//! hardware.rs
//! Hardware monitoring endpoints and WebSocket handler
//!
//! Responsibilities:
//! - Provides hardware statistics endpoint `/api/hardware`
//! - Manages WebSocket connections for real-time hardware updates
//! - Handles WebSocket lifecycle and broadcast forwarding
//!
//! Dependencies:
//! - axum framework
//! - futures_util for WebSocket handling
//! - tokio async runtime
//! - crate::hardware for statistics collection
//! - crate::AppState for shared state

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    Json,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tracing::{info, warn};

use crate::hardware;
use crate::AppState;

/// Returns current hardware utilization statistics
pub async fn hardware_stats(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let stats = hardware::get_stats(state.nvml.as_ref());
    Json(serde_json::json!({
        "success": true,
        "data": {
            "cpu_percent": stats.cpu_percent,
            "ram_used_gb": stats.ram_used_gb,
            "ram_total_gb": stats.ram_total_gb,
            "vram_used_gb": stats.vram_used_gb,
            "vram_total_gb": stats.vram_total_gb,
        }
    }))
}

/// WebSocket upgrade handler for real-time monitoring broadcasts
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Internal WebSocket connection handler - forwards broadcast messages to connected clients
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = state.ws_broadcast.subscribe();

    let send_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Close(_)) => {
                info!("WebSocket closed");
                break;
            }
            Err(e) => {
                warn!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    send_task.abort();
}
