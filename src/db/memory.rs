//! memory.rs
//! memory.db data access layer
//!
//! Responsibilities:
//! - CRUD operations for memories and embeddings tables
//! - Schema initialization for memory.db
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
pub struct Memory {
    pub id: i64,
    #[serde(rename = "type")]
    pub memory_type: String,
    pub content: String,
    pub source: String,
    pub importance_weight: f64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Embedding {
    pub id: i64,
    pub source_type: String,
    pub source_id: i64,
    pub embedding: Vec<f32>,
    pub model_version: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Schema init
// ---------------------------------------------------------------------------

pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
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

// ---------------------------------------------------------------------------
// memories
// ---------------------------------------------------------------------------

pub fn insert_memory(pool: &DbPool, memory: &Memory) -> Result<i64> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    conn.execute(
        "INSERT INTO memories (type, content, source, importance_weight) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![memory.memory_type, memory.content, memory.source, memory.importance_weight],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_memory_by_id(pool: &DbPool, id: i64) -> Result<Option<Memory>> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    let mut stmt = conn.prepare(
        "SELECT id, type, content, source, importance_weight, created_at, updated_at
         FROM memories WHERE id = ?1",
    )?;
    let result = stmt.query_row([id], |row| {
        Ok(Memory {
            id: row.get(0)?,
            memory_type: row.get(1)?,
            content: row.get(2)?,
            source: row.get(3)?,
            importance_weight: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    });
    match result {
        Ok(m) => Ok(Some(m)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Importance-ranked fallback search when embeddings are disabled.
/// Returns memories ordered by importance_weight DESC, limited by `limit`.
pub fn search_memories(pool: &DbPool, limit: usize) -> Result<Vec<Memory>> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    let mut stmt = conn.prepare(
        "SELECT id, type, content, source, importance_weight, created_at, updated_at
         FROM memories ORDER BY importance_weight DESC LIMIT ?1",
    )?;
    let rows = stmt.query_map([limit as i64], |row| {
        Ok(Memory {
            id: row.get(0)?,
            memory_type: row.get(1)?,
            content: row.get(2)?,
            source: row.get(3)?,
            importance_weight: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

pub fn delete_memory(pool: &DbPool, id: i64) -> Result<()> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    conn.execute("DELETE FROM memories WHERE id = ?1", [id])?;
    Ok(())
}

// ---------------------------------------------------------------------------
// embeddings
// ---------------------------------------------------------------------------

pub fn insert_embedding(pool: &DbPool, embedding: &Embedding) -> Result<()> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    // Serialize Vec<f32> to bytes for BLOB storage
    let bytes: Vec<u8> = embedding
        .embedding
        .iter()
        .flat_map(|f| f.to_le_bytes().to_vec())
        .collect();
    conn.execute(
        "INSERT INTO embeddings (source_type, source_id, embedding, model_version) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![embedding.source_type, embedding.source_id, bytes, embedding.model_version],
    )?;
    Ok(())
}

pub fn get_embeddings_for_source(
    pool: &DbPool,
    source_type: &str,
    source_id: i64,
) -> Result<Vec<Embedding>> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    let mut stmt = conn.prepare(
        "SELECT id, source_type, source_id, embedding, model_version, created_at
         FROM embeddings WHERE source_type = ?1 AND source_id = ?2",
    )?;
    let rows = stmt.query_map(rusqlite::params![source_type, source_id], |row| {
        let blob: Vec<u8> = row.get(3)?;
        // Deserialize bytes back to Vec<f32>
        let embedding: Vec<f32> = blob
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        Ok(Embedding {
            id: row.get(0)?,
            source_type: row.get(1)?,
            source_id: row.get(2)?,
            embedding,
            model_version: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
}

pub fn delete_embeddings_for_source(pool: &DbPool, source_type: &str, source_id: i64) -> Result<()> {
    let conn = pool.lock().map_err(|_e| rusqlite::Error::InvalidQuery)?;
    conn.execute(
        "DELETE FROM embeddings WHERE source_type = ?1 AND source_id = ?2",
        rusqlite::params![source_type, source_id],
    )?;
    Ok(())
}