//! core.rs
//! Database core module re-exports
//!
//! Responsibilities:
//! - Re-export all database submodules for backwards compatibility
//! - Maintain public interface integrity during decomposition
//!
//! All implementation logic has been moved to individual single-responsibility modules.

pub use super::seeds::{seed_action_registry, seed_personality};
pub use super::schema::init_schema;
pub use super::app_settings::{
    AppSettings,
    AppSettingsUpdate,
    get_app_settings,
    update_app_settings,
    update_app_settings_partial,
};
pub use super::actions::{ActionTag, get_action_registry};
pub use super::models::{ModelConfigUpdate, ModelInfo, discover_models};
