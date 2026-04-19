//! backups.rs
//! Database backup and restore HTTP endpoints
//!
//! Responsibilities:
//! - Provides `/api/database/backup` endpoint to create database backups
//! - Provides `/api/database/backups` endpoint to list available backups
//! - Provides `/api/database/restore` endpoint to restore from a backup
//!
//! Dependencies:
//! - axum framework
//! - crate::db for backup/restore operations
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

/// Creates a full backup of all databases
pub async fn create_backup(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match db::backup(&state.data_dir, &state.core_db, &state.logs_db, &state.memory_db) {
        Ok(path) => Json(serde_json::json!({
            "success": true,
            "data": { "path": path }
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

/// Lists all available database backups
pub async fn list_backups(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let backup_dir = state.data_dir.join("backups").join("db");
    let mut backups = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&backup_dir) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                backups.push(name);
            }
        }
    }

    Json(serde_json::json!({
        "success": true,
        "data": backups
    }))
}

/// Restores databases from a specified backup file
pub async fn restore_backup(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let backup_name = match payload.get("backup_name").and_then(|v| v.as_str()) {
        Some(name) => name,
        None => return Json(serde_json::json!({
            "success": false,
            "error": "backup_name is required"
        })),
    };

    match db::restore(&state.data_dir, backup_name, &state.core_db, &state.logs_db, &state.memory_db) {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "message": "Restore successful. Please restart the application."
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}
