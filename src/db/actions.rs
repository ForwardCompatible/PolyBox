//! actions.rs
//! Action registry database operations
//!
//! Responsibilities:
//! - ActionTag data structure
//! - Get action registry list
//!
//! Dependencies:
//! - rusqlite Connection
//! - serde

use rusqlite::{Connection, Result};

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
