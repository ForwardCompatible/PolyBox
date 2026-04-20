# PolyBox 2.0 — Database Schemas

## Overview

Three SQLite databases. Each has a distinct responsibility:

- **core.db** — System configuration, action registry, and personality. Required for the system to function.
- **logs.db** — Everything that happened. Every generation, every action tag execution, every server event.
- **memory.db** — Everything the agent remembers across all sessions.

---

## Pragmas (apply on every DB connection)

```sql
PRAGMA journal_mode=WAL;
PRAGMA synchronous=NORMAL;
PRAGMA foreign_keys=ON;
```

---

## core.db

```sql
CREATE TABLE IF NOT EXISTS app_settings (
    id                          INTEGER PRIMARY KEY CHECK (id = 1),
    agent_name                  TEXT    DEFAULT 'PolyBox',
    user_name                   TEXT    DEFAULT 'User',
    user_timezone               TEXT    DEFAULT 'America/New_York',
    orchestrator_ctx_size       INTEGER DEFAULT 256000,
    output_token_reserve        INTEGER DEFAULT 6144,
    max_iterations              INTEGER DEFAULT 5,        -- 0 = unlimited
    thinking_open_tag           TEXT,
    thinking_close_tag          TEXT,
    show_reasoning              BOOLEAN DEFAULT TRUE,
    reasoning_collapsed_default BOOLEAN DEFAULT TRUE,
    default_session_id          TEXT    DEFAULT 'default',
    embeddings_enabled          BOOLEAN DEFAULT TRUE,
    store_context_debug         BOOLEAN DEFAULT FALSE,
    embedding_server_port       INTEGER DEFAULT 11435,
    chat_history_turn_limit INTEGER DEFAULT 10,
    web_server_port INTEGER DEFAULT 9001,
    health_check_timeout_secs INTEGER DEFAULT 120,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS action_registry (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    tag            TEXT    NOT NULL UNIQUE,
    description    TEXT    NOT NULL,
    parameters     TEXT    NOT NULL,              -- JSON: parameter definitions
    enabled        BOOLEAN DEFAULT TRUE,
    handler        TEXT    NOT NULL,              -- 'system' | './workspace/toolbox/<script>.py'
    execution_mode TEXT    NOT NULL DEFAULT 'immediate', -- 'immediate' | 'async' | 'persistent'
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

CREATE TABLE IF NOT EXISTS model_configs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  model_type TEXT NOT NULL UNIQUE, -- 'orchestrator' | 'embedding'
  model_path TEXT NOT NULL,
  port INTEGER NOT NULL, -- 11434 for orchestrator, 11435 for embedding
  auto_start BOOLEAN DEFAULT FALSE,
  -- Orchestrator-only fields (NULL for embedding)
  ctx_size INTEGER,           -- Context window size, default 256000
  n_gpu_layers INTEGER,       -- GPU layers, -1 = all layers
  temperature REAL,           -- Sampling temperature, default 0.9
  repeat_penalty REAL,        -- Repeat penalty, default 1.1
  cache_type_k TEXT,          -- KV cache type K: NULL (N/A) | 'f16' | 'q8_0' | 'q4_0' - NULL means flag not passed to llama-server
  cache_type_v TEXT,          -- KV cache type V: NULL (N/A) | 'f16' | 'q8_0' | 'q4_0' - NULL means flag not passed to llama-server
  flash_attn BOOLEAN,         -- Flash attention flag
  cache_ram BOOLEAN,          -- Cache in RAM flag
  -- Embedding-only fields (NULL for orchestrator)
  embedding_ctx_size INTEGER, -- Embedding context size, default 512
  dim INTEGER,                -- Embedding dimension (required for embedding models)
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS model_registry (
id INTEGER PRIMARY KEY AUTOINCREMENT,
model_type TEXT NOT NULL, -- 'orchestrator' | 'embedding'
repo_name TEXT NOT NULL, -- e.g. 'llama-3.2-3b'
filename TEXT NOT NULL, -- e.g. 'Q4_K_M.gguf'
full_path TEXT NOT NULL, -- absolute path to the GGUF file
file_size_bytes INTEGER,
discovered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
UNIQUE(model_type, repo_name, filename)
);

CREATE INDEX IF NOT EXISTS idx_model_registry_type ON model_registry(model_type);
CREATE INDEX IF NOT EXISTS idx_model_registry_repo ON model_registry(repo_name);
```

---

## logs.db

```sql
CREATE TABLE IF NOT EXISTS chat_sessions (
    session_id TEXT PRIMARY KEY,                  -- UUID or 'default' for MVP
    name       TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS chat_logs (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id          TEXT    NOT NULL,
    role                TEXT    NOT NULL,          -- 'user' | 'assistant'
    turn_type           TEXT    NOT NULL,          -- 'user' | 'assistant' | 'regeneration'
    reasoning_content   TEXT,                      -- Thinking tokens, delimiters stripped
    response_content    TEXT,                      -- Clean final reply, action tags stripped
    prompt_tokens       INTEGER,
    completion_tokens   INTEGER,
    tokens_per_second   REAL,
    generation_time_ms  INTEGER,
    loop_iteration      INTEGER DEFAULT 0,         -- Tracks REGENERATE steps within a turn
    interrupted         BOOLEAN DEFAULT FALSE,      -- TRUE if generation was stopped by user
    timestamp           TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES chat_sessions(session_id)
);

CREATE INDEX IF NOT EXISTS idx_chat_logs_session   ON chat_logs(session_id);
CREATE INDEX IF NOT EXISTS idx_chat_logs_timestamp ON chat_logs(timestamp);

CREATE TABLE IF NOT EXISTS tool_calls (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_log_id INTEGER NOT NULL,
    session_id  TEXT    NOT NULL,
    tag_name    TEXT    NOT NULL,                  -- Matches action_registry.tag
    parameters  TEXT,                              -- JSON: parameters as parsed
    result      TEXT,                              -- JSON: full handler response
    success     BOOLEAN,
    timestamp   TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chat_log_id) REFERENCES chat_logs(id)
);

CREATE INDEX IF NOT EXISTS idx_tool_calls_chat_log ON tool_calls(chat_log_id);
CREATE INDEX IF NOT EXISTS idx_tool_calls_tag_name ON tool_calls(tag_name);

-- Only written when store_context_debug = TRUE in app_settings
CREATE TABLE IF NOT EXISTS context_debug (
    id                     INTEGER PRIMARY KEY AUTOINCREMENT,
    chat_log_id            INTEGER NOT NULL,
    session_id             TEXT    NOT NULL,
    tokens_core_values     INTEGER,
    tokens_personality     INTEGER,
    tokens_action_registry INTEGER,
    tokens_current_time    INTEGER,
    tokens_memory_summary  INTEGER,
    tokens_chat_history    INTEGER,
    tokens_tool_results    INTEGER,
    tokens_user_message    INTEGER,
    tokens_total           INTEGER,
    injected_memory_ids    TEXT,                   -- Comma-separated memory IDs injected this turn
    timestamp              TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (chat_log_id) REFERENCES chat_logs(id)
);

CREATE INDEX IF NOT EXISTS idx_context_debug_chat_log ON context_debug(chat_log_id);

-- Server lifecycle events only. Per-generation stats live on chat_logs.
CREATE TABLE IF NOT EXISTS llama_server_log (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,                      -- 'started' | 'stopped' | 'model_loaded' | 'crash'
    model_name TEXT,
    detail     TEXT,
    timestamp  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

## memory.db

```sql
CREATE TABLE IF NOT EXISTS memories (
    id                INTEGER PRIMARY KEY AUTOINCREMENT,
    type              TEXT    NOT NULL,             -- e.g. 'fact', 'preference', 'instruction'
    content           TEXT    NOT NULL,
    source            TEXT    NOT NULL,             -- session_id that produced this memory
    importance_weight REAL    DEFAULT 1.0,          -- Used for ranking when embeddings disabled
    created_at        TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at        TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_memories_importance ON memories(importance_weight DESC);

-- Embeddings generated for memories and chat_logs.
CREATE TABLE IF NOT EXISTS embeddings (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    source_type   TEXT    NOT NULL,                -- 'memory' | 'chat_log'
    source_id     INTEGER NOT NULL,                -- References memories.id or chat_logs.id based on source_type
    embedding     BLOB    NOT NULL,                -- Raw vector from embedding model
    model_version TEXT    NOT NULL,                -- Embedding model identifier
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_embeddings_source ON embeddings(source_type, source_id);
```
