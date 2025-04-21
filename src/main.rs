mod local_storage;

use local_storage::contract::contract_local_storage::ContractLocalStorage;
use local_storage::contract::contract_tables::ContractTable;
use local_storage::core_local_storage::CoreLocalStorage;
use local_storage::location::location_local_storage::LocationLocalStorage;
use local_storage::location::location_tables::{LocationSawmillJunctionTable, LocationTable};
use local_storage::note::note_local_storage::NoteLocalStorage;
use local_storage::note::note_tables::NoteTable;
use local_storage::photo::photo_local_storage::PhotoLocalStorage;
use local_storage::photo::photo_tables::PhotoTable;
use local_storage::sawmill::sawmill_local_storage::SawmillLocalStorage;
use local_storage::sawmill::sawmill_tables::SawmillTable;
use local_storage::shipment::shipment_local_storage::ShipmentLocalStorage;
use local_storage::shipment::shipment_tables::ShipmentTable;
use local_storage::user::user_local_storage::UserLocalStorage;
use local_storage::user::user_tables::UserTable;

use chrono;
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use rusqlite::{Result, Connection};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{Duration, timeout};
use uuid::Uuid;
use warp::Filter;
use warp::ws::{Message, WebSocket};

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::fs;
use std::path::Path;
use std::sync::Mutex as StdMutex;

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
    let dir_path = Path::new("databases");
    if !dir_path.exists() {
        fs::create_dir_all(dir_path).map_err(|_: std::io::Error| rusqlite::Error::ExecuteReturnedResults)?;
    }

    let conn = rusqlite::Connection::open(db_path)?;

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
    let db_path = format!("databases/{}.db", tenant);
    Path::new(&db_path).exists()
}

fn get_db_path(tenant: &str) -> String {
    format!("databases/{}.db", tenant)
}

fn get_db_pool(tenant: &str, db_pools: &DbPoolMap) -> Result<DbPool> {
    let mut pools = db_pools.lock().unwrap();

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
        let pool = Pool::new(manager).map_err(|_| rusqlite::Error::InvalidQuery)?; // Convert the error type

        pools.insert(tenant.to_string(), pool);
    }

    Ok(pools.get(tenant).unwrap().clone())
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

    let conn = match pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            println!("Failed to get database connection: {:?}", e);
            return false;
        }
    };

    // get user from userlocalstorage

    if user == null {
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

    // Update the client entry with the database name and user ID
    {
        let mut clients_lock = clients.lock().unwrap();
        if let Some(client) = clients_lock.get_mut(&client_id) {
            client.db_name = tenant.to_string();
            client.user_id = user_id.to_string();
        }
    }

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

fn get_client_db_path(client_id: &str, clients: &Clients) -> Option<String> {
    let clients_lock = clients.lock().unwrap();
    if let Some(client) = clients_lock.get(client_id) {
        Some(format!("databases/{}.db", client.db_name))
    } else {
        None
    }
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

    match msg_type {
        "contract_update" => handle_contract_update(&data, &db_path),
        "location_update" => handle_location_update(&data, &db_path),
        "note_update" => handle_note_update(&data, &db_path),
        "photo_update" => handle_photo_update(&data, &db_path),
        "sawmill_update" => handle_sawmill_update(&data, &db_path),
        "shipment_update" => handle_shipment_update(&data, &db_path),
        "user_update" => handle_user_update(&data, &db_path),
        _ => println!("Unknown message type: {}", msg_type),
    };
}

fn handle_contract_update(data: &Value, db_path: &str) {
    // TODO: Implement contract update logic
    println!("Contract update received for {}: {:?}", db_path, data);
}

fn handle_location_update(data: &Value, db_path: &str) {
    // TODO: Implement location update logic
    println!("Location update received for {}: {:?}", db_path, data);
}

fn handle_note_update(data: &Value, db_path: &str) {
    // TODO: Implement note update logic
    println!("Note update received for {}: {:?}", db_path, data);
}

fn handle_photo_update(data: &Value, db_path: &str) {
    // TODO: Implement photo update logic
    println!("Photo update received for {}: {:?}", db_path, data);
}

fn handle_sawmill_update(data: &Value, db_path: &str) {
    // TODO: Implement sawmill update logic
    println!("Sawmill update received for {}: {:?}", db_path, data);
}

fn handle_shipment_update(data: &Value, db_path: &str) {
    // TODO: Implement shipment update logic
    println!("Shipment update received for {}: {:?}", db_path, data);
}

fn handle_user_update(data: &Value, db_path: &str) {
    // TODO: Implement user update logic
    println!("User update received for {}: {:?}", db_path, data);
}

async fn handle_sync_request(data: &Value, client_id: String, clients: &Clients) {
    let db_path = match get_client_db_path(&client_id, clients) {
        Some(path) => path,
        None => {
            println!("No database associated with client {}", client_id);
            return;
        }
    };

    // Implement basic sync response
    println!("Sync request for database: {}", db_path);

    let response = json!({
        "type": "sync_response",
        "data": {
            "status": "success",
            "message": format!("Sync request received for database {}", db_path)
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    send_message(client_id, &response.to_string(), clients).await;
}

async fn send_message(client_id: String, msg: &str, clients: &Clients) {
    let clients_lock = clients.lock().unwrap();
    if let Some(client) = clients_lock.get(&client_id) {
        if let Err(e) = client.sender.send(Message::text(msg)) {
            println!("Error sending message to client {}: {:?}", client_id, e);
        }
    } else {
        println!("Client {} not found", client_id);
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
    let clients_lock = clients.lock().unwrap();
    for (id, client) in clients_lock.iter() {
        if id != &client_id {
            if let Err(e) = client.sender.send(Message::text(msg)) {
                println!("Error sending message to client {}: {:?}", id, e);
            }
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
}

async fn handle_connection(ws: WebSocket, clients: Clients, db_pools: DbPoolMap) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let client_id = format!("client-{}", Uuid::new_v4());

    clients.lock().unwrap().insert(
        client_id.clone(),
        Client {
            sender: tx.clone(),
            db_name: "".to_string(),
            user_id: "".to_string(),
        },
    );

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
    }

    clients.lock().unwrap().remove(&client_id);
    println!("Client disconnected: {}", client_id);
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
        fs::create_dir_all(dir_path).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?;
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
