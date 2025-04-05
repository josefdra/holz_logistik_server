use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::time::Duration;

pub mod user_store;

pub type DbPool = Pool<Sqlite>;

/// Initialize the database connection pool
pub async fn init_db_pool(database_url: &str) -> Result<DbPool> {
    // Create the database if it doesn't exist
    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
        Sqlite::create_database(database_url).await?;
    }

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await?;

    // Run migrations
    setup_database(&pool).await?;

    Ok(pool)
}

/// Set up the database schema
async fn setup_database(pool: &DbPool) -> Result<()> {
    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            role INTEGER NOT NULL,
            api_key TEXT,
            password_hash TEXT,
            last_edit TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Add some sample users if the table is empty
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    if count.0 == 0 {
        // Insert a sample admin user
        sqlx::query(
            r#"
            INSERT INTO users (id, name, role, api_key, last_edit)
            VALUES (1, 'Admin User', 2, 'admin-api-key', CURRENT_TIMESTAMP);
            "#,
        )
        .execute(pool)
        .await?;

        // Insert a sample privileged user
        sqlx::query(
            r#"
            INSERT INTO users (id, name, role, api_key, last_edit)
            VALUES (2, 'Privileged User', 1, 'privileged-api-key', CURRENT_TIMESTAMP);
            "#,
        )
        .execute(pool)
        .await?;

        // Insert a sample basic user
        sqlx::query(
            r#"
            INSERT INTO users (id, name, role, api_key, last_edit)
            VALUES (3, 'Basic User', 0, 'basic-api-key', CURRENT_TIMESTAMP);
            "#,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
