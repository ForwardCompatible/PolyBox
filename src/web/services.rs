//! services.rs
//! Service lifecycle management endpoints
//!
//! Responsibilities:
//! - Provides service status endpoint `/api/services/:type/status`
//! - Provides service start endpoint `/api/services/:type/start`
//! - Provides service stop endpoint `/api/services/:type/stop`
//! - Manages orchestrator and embedding service lifecycle
//!
//! Dependencies:
//! - axum framework
//! - tokio async runtime
//! - crate::config for service configuration
//! - crate::AppState for shared state and service manager

use axum::{
    extract::{
        Path, State,
    },
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tracing::info;

use crate::config;
use crate::AppState;

/// Returns current running status for a specified service
pub async fn get_service_status(
    Path(service_type): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let running = match service_type.as_str() {
        "orchestrator" => state.service_manager.is_orchestrator_running(),
        "embedding" => state.service_manager.is_embedding_running(),
        _ => false,
    };
    Json(serde_json::json!({
        "type": service_type,
        "running": running
    }))
}

/// Starts the specified service type
pub async fn start_service(
    Path(service_type): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Starting service: {}", service_type);

    let (_model_path, result) = match service_type.as_str() {
        "orchestrator" => {
            let config = match config::load_orchestrator_config(&state.data_dir) {
                Ok(c) => c,
                Err(e) => return Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to load orchestrator config: {}", e)
                })),
            };

            let model_path = match &config.model_path {
                Some(path) if !path.is_empty() => path.clone(),
                _ => return Json(serde_json::json!({
                    "success": false,
                    "error": "No model path configured for orchestrator"
                })),
            };

            let model_path = if model_path.starts_with('/') {
                model_path
            } else {
                std::env::current_dir()
                    .map(|cwd| cwd.join(model_path.as_str()).to_string_lossy().to_string())
                    .unwrap_or(model_path)
            };

            let result = state.service_manager.start_orchestrator(
                &model_path,
                config.port as u16,
                config.ctx_size.unwrap_or(32000),
                &config,
            );
            (model_path, result)
        }
        "embedding" => {
            let config = match config::load_embedding_config(&state.data_dir) {
                Ok(c) => c,
                Err(e) => return Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to load embedding config: {}", e)
                })),
            };

            let model_path = match &config.model_path {
                Some(path) if !path.is_empty() => path.clone(),
                _ => return Json(serde_json::json!({
                    "success": false,
                    "error": "No model path configured for embedding"
                })),
            };

            let model_path = if model_path.starts_with('/') {
                model_path
            } else {
                std::env::current_dir()
                    .map(|cwd| cwd.join(model_path.as_str()).to_string_lossy().to_string())
                    .unwrap_or(model_path)
            };

            let result = state.service_manager.start_embedding(
                &model_path,
                config.port as u16,
                config.embedding_ctx_size.unwrap_or(512),
                &config,
            );
            (model_path, result)
        }
        _ => return Json(serde_json::json!({
            "success": false,
            "error": format!("Unknown service type: {}", service_type)
        })),
    };

    match result {
        Ok(_) => {
            // Broadcast updated service status to all WebSocket clients
            let status_msg = serde_json::json!({
                "type": "service_status",
                "orchestrator": state.is_orchestrator_running(),
                "embedding": state.is_embedding_running(),
            }).to_string();
            let _ = state.ws_broadcast.send(status_msg);

            Json(serde_json::json!({
                "success": true,
                "message": format!("{} started", service_type)
            }))
        }
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e
        })),
    }
}

/// Stops the specified service type
pub async fn stop_service(
    Path(service_type): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Stopping service: {}", service_type);

    let result = match service_type.as_str() {
        "orchestrator" => state.service_manager.stop_orchestrator(),
        "embedding" => state.service_manager.stop_embedding(),
        _ => return Json(serde_json::json!({
            "success": false,
            "error": format!("Unknown service type: {}", service_type)
        })),
    };

    match result {
        Ok(_) => {
            // Broadcast updated service status to all WebSocket clients
            let status_msg = serde_json::json!({
                "type": "service_status",
                "orchestrator": state.is_orchestrator_running(),
                "embedding": state.is_embedding_running(),
            }).to_string();
            let _ = state.ws_broadcast.send(status_msg);

            Json(serde_json::json!({
                "success": true,
                "message": format!("{} stopped", service_type)
            }))
        }
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e
        })),
    }
}
