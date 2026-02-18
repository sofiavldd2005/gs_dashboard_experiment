//! user imports
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use serde_json;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tower_http::services::ServeFile;

mod protocol;

struct AppState {
    // Broadcasts telemetry to all connected WebSocket clients
    // When telemetry comes in, we shout it out to every connected dashboard
    telemetry_tx: broadcast::Sender<protocol::Telemetry>,
    // Sends commands from Web interface to rocket : any dashboards can send commands,
    // but they all funnel down to one single "Radio Link" task that talks to the rocke
    command_tx: mpsc::Sender<String>,
}
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

// 2. WebSocket Upgrade
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut telemetry_rx = state.telemetry_tx.subscribe();

    loop {
        tokio::select! {
            // Forward Telemetry -> Browser
            res = telemetry_rx.recv() => {
                match res {
                    Ok(data) => {
                        if let Ok(msg) = serde_json::to_string(&data) {
                            if socket.send(Message::Text(msg)).await.is_err() {
                                break; // Client disconnected, exit loop
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // We are receiving data faster than we can send it.
                        // We just continue to the next loop to get the newest data.
                        continue;
                    }
                    Err(_) => break, // Channel closed
                }
            }

            // Receive Command <- Browser
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let _ = state.command_tx.send(text).await;
                    }
                    _ => break, // Socket closed or error
                }
            }
        }
    }
}
