//! actions.rs
//! Action registry HTTP endpoints
//!
//! Responsibilities:
//! - Provides `/api/action-registry` endpoint to retrieve registered actions
//!
//! Dependencies:
//! - axum framework
//! - crate::db for action registry data
//! - crate::AppState for shared state

use axum::{
    extract::{
        State,
    },
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::db;
use crate::AppState;

/// Retrieves the full list of registered system actions
pub async fn get_action_registry(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let conn = match state.core_db.lock() {
        Ok(c) => c,
        Err(e) => return Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    };

    match db::core::get_action_registry(&conn) {
        Ok(actions) => Json(serde_json::json!({
            "success": true,
            "data": actions
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}
