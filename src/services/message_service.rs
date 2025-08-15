use crate::handlers::ClientHandler;
use serde_json::{Value, json};
use std::sync::Arc;
use warp::ws::Message;

pub struct MessageService {
	client_handler: Arc<ClientHandler>,
}

impl MessageService {
	pub fn new(client_handler: Arc<ClientHandler>) -> Self {
		Self { client_handler }
	}

	pub async fn send_message(&self, client_id: String, message: &str) -> Result<(), MessageError> {
		if let Some(client) = self.client_handler.get_client(&client_id).await {
			log::debug!("Sending message to client {}: {}", client_id, message);

			client
				.sender
				.send(Message::text(message))
				.map_err(|e| MessageError::SendFailed(e.to_string()))?;

			Ok(())
		} else {
			Err(MessageError::ClientNotFound(client_id))
		}
	}

	pub async fn send_pong(&self, client_id: String) -> Result<(), MessageError> {
		let response = json!({
				"type": "pong",
				"timestamp": chrono::Utc::now().timestamp_millis()
		});

		self.send_message(client_id, &response.to_string()).await
	}

	pub async fn broadcast_update(
		&self,
		sender_id: String,
		message: &Value,
	) -> Result<(), MessageError> {
		let sender_client = self
			.client_handler
			.get_client(&sender_id)
			.await
			.ok_or_else(|| MessageError::ClientNotFound(sender_id.clone()))?;

		if sender_client.db_name.is_empty() {
			return Err(MessageError::ClientNotAuthenticated);
		}

		let is_deleted = message
			.get("data")
			.and_then(|data| data.get("deleted"))
			.and_then(|deleted| deleted.as_i64())
			.unwrap_or(0)
			== 1;

		let msg_type = message
			.get("type")
			.and_then(|v| v.as_str())
			.unwrap_or("unknown");

		let entity_id = message
			.get("data")
			.and_then(|data| data.get("id"))
			.cloned()
			.unwrap_or(json!("unknown"));

		// Get all clients in the same tenant
		let clients = self
			.client_handler
			.get_clients_by_tenant(&sender_client.db_name)
			.await;

		for client in clients {
			if client.id != sender_id {
				// Send the original message to other clients
				if let Err(e) = client.sender.send(Message::text(&message.to_string())) {
					log::error!("Failed to send message to client {}: {:?}", client.id, e);
				}
			} else {
				// Send confirmation to sender
				let confirm_msg = if is_deleted {
					message.clone()
				} else {
					json!({
							"type": msg_type,
							"data": {
									"id": entity_id,
									"synced": 1
							},
							"timestamp": chrono::Utc::now().timestamp_millis()
					})
				};

				if let Err(e) = client.sender.send(Message::text(&confirm_msg.to_string())) {
					log::error!(
						"Failed to send confirmation to client {}: {:?}",
						client.id,
						e
					);
				}
			}
		}

		Ok(())
	}

	pub async fn broadcast_to_tenant(
		&self,
		tenant: &str,
		message: &str,
		exclude_client: Option<String>,
	) -> Result<(), MessageError> {
		let clients = self.client_handler.get_clients_by_tenant(tenant).await;

		for client in clients {
			if let Some(ref exclude) = exclude_client {
				if client.id == *exclude {
					continue;
				}
			}

			if let Err(e) = client.sender.send(Message::text(message)) {
				log::error!("Failed to send message to client {}: {:?}", client.id, e);
			}
		}

		Ok(())
	}
}

#[derive(Debug, thiserror::Error)]
pub enum MessageError {
	#[error("Client not found: {0}")]
	ClientNotFound(String),
	#[error("Client not authenticated")]
	ClientNotAuthenticated,
	#[error("Failed to send message: {0}")]
	SendFailed(String),
}
