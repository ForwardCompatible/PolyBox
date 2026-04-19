//! logs.rs
//! logs.db data access layer
//!
//! Responsibilities:
//! - CRUD operations for chat_sessions, chat_logs, react_iterations, tool_calls
//! - Schema initialization for logs.db
//!
//! Dependencies:
//! - rusqlite Connection
//! - super::DbPool

use rusqlite::{Connection, Result};
use crate::db::DbPool;

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatSession {
    pub session_id: String,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatLog {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub reasoning_content: Option<String>,
    pub response_content: Option<String>,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub tokens_per_second: Option<f64>,
    pub generation_time_ms: Option<i64>,
    pub interrupted: bool,
    pub timestamp: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReactIteration {
    pub id: i64,
    pub chat_log_id: i64,
    pub iteration_index: i64,
    pub thought_content: Option<String>,
    pub response_content: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    pub id: i64,
    pub iteration_id: i64,
    pub tag_name: String,
    pub parameters: Option<String>,
    pub result: Option<String>,
    pub success: Option<bool>,
}

// ---------------------------------------------------------------------------
// Schema init
// ---------------------------------------------------------------------------

pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
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

// ---------------------------------------------------------------------------
// chat_sessions
// ---------------------------------------------------------------------------

pub fn create_session(pool: &DbPool, session_id: &str, name: &str) -> Result<()> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    conn.execute(
        "INSERT INTO chat_sessions (session_id, name) VALUES (?1, ?2)",
        [session_id, name],
    )?;
    Ok(())
}

pub fn get_session(pool: &DbPool, session_id: &str) -> Result<Option<ChatSession>> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    let mut stmt = conn.prepare("SELECT session_id, name, created_at FROM chat_sessions WHERE session_id = ?1")?;
    let result = stmt.query_row([session_id], |row| {
        Ok(ChatSession {
            session_id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    });
    match result {
        Ok(session) => Ok(Some(session)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

// ---------------------------------------------------------------------------
// chat_logs
// ---------------------------------------------------------------------------

pub fn create_chat_log(
    pool: &DbPool,
    session_id: &str,
    role: &str,
) -> Result<i64> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    conn.execute(
        "INSERT INTO chat_logs (session_id, role) VALUES (?1, ?2)",
        [session_id, role],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_chat_log_response(
    pool: &DbPool,
    chat_log_id: i64,
    reasoning_content: Option<&str>,
    response_content: Option<&str>,
    prompt_tokens: Option<i64>,
    completion_tokens: Option<i64>,
    tokens_per_second: Option<f64>,
    generation_time_ms: Option<i64>,
    interrupted: bool,
) -> Result<()> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    conn.execute(
        r#"UPDATE chat_logs SET
            reasoning_content = ?2,
            response_content = ?3,
            prompt_tokens = ?4,
            completion_tokens = ?5,
            tokens_per_second = ?6,
            generation_time_ms = ?7,
            interrupted = ?8
           WHERE id = ?1"#,
        rusqlite::params![
            chat_log_id,
            reasoning_content,
            response_content,
            prompt_tokens,
            completion_tokens,
            tokens_per_second,
            generation_time_ms,
            interrupted,
        ],
    )?;
    Ok(())
}

pub fn get_chat_logs_for_session(
    pool: &DbPool,
    session_id: &str,
    limit: Option<i64>,
) -> Result<Vec<ChatLog>> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    let limit = limit.unwrap_or(50);
    let mut stmt = conn.prepare(
        "SELECT id, session_id, role, reasoning_content, response_content,
                prompt_tokens, completion_tokens, tokens_per_second,
                generation_time_ms, interrupted, timestamp
         FROM chat_logs WHERE session_id = ?1
         ORDER BY timestamp DESC LIMIT ?2",
    )?;
    let rows = stmt.query_map([session_id, &limit.to_string()], |row| {
        Ok(ChatLog {
            id: row.get(0)?,
            session_id: row.get(1)?,
            role: row.get(2)?,
            reasoning_content: row.get(3)?,
            response_content: row.get(4)?,
            prompt_tokens: row.get(5)?,
            completion_tokens: row.get(6)?,
            tokens_per_second: row.get(7)?,
            generation_time_ms: row.get(8)?,
            interrupted: row.get(9)?,
            timestamp: row.get(10)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

// ---------------------------------------------------------------------------
// react_iterations
// ---------------------------------------------------------------------------

pub fn create_react_iteration(
    pool: &DbPool,
    chat_log_id: i64,
    iteration_index: i64,
    thought_content: Option<&str>,
    response_content: Option<&str>,
) -> Result<i64> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    conn.execute(
        "INSERT INTO react_iterations (chat_log_id, iteration_index, thought_content, response_content) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![chat_log_id, iteration_index, thought_content, response_content],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_react_iterations_for_chat_log(
    pool: &DbPool,
    chat_log_id: i64,
) -> Result<Vec<ReactIteration>> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    let mut stmt = conn.prepare(
        "SELECT id, chat_log_id, iteration_index, thought_content, response_content, timestamp
         FROM react_iterations WHERE chat_log_id = ?1 ORDER BY iteration_index",
    )?;
    let rows = stmt.query_map([chat_log_id], |row| {
        Ok(ReactIteration {
            id: row.get(0)?,
            chat_log_id: row.get(1)?,
            iteration_index: row.get(2)?,
            thought_content: row.get(3)?,
            response_content: row.get(4)?,
            timestamp: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

pub fn get_iteration_count(pool: &DbPool, chat_log_id: i64) -> Result<i64> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM react_iterations WHERE chat_log_id = ?1",
        [chat_log_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

// ---------------------------------------------------------------------------
// tool_calls
// ---------------------------------------------------------------------------

pub fn create_tool_call(
    pool: &DbPool,
    iteration_id: i64,
    tag_name: &str,
    parameters: Option<&str>,
) -> Result<i64> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    conn.execute(
        "INSERT INTO tool_calls (iteration_id, tag_name, parameters) VALUES (?1, ?2, ?3)",
        rusqlite::params![iteration_id, tag_name, parameters],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_tool_call_result(
    pool: &DbPool,
    tool_call_id: i64,
    result: &str,
    success: bool,
) -> Result<()> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    conn.execute(
        "UPDATE tool_calls SET result = ?2, success = ?3 WHERE id = ?1",
        rusqlite::params![tool_call_id, result, success],
    )?;
    Ok(())
}

pub fn get_tool_calls_for_iteration(
    pool: &DbPool,
    iteration_id: i64,
) -> Result<Vec<ToolCall>> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    let mut stmt = conn.prepare(
        "SELECT id, iteration_id, tag_name, parameters, result, success
         FROM tool_calls WHERE iteration_id = ?1",
    )?;
    let rows = stmt.query_map([iteration_id], |row| {
        Ok(ToolCall {
            id: row.get(0)?,
            iteration_id: row.get(1)?,
            tag_name: row.get(2)?,
            parameters: row.get(3)?,
            result: row.get(4)?,
            success: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}