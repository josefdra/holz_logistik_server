use axum::{
    routing::{get, post, delete},
    Router,
};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod error;
mod handlers;
mod models;
mod ws;

use crate::{
    config::CONFIG,
    db::init_db_pool,
    handlers::{
        auth::{authenticate_api_key, generate_api_key, AuthState},
        user::{delete_user, get_all_users, get_user_by_id, save_user, UserState},
        ws::{ws_handler, AppState},
    },
    ws::ConnectionManager,
};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let pool = match init_db_pool(&CONFIG.database_url).await {
        Ok(pool) => {
            tracing::info!("Database initialized successfully");
            pool
        }
        Err(e) => {
            tracing::error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    // Create user store
    let user_store = db::user_store::UserStore::new(pool);
    
    // Create connection manager
    let connection_manager = std::sync::Arc::new(ConnectionManager::new());
    
    // Create app states
    let ws_state = AppState::new(user_store.clone());
    let auth_state = AuthState {
        user_store: user_store.clone(),
    };
    let user_state = UserState {
        user_store: user_store.clone(),
        connection_manager: connection_manager.clone(),
    };

    // Build the routes
    let app = Router::new()
        // WebSocket route
        .route("/ws", get(ws_handler))
        // REST routes for authentication
        .route("/api/auth", post(authenticate_api_key))
        .route("/api/auth/api-key", post(generate_api_key))
        // REST routes for users
        .route("/api/users", get(get_all_users).post(save_user))
        .route("/api/users/:id", get(get_user_by_id).delete(delete_user))
        // Add tracing
        .layer(TraceLayer::new_for_http())
        // Add state
        .with_state(ws_state)
        .with_state(auth_state)
        .with_state(user_state);

    // Parse the socket address
    let addr = SocketAddr::from(([0, 0, 0, 0], CONFIG.server_port));
    tracing::info!("Starting server on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
