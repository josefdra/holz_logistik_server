use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// WebSocket message types that can be handled by the server
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Authentication request from a client
    AuthenticationRequest,
    /// Authentication response to a client
    AuthenticationResponse,
    /// User data update from a client
    UserUpdate,
    /// User data deletion from a client
    UserDeletion,
    /// Connection status update to client
    ConnectionStatus,
    /// Ping message
    Ping,
    /// Pong message
    Pong,
}

/// WebSocket message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage<T> {
    /// Message type
    pub type_: String,
    /// Message data
    pub data: T,
    /// Timestamp when the message was created
    pub timestamp: DateTime<Utc>,
    /// Optional message ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
}

impl<T> WebSocketMessage<T> {
    pub fn new(type_: &str, data: T) -> Self {
        Self {
            type_: type_.to_string(),
            data,
            timestamp: Utc::now(),
            id: Some(Uuid::new_v4()),
        }
    }
}

/// Connection status message sent to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatusData {
    pub status: ConnectionStatus,
}

/// Connection status enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
}

/// User deletion message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDeletionData {
    pub id: i64,
    pub deleted: bool,
    pub timestamp: DateTime<Utc>,
}
