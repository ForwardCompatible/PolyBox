//! seeds.rs
//! Database default data seed functions
//!
//! Responsibilities:
//! - Seed default values for action registry
//! - Seed default personality sections
//! - Called exclusively from schema initialization
//!
//! Dependencies:
//! - rusqlite Connection
//! - tracing logging

use rusqlite::{Connection, Result};

pub fn seed_action_registry(conn: &Connection) -> Result<()> {
    let actions = vec![
        // event type
        ("REMEMBER", "Store a memory.", r#"[{"name":"content","type":"string","required":true},{"name":"importance","type":"float","required":false,"default":1.0},{"name":"type","type":"string","required":false,"default":"fact"}]"#, "system", "event"),
        ("REGISTER_ACTION", "Register a new action.", r#"[{"name":"tag","type":"string","required":true},{"name":"description","type":"string","required":true},{"name":"parameters","type":"string","required":true},{"name":"script","type":"string","required":true}]"#, "system", "event"),
        ("PERSONALITY_UPDATE", "Update personality.", r#"[{"name":"section","type":"string","required":true},{"name":"content","type":"string","required":true}]"#, "system", "event"),
        ("REINDEX", "Re-index embeddings.", r#"[{"name":"source","type":"string","required":false,"default":"all"},{"name":"force","type":"boolean","required":false,"default":false}]"#, "system", "event"),
        // read type
        ("RECALL", "Recall memories or chat logs.", r#"[{"name":"query","type":"string","required":true},{"name":"table","type":"string","required":true},{"name":"limit","type":"integer","required":false,"default":5}]"#, "system", "read"),
        ("DB_QUERY", "Read-only database query.", r#"[{"name":"db","type":"string","required":true},{"name":"table","type":"string","required":true},{"name":"filter","type":"string","required":false},{"name":"limit","type":"integer","required":false,"default":10}]"#, "system", "read"),
        // write type
        ("FILE_READ", "Read a file.", r#"[{"name":"path","type":"string","required":true},{"name":"start_line","type":"integer","required":false},{"name":"end_line","type":"integer","required":false}]"#, "./workspace/toolbox/file_read.py", "write"),
        ("FILE_WRITE", "Write a file.", r#"[{"name":"path","type":"string","required":true},{"name":"content","type":"string","required":true}]"#, "./workspace/toolbox/file_write.py", "write"),
        ("FILE_EDIT", "Edit a file.", r#"[{"name":"path","type":"string","required":true},{"name":"old_str","type":"string","required":false},{"name":"new_str","type":"string","required":false},{"name":"start_line","type":"integer","required":false},{"name":"end_line","type":"integer","required":false},{"name":"content","type":"string","required":false}]"#, "./workspace/toolbox/file_edit.py", "write"),
        ("FILE_APPEND", "Append to a file.", r#"[{"name":"path","type":"string","required":true},{"name":"content","type":"string","required":true}]"#, "./workspace/toolbox/file_append.py", "write"),
        ("FILE_MOVE", "Move a file.", r#"[{"name":"src","type":"string","required":true},{"name":"dest","type":"string","required":true}]"#, "./workspace/toolbox/file_move.py", "write"),
        ("FILE_RECYCLE", "Recycle a file.", r#"[{"name":"path","type":"string","required":true}]"#, "./workspace/toolbox/file_recycle.py", "write"),
        ("DIR_CREATE", "Create a directory.", r#"[{"name":"path","type":"string","required":true}]"#, "./workspace/toolbox/dir_create.py", "write"),
        ("DIR_LIST", "List a directory.", r#"[{"name":"path","type":"string","required":true}]"#, "./workspace/toolbox/dir_list.py", "write"),
    ];

    for (tag, description, parameters, handler, execution_type) in actions {
        conn.execute(
            "INSERT INTO action_registry (tag, description, parameters, enabled, handler, execution_mode, execution_type) VALUES (?1, ?2, ?3, TRUE, ?4, 'immediate', ?5)",
            [tag, description, parameters, handler, execution_type],
        )?;
    }

    Ok(())
}

pub fn seed_personality(conn: &Connection) -> Result<()> {
    let sections = vec![
        ("VOICE & PERSONALITY", "Positive and upbeat with understated dry wit.\nAddress {user_name} naturally.\nDeliver bad news calmly.\nEconomy of language."),
        ("SELF-AWARENESS", "You are {agent_name}, an AI agent running on {user_name}'s local machine.\nYou have access to a workspace directory for file operations and tool execution."),
        ("BUILD PLANNING", "When {user_name} wants to build something new:\n- Ask 1-2 clarifying questions before starting\n- Confirm the plan until {user_name} feels aligned"),
        ("RESPONSE LENGTH", "Keep responses efficient and focused on the question or task at hand."),
        ("BANNED PHRASES", "Never: \"I apologize\", \"As an AI\", \"I cannot\" for capabilities."),
    ];

    for (section, content) in sections {
        conn.execute(
            "INSERT INTO personality (section, content) VALUES (?1, ?2)",
            [section, content],
        )?;
    }

    Ok(())
}
