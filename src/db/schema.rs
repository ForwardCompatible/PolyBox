//! schema.rs
//! Database schema definition and initialization
//!
//! Responsibilities:
//! - Define all database tables and indexes
//! - Initialize database schema on first run
//! - Call seed functions after schema creation
//!
//! Dependencies:
//! - rusqlite Connection
//! - tracing logging
//! - crate::db::seeds

use rusqlite::{Connection, Result};
use tracing::info;
use super::seeds;

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS app_settings (
            id                          INTEGER PRIMARY KEY CHECK (id = 1),
            agent_name                  TEXT    DEFAULT 'PolyBox',
            user_name                   TEXT    DEFAULT 'User',
            user_timezone               TEXT    DEFAULT 'America/New_York',
            orchestrator_ctx_size       INTEGER DEFAULT 32000,
            output_token_reserve        INTEGER DEFAULT 6144,
            max_iterations              INTEGER DEFAULT 5,
            thinking_open_tag           TEXT,
            thinking_close_tag          TEXT,
            show_reasoning              BOOLEAN DEFAULT TRUE,
            reasoning_collapsed_default BOOLEAN DEFAULT TRUE,
            default_session_id          TEXT    DEFAULT 'default',
            embeddings_enabled          BOOLEAN DEFAULT TRUE,
            embeddings_dim              INTEGER,
            store_context_debug         BOOLEAN DEFAULT FALSE,
            chat_history_turn_limit    INTEGER DEFAULT 10,
            web_server_port             INTEGER DEFAULT 9001,
            health_check_timeout_secs   INTEGER DEFAULT 120,
            updated_at                  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS action_registry (
            id             INTEGER PRIMARY KEY AUTOINCREMENT,
            tag            TEXT    NOT NULL UNIQUE,
            description    TEXT    NOT NULL,
            parameters     TEXT    NOT NULL,
            enabled        BOOLEAN DEFAULT TRUE,
            handler        TEXT    NOT NULL,
            execution_mode TEXT    NOT NULL DEFAULT 'immediate',
            execution_type TEXT    NOT NULL DEFAULT 'read',
            created_at     TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at     TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS personality (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            section    TEXT    NOT NULL UNIQUE,
            content    TEXT    NOT NULL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_personality_section ON personality(section);

        CREATE TABLE IF NOT EXISTS model_registry (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            model_type       TEXT NOT NULL,
            repo_name        TEXT NOT NULL,
            filename         TEXT NOT NULL,
            full_path        TEXT NOT NULL,
            file_size_bytes  INTEGER,
            discovered_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(model_type, repo_name, filename)
        );

        CREATE INDEX IF NOT EXISTS idx_model_registry_type ON model_registry(model_type);
        CREATE INDEX IF NOT EXISTS idx_model_registry_repo ON model_registry(repo_name);
        "#,
    )?;

    // Seed app_settings if empty
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM app_settings",
        [],
        |row| row.get(0),
    )?;

    if count == 0 {
        conn.execute(
            r#"INSERT INTO app_settings (
                id, agent_name, user_name, user_timezone,
                orchestrator_ctx_size, output_token_reserve,
                max_iterations, thinking_open_tag, thinking_close_tag,
                show_reasoning, reasoning_collapsed_default,
                default_session_id, embeddings_enabled, embeddings_dim,
                store_context_debug, chat_history_turn_limit,
                web_server_port, health_check_timeout_secs
            ) VALUES (
                1, 'PolyBox', 'User', 'America/New_York',
                32000, 6144,
                5, NULL, NULL,
                TRUE, TRUE,
                'default', TRUE, NULL,
                FALSE, 10,
                9001, 120
            )"#,
            [],
        )?;
        info!("Seeded app_settings defaults");
    }

    // Seed action_registry if empty
    let action_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM action_registry",
        [],
        |row| row.get(0),
    )?;

    if action_count == 0 {
        seeds::seed_action_registry(conn)?;
        info!("Seeded action_registry defaults");
    }

    // Seed personality if empty
    let personality_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM personality",
        [],
        |row| row.get(0),
    )?;

    if personality_count == 0 {
        seeds::seed_personality(conn)?;
        info!("Seeded personality defaults");
    }

    Ok(())
}

/// Initialize logs.db schema (chat_sessions, chat_logs, react_iterations, tool_calls)
pub fn init_logs_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS chat_sessions (
            session_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS chat_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            reasoning_content TEXT,
            response_content TEXT,
            prompt_tokens INTEGER,
            completion_tokens INTEGER,
            tokens_per_second REAL,
            generation_time_ms INTEGER,
            interrupted BOOLEAN DEFAULT FALSE,
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (session_id) REFERENCES chat_sessions(session_id)
        );

        CREATE INDEX IF NOT EXISTS idx_chat_logs_session ON chat_logs(session_id);
        CREATE INDEX IF NOT EXISTS idx_chat_logs_timestamp ON chat_logs(timestamp);

        CREATE TABLE IF NOT EXISTS react_iterations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chat_log_id INTEGER NOT NULL,
            iteration_index INTEGER NOT NULL,
            thought_content TEXT,
            response_content TEXT,
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (chat_log_id) REFERENCES chat_logs(id)
        );

        CREATE INDEX IF NOT EXISTS idx_react_iterations_chat_log ON react_iterations(chat_log_id);

        CREATE TABLE IF NOT EXISTS tool_calls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            iteration_id INTEGER NOT NULL,
            tag_name TEXT NOT NULL,
            parameters TEXT,
            result TEXT,
            success BOOLEAN,
            FOREIGN KEY (iteration_id) REFERENCES react_iterations(id)
        );

        CREATE INDEX IF NOT EXISTS idx_tool_calls_iteration ON tool_calls(iteration_id);
        CREATE INDEX IF NOT EXISTS idx_tool_calls_tag_name ON tool_calls(tag_name);
        "#,
    )?;
    Ok(())
}

/// Initialize memory.db schema (memories, embeddings)
pub fn init_memory_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS memories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            type TEXT NOT NULL,
            content TEXT NOT NULL,
            source TEXT NOT NULL,
            importance_weight REAL DEFAULT 1.0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_memories_importance ON memories(importance_weight DESC);

        CREATE TABLE IF NOT EXISTS embeddings (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_type TEXT NOT NULL,
            source_id INTEGER NOT NULL,
            embedding BLOB NOT NULL,
            model_version TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_embeddings_source ON embeddings(source_type, source_id);
        "#,
    )?;
    Ok(())
}
