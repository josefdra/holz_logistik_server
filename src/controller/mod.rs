use crate::config::Config;
use crate::handlers::{ClientHandler, ConnectionHandler, DatabaseHandler, ProcessMessage};
use crate::services::{AuthService, MessageService, SyncService};
use std::sync::Arc;

pub struct Controller {
    client_handler: Arc<ClientHandler>,
    database_handler: Arc<DatabaseHandler>,
    connection_handler: Arc<ConnectionHandler>,
    auth_service: Arc<AuthService>,
    sync_service: Arc<SyncService>,
    message_service: Arc<MessageService>,
    config: Arc<Config>,
}

impl Controller {
    pub fn new(config: Config) -> Self {
        let config = Arc::new(config);
        let client_handler = Arc::new(ClientHandler::new());
        let database_handler = Arc::new(DatabaseHandler::new(config.clone(), client_handler.clone()));
        let message_service = Arc::new(MessageService::new(client_handler.clone()));
        let auth_service = Arc::new(AuthService::new(
            database_handler.clone(),
            client_handler.clone(),
        ));
        let sync_service = Arc::new(SyncService::new(
            database_handler.clone(),
            message_service.clone(),
            client_handler.clone(),
        ));
        let connection_handler = Arc::new(ConnectionHandler::new(
            client_handler.clone(),
            auth_service.clone(),
            config.auth_timeout_secs,
        ));

        Self {
            client_handler,
            database_handler,
            connection_handler,
            auth_service,
            sync_service,
            message_service,
            config,
        }
    }

    pub async fn handle_websocket_connection(&self, ws: warp::ws::WebSocket) {
        let controller = Arc::new(self.clone());
        self.connection_handler.handle_new_connection(ws, controller).await;
    }
		
    async fn handle_data_update(
        &self,
        client_id: String,
        msg_type: &str,
        message: &serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = message.get("data").cloned().unwrap_or(serde_json::json!({}));

        let update_successful = self.database_handler
            .process_update(client_id.clone(), msg_type, &data)
            .await?;

        if update_successful {
            self.message_service.broadcast_update(client_id, &message).await?;
        }

        Ok(())
    }
}

impl Clone for Controller {
    fn clone(&self) -> Self {
        Self {
            client_handler: self.client_handler.clone(),
            database_handler: self.database_handler.clone(),
            connection_handler: self.connection_handler.clone(),
            auth_service: self.auth_service.clone(),
            sync_service: self.sync_service.clone(),
            message_service: self.message_service.clone(),
            config: self.config.clone(),
        }
    }
}

#[async_trait::async_trait]
impl ProcessMessage for Controller {
    async fn process_message(
        &self,
        client_id: String,
        message: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg_type = message
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        match msg_type {
            "authentication_request" => {
                self.auth_service
                    .authenticate(client_id, message.get("data").cloned())
                    .await?;
            }
            "sync_request" => {
                self.sync_service
                    .handle_sync_request(client_id, message.get("data").cloned())
                    .await?;
            }
            "sync_complete" => {
                self.sync_service.handle_sync_complete(client_id).await?;
            }
            "ping" => {
                self.message_service.send_pong(client_id).await?;
            }
            msg_type if msg_type.ends_with("_update") => {
                self.handle_data_update(client_id, msg_type, &message).await?;
            }
            _ => {
                log::warn!("Unknown message type: {}", msg_type);
            }
        }

        Ok(())
    }
}
