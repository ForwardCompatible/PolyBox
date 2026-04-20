# PolyBox 2.0 — API Contracts

## Services

Two llama.cpp server instances (build 8661) run as child processes managed by the Rust host.

| Service | Model | Purpose | Default Port |
|---|---|---|---|
| Orchestrator | GGUF from `./models/orchestrator/` | Primary generation, tokenization | 11434 |
| Embedding | GGUF from `./models/embedding/` | Memory embedding and semantic search | 11435 |

Both ports are configurable in `app_settings`. The Rust host reads port values from the database on startup and uses them for all requests.

---

## Orchestrator Service

### Launch Command

```
llama-server \
  --model ./models/orchestrator/<model-name>/<selected.gguf> \
  --ctx-size <orchestrator_ctx_size> \
  --n-predict <output_token_reserve> \
  --port <orchestrator_port> \
  --host 127.0.0.1 \
  [user-configured gpu/cpu offload flags]
```

All launch parameters except port are derived from `app_settings`. Model path is resolved from `./models/orchestrator/` by scanning for `.gguf` files.

---

### POST /completion

Primary generation endpoint. Called once per generation turn including each ReAct loop iteration.

**Request:**
```json
{
  "prompt": "<fully assembled context string>",
  "stream": true,
  "n_predict": <output_token_reserve>,
  "stop": ["<|im_end|>"],
  "cache_prompt": true
}
```

- `prompt` contains the fully assembled context including system prompt, chat history, tool results, and user message.
- `stream` is always `true`. Tokens are streamed to the UI via WebSocket as they arrive.
- `n_predict` is read from `app_settings.output_token_reserve`.
- `stop` sequences are configured to prevent the model from generating the next user turn. Default: `["<|im_end|>"]` (ChatML). Hardcoded for MVP.
- `cache_prompt` enables KV cache reuse across turns where the prompt prefix is unchanged.

**Streaming Response:**

Each chunk is a server-sent event:
```
data: {"token":{"content":"<token_text>"},"stop":false}
data: {"token":{"content":""},"stop":true,"timings":{"predicted_n":<n>,"predicted_ms":<ms>,"prompt_n":<n>,"prompt_ms":<ms>}}
```

The Rust host:
1. Forwards each `content` token to the UI via WebSocket as it arrives
2. Accumulates the full response in memory
3. On `stop: true`, reads `timings` to derive `completion_tokens`, `generation_time_ms`, and `tokens_per_second`
4. Writes the completed generation to `chat_logs` after full response is received

**Derived metrics written to chat_logs:**
- `completion_tokens` — `timings.predicted_n`
- `generation_time_ms` — `timings.predicted_ms`
- `tokens_per_second` — `predicted_n / (predicted_ms / 1000)`
- `prompt_tokens` — `timings.prompt_n`

---

### POST /tokenize

Used to measure token counts during context assembly. Called once per context section during assembly to calculate the token budget.

**Request:**
```json
{
  "content": "<text to tokenize>"
}
```

**Response:**
```json
{
  "tokens": [<integer>, <integer>, ...]
}
```

Token count for a section = `tokens.length`.

The Rust host calls `/tokenize` on each context section independently during assembly. Total context tokens must not exceed `orchestrator_ctx_size - output_token_reserve`.

---

### GET /health

Used to verify the orchestrator is ready before accepting user input.

**Response (ready):**
```json
{ "status": "ok" }
```

**Response (loading):**
```json
{ "status": "loading model" }
```

The Rust host polls `/health` after launching the orchestrator process. UI displays a loading state until `status: "ok"` is returned. If the process exits before returning `ok`, Rust logs the event to `llama_server_log` with `event_type='crash'` and surfaces an error in the UI.

---

## Embedding Service

### Launch Command

```
llama-server \
  --model ./models/embedding/<model-name>/<selected.gguf> \
  --port <embedding_server_port> \
  --host 127.0.0.1 \
  --embedding \
  --ctx-size 512 \
  --cpu-only
```

- Always CPU-only. GPU resources are reserved for the orchestrator.
- Only launched when `embeddings_enabled=TRUE` in `app_settings`.
- If launch fails, Rust sets `embeddings_enabled=FALSE` for the session, logs the event, and continues. The system does not halt.

---

### POST /embedding

Generates a vector embedding for a given string. Called during `REMEMBER` (to embed new memories) and `RECALL` (to embed the search query).

**Request:**
```json
{
  "content": "<text to embed>"
}
```

**Response:**
```json
{
  "embedding": [<float>, <float>, ...]
}
```

The returned vector is stored as a BLOB in `embeddings.embedding`. Similarity ranking uses cosine similarity computed in Rust.

---

### GET /health

Same contract as orchestrator `/health`. Rust polls after launch before marking the embedding service as ready.

---

## Port Management

| Service | Column | Default |
|---|---|---|
| Orchestrator | `model_configs.port` where `model_type='orchestrator'` | 11434 |
| Embedding | `model_configs.port` where `model_type='embedding'` | 11435 |
| Web Server | `app_settings.web_server_port` | 9001 |

All services bind to `127.0.0.1` only. No external network exposure.

On startup, Rust reads port values from `model_configs` for llama.cpp services and `app_settings.web_server_port` for the web server. Rust checks all ports are free before launching. If a port is already in use, Rust logs the conflict and surfaces an error in the UI rather than attempting to bind.

---

## Service Lifecycle

### Startup Sequence
1. Rust reads port and model configuration from `model_configs`
2. Rust reads application configuration from `app_settings` (including `web_server_port`)
3. Rust checks port availability for all services (orchestrator, embedding, web server)
4. Rust starts web server on `web_server_port` — serves static UI files and WebSocket endpoint
5. Orchestrator process launched — Rust polls `/health` until ready
6. If `embeddings_enabled=TRUE`: embedding service launched — Rust polls `/health` until ready
7. UI unlocked for user input

### Shutdown Sequence
1. Rust sends SIGTERM to both service processes
2. Waits up to 10 seconds for graceful exit
3. If process has not exited: SIGKILL

### Crash Recovery
- If either service exits unexpectedly mid-session: Rust logs to `llama_server_log` with `event_type='crash'`
- UI displays service status error
- Rust does not attempt automatic restart in MVP
- User must restart the service via the UI

---

## Error Handling

| Condition | Rust behavior |
|---|---|
| Service unreachable on request | Return error to dispatcher, surface in UI |
| `/completion` returns non-200 | Log to `llama_server_log`, surface error in UI, do not write partial response to `chat_logs` |
| `/tokenize` returns non-200 | Halt context assembly, surface error in UI |
| `/embedding` returns non-200 | Fall back to importance ranking for this request, log warning |
| Service process exits unexpectedly | Log `event_type='crash'` to `llama_server_log`, surface in UI |

---

## WebSocket Contract

The Rust web server exposes a single WebSocket endpoint at `/ws`. The UI connects on page load and maintains a persistent connection. The WebSocket is bidirectional — it carries streaming tokens and status from server to client, and chat messages, stop/continue commands, and history requests from client to server.

The UI derives the WebSocket URL from `window.location` at runtime. No API call is needed to discover the port.

### Message Format

All messages are JSON with a `type` field:

```json
{ "type": "<message_type>", ...additional fields }
```

### Server → Client Types

Defined in `PB_Generation_Loop.md` §WebSocket Messages and `PB_UI_and_Logging.md` §WebSocket Message Handling.

### Client → Server Types

| Type | Payload | Purpose |
|---|---|---|
| `message` | `{ "type": "message", "content": "<user text>" }` | User submits a chat message |
| `stop` | `{ "type": "stop" }` | User presses stop button |
| `continue` | `{ "type": "continue" }` | User continues after max iterations |
| `get_history` | `{ "type": "get_history", "session_id": "...", "limit": N }` | Request chat history for a session |

### Rust Handling of Client Messages

- `message`: Triggers the full generation loop (see `PB_Generation_Loop.md` §Turn Lifecycle)
- `stop`: Cancels current generation (see `PB_Generation_Loop.md` §Stop Button)
- `continue`: Resets iteration counter and resumes the loop (see `PB_Generation_Loop.md` §Max Iterations)
- `get_history`: Queries `chat_logs` for the requested session, responds with `chat_history` type

### Connection Lifecycle

- UI connects on page load
- If the connection drops, the UI attempts reconnection with exponential backoff
- The Rust host sends a `turn_complete` or `error` message when the current operation finishes
- The Rust host does not persist WebSocket state — reconnection starts fresh
