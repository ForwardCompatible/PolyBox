# PolyBox 2.0 — Context Assembly Specification

## Overview

The Rust host assembles a complete context string before every generation call. This includes the initial user turn and every subsequent ReAct loop iteration. The assembled string is passed as the `prompt` field to `POST /completion`.

Context is assembled fresh each generation. Nothing is cached at the application level — KV cache reuse is handled by llama.cpp via `cache_prompt: true`.

---

## Token Budget

```
available_tokens = orchestrator_ctx_size - output_token_reserve
```

Both values are read from `app_settings`. Token counts are measured by calling `POST /tokenize` on the Rust host against the orchestrator service. No estimation or approximation is used.

Assembly is rejected and an error surfaced to the UI if the minimum required sections (1–5 + user message) exceed `available_tokens`.

---

## Assembly Order

Sections are assembled in this fixed order. Order is not configurable.

| Position | Section | Source |
|---|---|---|
| 1 | Core Values | `core_values.md` at project root |
| 2 | Personality | `personality` table, all enabled sections in `section` order |
| 3 | Action Registry | `action_registry` table, all enabled tags |
| 4 | Current Time | System clock, formatted with `app_settings.user_timezone` |
| 5 | Memory Summary | Semantic retrieval against `memories` (see §4) |
| 6 | Chat History | Recent turns from `chat_logs` (see §5) |
| 7 | Tool Results | Accumulated results from current ReAct loop iteration (see §6) |
| 8 | User Message | Current user input |

Sections 1–5 are fixed and always included in full. Sections 6–7 are variable. Section 8 is always included in full.

If sections 1–5 and section 8 together exceed `available_tokens`, assembly halts and an error is returned. This condition indicates a configuration problem (context window too small for the system prompt).

---

## Section Specifications

### 1. Core Values

- Read from `core_values.md` at the project root on startup and cached in memory.
- To update core values, edit `core_values.md` and restart the application.
- Displayed to the user in the terminal on system launch.

### 2. Personality

- All rows from `personality` table, ordered by `section` column ascending.
- Each section rendered as:
```
## <section>
<content>
```
- `{user_name}` and `{agent_name}` placeholders in `content` are substituted with values from `app_settings` before inclusion.

### 3. Action Registry

- All rows from `action_registry` where `enabled=TRUE`.
- Each entry rendered as:
```
[ACTION:<tag>]
Description: <description>
Parameters: <parameters JSON>
```
- Disabled tags are excluded entirely. The agent has no knowledge of disabled tags.

### 4. Current Time

- Single line rendered as:
```
Current time: <ISO 8601 datetime in user_timezone>
```
- Timezone read from `app_settings.user_timezone`.

### 5. Memory Summary

- Triggered automatically on every generation. Not agent-initiated.
- Query: embed the current user message via `POST /embedding`, retrieve top 5 memories by cosine similarity from `memories`.
- If `embeddings_enabled=FALSE`: retrieve top 5 memories ordered by `importance_weight DESC`, `created_at DESC`.
- Rendered as a numbered list:
```
Relevant memories:
1. <content> (importance: <importance_weight>)
2. ...
```
- If no memories exist, this section is omitted entirely.

### 6. Chat History

- Retrieved from `chat_logs` for the current `session_id`.
- Only `role='user'` and `role='assistant'` rows. `turn_type='regeneration'` rows are excluded.
- Maximum turns: `chat_history_turn_limit` from `app_settings` (default: 10). One turn = one user message + one assistant response.
- Oldest messages trimmed first if token budget requires further reduction after turn limit is applied.
- Each turn rendered as:
```
User: <response_content>
Assistant: <response_content>
```
- `response_content` is used — clean text only, action tags and thinking tokens already stripped at write time.

### 7. Tool Results

- Only present during ReAct loop iterations (not on the initial generation).
- Contains the accumulated results of all action tags executed in the previous generation.
- Rendered as:
```
Tool results:
[ACTION:<tag_name>]: <result JSON>
[ACTION:<tag_name>]: <result JSON>
```
- All results from the previous iteration are included regardless of success or failure.
- On the initial user turn this section is empty and omitted.

### 8. User Message

- The raw user input for the current turn.
- Always the final section.
- Rendered as:
```
User: <message>
```

---

## Token Measurement

Token counts per section are measured by calling `POST /tokenize` on the rendered section string. Counts are used for:

1. Budget enforcement during assembly
2. Writing to `context_debug` when `store_context_debug=TRUE`

Measurement calls are made in assembly order. Assembly halts immediately if the running total would exceed `available_tokens` before a required section is included.

---

## Trimming Strategy

When total assembled tokens would exceed `available_tokens`:

1. Reduce chat history turn count by removing the oldest turn pair (user + assistant) and re-measuring.
2. Repeat until either the budget is met or chat history is exhausted.
3. If budget is still exceeded with zero chat history turns, assembly halts and an error is returned.

Sections 1–5 and section 8 are never trimmed.

---

## context_debug Write

When `store_context_debug=TRUE` in `app_settings`, after every successful assembly Rust writes one row to `context_debug` with:

- `chat_log_id` — the `chat_logs.id` for this generation (written after generation completes)
- `session_id`
- Token count per section column
- `tokens_total` — sum of all section token counts
- `injected_memory_ids` — comma-separated `memories.id` values injected in section 5

The `chat_log_id` is set after the generation completes and the `chat_logs` row is written. The `context_debug` row is inserted in the same transaction.

---

## ReAct Loop Context Behavior

On each ReAct loop iteration, context is fully reassembled from scratch with one addition: section 7 (Tool Results) is populated with results from the previous iteration.

Chat history does not change between ReAct iterations within the same turn. The user message remains section 8 unchanged. Only section 7 changes between iterations.

---

## Thinking Token Handling

If `thinking_open_tag` and `thinking_close_tag` are set in `app_settings`:

- The Rust host strips content between these tags from the completed generation before writing `response_content` to `chat_logs`.
- Stripped thinking content is written to `reasoning_content` in `chat_logs`.
- Thinking content is never included in chat history passed back into context.
- Thinking content is never shown in the chat UI inline — it is displayed separately in a collapsible block.
