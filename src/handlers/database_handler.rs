use crate::config::Config;
use crate::handlers::ClientHandler;
use crate::local_storage::{
	CoreLocalStorage, contract::ContractLocalStorage, location::LocationLocalStorage,
	note::NoteLocalStorage, photo::PhotoLocalStorage, sawmill::SawmillLocalStorage,
	shipment::ShipmentLocalStorage, user::UserLocalStorage,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

type DbPool = Pool<SqliteConnectionManager>;

pub struct DatabaseHandler {
	pools: Arc<RwLock<HashMap<String, DbPool>>>,
	config: Arc<Config>,
	client_handler: Arc<ClientHandler>,
}

impl DatabaseHandler {
	pub fn new(config: Arc<Config>, client_handler: Arc<ClientHandler>) -> Self {
		Self {
			pools: Arc::new(RwLock::new(HashMap::new())),
			config,
			client_handler,
		}
	}

	pub async fn get_or_create_pool(&self, tenant: &str) -> Result<DbPool, DatabaseError> {
		let mut pools = self.pools.write().await;

		if let Some(pool) = pools.get(tenant) {
			return Ok(pool.clone());
		}

		let db_path = self.get_db_path(tenant);
		log::info!(
			"Creating new connection pool for tenant {} at {}",
			tenant,
			db_path
		);

		if !Path::new(&db_path).exists() {
			self.initialize_database(&db_path)?;
		}

		let manager = SqliteConnectionManager::file(&db_path);
		let pool = Pool::builder()
			.max_size(self.config.max_pool_size)
			.build(manager)
			.map_err(|e| DatabaseError::PoolCreation(e.to_string()))?;

		pools.insert(tenant.to_string(), pool.clone());
		Ok(pool)
	}

	pub fn get_db_path(&self, tenant: &str) -> String {
		format!("{}/{}.db", self.config.database_dir, tenant)
	}

	pub async fn database_exists(&self, tenant: &str) -> bool {
		Path::new(&self.get_db_path(tenant)).exists()
	}

	fn initialize_database(&self, db_path: &str) -> Result<(), DatabaseError> {
		let dir_path = Path::new(&db_path).parent().unwrap_or(Path::new(""));
		if !dir_path.exists() {
			fs::create_dir_all(dir_path).map_err(|e| DatabaseError::Initialization(e.to_string()))?;
		}

		let conn =
			Connection::open(db_path).map_err(|e| DatabaseError::Initialization(e.to_string()))?;

		conn
			.execute("PRAGMA foreign_keys = ON;", [])
			.map_err(|e| DatabaseError::Initialization(e.to_string()))?;

		log::info!("Database initialized: {}", db_path);
		Ok(())
	}

	pub async fn get_client_db_path(&self, client_id: &str) -> Result<String, DatabaseError> {
		if let Some(client) = self.client_handler.get_client(client_id).await {
			if client.db_name.is_empty() {
				return Err(DatabaseError::ClientNotAuthenticated);
			}
			Ok(self.get_db_path(&client.db_name))
		} else {
			Err(DatabaseError::ClientNotFound)
		}
	}

	pub async fn process_update(
		&self,
		client_id: String,
		update_type: &str,
		data: &Value,
	) -> Result<bool, DatabaseError> {
		let db_path = self.get_client_db_path(&client_id).await?;
		let core_storage =
			Arc::new(CoreLocalStorage::new(&db_path).map_err(|e| DatabaseError::Storage(e.to_string()))?);

		let is_deleted = data.get("deleted").and_then(|v| v.as_i64()).unwrap_or(0) == 1;

		match update_type {
			"contract_update" => self.handle_contract_update(data, core_storage, is_deleted),
			"location_update" => self.handle_location_update(data, core_storage, is_deleted),
			"note_update" => self.handle_note_update(data, core_storage, is_deleted),
			"photo_update" => self.handle_photo_update(data, core_storage, is_deleted),
			"sawmill_update" => self.handle_sawmill_update(data, core_storage, is_deleted),
			"shipment_update" => self.handle_shipment_update(data, core_storage, is_deleted),
			"user_update" => self.handle_user_update(data, core_storage, is_deleted),
			_ => {
				log::warn!("Unknown update type: {}", update_type);
				Ok(false)
			}
		}
	}

	fn handle_contract_update(
		&self,
		data: &Value,
		core_storage: Arc<CoreLocalStorage>,
		is_deleted: bool,
	) -> Result<bool, DatabaseError> {
		log::debug!("Contract update received: {:?}", data);

		if !is_deleted {
			let contract_storage = ContractLocalStorage::new(core_storage)
				.map_err(|e| DatabaseError::Storage(e.to_string()))?;
			contract_storage
				.save_contract(data)
				.map_err(|e| DatabaseError::Storage(e.to_string()))
		} else {
			self.handle_deletion(data, core_storage, "contracts")
		}
	}

	fn handle_location_update(
		&self,
		data: &Value,
		core_storage: Arc<CoreLocalStorage>,
		is_deleted: bool,
	) -> Result<bool, DatabaseError> {
		log::debug!("Location update received: {:?}", data);

		if !is_deleted {
			let location_storage = LocationLocalStorage::new(core_storage)
				.map_err(|e| DatabaseError::Storage(e.to_string()))?;
			location_storage
				.save_location(data)
				.map_err(|e| DatabaseError::Storage(e.to_string()))
		} else {
			self.handle_deletion(data, core_storage, "locations")
		}
	}

	fn handle_note_update(
		&self,
		data: &Value,
		core_storage: Arc<CoreLocalStorage>,
		is_deleted: bool,
	) -> Result<bool, DatabaseError> {
		log::debug!("Note update received: {:?}", data);

		if !is_deleted {
			let note_storage =
				NoteLocalStorage::new(core_storage).map_err(|e| DatabaseError::Storage(e.to_string()))?;
			note_storage
				.save_note(data)
				.map_err(|e| DatabaseError::Storage(e.to_string()))
		} else {
			self.handle_deletion(data, core_storage, "notes")
		}
	}

	fn handle_photo_update(
		&self,
		data: &Value,
		core_storage: Arc<CoreLocalStorage>,
		is_deleted: bool,
	) -> Result<bool, DatabaseError> {
		log::debug!("Photo update received");

		if !is_deleted {
			let photo_storage =
				PhotoLocalStorage::new(core_storage).map_err(|e| DatabaseError::Storage(e.to_string()))?;
			photo_storage
				.save_photo(data)
				.map_err(|e| DatabaseError::Storage(e.to_string()))
		} else {
			self.handle_deletion(data, core_storage, "photos")
		}
	}

	fn handle_sawmill_update(
		&self,
		data: &Value,
		core_storage: Arc<CoreLocalStorage>,
		is_deleted: bool,
	) -> Result<bool, DatabaseError> {
		log::debug!("Sawmill update received: {:?}", data);

		if !is_deleted {
			let sawmill_storage = SawmillLocalStorage::new(core_storage)
				.map_err(|e| DatabaseError::Storage(e.to_string()))?;
			sawmill_storage
				.save_sawmill(data)
				.map_err(|e| DatabaseError::Storage(e.to_string()))
		} else {
			self.handle_deletion(data, core_storage, "sawmills")
		}
	}

	fn handle_shipment_update(
		&self,
		data: &Value,
		core_storage: Arc<CoreLocalStorage>,
		is_deleted: bool,
	) -> Result<bool, DatabaseError> {
		log::debug!("Shipment update received: {:?}", data);

		if !is_deleted {
			let shipment_storage = ShipmentLocalStorage::new(core_storage)
				.map_err(|e| DatabaseError::Storage(e.to_string()))?;
			shipment_storage
				.save_shipment(data)
				.map_err(|e| DatabaseError::Storage(e.to_string()))
		} else {
			self.handle_deletion(data, core_storage, "shipments")
		}
	}

	fn handle_user_update(
		&self,
		data: &Value,
		core_storage: Arc<CoreLocalStorage>,
		is_deleted: bool,
	) -> Result<bool, DatabaseError> {
		log::debug!("User update received: {:?}", data);

		if !is_deleted {
			// Validate user name
			if let Some(name) = data.get("name").and_then(|n| n.as_str()) {
				if name.is_empty() {
					log::warn!("Empty user name");
					return Ok(false);
				}
			}

			let user_storage =
				UserLocalStorage::new(core_storage).map_err(|e| DatabaseError::Storage(e.to_string()))?;
			user_storage
				.save_user(data)
				.map_err(|e| DatabaseError::Storage(e.to_string()))
		} else {
			self.handle_deletion(data, core_storage, "users")
		}
	}

	fn handle_deletion(
		&self,
		data: &Value,
		core_storage: Arc<CoreLocalStorage>,
		table: &str,
	) -> Result<bool, DatabaseError> {
		if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
			core_storage
				.mark_as_deleted(table, id)
				.map(|_| true)
				.map_err(|e| DatabaseError::Storage(e.to_string()))
		} else {
			Err(DatabaseError::MissingId)
		}
	}

	pub async fn cleanup_pools(&self) {
		let mut pools = self.pools.write().await;
		pools.clear();
		log::info!("All database pools cleaned up");
	}
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
	#[error("Database pool creation failed: {0}")]
	PoolCreation(String),
	#[error("Database initialization failed: {0}")]
	Initialization(String),
	#[error("Storage operation failed: {0}")]
	Storage(String),
	#[error("Missing entity ID")]
	MissingId,
	#[error("Database not found")]
	NotFound,
	#[error("Client not found")]
	ClientNotFound,
	#[error("Client not authenticated")]
	ClientNotAuthenticated,
}
