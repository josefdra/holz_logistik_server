use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    db::user_store::UserStore,
    error::{AppError, Result},
    models::user::{User, UserDto},
    ws::SharedConnectionManager,
};

/// State for user handlers
pub struct UserState {
    pub user_store: UserStore,
    pub connection_manager: SharedConnectionManager,
}

/// Get all users handler
pub async fn get_all_users(State(state): State<UserState>) -> Result<impl IntoResponse> {
    let users = state.user_store.get_all_users().await?;
    let user_dtos: Vec<UserDto> = users.into_iter().map(UserDto::from).collect();
    Ok((StatusCode::OK, Json(user_dtos)))
}

/// Get user by ID handler
pub async fn get_user_by_id(
    State(state): State<UserState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse> {
    let user = state.user_store.get_user_by_id(id).await?;
    Ok((StatusCode::OK, Json(UserDto::from(user))))
}

/// Create or update user handler
pub async fn save_user(
    State(state): State<UserState>,
    Json(user_dto): Json<UserDto>,
) -> Result<impl IntoResponse> {
    // Save the user
    let updated_user = state.user_store.save_user(user_dto.clone()).await?;
    
    // Broadcast the update to all clients
    let updated_dto = UserDto::from(updated_user.clone());
    let updated_json = serde_json::to_value(updated_dto).map_err(AppError::Json)?;
    state.connection_manager.broadcast_user_update(updated_json).await?;
    
    Ok((StatusCode::OK, Json(UserDto::from(updated_user))))
}

/// Delete user handler
pub async fn delete_user(
    State(state): State<UserState>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse> {
    // Delete the user
    state.user_store.delete_user(id).await?;
    
    // Broadcast the deletion to all clients
    let deletion_data = serde_json::json!({
        "id": id,
        "deleted": true,
        "timestamp": chrono::Utc::now(),
    });
    state.connection_manager.broadcast_user_deletion(deletion_data).await?;
    
    Ok(StatusCode::NO_CONTENT)
}
