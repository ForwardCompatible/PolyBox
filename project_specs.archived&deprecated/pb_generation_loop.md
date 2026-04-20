# PolyBox 2.0 — Generation Loop Specification

## Overview

The generation loop is the core turn lifecycle managed entirely by the Rust host. It begins when the user submits a message and ends when no further generation is required. The UI is updated throughout via WebSocket.

---

## Turn Lifecycle

### 1. User Input

- User submits a message via the UI over the WebSocket connection (client sends `{ type: "message", content: "..." }`).
- Send button is disabled immediately. User may continue typing but cannot submit.
- Stop button is enabled.
- Rust receives the WebSocket message, extracts the content, and writes the user message to `chat_logs`:
  - `role='user'`
  - `turn_type='user'`
  - `response_content=<message>`
  - `loop_iteration=0`

### 2. Context Assembly

- Rust assembles the full context package per `PB_Context_Assembly.md`.
- If assembly fails, an error is surfaced in the UI and the turn ends. Input is re-enabled.

### 3. Generation

- Rust calls `POST /completion` on the orchestrator with `stream=true`.
- Tokens are forwarded to the UI via WebSocket as they arrive.
- The full response is accumulated in memory.
- On completion, Rust reads `timings` from the final SSE chunk.

### 4. Response Processing

After generation completes:

1. Strip thinking tokens — content between `thinking_open_tag` and `thinking_close_tag` is removed from the response and stored separately.
2. Parse all action tags from the remaining response text.
3. Strip action tags from the visible response text.
4. The cleaned response and reasoning content are held in memory. Nothing is written to `chat_logs` yet.

### 5. Tag Dispatch

- Tags execute per the rules in `PB_Action_Tag_System.md`.
- Write operations execute first, then all other non-`REGENERATE` tags, then `REGENERATE` last.
- Each tag execution writes one row to `tool_calls`.
- All results are accumulated.

### 6. Generation Complete — Write to DB

After all non-`REGENERATE` tags have executed, Rust writes the generation to `chat_logs`:

- `role='assistant'`
- `turn_type='assistant'` for the first generation, `'regeneration'` for subsequent ReAct iterations
- `reasoning_content` — stripped thinking tokens
- `response_content` — cleaned response text, action tags stripped
- `prompt_tokens`, `completion_tokens`, `tokens_per_second`, `generation_time_ms` — from `/completion` timings
- `loop_iteration` — current iteration count (0-based)
- `interrupted=FALSE`

If `store_context_debug=TRUE`, write `context_debug` row in the same transaction.

**Chat log embedding:** After writing the `chat_logs` row, if `embeddings_enabled=TRUE` and the embedding service is running, generate an embedding for `response_content` and INSERT into `embeddings` with `source_type='chat_log'` and `source_id=<chat_log_id>`. If embedding generation fails, log a warning and continue — the row is still written without an embedding. REINDEX can add the embedding later.

### 7. ReAct Loop

If tool results were collected in step 5:

1. Increment loop iteration counter.
2. Check loop iteration counter against `max_iterations` (see §Max Iterations).
3. Reassemble context with tool results injected into section 7.
4. Return to step 3 (Generation).

If no tool results exist and no `REGENERATE` tag was parsed, the turn is complete. Proceed to step 9.

### 8. REGENERATE Handling

If a `REGENERATE` tag was parsed:

1. If `msg` is set: send status message to UI via WebSocket.
2. Wait `delay` seconds. **Stop button remains active during this delay.** If stop is pressed: cancel pending re-entry, turn ends. Proceed to step 9.
3. Increment loop iteration counter.
4. Check loop iteration counter against `max_iterations`.
5. Reassemble context. Section 7 is empty unless tool results also exist from this iteration.
6. Return to step 3 (Generation).

### 9. Turn Complete

- Stop button disabled.
- Send button re-enabled.
- Any pre-typed user message remains in the input field.

---

## Max Iterations

`max_iterations` is read from `app_settings` at the start of each turn. `0` = unlimited — the iteration check is skipped entirely.

When the loop iteration counter reaches `max_iterations`:

1. The current generation is written to `chat_logs` normally.
2. Rust sends a continuation prompt to the UI via WebSocket:
   > "Maximum iterations reached. Continue?"
3. UI presents Continue / Stop buttons. Send button remains disabled.
4. **If Continue:** reset the iteration counter to 0 and resume the loop for another full `max_iterations` block.
5. **If Stop:** turn ends. Proceed to step 9.

The iteration counter resets to 0 on each continuation. Each block runs the full `max_iterations` count before prompting again.

---

## Stop Button

The stop button is available whenever a generation is in progress, and remains active during REGENERATE delay periods.

**When pressed during generation:**

1. Rust sends a cancellation signal to the orchestrator — closes the streaming connection.
2. Whatever response has been accumulated to that point is processed as if generation completed:
- Thinking tokens stripped.
- Action tags parsed and dispatched from the partial response.
- Cleaned partial response written to `chat_logs` with `interrupted=TRUE`.
3. No further ReAct iterations are triggered regardless of parsed tags.
4. If a `REGENERATE` tag was present in the partial response, it is discarded.
5. Turn ends. Proceed to step 9.

**When pressed during REGENERATE delay:**

1. Cancel the pending re-entry. No new generation is triggered.
2. Turn ends. Proceed to step 9.

---

## Error States

### Context Assembly Failure
- Error surfaced in UI.
- Nothing written to `chat_logs`.
- Turn ends. Input re-enabled.

### Orchestrator Unreachable
- Error surfaced in UI.
- Nothing written to `chat_logs`.
- Turn ends. Input re-enabled.

### Generation Returns Non-200
- Error logged to `llama_server_log`.
- Error surfaced in UI.
- Partial response discarded.
- Turn ends. Input re-enabled.

### Tag Execution Failure
- Failure returned as tool result for that tag.
- Logged to `tool_calls` with `success=FALSE`.
- Turn continues. Failure injected into next ReAct iteration so agent can respond.
- Turn does not halt on individual tag failure.

### Orchestrator Crash Mid-Generation
- Rust detects process exit.
- Logs `event_type='crash'` to `llama_server_log`.
- Partial response discarded. Nothing written to `chat_logs`.
- Error surfaced in UI.
- Turn ends. Input re-enabled.
- Rust does not attempt automatic restart.

---

## WebSocket Messages

The WebSocket connection is bidirectional. The Rust host sends server-to-client messages during a turn and receives client-to-server commands from the UI.

**Server → Client:**

| Type | Payload | When |
|---|---|---|
| `token` | `{ "content": "<token>" }` | Each streamed token |
| `status` | `{ "message": "<text>" }` | REGENERATE `msg=`, tool execution status |
| `reasoning_start` | `{}` | Thinking token block begins |
| `reasoning_end` | `{}` | Thinking token block ends |
| `turn_complete` | `{}` | Turn fully complete |
| `interrupted` | `{}` | Stop button pressed, generation halted |
| `error` | `{ "message": "<text>" }` | Any error condition |
| `max_iterations` | `{}` | Max iterations reached, awaiting user decision |
| `continuation_prompt` | `{ "message": "Maximum iterations reached. Continue?" }` | Sent when loop hits max_iterations, awaiting user Continue/Stop response |
| `chat_history` | `{ "messages": [...] }` | Response to `get_history` — chat log rows for the requested session |

**Client → Server:**

| Type | Payload | When |
|---|---|---|
| `message` | `{ "content": "<user text>" }` | User submits a chat message |
| `stop` | `{}` | User presses the stop button |
| `continue` | `{}` | User presses Continue after max iterations prompt |
| `get_history` | `{ "session_id": "...", "limit": N }` | UI requests chat history for a session |

---

## Iteration Counter Behavior

- Counter starts at 0 on each new user turn.
- Incremented once per generation after the first.
- First generation of a turn is iteration 0.
- Counter is reset to 0 when the user confirms continuation after `max_iterations` is reached.
- Counter is local to the current turn. It is not persisted — `chat_logs.loop_iteration` records the iteration number per row.
