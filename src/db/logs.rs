//! logs.rs
//! logs.db data access layer
//!
//! Responsibilities:
//! - Schema initialization for logs.db
//!
//! Dependencies:
//! - rusqlite Connection
//! - super::DbPool

use rusqlite::Connection;

// ---------------------------------------------------------------------------
// Schema init
// ---------------------------------------------------------------------------

pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;

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
