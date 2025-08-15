use std::sync::Arc;
use serde_json::{json, Value};
use crate::handlers::{ClientHandler, DatabaseHandler};
use crate::local_storage::user::UserLocalStorage;
use crate::local_storage::CoreLocalStorage;

pub struct AuthService {
    database_handler: Arc<DatabaseHandler>,
    client_handler: Arc<ClientHandler>,
}

impl AuthService {
    pub fn new(
        database_handler: Arc<DatabaseHandler>,
        client_handler: Arc<ClientHandler>,
    ) -> Self {
        Self {
            database_handler,
            client_handler,
        }
    }

    pub async fn authenticate(
        &self,
        client_id: String,
        data: Option<Value>,
    ) -> Result<bool, AuthError> {
        let data = data.ok_or(AuthError::MissingData)?;
        
        let api_key = data
            .get("apiKey")
            .and_then(|v| v.as_str())
            .ok_or(AuthError::MissingApiKey)?;

        let parts: Vec<&str> = api_key.splitn(2, '-').collect();
        if parts.len() != 2 {
            return Err(AuthError::InvalidApiKeyFormat);
        }

        let tenant = parts[0];
        let user_id = parts[1];

        log::info!("Authentication attempt for tenant: {}, user_id: {}", tenant, user_id);

        // Check if database exists
        if !self.database_handler.database_exists(tenant).await {
            self.send_auth_rejection(
                &client_id,
                "Invalid tenant",
            ).await?;
            return Ok(false);
        }

        // Get database pool
        let pool = self.database_handler
            .get_or_create_pool(tenant)
            .await
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        // Verify connection
        pool.get()
            .map_err(|e| AuthError::DatabaseError(e.to_string()))?;

        // Get user from database
        let db_path = self.database_handler.get_db_path(tenant);
        let core_storage = Arc::new(
            CoreLocalStorage::new(&db_path)
                .map_err(|e| AuthError::StorageError(e.to_string()))?
        );

        let user_storage = UserLocalStorage::new(core_storage)
            .map_err(|e| AuthError::StorageError(e.to_string()))?;

        let user_result = user_storage
            .get_user_by_id(user_id)
            .map_err(|e| AuthError::StorageError(e.to_string()))?;

        if user_result.is_none() {
            self.send_auth_rejection(
                &client_id,
                "User not found",
            ).await?;
            return Ok(false);
        }

        let user_data = user_result.unwrap();

        // Update client with auth info
        self.client_handler
            .update_client_auth(&client_id, tenant.to_string(), user_id.to_string())
            .await
            .map_err(|e| AuthError::ClientError(e.to_string()))?;

        // Send success response
        let response = json!({
            "type": "authentication_response",
            "data": {
                "id": user_data.get("id").unwrap_or(&json!("")).as_str(),
                "role": user_data.get("role").unwrap_or(&json!(0)),
                "lastEdit": user_data.get("lastEdit").unwrap_or(&json!(chrono::Utc::now().timestamp_millis())),
                "name": user_data.get("name").unwrap_or(&json!("Unknown User")).as_str(),
                "authenticated": 1,
            },
            "timestamp": chrono::Utc::now().timestamp_millis()
        });

        self.send_message(&client_id, &response.to_string()).await?;
        
        Ok(true)
    }

    async fn send_auth_rejection(
        &self,
        client_id: &str,
        error: &str,
    ) -> Result<(), AuthError> {
        let response = json!({
            "type": "authentication_response",
            "data": {
                "authenticated": 0,
                "error": error
            },
            "timestamp": chrono::Utc::now().timestamp_millis()
        });

        self.send_message(client_id, &response.to_string()).await
    }

    async fn send_message(&self, client_id: &str, message: &str) -> Result<(), AuthError> {
        if let Some(client) = self.client_handler.get_client(client_id).await {
            client.sender
                .send(warp::ws::Message::text(message))
                .map_err(|e| AuthError::MessageError(e.to_string()))?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Missing authentication data")]
    MissingData,
    #[error("Missing API key")]
    MissingApiKey,
    #[error("Invalid API key format")]
    InvalidApiKeyFormat,
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Client error: {0}")]
    ClientError(String),
    #[error("Message send error: {0}")]
    MessageError(String),
}
