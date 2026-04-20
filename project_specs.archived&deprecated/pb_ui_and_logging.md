# PolyBox 2.0 — UI and Logging Specification

## Technology

Static HTML, CSS, and JavaScript served by the Rust host over HTTPS (self-signed certificate). WebSocket connections are established to the same Rust host. The UI never queries the database directly — all data is fetched via the Rust host.

**UI Stack:**
- **Tailwind CSS** via CDN — utility-first styling, no build pipeline, no npm, single script tag. Chosen for zero installation overhead and full compatibility with Rust-served static files.
- **Shoelace** via CDN — pre-built web components for interactive elements (modals, tabs, toggles, dropdowns). Single script tag.
- No frontend framework. No build step. No node_modules.

**JavaScript Architecture:**
- **ES Modules** — all JavaScript files use `type="module"` and native `import`/`export` syntax. No build step required (native browser support since ES2015).
- **Single entry point** — `web/js/main.js` imports and orchestrates all other modules. Loaded via `<script type="module" src="js/main.js">` in `index.html`.
- **Module responsibility:**
  - `main.js` — entry point, initializes router and WebSocket on DOMContentLoaded
  - `api.js` — all REST API calls (exported functions: `fetchModels`, `refreshModels`, `fetchModelConfigs`, `saveModelConfigs`, `startService`, `stopService`, `fetchServiceStatus`, `fetchAppSettings`, `saveAppSettings`, `fetchActionRegistry`, `toggleActionTag`, `createDatabaseBackup`, `fetchDatabaseBackups`, `restoreDatabase`)
  - `websocket.js` — WebSocket connection management (exported functions: `initWebSocket`, `sendMessage`, `closeWebSocket`, `addMessageHandler`)
  - `router.js` — SPA routing, imports page modules, calls their init functions (exported function: `initRouter`)
  - `system-orch.js` — orchestrator tab UI logic (exported functions: `initOrchestratorTab`, `stopOrchestratorPolling`)
  - `system-embed.js` — embedding tab UI logic (exported functions: `initEmbeddingTab`, `stopEmbeddingPolling`)
  - `settings.js` — settings page UI logic (exported function: `initSettingsPage`)
  - `database.js` — database backup/restore UI logic (exported function: `initDatabaseTab`)
  - `action-modal.js` — action tag modal logic (exported function: `initActionModal`)
  - `sidebar.js` — sidebar navigation logic (exported function: `initSidebar`)
- **No global scope pollution** — modules do not assign to `window` or declare globals that conflict with other modules. Each module's variables are scoped to that module.
- **Import pattern** — modules that need shared functionality import from `api.js` or `websocket.js` directly.

## Color Palette & Aesthetic

Dark mode only for MVP. Light mode is a post-MVP enhancement.

The aesthetic is warm and alive — deep near-black backgrounds with amber-orange accent that glows against them. Not clinical. Not cold.

| Role | Tailwind Class | Usage |
|---|---|---|
| Background | `zinc-950` | Page background |
| Surface | `zinc-900` | Cards, panels, sidebars |
| Surface elevated | `zinc-800` | Inputs, modals, hover states |
| Accent | `orange-400` / `amber-500` | Primary actions, status indicators, highlights |
| Text primary | `zinc-100` | Main readable text |
| Text secondary | `zinc-400` | Labels, metadata, secondary info |
| Error | `red-400` | Error states |
| Success | `green-400` | Success states |

Light mode inverts the background scale while preserving the orange accent. Post-MVP.

---

## Global Layout

### Left Navigation Menu

Collapsible vertical sidebar. When collapsed, a centered visual handle indicates it can be expanded. Two navigation items:

- **Chat** (top)
- **System** (below Chat)

### Footer (always visible)

Three zones:

- **Left:** Empty (reserved). Settings navigation is in the sidebar.
- **Center:** Hardware monitor — CPU %, RAM used/total GB, VRAM used/total GB. Updated via WebSocket push every 3 seconds from Rust host using `sysinfo` (CPU/RAM) and `nvml-wrapper` (VRAM). Not persisted to database.
- **Right:** Two service status indicators — `ORCH` and `EMBED`. Each is a small circular light: green = running, red = stopped. Updated via WebSocket on `llama_server_log` events (`started`, `stopped`, `crash`).

---

## Pages

---

### Chat Page

#### Layout

- Left: collapsible navigation menu
- Center: chat history + floating input bar
- Right: collapsible session panel (single session for MVP — reserved for future multi-session support)

#### Chat History

- Displays turns from `chat_logs` for the active `session_id`
- `role='user'` turns rendered as user messages
- `role='assistant'` turns rendered as agent messages
- `turn_type='regeneration'` rows are not displayed — internal loop iterations are hidden from the user
- Interrupted responses (`interrupted=TRUE`) displayed with an "RESPONSE INTERRUPTED" label
- Chat history scrolls upward. New messages append at the bottom.

#### Thinking Display

- If `show_reasoning=TRUE` in `app_settings`: reasoning content is displayed in a collapsible block above the agent response
- Collapsed by default if `reasoning_collapsed_default=TRUE`
- Label: "Reasoning" with a toggle chevron
- Thinking content is visually distinct from the response (different background, monospace or muted style)

#### Floating Input Bar

- Fixed position. Does not scroll with chat history.
- On desktop: anchored to bottom of chat area
- On mobile: locked to bottom of screen
- Chat history bottom padding ensures the last message is never obscured by the input bar
- When the user scrolls up through history, the input bar remains fixed above the viewport bottom

**Left of input field:** `+` icon — opens Action Tag Modal (see below)

**Input field:** text area, expands vertically with content

**Right of input field:** Send button (disabled during generation) and Stop button (enabled during generation only)

#### Action Tag Modal

Opened via `+` icon. Displays all rows from `action_registry`.

- Each tag shown with its name and description
- **Enabled** (`enabled=TRUE`): tag name displayed in green
- **Disabled** (`enabled=FALSE`): tag name displayed in gray
- Toggling a tag writes `enabled=TRUE/FALSE` directly to `action_registry` in `core.db`
- Changes take effect on the next generation

#### Document Upload

Two upload mechanisms:
- Paperclip icon in the input bar area
- Drag and drop onto the chat window

File uploads use HTTP POST (multipart/form-data) to the Rust host — not the WebSocket. The WebSocket is reserved for chat messages and streaming tokens.

**On upload:**
1. File written to `./workspace/chat_docs/` with filename formatted as `<name>_<YYYY-MM-DD_HHmm>.<ext>` (zero-padded, 24hr)
   - Example: `report_2026-04-09_1741.md`
2. A file attachment indicator appears in the user message area showing the filename as a link
3. The agent receives the file path in context so it can use `FILE_READ` to access the content

Supported file types for MVP: any file the agent can meaningfully read as text. No binary parsing.

#### WebSocket Message Handling

The chat UI maintains a persistent WebSocket connection during active sessions. The WebSocket is bidirectional — it carries both server-to-client streaming data and client-to-server commands. The UI derives the WebSocket URL from `window.location` at runtime (self-discovering, no API fetch needed).

**Server → Client messages:**

| Message Type | UI Behavior |
|---|---|
| `token` | Append token content to current agent message bubble |
| `status` | Display status text below current message bubble |
| `reasoning_start` | Open reasoning collapsible block, begin appending to it |
| `reasoning_end` | Close reasoning collapsible block |
| `turn_complete` | Finalize message, re-enable send button, disable stop button |
| `interrupted` | Append "RESPONSE INTERRUPTED" label, re-enable send button |
| `error` | Display error message in chat, re-enable send button |
| `continuation_prompt` | Display "Maximum iterations reached. Continue?" with Continue / Stop buttons inline in chat |
| `chat_history` | Populate chat history from server response — used on page load |
| `hardware_monitor` | Update footer CPU/RAM/VRAM stats |
| `service_status` | Update footer ORCH/EMBED status dots |

**Client → Server messages:**

| Message Type | Payload | When |
|---|---|---|
| `message` | `{ content: "..." }` | User submits chat message |
| `stop` | `{}` | User presses stop button |
| `continue` | `{}` | User presses Continue after max iterations |
| `get_history` | `{ session_id: "default", limit: 10 }` | Chat page loads, requests history |

---

### System Page

Four tabs with left-side tab navigation. A status section at the top shows current service states.

---

#### Tab 1: Orchestrator

Model configuration for the primary generation model. Organized into two card sections.

**Section 1: Model Selection**

| Setting | maps to `model_configs` column | Input type |
|---|---|---|
| Model | Populated from `model_registry` table | Grouped dropdown — `repo_name` as header, `filename` as options |
| Port | `port` | Number input |
| Context size | `ctx_size` | Number input |
| Auto-start | `auto_start` | Checkbox |

**Section 2: Launch Configuration**

| Setting | maps to `model_configs` column | Input type |
|---|---|---|
| GPU layers | `n_gpu_layers` | Number input (-1 = all layers) |
| Cache type K | `cache_type_k` | Dropdown: N/A, f16, q8_0, q4_0 |
| Cache type V | `cache_type_v` | Dropdown: N/A, f16, q8_0, q4_0 |
| Flash attention | `flash_attn` | Checkbox |
| Temperature | `temperature` | Number input (0.0 - 2.0) |
| Repeat penalty | `repeat_penalty` | Number input (1.0 - 2.0) |
| Cache in RAM | `cache_ram` | Checkbox |

**Actions:**
- **Save** — writes all settings to `model_configs` where `model_type='orchestrator'`
- **Start / Stop** — starts or stops the orchestrator process. Status reflected in footer indicator.
- **Auto-start checkbox** — if checked, orchestrator launches automatically on next system startup using saved settings

---

#### Tab 2: Embedding

Model configuration for the embedding service.

**Controls:**

| Setting | maps to `model_configs` column | Input type |
|---|---|---|
| Model | Populated from `model_registry` table | Grouped dropdown — `repo_name` as header, `filename` as options |
| Port | `port` | Number input |
| Context size | `embedding_ctx_size` | Number input |
| DIM | `dim` | Number input (required for embedding models) |
| Auto-start | `auto_start` | Checkbox |

**Actions:**
- **Save** — writes settings to `model_configs` where `model_type='embedding'`
- **Start / Stop** — starts or stops the embedding service
- **Auto-start checkbox** — if checked, embedding service launches on next system startup

Note: embedding service always runs CPU-only. No GPU settings exposed.

---

#### Tab 3: Container

Podman sandbox management. Full UI controls are present but disabled until Podman integration is complete.

**Controls and Actions (disabled until post-MVP):**

- **Start / Stop** — starts or stops `polybox-sandbox`. Auto-start checkbox saves preference.
- **Auto-start checkbox** — container starts automatically on next system startup if checked
- **Backup** — triggers `podman export polybox-sandbox` to `backups/container/container_<YYYY-MM-DD_HHmm>.tar`. Container must be stopped first. UI enforces this — backup button disabled while container is running.
- **Restore** — dropdown list of available `.tar` files in `backups/container/`. Selecting one and confirming triggers `podman import`. Container must be stopped. Does not affect `.env.container`.

See `PB_Podman.md` for the full specification.

---

#### Tab 4: Database

Database backup operations.

**Controls and Actions:**

- **Backup** — copies all three database files to `backups/db/db_<YYYY-MM-DD_HHmm>/` as a timestamped folder containing `core.db`, `logs.db`, `memory.db`
- **Restore** — dropdown list of available backup folders in `backups/db/`. Selecting one and confirming replaces live database files. Rust host must restart after restore.

---

### Settings Page

User-facing application settings. All values read from and written to `app_settings` in `core.db`.

**Controls:**

| Setting | `app_settings` column | Input type |
|---|---|---|
| Agent name | `agent_name` | Text input |
| User name | `user_name` | Text input |
| Timezone | `user_timezone` | Dropdown (IANA timezone list) |
| Max iterations | `max_iterations` | Number input (0 = unlimited) |
| Show reasoning | `show_reasoning` | Checkbox |
| Reasoning collapsed by default | `reasoning_collapsed_default` | Checkbox |
| Chat history turn limit | `chat_history_turn_limit` | Number input |
| Output token reserve | `output_token_reserve` | Number input |
| Thinking open tag | `thinking_open_tag` | Text input |
| Thinking close tag | `thinking_close_tag` | Text input |
| Store context debug | `store_context_debug` | Checkbox |
| Embeddings enabled | `embeddings_enabled` | Checkbox |
| Web server port | `web_server_port` | Number input |
| Health check timeout | `health_check_timeout_secs` | Number input (seconds, minimum 30) |

**Save button** — writes all values to `app_settings` in a single transaction.

---

## REST API Endpoints

All API endpoints return JSON with a consistent structure:

**Success:**
```json
{ "success": true, "data": { ... } }
```

**Failure:**
```json
{ "success": false, "error": "description" }
```

---

### Services

| Endpoint | Method | Purpose |
|---|---|---|
| `/api/services/:type/status` | GET | Get status for orchestrator or embedding service |
| `/api/services/:type/start` | POST | Start the specified service |
| `/api/services/:type/stop` | POST | Stop the specified service |

`:type` is either `orchestrator` or `embedding`.

**GET /api/services/:type/status response:**
```json
{
  "success": true,
  "data": {
    "running": true,
    "model": "model-name",
    "port": 11434,
    "uptime_seconds": 3600
  }
}
```

---

### Models

| Endpoint | Method | Purpose |
|---|---|---|
| `/api/models` | GET | List all discovered models |
| `/api/models?type=:type` | GET | List models for a specific type |
| `/api/models/refresh` | POST | Trigger filesystem scan for new models |

**GET /api/models response:**
```json
{
  "success": true,
  "data": {
    "orchestrator": [
      { "repo_name": "llama-3", "filename": "llama-3-8b.q4_k_m.gguf", "full_path": "./models/orchestrator/llama-3/llama-3-8b.q4_k_m.gguf", "file_size_bytes": 4337231000 }
    ],
    "embedding": [
      { "repo_name": "qwen3", "filename": "qwen3-0.6b.gguf", "full_path": "./models/embedding/qwen3/qwen3-0.6b.gguf", "file_size_bytes": 1234567890 }
    ]
  }
}
```

---

### Model Configs

| Endpoint | Method | Purpose |
|---|---|---|
| `/api/model-configs/:type` | GET | Get config for orchestrator or embedding |
| `/api/model-configs/:type` | PUT | Replace entire config for that type |

`:type` is either `orchestrator` or `embedding`.

**PUT /api/model-configs/:type request:**
```json
{
  "model_path": "./models/orchestrator/llama-3/llama-3-8b.q4_k_m.gguf",
  "port": 11434,
  "auto_start": true,
  "temperature": 0.7,
  "top_k": 40,
  "ctx_size": 256000,
  "n_gpu_layers": 35,
  "flash_attn": true
}
```

All model_configs columns are accepted. Omitted fields retain current values.

---

### App Settings

| Endpoint | Method | Purpose |
|---|---|---|
| `/api/app-settings` | GET | Get all app_settings values |
| `/api/app-settings` | PUT | Replace all app_settings values |

**PUT /api/app-settings request:**
```json
{
  "agent_name": "PolyBox",
  "user_name": "User",
  "max_iterations": 5,
  "show_reasoning": true
}
```

All app_settings columns except `id` and `updated_at` are accepted.

---

### Action Registry

| Endpoint | Method | Purpose |
|---|---|---|
| `/api/action-registry` | GET | List all action tags |
| `/api/action-registry/:id/toggle` | PUT | Toggle enabled state for a tag |

**PUT /api/action-registry/:id/toggle request:**
```json
{ "enabled": false }
```

---

### Database Backup/Restore

| Endpoint | Method | Purpose |
|---|---|---|
| `/api/database/backup` | POST | Create timestamped backup of all three databases |
| `/api/database/backups` | GET | List available backup folders |
| `/api/database/restore` | POST | Restore from a specified backup folder |

**POST /api/database/backup response:**
```json
{
  "success": true,
  "data": {
    "backup_path": "backups/db/db_2026-04-13_1405/",
    "files": ["core.db", "logs.db", "memory.db"]
  }
}
```

**POST /api/database/restore request:**
```json
{ "backup_folder": "db_2026-04-13_1405" }
```

---

### Chat History

| Endpoint | Method | Purpose |
|---|---|---|
| `/api/chat/:session_id` | GET | Get chat logs for a session |
| `/api/chat/:session_id` | POST | Submit a user message |

**GET /api/chat/:session_id response:**
```json
{
  "success": true,
  "data": {
    "session_id": "default",
    "messages": [
      { "id": 1, "role": "user", "response_content": "Hello", "timestamp": "2026-04-13T14:00:00Z" },
      { "id": 2, "role": "assistant", "response_content": "Hi there!", "timestamp": "2026-04-13T14:00:05Z" }
    ]
  }
}
```

---

## WebSocket Messages

The UI maintains a persistent WebSocket connection during active sessions. All messages are JSON with a `type` field.

### Server-to-Client Messages

| Type | Payload | When |
|---|---|---|
| `hardware_monitor` | `{ cpu_percent, ram_used_gb, ram_total_gb, vram_used_gb, vram_total_gb }` | Every 3 seconds |
| `service_status` | `{ service_type, running }` | On service start/stop/crash |
| `token` | `{ content }` | Each streamed token |
| `status` | `{ message }` | REGENERATE msg, tool execution status |
| `reasoning_start` | `{}` | Thinking block begins |
| `reasoning_end` | `{}` | Thinking block ends |
| `turn_complete` | `{}` | Turn fully complete |
| `interrupted` | `{}` | Stop button pressed |
| `error` | `{ message }` | Any error condition |
| `max_iterations` | `{}` | Max iterations reached |
| `continuation_prompt` | `{ message }` | Awaiting user Continue/Stop response |

### Client-to-Server Messages

| Type | Payload | When |
|---|---|---|
| `stop` | `{}` | User presses stop button |
| `continue` | `{}` | User presses Continue after max_iterations |
| `message` | `{ content }` | User submits a chat message |

---

## Logging

All logging is handled exclusively by the Rust host. The UI reads log data via Rust — never directly.

### What Gets Logged Where

| Event | Table | Database |
|---|---|---|
| User message | `chat_logs` | `logs.db` |
| Agent generation | `chat_logs` | `logs.db` |
| Action tag execution + result | `tool_calls` | `logs.db` |
| Context token breakdown | `context_debug` | `logs.db` (when enabled) |
| llama-server lifecycle event | `llama_server_log` | `logs.db` |
| Memory written by agent | `memories` | `memory.db` |
| Memory embedding | `embeddings` | `memory.db` |
| Action tag registered | `action_registry` | `core.db` |
| Personality section updated | `personality` | `core.db` |

### What Is Never Logged

- Raw assembled context (unless `store_context_debug=TRUE`)
- Thinking tokens in `response_content` — stored only in `reasoning_content`
- Action tags in `response_content` — stripped before write
- Hardware stats — pushed via websocket

---

## First Run

On first launch, `app_settings` and `model_configs` are created with defaults. The user is directed to the System page to configure and start the orchestrator before the chat page becomes usable. Chat input is disabled until the orchestrator service returns `status: "ok"` from `GET /health`.
