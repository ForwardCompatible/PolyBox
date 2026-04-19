//! app_settings.rs
//! Application settings database operations
//!
//! Responsibilities:
//! - AppSettings data structure
//! - AppSettingsUpdate partial update structure
//! - Get full application settings
//! - Full settings update
//! - Partial settings update
//!
//! Dependencies:
//! - rusqlite Connection
//! - super::DbPool
//! - serde
//! - tracing logging

use rusqlite::{Connection, Result};
use super::DbPool;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub id: i64,
    pub agent_name: String,
    pub user_name: String,
    pub user_timezone: String,
    pub orchestrator_ctx_size: i64,
    pub output_token_reserve: i64,
    pub max_iterations: i64,
    pub thinking_open_tag: Option<String>,
    pub thinking_close_tag: Option<String>,
    pub show_reasoning: bool,
    pub reasoning_collapsed_default: bool,
    pub default_session_id: String,
    pub embeddings_enabled: bool,
    pub embeddings_dim: Option<i64>,
    pub store_context_debug: bool,
    pub chat_history_turn_limit: i64,
    pub web_server_port: i64,
    pub health_check_timeout_secs: i64,
}

/// Request struct for partial app settings update (from REST API)
#[derive(Debug, serde::Deserialize)]
pub struct AppSettingsUpdate {
    pub agent_name: Option<String>,
    pub user_name: Option<String>,
    pub user_timezone: Option<String>,
    pub orchestrator_ctx_size: Option<i64>,
    pub output_token_reserve: Option<i64>,
    pub max_iterations: Option<i64>,
    pub thinking_open_tag: Option<String>,
    pub thinking_close_tag: Option<String>,
    pub show_reasoning: Option<bool>,
    pub reasoning_collapsed_default: Option<bool>,
    pub default_session_id: Option<String>,
    pub embeddings_enabled: Option<bool>,
    pub embeddings_dim: Option<i64>,
    pub store_context_debug: Option<bool>,
    pub chat_history_turn_limit: Option<i64>,
    pub web_server_port: Option<i64>,
    pub health_check_timeout_secs: Option<i64>,
}

pub fn get_app_settings(pool: &DbPool) -> Result<AppSettings> {
    let conn = pool.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    conn.query_row(
        "SELECT id, agent_name, user_name, user_timezone, orchestrator_ctx_size,
                output_token_reserve, max_iterations, thinking_open_tag, thinking_close_tag,
                show_reasoning, reasoning_collapsed_default, default_session_id,
                embeddings_enabled, embeddings_dim, store_context_debug, chat_history_turn_limit,
                web_server_port, health_check_timeout_secs
         FROM app_settings WHERE id = 1",
        [],
        |row| {
            Ok(AppSettings {
                id: row.get(0)?,
                agent_name: row.get(1)?,
                user_name: row.get(2)?,
                user_timezone: row.get(3)?,
                orchestrator_ctx_size: row.get(4)?,
                output_token_reserve: row.get(5)?,
                max_iterations: row.get(6)?,
                thinking_open_tag: row.get(7)?,
                thinking_close_tag: row.get(8)?,
                show_reasoning: row.get(9)?,
                reasoning_collapsed_default: row.get(10)?,
                default_session_id: row.get(11)?,
                embeddings_enabled: row.get(12)?,
                embeddings_dim: row.get(13)?,
                store_context_debug: row.get(14)?,
                chat_history_turn_limit: row.get(15)?,
                web_server_port: row.get(16)?,
                health_check_timeout_secs: row.get(17)?,
            })
        },
    )
}

pub fn update_app_settings(conn: &Connection, settings: &AppSettings) -> Result<()> {
    conn.execute(
        r#"UPDATE app_settings SET
            agent_name = ?1, user_name = ?2, user_timezone = ?3,
            orchestrator_ctx_size = ?4, output_token_reserve = ?5,
            max_iterations = ?6, thinking_open_tag = ?7, thinking_close_tag = ?8,
            show_reasoning = ?9, reasoning_collapsed_default = ?10,
            default_session_id = ?11, embeddings_enabled = ?12,
            embeddings_dim = ?13, store_context_debug = ?14, chat_history_turn_limit = ?15,
            web_server_port = ?16, health_check_timeout_secs = ?17,
            updated_at = CURRENT_TIMESTAMP
         WHERE id = 1"#,
        rusqlite::params![
            settings.agent_name,
            settings.user_name,
            settings.user_timezone,
            settings.orchestrator_ctx_size,
            settings.output_token_reserve,
            settings.max_iterations,
            settings.thinking_open_tag,
            settings.thinking_close_tag,
            settings.show_reasoning,
            settings.reasoning_collapsed_default,
            settings.default_session_id,
            settings.embeddings_enabled,
            settings.embeddings_dim,
            settings.store_context_debug,
            settings.chat_history_turn_limit,
            settings.web_server_port,
            settings.health_check_timeout_secs,
        ],
    )?;
    Ok(())
}

/// Update app settings with partial fields (from REST API)
pub fn update_app_settings_partial(
    pool: &DbPool,
    update: &AppSettingsUpdate,
) -> Result<()> {
    // Fetch existing settings
    let existing = get_app_settings(pool)?;

    // Merge: existing values + update values (update wins if Some)
    let merged = AppSettings {
        id: existing.id,
        agent_name: update.agent_name.clone().unwrap_or(existing.agent_name),
        user_name: update.user_name.clone().unwrap_or(existing.user_name),
        user_timezone: update.user_timezone.clone().unwrap_or(existing.user_timezone),
        orchestrator_ctx_size: update.orchestrator_ctx_size.unwrap_or(existing.orchestrator_ctx_size),
        output_token_reserve: update.output_token_reserve.unwrap_or(existing.output_token_reserve),
        max_iterations: update.max_iterations.unwrap_or(existing.max_iterations),
        thinking_open_tag: update.thinking_open_tag.clone().or(existing.thinking_open_tag),
        thinking_close_tag: update.thinking_close_tag.clone().or(existing.thinking_close_tag),
        show_reasoning: update.show_reasoning.unwrap_or(existing.show_reasoning),
        reasoning_collapsed_default: update.reasoning_collapsed_default.unwrap_or(existing.reasoning_collapsed_default),
        default_session_id: update.default_session_id.clone().unwrap_or(existing.default_session_id),
        embeddings_enabled: update.embeddings_enabled.unwrap_or(existing.embeddings_enabled),
        embeddings_dim: update.embeddings_dim.or(existing.embeddings_dim),
        store_context_debug: update.store_context_debug.unwrap_or(existing.store_context_debug),
        chat_history_turn_limit: update.chat_history_turn_limit.unwrap_or(existing.chat_history_turn_limit),
        web_server_port: update.web_server_port.unwrap_or(existing.web_server_port),
        health_check_timeout_secs: update.health_check_timeout_secs.unwrap_or(existing.health_check_timeout_secs),
    };

    let conn = pool.lock().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
    update_app_settings(&conn, &merged)
}
