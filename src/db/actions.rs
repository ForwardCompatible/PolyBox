//! actions.rs
//! Action registry database operations
//!
//! Responsibilities:
//! - ActionTag data structure
//! - ActionRegistryEntry data structure
//! - Get full action registry list
//! - Get all enabled actions
//! - Get single action by tag
//!
//! Dependencies:
//! - rusqlite Connection
//! - serde
//! - DbPool from crate::db

use rusqlite::{Connection, Result};
use crate::db::{DbPool, get_conn};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActionTag {
    pub id: i64,
    pub tag: String,
    pub description: String,
    pub parameters: String,
    pub enabled: bool,
    pub handler: String,
    pub execution_mode: String,
    pub execution_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActionRegistryEntry {
    pub id: i64,
    pub tag: String,
    pub description: String,
    pub parameters: String,
    pub enabled: bool,
    pub handler: String,
    pub execution_type: String,
    pub execution_mode: String,
}

pub fn get_action_registry(conn: &Connection) -> Result<Vec<ActionTag>> {
    let mut stmt = conn.prepare(
        "SELECT id, tag, description, parameters, enabled, handler, execution_mode, execution_type
         FROM action_registry ORDER BY id",
    )?;

    let tags = stmt
        .query_map([], |row| {
            Ok(ActionTag {
                id: row.get(0)?,
                tag: row.get(1)?,
                description: row.get(2)?,
                parameters: row.get(3)?,
                enabled: row.get(4)?,
                handler: row.get(5)?,
                execution_mode: row.get(6)?,
                execution_type: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(tags)
}

pub fn get_all_enabled_actions(pool: &DbPool) -> Result<Vec<ActionRegistryEntry>> {
    let conn = get_conn(pool)?;
    let mut stmt = conn.prepare(
        "SELECT id, tag, description, parameters, enabled, handler, execution_type, execution_mode
         FROM action_registry
         WHERE enabled = 1
         ORDER BY tag",
    )?;

    let actions = stmt
        .query_map([], |row| {
            Ok(ActionRegistryEntry {
                id: row.get(0)?,
                tag: row.get(1)?,
                description: row.get(2)?,
                parameters: row.get(3)?,
                enabled: row.get(4)?,
                handler: row.get(5)?,
                execution_type: row.get(6)?,
                execution_mode: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(actions)
}

pub fn get_action_by_tag(pool: &DbPool, tag: &str) -> Result<Option<ActionRegistryEntry>> {
    let conn = get_conn(pool)?;
    let mut stmt = conn.prepare(
        "SELECT id, tag, description, parameters, enabled, handler, execution_type, execution_mode
         FROM action_registry
         WHERE tag = ?1",
    )?;

    let mut rows = stmt.query([tag])?;

    if let Some(row) = rows.next()? {
        Ok(Some(ActionRegistryEntry {
            id: row.get(0)?,
            tag: row.get(1)?,
            description: row.get(2)?,
            parameters: row.get(3)?,
            enabled: row.get(4)?,
            handler: row.get(5)?,
            execution_type: row.get(6)?,
            execution_mode: row.get(7)?,
        }))
    } else {
        Ok(None)
    }
}
