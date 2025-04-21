mod local_storage;

use local_storage::contract::contract_local_storage::{ContractLocalStorage, Contract};
use local_storage::contract::contract_tables::ContractTable;
use local_storage::core_local_storage::CoreLocalStorage;
use local_storage::location::location_local_storage::{LocationLocalStorage, Location};
use local_storage::location::location_tables::{LocationSawmillJunctionTable, LocationTable};
use local_storage::note::note_local_storage::{NoteLocalStorage, Note};
use local_storage::note::note_tables::NoteTable;
use local_storage::photo::photo_local_storage::{PhotoLocalStorage, Photo};
use local_storage::photo::photo_tables::PhotoTable;
use local_storage::sawmill::sawmill_local_storage::{SawmillLocalStorage, Sawmill};
use local_storage::sawmill::sawmill_tables::SawmillTable;
use local_storage::shipment::shipment_local_storage::{ShipmentLocalStorage, Shipment};
use local_storage::shipment::shipment_tables::ShipmentTable;
use local_storage::user::user_local_storage::{UserLocalStorage, User};
use local_storage::user::user_tables::UserTable;

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

    conn.execute(&UserTable::create_table(), [])?;
    conn.execute(&SawmillTable::create_table(), [])?;
    conn.execute(&ContractTable::create_table(), [])?;
    conn.execute(&LocationTable::create_table(), [])?;
    conn.execute(&LocationSawmillJunctionTable::create_table(), [])?;
    conn.execute(&NoteTable::create_table(), [])?;
    conn.execute(&PhotoTable::create_table(), [])?;
    conn.execute(&ShipmentTable::create_table(), [])?;

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
            "timestamp": chrono::Utc::now().to_rfc3339()
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

    // Update client first, so we can use the db_path later
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

    // Now get the DB path after updating the client
    let db_path = match get_client_db_path(&client_id, clients) {
        Some(path) => path,
        None => {
            println!("No database associated with client {}", client_id);
            return false;
        }
    };

    // Create a CoreLocalStorage instance
    let core_storage = match CoreLocalStorage::new(&db_path) {
        Ok(storage) => Arc::new(storage),
        Err(e) => {
            println!("Failed to create core storage: {:?}", e);
            return false;
        }
    };

    // Get the user by ID
    let user_result = {
        // Create a UserLocalStorage instance
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
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        send_message(
            client_id,
            &serde_json::to_string(&rejection_response).unwrap(),
            clients,
        )
        .await;

        return false;
    }

    let user_data = user_result.unwrap().to_json();

    let response = json!({
        "type": "authentication_response",
        "data": {
            "id": user_data.get("id").unwrap_or(&json!("")).as_str(),
            "role": user_data.get("role").unwrap_or(&json!(0)),
            "lastEdit": user_data.get("lastEdit").unwrap_or(&json!(chrono::Utc::now().to_rfc3339())),
            "name": user_data.get("name").unwrap_or(&json!("Unknown User")).as_str(),
            "authenticated": 1,
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    send_message(
        client_id,
        &serde_json::to_string(&response).unwrap(),
        clients,
    )
    .await;

    true
}

async fn handle_client_message(msg_type: &str, data: &Value, client_id: &str, clients: &Clients) {
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

    // Create CoreLocalStorage instance
    let core_storage = match CoreLocalStorage::new(&db_path) {
        Ok(storage) => Arc::new(storage),
        Err(e) => {
            println!("Failed to create core storage: {:?}", e);
            return;
        }
    };

    match msg_type {
        "contract_update" => handle_contract_update(&data, core_storage.clone()),
        "location_update" => handle_location_update(&data, core_storage.clone()),
        "note_update" => handle_note_update(&data, core_storage.clone()),
        "photo_update" => handle_photo_update(&data, core_storage.clone()),
        "sawmill_update" => handle_sawmill_update(&data, core_storage.clone()),
        "shipment_update" => handle_shipment_update(&data, core_storage.clone()),
        "user_update" => handle_user_update(&data, core_storage.clone()),
        _ => println!("Unknown message type: {}", msg_type),
    }
}

fn handle_contract_update(data: &Value, core_storage: Arc<CoreLocalStorage>) {
    match ContractLocalStorage::new(core_storage) {
        Ok(contract_storage) => {
            println!("Contract update received: {:?}", data);
            match Contract::from_json(data) {
                Ok(contract) => {
                    if let Err(e) = contract_storage.save_contract(&contract) {
                        println!("Failed to save contract: {:?}", e);
                    }
                },
                Err(e) => {
                    println!("Failed to parse contract data: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create contract storage: {:?}", e);
        }
    }
}

fn handle_location_update(data: &Value, core_storage: Arc<CoreLocalStorage>) {
    match LocationLocalStorage::new(core_storage) {
        Ok(location_storage) => {
            println!("Location update received: {:?}", data);
            match Location::from_json(data) {
                Ok(location) => {
                    if let Err(e) = location_storage.save_location(&location) {
                        println!("Failed to save location: {:?}", e);
                    }
                },
                Err(e) => {
                    println!("Failed to parse location data: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create location storage: {:?}", e);
        }
    }
}

fn handle_note_update(data: &Value, core_storage: Arc<CoreLocalStorage>) {
    match NoteLocalStorage::new(core_storage) {
        Ok(note_storage) => {
            println!("Note update received: {:?}", data);
            match Note::from_json(data) {
                Ok(note) => {
                    if let Err(e) = note_storage.save_note(&note) {
                        println!("Failed to save note: {:?}", e);
                    }
                },
                Err(e) => {
                    println!("Failed to parse note data: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create note storage: {:?}", e);
        }
    }
}

fn handle_photo_update(data: &Value, core_storage: Arc<CoreLocalStorage>) {
    match PhotoLocalStorage::new(core_storage) {
        Ok(photo_storage) => {
            println!("Photo update received: {:?}", data);
            match Photo::from_json(data) {
                Ok(photo) => {
                    if let Err(e) = photo_storage.save_photo(&photo) {
                        println!("Failed to save photo: {:?}", e);
                    }
                },
                Err(e) => {
                    println!("Failed to parse photo data: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create photo storage: {:?}", e);
        }
    }
}

fn handle_sawmill_update(data: &Value, core_storage: Arc<CoreLocalStorage>) {
    match SawmillLocalStorage::new(core_storage) {
        Ok(sawmill_storage) => {
            println!("Sawmill update received: {:?}", data);
            match Sawmill::from_json(data) {
                Ok(sawmill) => {
                    if let Err(e) = sawmill_storage.save_sawmill(&sawmill) {
                        println!("Failed to save sawmill: {:?}", e);
                    }
                },
                Err(e) => {
                    println!("Failed to parse sawmill data: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create sawmill storage: {:?}", e);
        }
    }
}

fn handle_shipment_update(data: &Value, core_storage: Arc<CoreLocalStorage>) {
    match ShipmentLocalStorage::new(core_storage) {
        Ok(shipment_storage) => {
            println!("Shipment update received: {:?}", data);
            match Shipment::from_json(data) {
                Ok(shipment) => {
                    if let Err(e) = shipment_storage.save_shipment(&shipment) {
                        println!("Failed to save shipment: {:?}", e);
                    }
                },
                Err(e) => {
                    println!("Failed to parse shipment data: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create shipment storage: {:?}", e);
        }
    }
}

fn handle_user_update(data: &Value, core_storage: Arc<CoreLocalStorage>) {
    match UserLocalStorage::new(core_storage) {
        Ok(user_storage) => {
            println!("User update received: {:?}", data);
            match User::from_json(data) {
                Ok(user) => {
                    if user.name == "" {
                        println!("Empty user");
                        return
                    }
                    if let Err(e) = user_storage.save_user(&user) {
                        println!("Failed to save user: {:?}", e);
                    }
                },
                Err(e) => {
                    println!("Failed to parse user data: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to create user storage: {:?}", e);
        }
    }
}

async fn handle_sync_request(data: &Value, client_id: String, clients: &Clients) {
    let db_path = match get_client_db_path(&client_id, clients) {
        Some(path) => path,
        None => {
            println!("No database associated with client {}", client_id);
            return;
        }
    };

    // Create CoreLocalStorage instance
    match CoreLocalStorage::new(&db_path) {
        Ok(storage) => Arc::new(storage),
        Err(e) => {
            println!("Failed to create core storage: {:?}", e);

            let error_response = json!({
                "type": "sync_response",
                "data": {
                    "status": "error",
                    "message": format!("Database error: {}", e)
                },
                "timestamp": chrono::Utc::now().to_rfc3339()
            });

            send_message(client_id, &error_response.to_string(), clients).await;
            return;
        }
    };

    // Implement sync response with actual data
    println!("Sync request for database: {}", db_path);

    // TODO: Implement proper sync logic here using core_storage

    let response = json!({
        "type": "sync_response",
        "data": {
            "status": "success",
            "message": format!("Sync request processed for database {}", db_path)
            // Add actual sync data here
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    send_message(client_id, &response.to_string(), clients).await;
}

async fn send_message(client_id: String, msg: &str, clients: &Clients) {
    match clients.lock() {
        Ok(clients_lock) => {
            if let Some(client) = clients_lock.get(&client_id) {
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
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    send_message(client_id, &response.to_string(), clients).await;
}

async fn broadcast_message(client_id: String, msg: &str, clients: &Clients) {
    match clients.lock() {
        Ok(clients_lock) => {
            for (id, client) in clients_lock.iter() {
                if id != &client_id {
                    if let Err(e) = client.sender.send(Message::text(msg)) {
                        println!("Error sending message to client {}: {:?}", id, e);
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to lock clients: {:?}", e);
        }
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
                println!("Received message: {:?}", msg);

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
                println!("Received message from client {}: {:?}", client_id, msg);

                if let Some(text) = msg.to_str().ok() {
                    if let Ok(json_msg) = serde_json::from_str::<Value>(text) {
                        let msg_type = json_msg
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown");

                        let data = json_msg.get("data").cloned().unwrap_or(json!({}));

                        if msg_type == "ping" {
                            send_pong(client_id.clone(), &clients).await;
                        } else if msg_type == "sync_request" {
                            handle_sync_request(&data, client_id.clone(), &clients).await;
                        } else {
                            handle_client_message(msg_type, &data, &client_id, &clients).await;
                            broadcast_message(client_id.clone(), text, &clients).await;
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
