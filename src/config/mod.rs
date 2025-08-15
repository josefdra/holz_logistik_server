use std::env;

#[derive(Clone)]
pub struct Config {
	pub port: u16,
	pub database_dir: String,
	pub auth_timeout_secs: u64,
	pub max_pool_size: u32,
}

impl Config {
	pub fn from_env() -> Result<Self, ConfigError> {
		Ok(Self {
			port: env::var("PORT")
				.unwrap_or_else(|_| "8080".to_string())
				.parse()
				.map_err(|_| ConfigError::InvalidPort)?,
			database_dir: env::var("DATABASE_DIR").unwrap_or_else(|_| "databases".to_string()),
			auth_timeout_secs: env::var("AUTH_TIMEOUT")
				.unwrap_or_else(|_| "10".to_string())
				.parse()
				.unwrap_or(10),
			max_pool_size: env::var("MAX_POOL_SIZE")
				.unwrap_or_else(|_| "20".to_string())
				.parse()
				.unwrap_or(20),
		})
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
	#[error("Invalid port number")]
	InvalidPort,
}
