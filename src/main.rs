mod config;
mod controller;
mod handlers;
mod local_storage;
mod models;
mod services;

use crate::config::Config;
use crate::controller::Controller;
use dotenv::dotenv;
use std::sync::Arc;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	dotenv().ok();
	env_logger::init();

	// Load configuration
	let config = Config::from_env()?;
	let port = config.port;

	// Initialize controller
	let controller = Arc::new(Controller::new(config));

	log::info!("Starting WebSocket server on port {}...", port);

	// Define routes
	let ws_route = warp::path("ws")
		.and(warp::ws())
		.and(with_controller(controller.clone()))
		.map(|ws: warp::ws::Ws, controller: Arc<Controller>| {
			ws.on_upgrade(
				move |socket| async move { controller.handle_websocket_connection(socket).await },
			)
		});

	let health_route = warp::path::end().map(|| "User Sync WebSocket Server is running.");

	let routes = ws_route.or(health_route);

	warp::serve(routes).run(([0, 0, 0, 0], port)).await;

	Ok(())
}

fn with_controller(
	controller: Arc<Controller>,
) -> impl Filter<Extract = (Arc<Controller>,), Error = std::convert::Infallible> + Clone {
	warp::any().map(move || controller.clone())
}
