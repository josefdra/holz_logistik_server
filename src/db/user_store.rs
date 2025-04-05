use chrono::Utc;
use sqlx::query_as;

use crate::{
    db::DbPool,
    error::{AppError, Result},
    models::user::{AuthResponse, Role, User, UserDto},
};

/// User store for database operations
pub struct UserStore {
    pool: DbPool,
}

impl UserStore {
    /// Create a new UserStore with the provided database pool
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Get a list of all users
    pub async fn get_all_users(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Database)?;

        Ok(users)
    }

    /// Get a user by ID
    pub async fn get_user_by_id(&self, id: i64) -> Result<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::UserNotFound)?;

        Ok(user)
    }

    /// Get a user by API key
    pub async fn get_user_by_api_key(&self, api_key: &str) -> Result<User> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE api_key = ?")
            .bind(api_key)
            .fetch_optional(&self.pool)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Auth("Invalid API key".into()))?;

        Ok(user)
    }

    /// Authenticate a user with an API key
    pub async fn authenticate(&self, api_key: &str) -> Result<AuthResponse> {
        match self.get_user_by_api_key(api_key).await {
            Ok(user) => Ok(AuthResponse::success(user)),
            Err(_) => Ok(AuthResponse::failure()),
        }
    }

    /// Create or update a user
    pub async fn save_user(&self, user: UserDto) -> Result<User> {
        // Check if user exists
        let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(user.id)
            .fetch_optional(&self.pool)
            .await
            .map_err(AppError::Database)?;

        let now = Utc::now();

        if existing_user.is_some() {
            // Update existing user
            sqlx::query(
                r#"
                UPDATE users 
                SET name = ?, role = ?, last_edit = ?
                WHERE id = ?
                "#,
            )
            .bind(&user.name)
            .bind(user.role as i32)
            .bind(now)
            .bind(user.id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;
        } else {
            // Create new user
            sqlx::query(
                r#"
                INSERT INTO users (id, name, role, last_edit)
                VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(user.id)
            .bind(&user.name)
            .bind(user.role as i32)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;
        }

        // Return the updated user
        self.get_user_by_id(user.id).await
    }

    /// Delete a user by ID
    pub async fn delete_user(&self, id: i64) -> Result<()> {
        // Check if user exists
        let existing_user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::UserNotFound)?;

        // Delete the user
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(existing_user.id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    /// Generate a new API key for a user
    pub async fn generate_api_key(&self, user_id: i64) -> Result<String> {
        // Generate a random API key
        let api_key = uuid::Uuid::new_v4().to_string();

        // Update the user's API key
        sqlx::query("UPDATE users SET api_key = ? WHERE id = ?")
            .bind(&api_key)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;

        Ok(api_key)
    }
}
