use std::sync::Arc;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};
use warp::ws::{Message, WebSocket};
use uuid::Uuid;
use serde_json::Value;
use crate::handlers::ClientHandler;
use crate::services::AuthService;

pub struct ConnectionHandler {
    client_handler: Arc<ClientHandler>,
    auth_service: Arc<AuthService>,
    auth_timeout_secs: u64,
}

impl ConnectionHandler {
    pub fn new(
        client_handler: Arc<ClientHandler>,
        auth_service: Arc<AuthService>,
        auth_timeout_secs: u64,
    ) -> Self {
        Self {
            client_handler,
            auth_service,
            auth_timeout_secs,
        }
    }

    pub async fn handle_new_connection<C>(
        &self,
        ws: WebSocket,
        controller: Arc<C>,
    ) where
        C: ProcessMessage + Send + Sync + 'static,
    {
        let (mut ws_tx, mut ws_rx) = ws.split();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let client_id = format!("client-{}", Uuid::new_v4());

        // Add client to handler
        if let Err(e) = self.client_handler.add_client(client_id.clone(), tx.clone()).await {
            log::error!("Failed to add client: {:?}", e);
            return;
        }

        // Spawn task to handle outgoing messages
        let tx_client_id = client_id.clone();
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = ws_tx.send(message).await {
                    log::error!("Error sending message to client {}: {:?}", tx_client_id, e);
                    break;
                }
            }
        });

        // Handle authentication with timeout
        let auth_result = timeout(
            Duration::from_secs(self.auth_timeout_secs),
            self.wait_for_authentication(client_id.clone(), &mut ws_rx, controller.clone()),
        ).await;

        match auth_result {
            Ok(Ok(true)) => {
                log::info!("Client {} authenticated successfully", client_id);
                self.handle_authenticated_connection(
                    client_id.clone(),
                    ws_rx,
                    controller,
                ).await;
            }
            Ok(Ok(false)) => {
                log::warn!("Authentication failed for client {}", client_id);
            }
            Ok(Err(e)) => {
                log::error!("Authentication error for client {}: {:?}", client_id, e);
            }
            Err(_) => {
                log::warn!("Authentication timeout for client {}", client_id);
            }
        }

        // Clean up
        if let Err(e) = self.client_handler.remove_client(&client_id).await {
            log::error!("Failed to remove client {}: {:?}", client_id, e);
        }
    }

    async fn wait_for_authentication<C>(
        &self,
        client_id: String,
        ws_rx: &mut futures_util::stream::SplitStream<WebSocket>,
        controller: Arc<C>,
    ) -> Result<bool, ConnectionError>
    where
        C: ProcessMessage + Send + Sync,
    {
        while let Some(result) = ws_rx.next().await {
            match result {
                Ok(msg) => {
                    if let Some(text) = msg.to_str().ok() {
                        if let Ok(json_msg) = serde_json::from_str::<Value>(text) {
                            let msg_type = json_msg.get("type").and_then(|v| v.as_str());
                            
                            if msg_type == Some("authentication_request") {
                                if let Err(e) = controller.process_message(client_id.clone(), json_msg).await {
                                    log::error!("Error processing auth message: {:?}", e);
                                    return Ok(false);
                                }
                                
                                // Check if client is now authenticated
                                if let Some(client) = self.client_handler.get_client(&client_id).await {
                                    return Ok(client.authenticated);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(ConnectionError::WebSocket(e.to_string()));
                }
            }
        }
        
        Ok(false)
    }

    async fn handle_authenticated_connection<C>(
        &self,
        client_id: String,
        mut ws_rx: futures_util::stream::SplitStream<WebSocket>,
        controller: Arc<C>,
    ) where
        C: ProcessMessage + Send + Sync,
    {
        log::info!("Client {} entering authenticated state", client_id);

        while let Some(result) = ws_rx.next().await {
            match result {
                Ok(msg) => {
                    if msg.is_close() {
                        log::info!("Client {} sent close frame", client_id);
                        break;
                    }
                    
                    if let Some(text) = msg.to_str().ok() {
                        if let Ok(json_msg) = serde_json::from_str::<Value>(text) {
                            let msg_type = json_msg.get("type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            
                            log::debug!("Received {} from client {}", msg_type, client_id);
                            
                            if let Err(e) = controller.process_message(client_id.clone(), json_msg).await {
                                log::error!("Error processing message from client {}: {:?}", client_id, e);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("WebSocket error for client {}: {:?}", client_id, e);
                    break;
                }
            }
        }

        log::info!("Client {} disconnecting", client_id);
    }
}

// Trait for the controller to implement
pub trait ProcessMessage {
    async fn process_message(
        &self,
        client_id: String,
        message: Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    #[error("Authentication failed")]
    AuthenticationFailed,
    #[error("Client error: {0}")]
    ClientError(String),
}
