use chrono;
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{timeout, Duration};
use uuid::Uuid;
use warp::ws::{Message, WebSocket};
use warp::Filter;

#[derive(Debug)]
struct Client {
    sender: UnboundedSender<Message>,
}

type Clients = Arc<Mutex<HashMap<String, Client>>>;

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

async fn handle_authentication_request(client_id: String, clients: &Clients, data: Value) -> bool {
    let admin_key = env::var("ADMIN_API_KEY").unwrap_or_else(|_| "".to_string());

    let authenticated = data.get("apiKey") == Some(&json!(admin_key));

    if authenticated {
        let user_data = json!({
            "type": "authentication_response",
            "data": {
                "id": "1".to_string(),
                "role": 2,
                "lastEdit": chrono::Utc::now().to_rfc3339(),
                "name": "Sepp Dräxl".to_string(),
                "authenticated": 1,
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        send_message(
            client_id,
            &serde_json::to_string(&user_data).unwrap(),
            clients,
        )
        .await;
    }

    authenticated
}

async fn send_pong(client_id: String, clients: &Clients) {
    // TODO: Implement actual sync logic here
    // For now, send a simple acknowledgment
    let response = json!({
        "type": "pong",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    send_message(client_id, &response.to_string(), clients).await;
}

async fn handle_sync_request(data: &Value, client_id: String, clients: &Clients) {
    // TODO: Implement actual sync logic here
    // For now, send a simple acknowledgment
    let response = json!({
        "type": "sync_response",
        "data": {
            "status": "success",
            "message": "Sync request received"
        }
    });
    send_message(client_id, &response.to_string(), clients).await;
}

fn handle_contract_update(data: &Value) {
    // TODO: Implement contract update logic
    println!("Contract update received: {:?}", data);
}

fn handle_location_update(data: &Value) {
    // TODO: Implement location update logic
    println!("Location update received: {:?}", data);
}

fn handle_note_update(data: &Value) {
    // TODO: Implement note update logic
    println!("Note update received: {:?}", data);
}

fn handle_photo_update(data: &Value) {
    // TODO: Implement photo update logic
    println!("Photo update received: {:?}", data);
}

fn handle_sawmill_update(data: &Value) {
    // TODO: Implement sawmill update logic
    println!("Sawmill update received: {:?}", data);
}

fn handle_shipment_update(data: &Value) {
    // TODO: Implement shipment update logic
    println!("Shipment update received: {:?}", data);
}

fn handle_user_update(data: &Value) {
    // TODO: Implement user update logic
    println!("User update received: {:?}", data);
}

async fn handle_client_message(msg_type: &str, data: &Value) {
    println!("Message data: {}", data);

    match msg_type {
        "contract_update" => handle_contract_update(&data),
        "location_update" => handle_location_update(&data),
        "note_update" => handle_note_update(&data),
        "photo_update" => handle_photo_update(&data),
        "sawmill_update" => handle_sawmill_update(&data),
        "shipment_update" => handle_shipment_update(&data),
        "user_update" => handle_user_update(&data),
        _ => println!("Unknown message type: {}", msg_type),
    };
}

fn with_clients(
    clients: Clients,
) -> impl Filter<Extract = (Clients,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

async fn authenticate_client(
    client_id: String,
    ws_rx: &mut futures_util::stream::SplitStream<WebSocket>,
    clients: &Clients,
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
                            return handle_authentication_request(client_id, clients, data).await;
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
                            handle_client_message(msg_type, &data).await;
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

async fn handle_connection(ws: WebSocket, clients: Clients) {
    let (mut ws_tx, mut ws_rx) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel();
    let client_id = format!("client-{}", Uuid::new_v4());

    clients
        .lock()
        .unwrap()
        .insert(client_id.clone(), Client { sender: tx.clone() });

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
        authenticate_client(client_id.clone(), &mut ws_rx, &clients),
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

#[tokio::main]
async fn main() {
    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port: u16 = port.parse().expect("PORT must be a number");
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    println!("Starting WebSocket server on port {}...", port);

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .map(|ws: warp::ws::Ws, clients| {
            ws.on_upgrade(move |socket| handle_connection(socket, clients))
        });

    let health_route = warp::path::end().map(|| "User Sync WebSocket Server is running.");
    let routes = ws_route.or(health_route);

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
