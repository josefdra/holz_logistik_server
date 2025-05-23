mod local_storage;

use local_storage::contract::contract_local_storage::ContractLocalStorage;
use local_storage::core_local_storage::CoreLocalStorage;
use local_storage::location::location_local_storage::LocationLocalStorage;
use local_storage::note::note_local_storage::NoteLocalStorage;
use local_storage::photo::photo_local_storage::PhotoLocalStorage;
use local_storage::sawmill::sawmill_local_storage::SawmillLocalStorage;
use local_storage::shipment::shipment_local_storage::ShipmentLocalStorage;
use local_storage::user::user_local_storage::UserLocalStorage;

use chrono;
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, Result};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::Mutex as StdMutex;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{Duration, timeout};
use uuid::Uuid;
use warp::Filter;
use warp::ws::{Message, WebSocket};

type DbPool = Pool<SqliteConnectionManager>;
type DbPoolMap = Arc<StdMutex<HashMap<String, DbPool>>>;
type Clients = Arc<Mutex<HashMap<String, Client>>>;

#[derive(Debug)]
struct Client {
    sender: UnboundedSender<Message>,
    db_name: String,
    user_id: String,
    sync_completed: bool,
}

fn initialize_database(db_path: &str) -> Result<()> {
    let dir_path = Path::new(&db_path).parent().unwrap_or(Path::new(""));
    if !dir_path.exists() {
        fs::create_dir_all(dir_path).map_err(|e| {
            eprintln!("Failed to create directory: {:?}", e);
            rusqlite::Error::ExecuteReturnedResults
        })?;
    }

    let conn = Connection::open(db_path)?;

    conn.execute("PRAGMA foreign_keys = ON;", [])?;

    println!("Database initialized: {}", db_path);
    Ok(())
}

fn database_exists(tenant: &str) -> bool {
    let db_path = get_db_path(tenant);
    Path::new(&db_path).exists()
}

fn get_db_path(tenant: &str) -> String {
    format!("databases/{}.db", tenant)
}

fn get_db_pool(tenant: &str, db_pools: &DbPoolMap) -> Result<DbPool> {
    let mut pools = db_pools.lock().map_err(|_| {
        eprintln!("Failed to lock database pools");
        rusqlite::Error::ExecuteReturnedResults
    })?;

    if !pools.contains_key(tenant) {
        let db_path = get_db_path(tenant);
        println!(
            "Creating new connection pool for tenant {} at {}",
            tenant, db_path
        );

        if !Path::new(&db_path).exists() {
            initialize_database(&db_path)?;
        }

        let manager = SqliteConnectionManager::file(&db_path);
        let pool = Pool::new(manager).map_err(|e| {
            eprintln!("Failed to create connection pool: {:?}", e);
            rusqlite::Error::InvalidQuery
        })?;

        pools.insert(tenant.to_string(), pool);
    }

    Ok(pools.get(tenant).unwrap().clone())
}

fn get_client_db_path(client_id: &str, clients: &Clients) -> Option<String> {
    match clients.lock() {
        Ok(clients_lock) => {
            if let Some(client) = clients_lock.get(client_id) {
                Some(get_db_path(&client.db_name))
            } else {
                None
            }
        }
        Err(e) => {
            eprintln!("Failed to lock clients: {:?}", e);
            None
        }
    }
}

async fn handle_authentication_request(
    client_id: String,
    clients: &Clients,
    db_pools: &DbPoolMap,
    data: Value,
) -> bool {
    let api_key = match data.get("apiKey").and_then(|v| v.as_str()) {
        Some(key) => key,
        None => {
            println!("No API key provided");
            return false;
        }
    };

    let parts: Vec<&str> = api_key.splitn(2, '-').collect();
    if parts.len() != 2 {
        println!("Invalid API key format");
        return false;
    }

    let tenant = parts[0];
    let user_id = parts[1];

    println!(
        "Authentication attempt for tenant: {}, user_id: {}",
        tenant, user_id
    );

    if !database_exists(tenant) {
        println!("Database for tenant {} does not exist", tenant);

        let rejection_response = json!({
            "type": "authentication_response",
            "data": {
                "authenticated": 0,
                "error": "Invalid tenant"
            },
            "timestamp": chrono::Utc::now().timestamp_millis()
        });

        send_message(
            client_id,
            &serde_json::to_string(&rejection_response).unwrap(),
            clients,
        )
        .await;

        return false;
    }

    let pool = match get_db_pool(tenant, db_pools) {
        Ok(pool) => pool,
        Err(e) => {
            println!("Failed to get database pool: {:?}", e);
            return false;
        }
    };

    match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            println!("Failed to get database connection: {:?}", e);
            return false;
        }
    };

    {
        match clients.lock() {
            Ok(mut clients_lock) => {
                if let Some(client) = clients_lock.get_mut(&client_id) {
                    client.db_name = tenant.to_string();
                    client.user_id = user_id.to_string();
                } else {
                    println!("Client {} not found", client_id);
                    return false;
                }
            }
            Err(e) => {
                println!("Failed to lock clients: {:?}", e);
                return false;
            }
        }
    }

    let db_path = match get_client_db_path(&client_id, clients) {
        Some(path) => path,
        None => {
            println!("No database associated with client {}", client_id);
            return false;
        }
    };

    let core_storage = match CoreLocalStorage::new(&db_path) {
        Ok(storage) => Arc::new(storage),
        Err(e) => {
            println!("Failed to create core storage: {:?}", e);
            return false;
        }
    };

    let user_result = {
        let user_storage = match UserLocalStorage::new(core_storage) {
            Ok(storage) => storage,
            Err(e) => {
                println!("Failed to create user storage: {:?}", e);
                return false;
            }
        };

        match user_storage.get_user_by_id(user_id) {
            Ok(user_opt) => user_opt,
            Err(e) => {
                println!("Failed to get user: {:?}", e);
                return false;
            }
        }
    };

    if user_result.is_none() {
        println!(
            "User {} not found in database for tenant {}",
            user_id, tenant
        );

        let rejection_response = json!({
            "type": "authentication_response",
            "data": {
                "authenticated": 0,
                "error": "User not found"
            },
            "timestamp": chrono::Utc::now().timestamp_millis()
        });

        send_message(
            client_id,
            &serde_json::to_string(&rejection_response).unwrap(),
            clients,
        )
        .await;

        return false;
    }

    let user_data = user_result.unwrap();

    let response = json!({
        "type": "authentication_response",
        "data": {
            "id": user_data.get("id").unwrap_or(&json!("")).as_str(),
            "role": user_data.get("role").unwrap_or(&json!(0)),
            "lastEdit": user_data.get("lastEdit").unwrap_or(&json!(chrono::Utc::now().timestamp_millis())),
            "name": user_data.get("name").unwrap_or(&json!("Unknown User")).as_str(),
            "authenticated": 1,
        },
        "timestamp": chrono::Utc::now().timestamp_millis()
    });

    send_message(
        client_id,
        &serde_json::to_string(&response).unwrap(),
        clients,
    )
    .await;

    true
}

async fn handle_client_message(msg_type: &str, msg: &str, data: &Value, client_id: &str, clients: &Clients,) {
    let db_path = match get_client_db_path(client_id, clients) {
        Some(path) => path,
        None => {
            println!("No database associated with client {}", client_id);
            return;
        }
    };

    println!(
        "Processing message of type {} for database {}",
        msg_type, db_path
    );

    let core_storage = match CoreLocalStorage::new(&db_path) {
        Ok(storage) => Arc::new(storage),
        Err(e) => {
            println!("Failed to create core storage: {:?}", e);
            return;
        }
    };

    match msg_type {
        "contract_update" => {
            let update_happened = handle_contract_update(&data, core_storage.clone());
            if update_happened {
                broadcast_message(client_id.to_string(), msg, &clients).await;
            }
        },
        "location_update" => {
            let update_happened = handle_location_update(&data, core_storage.clone());
            if update_happened {
                broadcast_message(client_id.to_string(), msg, &clients).await;
            }
        },
        "note_update" => {
            let update_happened = handle_note_update(&data, core_storage.clone());
            if update_happened {
                broadcast_message(client_id.to_string(), msg, &clients).await;
            }
        },
        "photo_update" => {
            let update_happened = handle_photo_update(&data, core_storage.clone());
            if update_happened {
                broadcast_message(client_id.to_string(), msg, &clients).await;
            }
        },
        "sawmill_update" => {
            let update_happened = handle_sawmill_update(&data, core_storage.clone());
            if update_happened {
                broadcast_message(client_id.to_string(), msg, &clients).await;
            }
        },
        "shipment_update" => {
            let update_happened = handle_shipment_update(&data, core_storage.clone());
            if update_happened {
                broadcast_message(client_id.to_string(), msg, &clients).await;
            }
        },
        "user_update" => {
            let update_happened = handle_user_update(&data, core_storage.clone());
            if update_happened {
                broadcast_message(client_id.to_string(), msg, &clients).await;
            }
        },
        _ => println!("Unknown message type: {}", msg_type),
    }
}

fn handle_contract_update(data: &Value, core_storage: Arc<CoreLocalStorage>) -> bool {
    match ContractLocalStorage::new(core_storage.clone()) {
        Ok(contract_storage) => {
            println!("Contract update received: {:?}", data);

            let is_deleted = data.get("deleted").and_then(|v| v.as_i64()).unwrap_or(0) == 1;

            if !is_deleted {
                match contract_storage.save_contract(data) {
                    Ok(success) => {
                        success
                    },
                    Err(e) => {
                        println!("Failed to save contract: {:?}", e);
                        false
                    }
                }
            } else {
                if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                    match core_storage.mark_as_deleted("contracts", id) {
                        Ok(_) => true,
                        Err(e) => {
                            println!("Failed to mark contract as deleted: {:?}", e);
                            false
                        }
                    }
                } else {
                    println!("Failed to mark contract as deleted: Missing ID");
                    false
                }
            }
        }
        Err(e) => {
            println!("Failed to create contract storage: {:?}", e);
            false
        }
    }
}

fn handle_location_update(data: &Value, core_storage: Arc<CoreLocalStorage>) -> bool {
    match LocationLocalStorage::new(core_storage.clone()) {
        Ok(location_storage) => {
            println!("Location update received: {:?}", data);

            let is_deleted = data.get("deleted").and_then(|v| v.as_i64()).unwrap_or(0) == 1;

            if !is_deleted {
                match location_storage.save_location(data) {
                    Ok(success) => success,
                    Err(e) => {
                        println!("Failed to save location: {:?}", e);
                        false
                    }
                }
            } else {
                if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                    match core_storage.mark_as_deleted("locations", id) {
                        Ok(_) => true,
                        Err(e) => {
                            println!("Failed to mark location as deleted: {:?}", e);
                            false
                        }
                    }
                } else {
                    println!("Failed to mark location as deleted: Missing ID");
                    false
                }
            }
        }
        Err(e) => {
            println!("Failed to create location storage: {:?}", e);
            false
        }
    }
}

fn handle_note_update(data: &Value, core_storage: Arc<CoreLocalStorage>) -> bool {
    match NoteLocalStorage::new(core_storage.clone()) {
        Ok(note_storage) => {
            println!("Note update received: {:?}", data);

            let is_deleted = data.get("deleted").and_then(|v| v.as_i64()).unwrap_or(0) == 1;

            if !is_deleted {
                match note_storage.save_note(data) {
                    Ok(success) => success,
                    Err(e) => {
                        println!("Failed to save note: {:?}", e);
                        false
                    }
                }
            } else {
                if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                    match core_storage.mark_as_deleted("notes", id) {
                        Ok(_) => true,
                        Err(e) => {
                            println!("Failed to mark note as deleted: {:?}", e);
                            false
                        }
                    }
                } else {
                    println!("Failed to mark note as deleted: Missing ID");
                    false
                }
            }
        }
        Err(e) => {
            println!("Failed to create note storage: {:?}", e);
            false
        }
    }
}

fn handle_photo_update(data: &Value, core_storage: Arc<CoreLocalStorage>) -> bool {
    match PhotoLocalStorage::new(core_storage.clone()) {
        Ok(photo_storage) => {
            println!("Photo update received");

            let is_deleted = data.get("deleted").and_then(|v| v.as_i64()).unwrap_or(0) == 1;

            if !is_deleted {
                match photo_storage.save_photo(data) {
                    Ok(success) => success,
                    Err(e) => {
                        println!("Failed to save photo: {:?}", e);
                        false
                    }
                }
            } else {
                if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                    match core_storage.mark_as_deleted("photos", id) {
                        Ok(_) => true,
                        Err(e) => {
                            println!("Failed to mark photo as deleted: {:?}", e);
                            false
                        }
                    }
                } else {
                    println!("Failed to mark photo as deleted: Missing ID");
                    false
                }
            }
        }
        Err(e) => {
            println!("Failed to create photo storage: {:?}", e);
            false
        }
    }
}

fn handle_sawmill_update(data: &Value, core_storage: Arc<CoreLocalStorage>) -> bool {
    match SawmillLocalStorage::new(core_storage.clone()) {
        Ok(sawmill_storage) => {
            println!("Sawmill update received: {:?}", data);

            let is_deleted = data.get("deleted").and_then(|v| v.as_i64()).unwrap_or(0) == 1;

            if !is_deleted {
                match sawmill_storage.save_sawmill(data) {
                    Ok(success) => success,
                    Err(e) => {
                        println!("Failed to save sawmill: {:?}", e);
                        false
                    }
                }
            } else {
                if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                    match core_storage.mark_as_deleted("sawmills", id) {
                        Ok(_) => true,
                        Err(e) => {
                            println!("Failed to mark sawmill as deleted: {:?}", e);
                            false
                        }
                    }
                } else {
                    println!("Failed to mark sawmill as deleted: Missing ID");
                    false
                }
            }
        }
        Err(e) => {
            println!("Failed to create sawmill storage: {:?}", e);
            false
        }
    }
}

fn handle_shipment_update(data: &Value, core_storage: Arc<CoreLocalStorage>) -> bool {
    match ShipmentLocalStorage::new(core_storage.clone()) {
        Ok(shipment_storage) => {
            println!("Shipment update received: {:?}", data);

            let is_deleted = data.get("deleted").and_then(|v| v.as_i64()).unwrap_or(0) == 1;

            if !is_deleted {
                match shipment_storage.save_shipment(data) {
                    Ok(success) => success,
                    Err(e) => {
                        println!("Failed to save shipment: {:?}", e);
                        false
                    }
                }
            } else {
                if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                    match core_storage.mark_as_deleted("shipments", id) {
                        Ok(_) => true,
                        Err(e) => {
                            println!("Failed to mark shipment as deleted: {:?}", e);
                            false
                        }
                    }
                } else {
                    println!("Failed to mark shipment as deleted: Missing ID");
                    false
                }
            }
        }
        Err(e) => {
            println!("Failed to create shipment storage: {:?}", e);
            false
        }
    }
}

fn handle_user_update(data: &Value, core_storage: Arc<CoreLocalStorage>) -> bool {
    match UserLocalStorage::new(core_storage.clone()) {
        Ok(user_storage) => {
            println!("User update received: {:?}", data);

            let is_deleted = data.get("deleted").and_then(|v| v.as_i64()).unwrap_or(0) == 1;

            if !is_deleted {
                if let Some(name) = data.get("name").and_then(|n| n.as_str()) {
                    if name.is_empty() {
                        println!("Empty user name");
                        return false;
                    }
                }

                match user_storage.save_user(data) {
                    Ok(success) => success,
                    Err(e) => {
                        println!("Failed to save user: {:?}", e);
                        false
                    }
                }
            } else {
                if let Some(id) = data.get("id").and_then(|v| v.as_str()) {
                    match core_storage.mark_as_deleted("users", id) {
                        Ok(_) => true,
                        Err(e) => {
                            println!("Failed to mark user as deleted: {:?}", e);
                            false
                        }
                    }
                } else {
                    println!("Failed to mark user as deleted: Missing ID");
                    false
                }
            }
        }
        Err(e) => {
            println!("Failed to create user storage: {:?}", e);
            false
        }
    }
}

async fn send_user_data(
    last_sync: i64,
    client_id: String,
    core_storage: Arc<CoreLocalStorage>,
    clients: &Clients,
) -> i64 {
    let user_storage = match UserLocalStorage::new(core_storage) {
        Ok(storage) => storage,
        Err(e) => {
            println!("Failed to create user storage: {:?}", e);
            return last_sync;
        }
    };

    let mut date = last_sync;
    let mut should_continue = true;

    while should_continue {
        let users = match user_storage.get_user_updates_by_date(date) {
            Ok(users) => users,
            Err(e) => {
                println!("Failed to get user updates: {:?}", e);
                return last_sync;
            }
        };

        if users.is_empty() {
            should_continue = false;
        } else {
            for user in &users {
                let response = serde_json::json!({
                    "type": "user_update",
                    "data": user,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                });

                send_message(client_id.clone(), &response.to_string(), clients).await;
                if let Some(newest_date) = user["arrivalAtServer"].as_i64() {
                    if date <= newest_date {
                        date = newest_date + 1;
                    }
                }
            }
        }
    }

    let completion_message = serde_json::json!({
        "type": "user_update",
        "data": serde_json::json!({
            "newSyncDate": date,
        }),
        "timestamp": chrono::Utc::now().timestamp_millis()
    });

    send_message(client_id.clone(), &completion_message.to_string(), clients).await;

    date
}

async fn send_sawmill_data(
    last_sync: i64,
    client_id: String,
    core_storage: Arc<CoreLocalStorage>,
    clients: &Clients,
) -> i64 {
    let sawmill_storage = match SawmillLocalStorage::new(core_storage) {
        Ok(storage) => storage,
        Err(e) => {
            println!("Failed to create sawmill storage: {:?}", e);
            return last_sync;
        }
    };

    let mut date = last_sync;
    let mut should_continue = true;

    while should_continue {
        let sawmills = match sawmill_storage.get_sawmill_updates_by_date(date) {
            Ok(sawmills) => sawmills,
            Err(e) => {
                println!("Failed to get sawmill updates: {:?}", e);
                return last_sync;
            }
        };

        if sawmills.is_empty() {
            should_continue = false;
        } else {
            for sawmill in &sawmills {
                let response = serde_json::json!({
                    "type": "sawmill_update",
                    "data": sawmill,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                });

                send_message(client_id.clone(), &response.to_string(), clients).await;
                if let Some(newest_date) = sawmill["arrivalAtServer"].as_i64() {
                    if date <= newest_date {
                        date = newest_date + 1;
                    }
                }
            }
        }
    }

    let completion_message = serde_json::json!({
        "type": "sawmill_update",
        "data": serde_json::json!({
            "newSyncDate": date,
        }),
        "timestamp": chrono::Utc::now().timestamp_millis()
    });

    send_message(client_id.clone(), &completion_message.to_string(), clients).await;

    date
}

async fn send_contract_data(
    last_sync: i64,
    client_id: String,
    core_storage: Arc<CoreLocalStorage>,
    clients: &Clients,
) -> i64 {
    let contract_storage = match ContractLocalStorage::new(core_storage) {
        Ok(storage) => storage,
        Err(e) => {
            println!("Failed to create contract storage: {:?}", e);
            return last_sync;
        }
    };

    let mut date = last_sync;
    let mut should_continue = true;

    while should_continue {
        let contracts = match contract_storage.get_contract_updates_by_date(date) {
            Ok(contracts) => contracts,
            Err(e) => {
                println!("Failed to get contract updates: {:?}", e);
                return last_sync;
            }
        };

        if contracts.is_empty() {
            should_continue = false;
        } else {
            for contract in &contracts {
                let response = serde_json::json!({
                    "type": "contract_update",
                    "data": contract,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                });

                send_message(client_id.clone(), &response.to_string(), clients).await;
                if let Some(newest_date) = contract["arrivalAtServer"].as_i64() {
                    if date <= newest_date {
                        date = newest_date + 1;
                    }
                }
            }
        }
    }

    let completion_message = serde_json::json!({
        "type": "contract_update",
        "data": serde_json::json!({
            "newSyncDate": date,
        }),
        "timestamp": chrono::Utc::now().timestamp_millis()
    });

    send_message(client_id.clone(), &completion_message.to_string(), clients).await;

    date
}

async fn send_photo_data(
    last_sync: i64,
    client_id: String,
    core_storage: Arc<CoreLocalStorage>,
    clients: &Clients,
) -> i64 {
    let photo_storage = match PhotoLocalStorage::new(core_storage) {
        Ok(storage) => storage,
        Err(e) => {
            println!("Failed to create photo storage: {:?}", e);
            return last_sync;
        }
    };

    let mut date = last_sync;
    let mut should_continue = true;

    while should_continue {
        let photos = match photo_storage.get_photo_updates_by_date(date) {
            Ok(photos) => photos,
            Err(e) => {
                println!("Failed to get photo updates: {:?}", e);
                return last_sync;
            }
        };

        if photos.is_empty() {
            should_continue = false;
        } else {
            for photo in &photos {
                let response = serde_json::json!({
                    "type": "photo_update",
                    "data": photo,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                });

                send_message(client_id.clone(), &response.to_string(), clients).await;
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                if let Some(newest_date) = photo["arrivalAtServer"].as_i64() {
                    if date <= newest_date {
                        date = newest_date + 1;
                    }
                }
            }
        }
    }

    let completion_message = serde_json::json!({
        "type": "photo_update",
        "data": serde_json::json!({
            "newSyncDate": date,
        }),
        "timestamp": chrono::Utc::now().timestamp_millis()
    });

    send_message(client_id.clone(), &completion_message.to_string(), clients).await;

    date
}

async fn send_note_data(
    last_sync: i64,
    client_id: String,
    core_storage: Arc<CoreLocalStorage>,
    clients: &Clients,
) -> i64 {
    let note_storage = match NoteLocalStorage::new(core_storage) {
        Ok(storage) => storage,
        Err(e) => {
            println!("Failed to create note storage: {:?}", e);
            return last_sync;
        }
    };

    let mut date = last_sync;
    let mut should_continue = true;

    while should_continue {
        let notes = match note_storage.get_note_updates_by_date(date) {
            Ok(notes) => notes,
            Err(e) => {
                println!("Failed to get note updates: {:?}", e);
                return last_sync;
            }
        };

        if notes.is_empty() {
            should_continue = false;
        } else {
            for note in &notes {
                let response = serde_json::json!({
                    "type": "note_update",
                    "data": note,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                });

                send_message(client_id.clone(), &response.to_string(), clients).await;
                if let Some(newest_date) = note["arrivalAtServer"].as_i64() {
                    if date <= newest_date {
                        date = newest_date + 1;
                    }
                }
            }
        }
    }

    let completion_message = serde_json::json!({
        "type": "note_update",
        "data": serde_json::json!({
            "newSyncDate": date,
        }),
        "timestamp": chrono::Utc::now().timestamp_millis()
    });

    send_message(client_id.clone(), &completion_message.to_string(), clients).await;

    date
}

async fn send_location_data(
    last_sync: i64,
    client_id: String,
    core_storage: Arc<CoreLocalStorage>,
    clients: &Clients,
) -> i64 {
    let location_storage = match LocationLocalStorage::new(core_storage) {
        Ok(storage) => storage,
        Err(e) => {
            println!("Failed to create location storage: {:?}", e);
            return last_sync;
        }
    };

    let mut date = last_sync;
    let mut should_continue = true;

    while should_continue {
        let locations = match location_storage.get_location_updates_by_date(date) {
            Ok(locations) => locations,
            Err(e) => {
                println!("Failed to get location updates: {:?}", e);
                return last_sync;
            }
        };

        if locations.is_empty() {
            should_continue = false;
        } else {
            for location in &locations {
                let response = serde_json::json!({
                    "type": "location_update",
                    "data": location,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                });

                send_message(client_id.clone(), &response.to_string(), clients).await;

                println!("{}", &response.to_string());

                if let Some(newest_date) = location["arrivalAtServer"].as_i64() {
                    if date <= newest_date {
                        date = newest_date + 1;
                    }
                }
            }
        }
    }

    let completion_message = serde_json::json!({
        "type": "location_update",
        "data": serde_json::json!({
            "newSyncDate": date,
        }),
        "timestamp": chrono::Utc::now().timestamp_millis()
    });

    send_message(client_id.clone(), &completion_message.to_string(), clients).await;

    date
}

async fn send_shipment_data(
    last_sync: i64,
    client_id: String,
    core_storage: Arc<CoreLocalStorage>,
    clients: &Clients,
) -> i64 {
    let shipment_storage = match ShipmentLocalStorage::new(core_storage) {
        Ok(storage) => storage,
        Err(e) => {
            println!("Failed to create shipment storage: {:?}", e);
            return last_sync;
        }
    };

    let mut date = last_sync;
    let mut should_continue = true;

    while should_continue {
        let shipments = match shipment_storage.get_shipments_by_date(date) {
            Ok(shipments) => shipments,
            Err(e) => {
                println!("Failed to get shipment updates: {:?}", e);
                return last_sync;
            }
        };

        if shipments.is_empty() {
            should_continue = false;
        } else {
            for shipment in &shipments {
                let response = serde_json::json!({
                    "type": "shipment_update",
                    "data": shipment,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                });

                send_message(client_id.clone(), &response.to_string(), clients).await;
                if let Some(newest_date) = shipment["arrivalAtServer"].as_i64() {
                    if date <= newest_date {
                        date = newest_date + 1;
                    }
                }
            }
        }
    }

    let completion_message = serde_json::json!({
        "type": "shipment_update",
        "data": serde_json::json!({
            "newSyncDate": date,
        }),
        "timestamp": chrono::Utc::now().timestamp_millis()
    });

    send_message(client_id.clone(), &completion_message.to_string(), clients).await;

    date
}

async fn handle_sync_request(data: &Value, client_id: String, clients: &Clients) -> bool {
    let db_path = match get_client_db_path(&client_id, clients) {
        Some(path) => path,
        None => {
            println!("No database associated with client {}", client_id);
            return false;
        }
    };

    let core_storage = match CoreLocalStorage::new(&db_path) {
        Ok(storage) => Arc::new(storage),
        Err(e) => {
            println!("Failed to create core storage: {:?}", e);
            return false;
        }
    };

    let last_user_sync = data
        .get("user_update")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let last_sawmill_sync = data
        .get("sawmill_update")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let last_contract_sync = data
        .get("contract_update")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let last_note_sync = data
        .get("note_update")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let last_location_sync = data
        .get("location_update")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let last_shipment_sync = data
        .get("shipment_update")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let last_photo_sync = data
        .get("photo_update")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    send_user_data(
        last_user_sync,
        client_id.clone(),
        core_storage.clone(),
        clients,
    )
    .await;

    send_sawmill_data(
        last_sawmill_sync,
        client_id.clone(),
        core_storage.clone(),
        clients,
    )
    .await;

    send_contract_data(
        last_contract_sync,
        client_id.clone(),
        core_storage.clone(),
        clients,
    )
    .await;

    send_location_data(
        last_location_sync,
        client_id.clone(),
        core_storage.clone(),
        clients,
    )
    .await;

    send_shipment_data(
        last_shipment_sync,
        client_id.clone(),
        core_storage.clone(),
        clients,
    )
    .await;

    send_note_data(
        last_note_sync,
        client_id.clone(),
        core_storage.clone(),
        clients,
    )
    .await;

    send_photo_data(
        last_photo_sync,
        client_id.clone(),
        core_storage.clone(),
        clients,
    )
    .await;

    true
}

async fn send_message(client_id: String, msg: &str, clients: &Clients) {
    match clients.lock() {
        Ok(clients_lock) => {
            if let Some(client) = clients_lock.get(&client_id) {
                println!("{}", msg.to_string());
                if let Err(e) = client.sender.send(Message::text(msg)) {
                    println!("Error sending message to client {}: {:?}", client_id, e);
                }
            } else {
                println!("Client {} not found", client_id);
            }
        }
        Err(e) => {
            println!("Failed to lock clients: {:?}", e);
        }
    }
}

async fn send_pong(client_id: String, clients: &Clients) {
    let response = json!({
        "type": "pong",
        "timestamp": chrono::Utc::now().timestamp_millis()
    });
    send_message(client_id, &response.to_string(), clients).await;
}

async fn broadcast_message(client_id: String, msg: &str, clients: &Clients) {
    if let Ok(json_msg) = serde_json::from_str::<Value>(msg) {
        match clients.lock() {
            Ok(clients_lock) => {
                for (id, client) in clients_lock.iter() {
                    if !client.db_name.is_empty() {
                        if id != &client_id {
                            if let Err(e) = client.sender.send(Message::text(msg)) {
                                println!("Error sending message to client {}: {:?}", id, e);
                            }
                        } else {
                            let is_deleted = json_msg
                                .get("data")
                                .and_then(|data| data.get("deleted"))
                                .and_then(|deleted| deleted.as_i64())
                                .unwrap_or(0)
                                == 1;

                            if is_deleted {
                                if let Err(e) = client.sender.send(Message::text(msg)) {
                                    println!(
                                        "Error sending delete confirmation to client {}: {:?}",
                                        id, e
                                    );
                                }
                            } else {
                                let msg_type = json_msg
                                    .get("type")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("unknown");

                                let entity_id = json_msg
                                    .get("data")
                                    .and_then(|data| data.get("id"))
                                    .cloned()
                                    .unwrap_or(json!("unknown"));

                                let confirm_msg = json!({
                                    "type": msg_type,
                                    "data": {
                                        "id": entity_id,
                                        "synced": 1
                                    },
                                    "timestamp": chrono::Utc::now().timestamp_millis()
                                });

                                if let Err(e) =
                                    client.sender.send(Message::text(&confirm_msg.to_string()))
                                {
                                    println!(
                                        "Error sending sync confirmation to client {}: {:?}",
                                        id, e
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("Failed to lock clients: {:?}", e);
            }
        }
    } else {
        println!("Failed to parse message as JSON: {}", msg);
    }
}

async fn authenticate_client(
    client_id: String,
    ws_rx: &mut futures_util::stream::SplitStream<WebSocket>,
    clients: &Clients,
    db_pools: &DbPoolMap,
) -> bool {
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let Some(text) = msg.to_str().ok() {
                    if let Ok(json_msg) = serde_json::from_str::<Value>(text) {
                        if json_msg.get("type").and_then(|v| v.as_str())
                            == Some("authentication_request")
                        {
                            let data = json_msg.get("data").cloned().unwrap_or(json!({}));
                            return handle_authentication_request(
                                client_id, clients, db_pools, data,
                            )
                            .await;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket error during authentication: {:?}", e);
                return false;
            }
        }
    }
    false
}

async fn handle_authenticated_client(
    client_id: String,
    mut ws_rx: futures_util::stream::SplitStream<WebSocket>,
    clients: Clients,
) {
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if let Some(text) = msg.to_str().ok() {
                    if let Ok(json_msg) = serde_json::from_str::<Value>(text) {
                        let msg_type = json_msg
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");

                        println!("Received message from client {}: {}", client_id, msg_type);

                        if msg_type != "photo_update" {
                            println!("{}", text);
                        } else {
                            println!("photo_update");
                        }

                        let data = json_msg.get("data").cloned().unwrap_or(json!({}));

                        if msg_type == "ping" {
                            send_pong(client_id.clone(), &clients).await;
                        } else if msg_type == "sync_request" {
                            if handle_sync_request(&data, client_id.clone(), &clients).await {
                                println!("Sync to client complete");
                                let should_send_message = {
                                    match clients.lock() {
                                        Ok(mut clients_lock) => {
                                            if let Some(client) = clients_lock.get_mut(&client_id) {
                                                client.sync_completed = true;
                                                println!(
                                                    "Client {} marked as fully synced",
                                                    client_id
                                                );
                                                true
                                            } else {
                                                false
                                            }
                                        }
                                        Err(e) => {
                                            println!(
                                                "Failed to lock clients to update sync status: {:?}",
                                                e
                                            );
                                            false
                                        }
                                    }
                                };

                                if should_send_message {
                                    let response = serde_json::json!({
                                        "type": "sync_from_server_complete",
                                        "timestamp": chrono::Utc::now().timestamp_millis()
                                    });

                                    send_message(
                                        client_id.clone(),
                                        &response.to_string(),
                                        &clients,
                                    )
                                    .await;
                                }
                            }
                        } else if msg_type == "sync_complete" {
                            let response = serde_json::json!({
                                "type": "sync_to_server_complete",
                                "timestamp": chrono::Utc::now().timestamp_millis()
                            });

                            send_message(client_id.clone(), &response.to_string(), &clients).await;
                        } else {
                            handle_client_message(msg_type, &text, &data, &client_id, &clients).await;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {:?}", e);
                break;
            }
        }
    }

    match clients.lock() {
        Ok(mut clients_lock) => {
            clients_lock.remove(&client_id);
            println!("Client disconnected: {}", client_id);
        }
        Err(e) => {
            eprintln!("Failed to lock clients for cleanup: {:?}", e);
        }
    }
}

async fn handle_connection(ws: WebSocket, clients: Clients, db_pools: DbPoolMap) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let client_id = format!("client-{}", Uuid::new_v4());

    match clients.lock() {
        Ok(mut clients_lock) => {
            clients_lock.insert(
                client_id.clone(),
                Client {
                    sender: tx.clone(),
                    db_name: "".to_string(),
                    user_id: "".to_string(),
                    sync_completed: false,
                },
            );
        }
        Err(e) => {
            eprintln!("Failed to lock clients for insertion: {:?}", e);
            return;
        }
    }

    tokio::task::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Err(e) = ws_tx.send(message).await {
                eprintln!("Error sending WebSocket message: {:?}", e);
                break;
            }
        }
    });

    let authenticated = match timeout(
        Duration::from_secs(10),
        authenticate_client(client_id.clone(), &mut ws_rx, &clients, &db_pools),
    )
    .await
    {
        Ok(result) => result,
        Err(_) => {
            eprintln!("Authentication timeout for client {}", client_id);
            false
        }
    };

    if authenticated {
        handle_authenticated_client(client_id.clone(), ws_rx, clients.clone()).await;
    } else {
        eprintln!("Authentication failed for client {}", client_id);

        match clients.lock() {
            Ok(mut clients_lock) => {
                clients_lock.remove(&client_id);
                println!("Removed unauthenticated client: {}", client_id);
            }
            Err(e) => {
                eprintln!("Failed to lock clients for cleanup: {:?}", e);
            }
        }
    }
}

fn with_clients(
    clients: Clients,
) -> impl Filter<Extract = (Clients,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

fn with_db_pools(
    db_pools: DbPoolMap,
) -> impl Filter<Extract = (DbPoolMap,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db_pools.clone())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let dir_path = Path::new("databases");
    if !dir_path.exists() {
        fs::create_dir_all(dir_path).map_err(|e| {
            eprintln!("Failed to create databases directory: {:?}", e);
            rusqlite::Error::ExecuteReturnedResults
        })?;
    }

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port: u16 = port.parse().expect("PORT must be a number");

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let db_pools: DbPoolMap = Arc::new(StdMutex::new(HashMap::new()));

    println!("Starting WebSocket server on port {}...", port);

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and(with_db_pools(db_pools.clone()))
        .map(|ws: warp::ws::Ws, clients, db_pools| {
            ws.on_upgrade(move |socket| handle_connection(socket, clients, db_pools))
        });

    let health_route = warp::path::end().map(|| "User Sync WebSocket Server is running.");
    let routes = ws_route.or(health_route);

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;

    Ok(())
}
