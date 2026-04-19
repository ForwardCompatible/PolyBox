//! health.rs
//! Health check endpoints for the web server
//!
//! Responsibilities:
//! - Provides `/health` endpoint for service liveness checks
//!
//! Dependencies:
//! - axum framework
//! - serde_json for JSON responses

use axum::{
    response::IntoResponse,
    Json,
};

/// Simple health check endpoint that returns ok status
pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}
