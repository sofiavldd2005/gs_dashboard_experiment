//! user imports
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tower_http::services::ServeFile;

mod protocol;
mod sockets_handler;
use sockets_handler::{ws_handler, AppState};

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    // 100 capacity buffer for broadcast
    let (telemetry_tx, _) = broadcast::channel::<protocol::Telemetry>(100);
    let (command_tx, mut command_rx) = mpsc::channel::<String>(32);

    let shared_state = Arc::new(AppState {
        telemetry_tx,
        command_tx,
    });

    // Dummy task: Listen for commands to relay to rocket
    tokio::spawn(async move {
        while let Some(cmd) = command_rx.recv().await {
            println!("RADIO LINK: Sending command '{}' to rocket", cmd);
        }
    });

    let app = Router::new()
        .nest_service("/", ServeFile::new("html/index.html")) // Standard way in Axum 0.7
        .route("/ws", get(ws_handler))
        .route("/mock-ingest", post(mock_ingest))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Ground Station Server live on port 3000");
    axum::serve(listener, app).await.unwrap();
}

///Mock for simulating  data comming from the rocket
async fn mock_ingest(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<protocol::Telemetry>,
) -> &'static str {
    // JSON data is into the broadcast pipe
    // It doesn't care who is listening; it just broadcasts the signal
    let _ = state.telemetry_tx.send(payload);
    "Data Received"
}
