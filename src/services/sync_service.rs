use crate::handlers::{ClientHandler, DatabaseHandler};
use crate::local_storage::CoreLocalStorage;
use crate::local_storage::contract::ContractLocalStorage;
use crate::local_storage::location::LocationLocalStorage;
use crate::local_storage::note::NoteLocalStorage;
use crate::local_storage::photo::PhotoLocalStorage;
use crate::local_storage::sawmill::SawmillLocalStorage;
use crate::local_storage::shipment::ShipmentLocalStorage;
use crate::local_storage::user::UserLocalStorage;
use crate::services::MessageService;
use serde_json::{Value, json};
use std::sync::Arc;

pub struct SyncService {
	database_handler: Arc<DatabaseHandler>,
	message_service: Arc<MessageService>,
	client_handler: Arc<ClientHandler>,
}

impl SyncService {
	pub fn new(
		database_handler: Arc<DatabaseHandler>,
		message_service: Arc<MessageService>,
		client_handler: Arc<ClientHandler>,
	) -> Self {
		Self {
			database_handler,
			message_service,
			client_handler,
		}
	}

	pub async fn handle_sync_request(
		&self,
		client_id: String,
		data: Option<Value>,
	) -> Result<(), SyncError> {
		let data = data.ok_or(SyncError::MissingData)?;

		let client = self
			.client_handler
			.get_client(&client_id)
			.await
			.ok_or(SyncError::ClientNotFound)?;

		if !client.authenticated {
			return Err(SyncError::NotAuthenticated);
		}

		let db_path = self.database_handler.get_db_path(&client.db_name);
		let core_storage = Arc::new(
			CoreLocalStorage::new(&db_path).map_err(|e| SyncError::StorageError(e.to_string()))?,
		);

		// Sync each entity type
		let sync_dates = SyncDates {
			user: data
				.get("user_update")
				.and_then(|v| v.as_i64())
				.unwrap_or(0),
			sawmill: data
				.get("sawmill_update")
				.and_then(|v| v.as_i64())
				.unwrap_or(0),
			contract: data
				.get("contract_update")
				.and_then(|v| v.as_i64())
				.unwrap_or(0),
			note: data
				.get("note_update")
				.and_then(|v| v.as_i64())
				.unwrap_or(0),
			location: data
				.get("location_update")
				.and_then(|v| v.as_i64())
				.unwrap_or(0),
			shipment: data
				.get("shipment_update")
				.and_then(|v| v.as_i64())
				.unwrap_or(0),
			photo: data
				.get("photo_update")
				.and_then(|v| v.as_i64())
				.unwrap_or(0),
		};

		// Send data for each entity type
		self
			.send_user_data(sync_dates.user, &client_id, core_storage.clone())
			.await?;
		self
			.send_sawmill_data(sync_dates.sawmill, &client_id, core_storage.clone())
			.await?;
		self
			.send_contract_data(sync_dates.contract, &client_id, core_storage.clone())
			.await?;
		self
			.send_location_data(sync_dates.location, &client_id, core_storage.clone())
			.await?;
		self
			.send_shipment_data(sync_dates.shipment, &client_id, core_storage.clone())
			.await?;
		self
			.send_note_data(sync_dates.note, &client_id, core_storage.clone())
			.await?;
		self
			.send_photo_data(sync_dates.photo, &client_id, core_storage.clone())
			.await?;

		// Mark sync as completed
		self
			.client_handler
			.mark_sync_completed(&client_id)
			.await
			.ok();

		// Send sync complete message
		let response = json!({
				"type": "sync_from_server_complete",
				"timestamp": chrono::Utc::now().timestamp_millis()
		});

		self
			.message_service
			.send_message(client_id, &response.to_string())
			.await?;

		Ok(())
	}

	async fn send_user_data(
		&self,
		last_sync: i64,
		client_id: &str,
		core_storage: Arc<CoreLocalStorage>,
	) -> Result<i64, SyncError> {
		let user_storage =
			UserLocalStorage::new(core_storage).map_err(|e| SyncError::StorageError(e.to_string()))?;

		let mut date = last_sync;
		let mut should_continue = true;

		while should_continue {
			let users = user_storage
				.get_user_updates_by_date(date)
				.map_err(|e| SyncError::StorageError(e.to_string()))?;

			if users.is_empty() {
				should_continue = false;
			} else {
				for user in &users {
					let response = json!({
							"type": "user_update",
							"data": user,
							"timestamp": chrono::Utc::now().timestamp_millis()
					});

					self
						.message_service
						.send_message(client_id.to_string(), &response.to_string())
						.await?;

					if let Some(newest_date) = user["arrivalAtServer"].as_i64() {
						if date <= newest_date {
							date = newest_date + 1;
						}
					}
				}
			}
		}

		let completion_message = json!({
				"type": "user_update",
				"data": json!({
						"newSyncDate": date,
				}),
				"timestamp": chrono::Utc::now().timestamp_millis()
		});

		self
			.message_service
			.send_message(client_id.to_string(), &completion_message.to_string())
			.await?;

		Ok(date)
	}

	async fn send_sawmill_data(
		&self,
		last_sync: i64,
		client_id: &str,
		core_storage: Arc<CoreLocalStorage>,
	) -> Result<i64, SyncError> {
		let sawmill_storage =
			SawmillLocalStorage::new(core_storage).map_err(|e| SyncError::StorageError(e.to_string()))?;

		let mut date = last_sync;
		let mut should_continue = true;

		while should_continue {
			let sawmills = sawmill_storage
				.get_sawmill_updates_by_date(date)
				.map_err(|e| SyncError::StorageError(e.to_string()))?;

			if sawmills.is_empty() {
				should_continue = false;
			} else {
				for sawmill in &sawmills {
					let response = json!({
							"type": "sawmill_update",
							"data": sawmill,
							"timestamp": chrono::Utc::now().timestamp_millis()
					});

					self
						.message_service
						.send_message(client_id.to_string(), &response.to_string())
						.await?;

					if let Some(newest_date) = sawmill["arrivalAtServer"].as_i64() {
						if date <= newest_date {
							date = newest_date + 1;
						}
					}
				}
			}
		}

		let completion_message = json!({
				"type": "sawmill_update",
				"data": json!({
						"newSyncDate": date,
				}),
				"timestamp": chrono::Utc::now().timestamp_millis()
		});

		self
			.message_service
			.send_message(client_id.to_string(), &completion_message.to_string())
			.await?;

		Ok(date)
	}

	// Similar implementations for other entity types (contract, location, note, photo, shipment)
	// Following the same pattern as above...

	async fn send_contract_data(
		&self,
		last_sync: i64,
		client_id: &str,
		core_storage: Arc<CoreLocalStorage>,
	) -> Result<i64, SyncError> {
		let contract_storage = ContractLocalStorage::new(core_storage)
			.map_err(|e| SyncError::StorageError(e.to_string()))?;

		let mut date = last_sync;
		let mut should_continue = true;

		while should_continue {
			let contracts = contract_storage
				.get_contract_updates_by_date(date)
				.map_err(|e| SyncError::StorageError(e.to_string()))?;

			if contracts.is_empty() {
				should_continue = false;
			} else {
				for contract in &contracts {
					let response = json!({
							"type": "contract_update",
							"data": contract,
							"timestamp": chrono::Utc::now().timestamp_millis()
					});

					self
						.message_service
						.send_message(client_id.to_string(), &response.to_string())
						.await?;

					if let Some(newest_date) = contract["arrivalAtServer"].as_i64() {
						if date <= newest_date {
							date = newest_date + 1;
						}
					}
				}
			}
		}

		let completion_message = json!({
				"type": "contract_update",
				"data": json!({"newSyncDate": date}),
				"timestamp": chrono::Utc::now().timestamp_millis()
		});

		self
			.message_service
			.send_message(client_id.to_string(), &completion_message.to_string())
			.await?;

		Ok(date)
	}

	async fn send_location_data(
		&self,
		last_sync: i64,
		client_id: &str,
		core_storage: Arc<CoreLocalStorage>,
	) -> Result<i64, SyncError> {
		let location_storage = LocationLocalStorage::new(core_storage)
			.map_err(|e| SyncError::StorageError(e.to_string()))?;

		let mut date = last_sync;
		let mut should_continue = true;

		while should_continue {
			let locations = location_storage
				.get_location_updates_by_date(date)
				.map_err(|e| SyncError::StorageError(e.to_string()))?;

			if locations.is_empty() {
				should_continue = false;
			} else {
				for location in &locations {
					let response = json!({
							"type": "location_update",
							"data": location,
							"timestamp": chrono::Utc::now().timestamp_millis()
					});

					self
						.message_service
						.send_message(client_id.to_string(), &response.to_string())
						.await?;

					if let Some(newest_date) = location["arrivalAtServer"].as_i64() {
						if date <= newest_date {
							date = newest_date + 1;
						}
					}
				}
			}
		}

		let completion_message = json!({
				"type": "location_update",
				"data": json!({"newSyncDate": date}),
				"timestamp": chrono::Utc::now().timestamp_millis()
		});

		self
			.message_service
			.send_message(client_id.to_string(), &completion_message.to_string())
			.await?;

		Ok(date)
	}

	async fn send_shipment_data(
		&self,
		last_sync: i64,
		client_id: &str,
		core_storage: Arc<CoreLocalStorage>,
	) -> Result<i64, SyncError> {
		let shipment_storage = ShipmentLocalStorage::new(core_storage)
			.map_err(|e| SyncError::StorageError(e.to_string()))?;

		let mut date = last_sync;
		let mut should_continue = true;

		while should_continue {
			let shipments = shipment_storage
				.get_shipments_by_date(date)
				.map_err(|e| SyncError::StorageError(e.to_string()))?;

			if shipments.is_empty() {
				should_continue = false;
			} else {
				for shipment in &shipments {
					let response = json!({
							"type": "shipment_update",
							"data": shipment,
							"timestamp": chrono::Utc::now().timestamp_millis()
					});

					self
						.message_service
						.send_message(client_id.to_string(), &response.to_string())
						.await?;

					if let Some(newest_date) = shipment["arrivalAtServer"].as_i64() {
						if date <= newest_date {
							date = newest_date + 1;
						}
					}
				}
			}
		}

		let completion_message = json!({
				"type": "shipment_update",
				"data": json!({"newSyncDate": date}),
				"timestamp": chrono::Utc::now().timestamp_millis()
		});

		self
			.message_service
			.send_message(client_id.to_string(), &completion_message.to_string())
			.await?;

		Ok(date)
	}

	async fn send_note_data(
		&self,
		last_sync: i64,
		client_id: &str,
		core_storage: Arc<CoreLocalStorage>,
	) -> Result<i64, SyncError> {
		let note_storage =
			NoteLocalStorage::new(core_storage).map_err(|e| SyncError::StorageError(e.to_string()))?;

		let mut date = last_sync;
		let mut should_continue = true;

		while should_continue {
			let notes = note_storage
				.get_note_updates_by_date(date)
				.map_err(|e| SyncError::StorageError(e.to_string()))?;

			if notes.is_empty() {
				should_continue = false;
			} else {
				for note in &notes {
					let response = json!({
							"type": "note_update",
							"data": note,
							"timestamp": chrono::Utc::now().timestamp_millis()
					});

					self
						.message_service
						.send_message(client_id.to_string(), &response.to_string())
						.await?;

					if let Some(newest_date) = note["arrivalAtServer"].as_i64() {
						if date <= newest_date {
							date = newest_date + 1;
						}
					}
				}
			}
		}

		let completion_message = json!({
				"type": "note_update",
				"data": json!({"newSyncDate": date}),
				"timestamp": chrono::Utc::now().timestamp_millis()
		});

		self
			.message_service
			.send_message(client_id.to_string(), &completion_message.to_string())
			.await?;

		Ok(date)
	}

	async fn send_photo_data(
		&self,
		last_sync: i64,
		client_id: &str,
		core_storage: Arc<CoreLocalStorage>,
	) -> Result<i64, SyncError> {
		let photo_storage =
			PhotoLocalStorage::new(core_storage).map_err(|e| SyncError::StorageError(e.to_string()))?;

		let mut date = last_sync;
		let mut should_continue = true;

		while should_continue {
			let photos = photo_storage
				.get_photo_updates_by_date(date)
				.map_err(|e| SyncError::StorageError(e.to_string()))?;

			if photos.is_empty() {
				should_continue = false;
			} else {
				for photo in &photos {
					let response = json!({
							"type": "photo_update",
							"data": photo,
							"timestamp": chrono::Utc::now().timestamp_millis()
					});

					self
						.message_service
						.send_message(client_id.to_string(), &response.to_string())
						.await?;

					// Add delay for photos to avoid overwhelming client
					tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

					if let Some(newest_date) = photo["arrivalAtServer"].as_i64() {
						if date <= newest_date {
							date = newest_date + 1;
						}
					}
				}
			}
		}

		let completion_message = json!({
				"type": "photo_update",
				"data": json!({
						"newSyncDate": date,
				}),
				"timestamp": chrono::Utc::now().timestamp_millis()
		});

		self
			.message_service
			.send_message(client_id.to_string(), &completion_message.to_string())
			.await?;

		Ok(date)
	}

	pub async fn handle_sync_complete(&self, client_id: String) -> Result<(), SyncError> {
		let response = json!({
				"type": "sync_to_server_complete",
				"timestamp": chrono::Utc::now().timestamp_millis()
		});

		self
			.message_service
			.send_message(client_id, &response.to_string())
			.await
			.map_err(|e| SyncError::MessageError(e.to_string()))
	}
}

struct SyncDates {
	user: i64,
	sawmill: i64,
	contract: i64,
	note: i64,
	location: i64,
	shipment: i64,
	photo: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
	#[error("Missing sync data")]
	MissingData,
	#[error("Client not found")]
	ClientNotFound,
	#[error("Client not authenticated")]
	NotAuthenticated,
	#[error("Storage error: {0}")]
	StorageError(String),
	#[error("Message error: {0}")]
	MessageError(String),
}

impl From<crate::services::message_service::MessageError> for SyncError {
    fn from(err: crate::services::message_service::MessageError) -> Self {
        SyncError::MessageError(err.to_string())
    }
}