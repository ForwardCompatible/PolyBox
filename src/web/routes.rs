//! routes.rs
//! Root web router assembly
//!
//! Responsibilities:
//! - Mounts all individual route modules into a single application router
//! - Provides single entry point for creating the complete web server router
//!
//! Dependencies:
//! - axum framework
//! - All individual web route modules

use axum::{
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;

use crate::AppState;

use crate::web::health;
use crate::web::hardware;
use crate::web::services;
use crate::web::route_settings;
use crate::web::actions;
use crate::web::models;
use crate::web::backups;

/// Builds and returns the complete application router with all mounted endpoints
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/ws", get(hardware::ws_handler))
        .route("/api/hardware", get(hardware::hardware_stats))
        .route("/api/services/:type/status", get(services::get_service_status))
        .route("/api/services/:type/start", post(services::start_service))
        .route("/api/services/:type/stop", post(services::stop_service))
        .route("/api/app-settings", get(route_settings::get_app_settings))
        .route("/api/app-settings", put(route_settings::update_app_settings_handler))
        .route("/api/action-registry", get(actions::get_action_registry))
        .route("/api/model-configs/:type", get(models::get_model_config))
        .route("/api/model-configs/:type", put(models::update_model_config_handler))
        .route("/api/models/:type", get(models::get_models))
        .route("/api/database/backup", post(backups::create_backup))
        .route("/api/database/backups", get(backups::list_backups))
        .route("/api/database/restore", post(backups::restore_backup))
        .with_state(state)
}
