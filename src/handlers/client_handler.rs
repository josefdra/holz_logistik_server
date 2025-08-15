use crate::models::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc::UnboundedSender;
use warp::ws::Message;

pub struct ClientHandler {
	clients: Arc<RwLock<HashMap<String, Client>>>,
}

impl ClientHandler {
	pub fn new() -> Self {
		Self {
			clients: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	pub async fn add_client(
		&self,
		client_id: String,
		sender: UnboundedSender<Message>,
	) -> Result<(), ClientError> {
		let mut clients = self.clients.write().await;
		clients.insert(
			client_id.clone(),
			Client {
				id: client_id.clone(),
				sender,
				db_name: String::new(),
				user_id: String::new(),
				sync_completed: false,
				authenticated: false,
			},
		);
		log::info!("Client {} connected", client_id);
		Ok(())
	}

	pub async fn remove_client(&self, client_id: &str) -> Result<(), ClientError> {
		let mut clients = self.clients.write().await;
		if clients.remove(client_id).is_some() {
			log::info!("Client {} disconnected", client_id);
			Ok(())
		} else {
			Err(ClientError::NotFound)
		}
	}

	pub async fn get_client(&self, client_id: &str) -> Option<Client> {
		let clients = self.clients.read().await;
		clients.get(client_id).cloned()
	}

	pub async fn update_client_auth(
		&self,
		client_id: &str,
		db_name: String,
		user_id: String,
	) -> Result<(), ClientError> {
		let mut clients = self.clients.write().await;
		if let Some(client) = clients.get_mut(client_id) {
			client.db_name = db_name;
			client.user_id = user_id;
			client.authenticated = true;
			log::info!(
				"Client {} authenticated for tenant {}",
				client_id,
				client.db_name
			);
			Ok(())
		} else {
			Err(ClientError::NotFound)
		}
	}

	pub async fn mark_sync_completed(&self, client_id: &str) -> Result<(), ClientError> {
		let mut clients = self.clients.write().await;
		if let Some(client) = clients.get_mut(client_id) {
			client.sync_completed = true;
			log::info!("Client {} marked as fully synced", client_id);
			Ok(())
		} else {
			Err(ClientError::NotFound)
		}
	}

	pub async fn get_authenticated_clients(&self) -> Vec<Client> {
		let clients = self.clients.read().await;
		clients
			.values()
			.filter(|c| c.authenticated)
			.cloned()
			.collect()
	}

	pub async fn get_clients_by_tenant(&self, tenant: &str) -> Vec<Client> {
		let clients = self.clients.read().await;
		clients
			.values()
			.filter(|c| c.db_name == tenant && c.authenticated)
			.cloned()
			.collect()
	}

	pub async fn client_count(&self) -> usize {
		let clients = self.clients.read().await;
		clients.len()
	}

	pub async fn authenticated_count(&self) -> usize {
		let clients = self.clients.read().await;
		clients.values().filter(|c| c.authenticated).count()
	}
}

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
	#[error("Client not found")]
	NotFound,
	#[error("Client not authenticated")]
	NotAuthenticated,
}
