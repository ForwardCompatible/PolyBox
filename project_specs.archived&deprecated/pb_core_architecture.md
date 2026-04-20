# PolyBox 2.0 — Core Architecture

## System Overview

PolyBox is a locally hosted AI agent framework. A single Rust process (the host) owns all system coordination — it manages two llama.cpp server instances, a `./workspace/` execution environment, three SQLite databases, and a web server that serves the chat UI.

All components run on the same machine. No external services. No cloud dependencies.

---

## Component Map

```
┌─────────────────────────────────────────────────────┐
│                     Rust Host                        │
│                                                      │
│  ┌──────────┐  ┌───────────┐  ┌──────────────────┐  │
│  │  Web     │  │ Generation│  │    Dispatcher    │  │
│  │  Server  │  │   Loop    │  │  (Tag Executor)  │  │
│  └────┬─────┘  └─────┬─────┘  └────────┬─────────┘  │
│       │               │                 │             │
│  ┌────▼───────────────▼─────────────────▼─────────┐  │
│  │              Database Layer                     │  │
│  │         core.db  │  logs.db  │  memory.db       │  │
│  └─────────────────────────────────────────────────┘  │
└──────────┬──────────────────────┬────────────────────┘
           │                      │
    ┌──────▼──────┐      ┌────────▼────────┐
    │  llama.cpp  │      │  llama.cpp      │
    │ Orchestrator│      │ Embedding Svc   │
    │  port 11434 │      │  port 11435     │
    └─────────────┘      └─────────────────┘
           
┌─────────────────────────────────────────────────────┐
│              Workspace Directory                     │
│                                                      │
│   ./workspace/          (agent execution root)        │
│   ./workspace/toolbox/  (Python tool scripts)         │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│                  Browser (UI)                        │
│         HTML / CSS / JS over HTTPS + WSS             │
└─────────────────────────────────────────────────────┘
```

---

## Components

### Rust Host

Single process. Owns all system state. Coordinates all other components.

**Responsibilities:**
- Launch and monitor orchestrator and embedding service processes
- Serve the chat UI over HTTPS (self-signed certificate, local network)
- Manage WebSocket connections for streaming and status messages
- Assemble context packages per `PB_Context_Assembly.md`
- Execute the generation loop per `PB_Generation_Loop.md`
- Parse and dispatch action tags per `PB_Action_Tag_System.md`
- Read and write all three databases
- Execute Python tools via host `python3` in `./workspace/toolbox/`
- Manage `./workspace/` directory structure

**Does not:**
- Accept connections from outside the local network
- Store any state outside the three SQLite databases and project root files

---

### Orchestrator Service

llama.cpp server (build 8661) running the user-configured GGUF model.

**Responsibilities:**
- Serve `POST /completion` — primary generation
- Serve `POST /tokenize` — token measurement for context assembly
- Serve `GET /health` — readiness check

**Configuration:** model path, port, context size, GPU/CPU offload, and all runtime settings are read from `model_configs` where `model_type='orchestrator'` at launch. See `PB_API_Contracts.md`.

---

### Embedding Service

llama.cpp server (build 8661) running Qwen3 0.6B GGUF with `--embedding` flag. CPU-only.

**Responsibilities:**
- Serve `POST /embedding` — generate vectors for memory storage and semantic search
- Serve `GET /health` — readiness check

Only launched when `embeddings_enabled=TRUE` in `app_settings`. If launch fails, system continues with importance-ranked memory fallback. See `PB_API_Contracts.md`.

---

### Workspace Directory

The `./workspace/` directory at the project root is the agent's execution environment. All agent file operations and Python tool execution occur here.

**Responsibilities:**
- Execution environment for all agent-initiated file operations
- Hosts `./workspace/toolbox/` — Python tool scripts for action tag handlers
- Agent has full control inside `./workspace/` — create, edit, move, recycle files and directories

**Rust interaction:** Python tools are executed via `python3 ./workspace/toolbox/<script>.py '<json-params>'` on the host. The TERMINAL action tag executes commands via `sh -c` scoped to `./workspace/`. See `PB_Action_Tag_System.md`.

---

### Database Layer

Three SQLite databases at the project data directory.

| Database | Contents |
|---|---|
| `core.db` | `app_settings`, `action_registry`, `personality` |
| `logs.db` | `chat_sessions`, `chat_logs`, `tool_calls`, `context_debug`, `llama_server_log` |
| `memory.db` | `memories`, `embeddings` |

All database access is exclusive to the Rust host. No other component reads or writes the databases directly. See `PB_Database_Schemas.md`.

---

### Web Server

Embedded in the Rust host. Serves the chat UI and WebSocket endpoint.

- HTTPS with self-signed certificate — local network access only
- Single bidirectional WebSocket connection per active session — carries streaming tokens and status (server→client) as well as chat messages, stop/continue commands, and history requests (client→server)
- Port configured via `app_settings.web_server_port` (default 9001)
- UI is static HTML/CSS/JS
- All UI data is read from `logs.db` via the Rust host — the UI never queries the database directly

---

## Project Root Files

| File | Purpose |
|---|---|
| `core_values.md` | Agent core values — read from disk at startup and cached in memory. Printed to terminal on launch. |
| `.env.system` | Environment variables for the Rust host and application |
| `./models/orchestrator/<model-name>/` | Orchestrator GGUF model files. Rust scans `./models/orchestrator/` and lists each `<model-name>` folder as a dropdown group, with `.gguf` files as selectable options. |
| `./models/embedding/<model-name>/` | Embedding GGUF model files. Same scanning logic applies. |

---

## Data Flow

### User Turn — Initial Generation

```
User submits message via WebSocket (type: "message")
  → Rust receives message, disables input, enables stop button
  → Rust assembles context (core_values, personality, registry, time, memories, history, user message)
  → Rust calls POST /completion (stream=true)
  → Tokens stream → Rust → WebSocket → UI
  → Generation completes
  → Rust strips thinking tokens, parses action tags, strips action tags from response
  → Rust dispatches tags → results collected → written to tool_calls
  → Rust writes chat_logs row
  → If tool results exist: ReAct loop (see below)
  → If REGENERATE parsed: REGENERATE handling (see below)
  → Turn complete → input re-enabled
```

### ReAct Loop

```
Tool results exist from previous generation
  → Rust increments iteration counter
  → Check against max_iterations
  → Reassemble context with tool results in section 7
  → Call POST /completion
  → Process response (strip, parse, dispatch)
  → Write chat_logs row
  → Repeat until no tool results and no REGENERATE
```

### REGENERATE

```
REGENERATE tag parsed (agent-initiated only)
  → Send msg= to UI if set
  → Wait delay= seconds
  → Reassemble context
  → Call POST /completion
  → Continue loop
```

### Memory Write

```
Agent outputs [ACTION:REMEMBER ...]
  → Dispatcher writes row to memories
  → If embeddings_enabled: POST /embedding → store vector in embeddings with source_type='memory'
  → If embedding fails: log warning, memory row still written, REINDEX can catch up later
```

### Chat Log Embedding

```
Assistant response written to chat_logs
  → If embeddings_enabled and embedding service running: POST /embedding → store vector in embeddings with source_type='chat_log'
  → If embedding fails: log warning, chat_log row still written, REINDEX can catch up later
```

### Memory Read (Automatic)

```
Context assembly — section 5
  → Embed current user message via POST /embedding
  → Cosine similarity search against embeddings
  → Top 5 results injected into context
  → If embeddings_enabled=FALSE: rank by importance_weight DESC
```

### Memory Read (Agent-Initiated)

```
Agent outputs [ACTION:RECALL ...]
  → Dispatcher queries memories or chat_logs
  → Results returned as tool result
  → Injected into next generation via ReAct loop
```

---

## Startup Sequence

1. Rust reads `app_settings` from `core.db` — creates with defaults if first run
2. Rust reads `model_configs` from `core.db` — creates with defaults if first run
3. Rust scans `./models/orchestrator/` and `./models/embedding/` directories, populates `model_registry` table
4. Rust checks port availability for orchestrator and embedding service
5. Rust ensures `./workspace/` and `./workspace/toolbox/` directories exist, seeds Python tool scripts if missing
6. Rust launches orchestrator process — polls `GET /health` until ready
7. If `embeddings_enabled=TRUE`: launch embedding service — poll `GET /health` until ready
8. Rust starts web server and WebSocket endpoint
9. UI becomes available to user

---

## Model Discovery

The Rust host maintains a registry of available GGUF models in the `model_registry` table. This avoids filesystem scans on every UI request and persists model metadata across restarts.

**Directory structure:**
```
models/
├── orchestrator/
│   ├── llama-3.2-3b/
│   │   ├── Q4_K_M.gguf
│   │   └── Q8_0.gguf
│   └── mistral-7b/
│       └── Q4_K_M.gguf
└── embedding/
    └── nomic-embed-text/
        └── nomic-embed-text-v1.Q4_K_M.gguf
```

**Discovery process:**
1. On startup, Rust scans both `./models/orchestrator/` and `./models/embedding/` directories
2. For each `<repo_name>/<filename>.gguf` found, inserts a row into `model_registry`
3. The `full_path` column stores the absolute path for verification
4. The `file_size_bytes` column stores the file size for UI display
5. Duplicate entries (same `model_type` + `repo_name` + `filename`) are ignored via UNIQUE constraint

**Refresh mechanism:**
- `POST /api/models/refresh` — triggers a re-scan of model directories
- Used when user adds new models without restarting the system
- Clears existing entries for the specified `model_type` and re-scans

---

## Shutdown Sequence

1. Complete or interrupt any active generation
2. Rust sends SIGTERM to orchestrator and embedding service
3. Wait up to 10 seconds for graceful exit — SIGKILL if timeout exceeded
4. Close database connections
5. Process exits

---

## Single-User Constraints

PolyBox is designed for single-user, private network use. The following are intentional constraints for MVP:

- One active chat session at a time
- One WebSocket connection at a time
- No authentication
- No multi-user access control
- HTTPS is self-signed — browser certificate warning is expected on first access
