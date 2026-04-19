//! route_settings.rs
//! Application settings HTTP endpoints
//!
//! Responsibilities:
//! - Provides GET endpoint `/api/app-settings` to retrieve application settings
//! - Provides PUT endpoint `/api/app-settings` to update application settings
//!
//! Dependencies:
//! - axum framework
//! - crate::db for settings persistence
//! - crate::AppState for shared state

use axum::{
    extract::{
        State, Json,
    },
    response::IntoResponse,
};
use std::sync::Arc;

use crate::db;
use crate::AppState;

/// Retrieves current application settings
pub async fn get_app_settings(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match db::core::get_app_settings(&state.core_db) {
        Ok(settings) => Json(serde_json::json!({
            "success": true,
            "data": settings
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

/// Updates application settings with partial changes
pub async fn update_app_settings_handler(
    State(state): State<Arc<AppState>>,
    Json(settings): Json<db::core::AppSettingsUpdate>,
) -> impl IntoResponse {
    match db::core::update_app_settings_partial(&state.core_db, &settings) {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "message": "Settings updated"
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}
