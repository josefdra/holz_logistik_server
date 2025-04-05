use std::{collections::HashMap, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    models::{ConnectionStatus, ConnectionStatusData, WebSocketMessage},
};

/// WebSocket connection manager
pub struct ConnectionManager {
    connections: RwLock<HashMap<Uuid, mpsc::Sender<Message>>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new connection
    pub async fn register(&self, id: Uuid, sender: mpsc::Sender<Message>) {
        let mut connections = self.connections.write().await;
        connections.insert(id, sender);
        tracing::info!("Registered connection {}", id);
    }

    /// Unregister a connection
    pub async fn unregister(&self, id: &Uuid) {
        let mut connections = self.connections.write().await;
        connections.remove(id);
        tracing::info!("Unregistered connection {}", id);
    }

    /// Send a message to a specific connection
    pub async fn send_to(&self, connection_id: &Uuid, message: impl Serialize) -> Result<()> {
        let connections = self.connections.read().await;
        if let Some(sender) = connections.get(connection_id) {
            let message_json = serde_json::to_string(&message).map_err(AppError::Json)?;
            if let Err(e) = sender.send(Message::Text(message_json)).await {
                tracing::error!("Failed to send message to {}: {}", connection_id, e);
                return Err(AppError::WebSocket(format!(
                    "Failed to send message: {}",
                    e
                )));
            }
        } else {
            return Err(AppError::WebSocket(format!(
                "Connection {} not found",
                connection_id
            )));
        }
        Ok(())
    }

    /// Broadcast a message to all connections
    pub async fn broadcast(&self, message: impl Serialize + Clone) -> Result<()> {
        let message_json = serde_json::to_string(&message).map_err(AppError::Json)?;
        let connections = self.connections.read().await;

        for (id, sender) in connections.iter() {
            if let Err(e) = sender.send(Message::Text(message_json.clone())).await {
                tracing::error!("Failed to send message to {}: {}", id, e);
            }
        }

        Ok(())
    }

    /// Broadcast a user update to all connections
    pub async fn broadcast_user_update(&self, user_data: Value) -> Result<()> {
        let message = WebSocketMessage::new("user_update", user_data);
        self.broadcast(message).await
    }

    /// Broadcast a user deletion to all connections
    pub async fn broadcast_user_deletion(&self, deletion_data: Value) -> Result<()> {
        let message = WebSocketMessage::new("user_update", deletion_data);
        self.broadcast(message).await
    }

    /// Send a connection status update to a specific connection
    pub async fn send_connection_status(
        &self,
        connection_id: &Uuid,
        status: ConnectionStatus,
    ) -> Result<()> {
        let status_data = ConnectionStatusData { status };
        let message = WebSocketMessage::new("connection_status", status_data);
        self.send_to(connection_id, message).await
    }

    /// Get the number of active connections
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.len()
    }
}

/// Shared state for the connection manager
pub type SharedConnectionManager = Arc<ConnectionManager>;

/// Handle a WebSocket connection
pub async fn handle_socket(
    socket: WebSocket,
    connection_manager: SharedConnectionManager,
    router: Arc<dyn MessageRouter>,
) {
    // Generate a connection ID
    let connection_id = Uuid::new_v4();
    tracing::info!("New websocket connection: {}", connection_id);

    // Split the socket into sender and receiver
    let (mut sender, mut receiver) = socket.split();

    // Create a channel for sending messages to this client
    let (tx, mut rx) = mpsc::channel::<Message>(100);

    // Register the connection
    connection_manager.register(connection_id, tx).await;

    // Send the connected status
    if let Err(e) = connection_manager
        .send_connection_status(&connection_id, ConnectionStatus::Connected)
        .await
    {
        tracing::error!("Error sending connection status: {}", e);
    }

    // Task to forward messages from the channel to the WebSocket
    let forward_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Err(e) = sender.send(message).await {
                tracing::error!("Error sending WebSocket message: {}", e);
                break;
            }
        }
    });

    // Task to handle incoming messages
    let connection_manager_clone = connection_manager.clone();
    let router_clone = router.clone();
    let receive_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    tracing::debug!("Received message: {}", text);

                    // Parse and route the message
                    match serde_json::from_str::<WebSocketMessage<Value>>(&text) {
                        Ok(message) => {
                            if let Err(e) = router_clone
                                .route_message(
                                    connection_id,
                                    message,
                                    connection_manager_clone.clone(),
                                )
                                .await
                            {
                                tracing::error!("Error routing message: {}", e);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Error parsing message: {}", e);
                        }
                    }
                }
                Ok(Message::Binary(_)) => {
                    tracing::debug!("Received binary message");
                }
                Ok(Message::Ping(data)) => {
                    // Automatically respond to pings
                    if let Err(e) = sender.send(Message::Pong(data)).await {
                        tracing::error!("Error sending pong: {}", e);
                    }
                }
                Ok(Message::Pong(_)) => {
                    // Can be used to check for client connectivity
                    tracing::debug!("Received pong");
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("Client disconnected");
                    break;
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = forward_task => {},
        _ = receive_task => {},
    }

    // Unregister the connection
    connection_manager.unregister(&connection_id).await;
    tracing::info!("WebSocket connection closed: {}", connection_id);
}
