//! Database layer for PolyBox
//! Three SQLite databases: core.db, logs.db, memory.db

pub mod core;
pub mod seeds;
pub mod schema;
pub mod app_settings;
pub mod actions;
pub mod models;
pub mod logs;
pub mod memory;

use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use std::path::Path;
use tracing::info;

pub type DbPool = Arc<Mutex<Connection>>;

pub fn init(data_dir: &Path) -> Result<(DbPool, DbPool, DbPool), Box<dyn std::error::Error + Send + Sync>> {
    let core_path = data_dir.join("core.db");
    let logs_path = data_dir.join("logs.db");
    let memory_path = data_dir.join("memory.db");

    fn configure(conn: &Connection) {
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;",
        )
        .expect("failed to configure database pragmas");
    }

    let core_conn = Connection::open(&core_path)?;
    configure(&core_conn);
    core::init_schema(&core_conn)?;
    let core_pool: DbPool = Arc::new(Mutex::new(core_conn));

    // logs and memory DBs schema init
    let logs_conn = Connection::open(&logs_path)?;
    configure(&logs_conn);
    logs::init_schema(&logs_conn)?;
    let logs_pool: DbPool = Arc::new(Mutex::new(logs_conn));

    let memory_conn = Connection::open(&memory_path)?;
    configure(&memory_conn);
    memory::init_schema(&memory_conn)?;
    let memory_pool: DbPool = Arc::new(Mutex::new(memory_conn));

    info!("Database layer initialized");
    info!("  core.db: {:?}", core_path);
    info!("  logs.db: {:?}", logs_path);
    info!("  memory.db: {:?}", memory_path);

    Ok((core_pool, logs_pool, memory_pool))
}

// Re-export pools - these would be set by main.rs
// For now, we'll pass them around

pub fn backup(data_dir: &Path, core: &DbPool, logs: &DbPool, memory: &DbPool) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use chrono::Local;
    use std::fs;

    let timestamp = Local::now().format("%Y-%m-%d_%H%M");
    let backup_dir = data_dir.join("backups").join(format!("db_{}", timestamp));
    fs::create_dir_all(&backup_dir)?;

    // Checkpoint WAL before backup
    {
        let conn = core.lock().map_err(|e| format!("lock error: {}", e))?;
        conn.execute_batch("PRAGMA wal_checkpoint(FULL)")?;
    }
    {
        let conn = logs.lock().map_err(|e| format!("lock error: {}", e))?;
        conn.execute_batch("PRAGMA wal_checkpoint(FULL)")?;
    }
    {
        let conn = memory.lock().map_err(|e| format!("lock error: {}", e))?;
        conn.execute_batch("PRAGMA wal_checkpoint(FULL)")?;
    }

    let backup_core = backup_dir.join("core.db");
    let backup_logs = backup_dir.join("logs.db");
    let backup_memory = backup_dir.join("memory.db");

    std::fs::copy(data_dir.join("core.db"), &backup_core)?;
    std::fs::copy(data_dir.join("logs.db"), &backup_logs)?;
    std::fs::copy(data_dir.join("memory.db"), &backup_memory)?;

    info!("Database backup created: {:?}", backup_dir);
    Ok(backup_dir.to_string_lossy().to_string())
}

pub fn restore(
    data_dir: &Path,
    backup_name: &str,
    core: &DbPool,
    logs: &DbPool,
    memory: &DbPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let backup_dir = data_dir.join("backups").join("db").join(backup_name);

    // Verify backup exists
    let backup_core = backup_dir.join("core.db");
    let backup_logs = backup_dir.join("logs.db");
    let backup_memory = backup_dir.join("memory.db");

    if !backup_core.exists() || !backup_logs.exists() || !backup_memory.exists() {
        return Err(format!("Backup not found: {}", backup_name).into());
    }

    // Close existing connections by dropping locks
    drop(core.lock());
    drop(logs.lock());
    drop(memory.lock());

    // Replace database files
    std::fs::copy(&backup_core, data_dir.join("core.db"))?;
    std::fs::copy(&backup_logs, data_dir.join("logs.db"))?;
    std::fs::copy(&backup_memory, data_dir.join("memory.db"))?;

    info!("Database restored from: {:?}", backup_dir);
    Ok(())
}
