use std::sync::Arc;

use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};

use crate::{
    ws::{handle_socket, ConnectionManager, DefaultMessageRouter, MessageRouter},
    db::user_store::UserStore,
};

/// Application state for websocket handlers
pub struct AppState {
    pub connection_manager: Arc<ConnectionManager>,
    pub message_router: Arc<dyn MessageRouter>,
}

impl AppState {
    pub fn new(user_store: UserStore) -> Self {
        let connection_manager = Arc::new(ConnectionManager::new());
        let message_router = Arc::new(DefaultMessageRouter::new(user_store));
        
        Self {
            connection_manager,
            message_router,
        }
    }
}

/// Handler for WebSocket connections
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Handle a new WebSocket connection
async fn handle_websocket(socket: WebSocket, state: AppState) {
    handle_socket(
        socket,
        state.connection_manager,
        state.message_router,
    ).await;
}
