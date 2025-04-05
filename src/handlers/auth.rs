use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::{
    db::user_store::UserStore,
    error::{AppError, Result},
    models::user::AuthResponse,
};

/// State for authentication handlers
pub struct AuthState {
    pub user_store: UserStore,
}

/// API key authentication request
#[derive(Debug, Deserialize)]
pub struct ApiKeyAuthRequest {
    pub api_key: String,
}

/// Handler for API key authentication
pub async fn authenticate_api_key(
    State(state): State<AuthState>,
    Json(request): Json<ApiKeyAuthRequest>,
) -> Result<impl IntoResponse> {
    // Authenticate the user
    let response = state.user_store.authenticate(&request.api_key).await?;

    // Return the response
    Ok((StatusCode::OK, Json(response)))
}

/// Generate API key request
#[derive(Debug, Deserialize)]
pub struct GenerateApiKeyRequest {
    pub user_id: i64,
}

/// Generate API key response
#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub api_key: String,
}

/// Handler for generating a new API key
pub async fn generate_api_key(
    State(state): State<AuthState>,
    Json(request): Json<GenerateApiKeyRequest>,
) -> Result<impl IntoResponse> {
    // Check if the user exists
    let _user = state.user_store.get_user_by_id(request.user_id).await?;

    // Generate a new API key
    let api_key = state.user_store.generate_api_key(request.user_id).await?;

    // Return the response
    Ok((StatusCode::OK, Json(ApiKeyResponse { api_key })))
}
