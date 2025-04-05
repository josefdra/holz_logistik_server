use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    db::user_store::UserStore,
    error::{AppError, Result},
    models::{
        UserDeletionData, WebSocketMessage,
        user::{AuthRequest, UserDto},
    },
    ws::connection::SharedConnectionManager,
};

/// Message router trait for handling websocket messages
#[async_trait]
pub trait MessageRouter: Send + Sync {
    async fn route_message(
        &self,
        connection_id: Uuid,
        message: WebSocketMessage<Value>,
        connection_manager: SharedConnectionManager,
    ) -> Result<()>;
}

/// Default implementation of the MessageRouter trait
pub struct DefaultMessageRouter {
    user_store: UserStore,
    // Map to track which connection belongs to which authenticated user
    auth_map: tokio::sync::RwLock<HashMap<Uuid, i64>>,
}

impl DefaultMessageRouter {
    /// Create a new message router
    pub fn new(user_store: UserStore) -> Self {
        Self {
            user_store,
            auth_map: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Store the authenticated user ID for a connection
    async fn set_authenticated_user(&self, connection_id: &Uuid, user_id: i64) {
        let mut map = self.auth_map.write().await;
        map.insert(*connection_id, user_id);
    }

    /// Get the authenticated user ID for a connection
    async fn get_authenticated_user(&self, connection_id: &Uuid) -> Option<i64> {
        let map = self.auth_map.read().await;
        map.get(connection_id).copied()
    }

    /// Remove the authenticated user ID for a connection
    async fn remove_authenticated_user(&self, connection_id: &Uuid) {
        let mut map = self.auth_map.write().await;
        map.remove(connection_id);
    }

    /// Handle authentication request
    async fn handle_auth_request(
        &self,
        connection_id: Uuid,
        data: Value,
        connection_manager: SharedConnectionManager,
    ) -> Result<()> {
        // Parse the authentication request
        let auth_request: AuthRequest = serde_json::from_value(data)
            .map_err(|e| AppError::BadRequest(format!("Invalid authentication request: {}", e)))?;

        // Authenticate the user
        let response = self.user_store.authenticate(&auth_request.api_key).await?;

        // If authenticated, store the user ID
        if response.authenticated && response.id.is_some() {
            self.set_authenticated_user(&connection_id, response.id.unwrap())
                .await;
        }

        // Send the response
        let message = WebSocketMessage::new("authentication_response", response);
        connection_manager.send_to(&connection_id, message).await?;

        Ok(())
    }

    /// Handle user update
    async fn handle_user_update(
        &self,
        connection_id: Uuid,
        data: Value,
        connection_manager: SharedConnectionManager,
    ) -> Result<()> {
        // Check if the connection is authenticated
        if self.get_authenticated_user(&connection_id).await.is_none() {
            return Err(AppError::Auth("Not authenticated".to_string()));
        }

        // Check if the message contains a "deleted" field
        if let Some(deleted) = data.get("deleted") {
            if deleted.as_bool().unwrap_or(false) {
                return self
                    .handle_user_deletion(connection_id, data, connection_manager)
                    .await;
            }
        }

        // Parse the user DTO
        let user_dto: UserDto = serde_json::from_value(data)
            .map_err(|e| AppError::BadRequest(format!("Invalid user data: {}", e)))?;

        // Save the user
        let updated_user = self.user_store.save_user(user_dto).await?;

        // Convert back to DTO for broadcasting
        let updated_dto = UserDto::from(updated_user);
        let updated_json = serde_json::to_value(updated_dto).map_err(AppError::Json)?;

        // Broadcast the update to all clients
        connection_manager
            .broadcast_user_update(updated_json)
            .await?;

        Ok(())
    }

    /// Handle user deletion
    async fn handle_user_deletion(
        &self,
        connection_id: Uuid,
        data: Value,
        connection_manager: SharedConnectionManager,
    ) -> Result<()> {
        // Check if the connection is authenticated
        if self.get_authenticated_user(&connection_id).await.is_none() {
            return Err(AppError::Auth("Not authenticated".to_string()));
        }

        // Parse the deletion data
        let id = data
            .get("id")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| AppError::BadRequest("Missing or invalid user id".to_string()))?;

        // Delete the user
        self.user_store.delete_user(id).await?;

        // Create deletion notification
        let deletion_data = UserDeletionData {
            id,
            deleted: true,
            timestamp: Utc::now(),
        };

        // Convert to JSON for broadcasting
        let deletion_json = serde_json::to_value(deletion_data).map_err(AppError::Json)?;

        // Broadcast the deletion to all clients
        connection_manager
            .broadcast_user_deletion(deletion_json)
            .await?;

        Ok(())
    }

    /// Handle ping message
    async fn handle_ping(
        &self,
        connection_id: Uuid,
        connection_manager: SharedConnectionManager,
    ) -> Result<()> {
        // Respond with a pong message
        let message = WebSocketMessage::new("pong", Value::Null);
        connection_manager.send_to(&connection_id, message).await?;

        Ok(())
    }
}

#[async_trait]
impl MessageRouter for DefaultMessageRouter {
    async fn route_message(
        &self,
        connection_id: Uuid,
        message: WebSocketMessage<Value>,
        connection_manager: SharedConnectionManager,
    ) -> Result<()> {
        // Route the message based on its type
        match message.type_.as_str() {
            "authentication_request" => {
                self.handle_auth_request(connection_id, message.data, connection_manager)
                    .await
            }
            "user_update" => {
                self.handle_user_update(connection_id, message.data, connection_manager)
                    .await
            }
            "ping" => self.handle_ping(connection_id, connection_manager).await,
            _ => {
                tracing::warn!("Unknown message type: {}", message.type_);
                Ok(())
            }
        }
    }
}
