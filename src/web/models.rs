//! models.rs
//! Model configuration and discovery HTTP endpoints
//!
//! Responsibilities:
//! - Provides `/api/model-configs/:type` GET endpoint to retrieve model configuration
//! - Provides `/api/model-configs/:type` PUT endpoint to update model configuration
//! - Provides `/api/models/:type` GET endpoint to discover available models
//! - Handles both orchestrator and embedding model types
//!
//! Dependencies:
//! - axum framework
//! - crate::config for model configuration
//! - crate::db for model discovery
//! - crate::AppState for shared state

use axum::{
    extract::{
        Path, State, Json,
    },
    response::IntoResponse,
};
use std::sync::Arc;

use crate::config;
use crate::db;
use crate::AppState;

/// Retrieves configuration for a specific model type
pub async fn get_model_config(
    Path(model_type): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match model_type.as_str() {
        "orchestrator" => {
            match config::load_orchestrator_config(&state.data_dir) {
                Ok(config) => Json(serde_json::json!({
                    "success": true,
                    "data": config
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to load orchestrator config: {}", e)
                })),
            }
        }
        "embedding" => {
            match config::load_embedding_config(&state.data_dir) {
                Ok(config) => Json(serde_json::json!({
                    "success": true,
                    "data": config
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to load embedding config: {}", e)
                })),
            }
        }
        _ => Json(serde_json::json!({
            "success": false,
            "error": format!("Unknown model type: {}", model_type)
        })),
    }
}

/// Discovers available models for a given model type
pub async fn get_models(
    Path(model_type): Path<String>,
) -> impl IntoResponse {
    match db::core::discover_models(&model_type) {
        Ok(models) => Json(serde_json::json!({
            "success": true,
            "data": models
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e
        })),
    }
}

/// Updates configuration for a specific model type
pub async fn update_model_config_handler(
    Path(model_type): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(update): Json<db::core::ModelConfigUpdate>,
) -> impl IntoResponse {
    match model_type.as_str() {
        "orchestrator" => {
            let mut config = match config::load_orchestrator_config(&state.data_dir) {
                Ok(c) => c,
                Err(e) => return Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to load orchestrator config: {}", e)
                })),
            };

            // Merge update — update wins if Some
            if update.backend_type.is_some() { config.backend_type = update.backend_type; }
            if update.model_path.is_some() { config.model_path = update.model_path.clone(); }
            if update.api_base_url.is_some() { config.api_base_url = update.api_base_url.clone(); }
            if update.model_name.is_some() { config.model_name = update.model_name.clone(); }
            if let Some(port) = update.port { config.port = port; }
            if let Some(auto_start) = update.auto_start { config.auto_start = auto_start; }
            if update.ctx_size.is_some() { config.ctx_size = update.ctx_size; }
            if update.output_token_reserve.is_some() { config.output_token_reserve = update.output_token_reserve; }
            if update.n_gpu_layers.is_some() { config.n_gpu_layers = update.n_gpu_layers; }
            if update.temperature.is_some() { config.temperature = update.temperature; }
            if update.repeat_penalty.is_some() { config.repeat_penalty = update.repeat_penalty; }
            if update.cache_type_k.is_some() { config.cache_type_k = update.cache_type_k.clone(); }
            if update.cache_type_v.is_some() { config.cache_type_v = update.cache_type_v.clone(); }
            if update.flash_attn.is_some() { config.flash_attn = update.flash_attn; }
            if update.cache_ram.is_some() { config.cache_ram = update.cache_ram; }
            // embedding_ctx_size and dim are intentionally ignored for orchestrator

            match config::save_orchestrator_config(&state.data_dir, &config) {
                Ok(_) => Json(serde_json::json!({
                    "success": true,
                    "message": "Model config updated"
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to save orchestrator config: {}", e)
                })),
            }
        }
        "embedding" => {
            let mut config = match config::load_embedding_config(&state.data_dir) {
                Ok(c) => c,
                Err(e) => return Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to load embedding config: {}", e)
                })),
            };

            // Merge update — update wins if Some
            if update.backend_type.is_some() { config.backend_type = update.backend_type; }
            if update.model_path.is_some() { config.model_path = update.model_path.clone(); }
            if update.api_base_url.is_some() { config.api_base_url = update.api_base_url.clone(); }
            if update.model_name.is_some() { config.model_name = update.model_name.clone(); }
            if let Some(port) = update.port { config.port = port; }
            if let Some(auto_start) = update.auto_start { config.auto_start = auto_start; }
            if update.embedding_ctx_size.is_some() { config.embedding_ctx_size = update.embedding_ctx_size; }
            // orchestrator-specific fields are intentionally ignored for embedding
            // dim is stored in app_settings, not embedding config

            match config::save_embedding_config(&state.data_dir, &config) {
                Ok(_) => Json(serde_json::json!({
                    "success": true,
                    "message": "Model config updated"
                })),
                Err(e) => Json(serde_json::json!({
                    "success": false,
                    "error": format!("Failed to save embedding config: {}", e)
                })),
            }
        }
        _ => Json(serde_json::json!({
            "success": false,
            "error": format!("Unknown model type: {}", model_type)
        })),
    }
}
