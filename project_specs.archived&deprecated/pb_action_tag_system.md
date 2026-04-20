# PolyBox 2.0 — Action Tag System Specification

## 1. Tag Grammar

```
tag        ::= "[ACTION:" name (S param)* "]"
name       ::= [A-Z_]+
param      ::= key "=" DQUOTE value DQUOTE
key        ::= [a-z_]+
value      ::= [^"]*
S          ::= [ \t]+
```

- Tag names are uppercase with underscores only.
- Parameter keys are lowercase with underscores only.
- Parameter values are always double-quoted.
- Tags may appear anywhere in agent output.
- Multiple tags may appear in a single generation.
- Tags are parsed in order of appearance after generation completes.

---

## 2. action_registry Schema (reference)

```sql
tag            TEXT NOT NULL UNIQUE      -- Matches tag name in grammar above
description    TEXT NOT NULL            -- Injected into agent context
parameters     TEXT NOT NULL            -- JSON schema (see §3)
enabled        BOOLEAN DEFAULT TRUE     -- Disabled tags are invisible to agent
handler        TEXT NOT NULL            -- 'system' | './workspace/toolbox/<script>.py'
execution_mode TEXT DEFAULT 'immediate' -- 'immediate' | 'async' | 'persistent'
```

All built-in tags have `handler='system'` and `execution_mode='immediate'`.

Agent-registered tags have `handler='./workspace/toolbox/<script>.py'` and `execution_mode='immediate'`.

The dispatcher uses the `handler` value to determine execution path — `'system'` invokes the native Rust handler; any other value is treated as a path to a Python script executed via `python3 <handler> '<json-params>'` on the host. Python tool scripts live in `./workspace/toolbox/`.

---

## 3. Parameter Schema Format

The `parameters` column in `action_registry` contains a JSON array:

```json
[
  {
    "name": "<parameter_name>",
    "type": "<string|integer|float|boolean>",
    "required": true,
    "default": null,
    "description": "<what this parameter does>"
  }
]
```

Valid types: `string`, `integer`, `float`, `boolean`.

---

## 4. tool_calls Row (written per tag execution)

One row is written to `tool_calls` for every action tag executed, in the order they execute.

```sql
chat_log_id  INTEGER   -- FK to chat_logs.id for this generation
session_id   TEXT      -- Current session
tag_name     TEXT      -- Matched action_registry.tag
parameters   TEXT      -- JSON: actual parameters as parsed
result       TEXT      -- JSON: handler response (see §5)
success      BOOLEAN   -- TRUE | FALSE
timestamp    TIMESTAMP
```

---

## 5. Result Structure

Every tag execution returns a JSON result written to `tool_calls.result`. No silent failures.

**Success:**
```json
{ "success": true, "data": <handler-specific payload> }
```

**Failure:**
```json
{ "success": false, "error": "description of failure" }
```

Failure conditions:
- Malformed tag (syntax error, unquoted value)
- Missing required parameter
- Unregistered or disabled tag
- Handler execution error

All failures are returned as tool results and injected into the next generation via the ReAct loop. The turn does not halt on a single tag failure.

---

## 6. Execution Order

Tags execute in the order they appear in the agent's output with one exception:

When a single generation contains both a write operation and a read operation targeting the same table, the write executes first. This means the read will reflect the freshly written data.

Write operations: `REMEMBER`, `REGISTER_ACTION`, `PERSONALITY_UPDATE`

`REGENERATE` always executes last regardless of position. If multiple `REGENERATE` tags appear, only the last one executes.

---

## 7. ReAct Loop vs. REGENERATE

**ReAct loop:** Managed entirely by the Rust host. When `immediate` tool results exist after tag execution, Rust injects them into context and triggers the next generation automatically. No tag required from the agent.

**REGENERATE:** Agent-initiated only. Used for delayed re-entry or self-directed continuation. Not related to tool result injection.

---

## 8. Built-in Action Tag Specifications

---

### REMEMBER

Writes one row to `memories`.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| content | string | yes | — |
| importance | float | no | 1.0 |
| type | string | no | "fact" |

**Handler:** system
**Execution mode:** immediate

**Rust behavior:**
1. Validate parameters
2. INSERT into `memories`: `content`, `importance_weight`, `type`, `source=session_id`, `created_at`, `updated_at`
3. If `embeddings_enabled=TRUE`: generate embedding via embedding service, INSERT into `embeddings` with `source_type='memory'`. If embedding generation fails, log a warning and continue — the memory row is still written. REINDEX can add the embedding later.
4. Return `{ "success": true, "data": { "id": <new memory id> } }`

---

### RECALL

Semantic search against `memories` or `chat_logs`.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| query | string | yes | — |
| table | string | yes | — |
| limit | integer | no | 5 |

Valid `table` values: `memories`, `chat_logs`.

**Handler:** system
**Execution mode:** immediate

**Rust behavior:**
1. Validate parameters
2. If `embeddings_enabled=TRUE`: generate query embedding, rank results by cosine similarity against `embeddings` table where `source_type` matches the target table (`'memory'` for `memories`, `'chat_log'` for `chat_logs`)
3. If `embeddings_enabled=FALSE`: rank `memories` by `importance_weight DESC`, `created_at DESC`; rank `chat_logs` by `timestamp DESC`
4. Return top `limit` rows as JSON array in `data`

---

### DB_QUERY

Read-only query against a specified table. No writes permitted via this tag.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| db | string | yes | — |
| table | string | yes | — |
| filter | string | no | null |
| limit | integer | no | 10 |

Valid `db` values: `core`, `logs`, `memory`.

**Handler:** system
**Execution mode:** immediate

**Rust behavior:**
1. Validate `db` and `table` against known schema — reject unknown tables
2. Construct SELECT query with SQL WHERE clause from `filter` if provided
3. Execute as read-only transaction
4. Return rows as JSON array in `data`

---

### FILE_READ

Reads file contents from within `./workspace/`.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| path | string | yes | — |
| start_line | integer | no | null |
| end_line | integer | no | null |

Path is relative to `./workspace/`. If `start_line` and `end_line` are omitted, full file is returned. If provided, both must be present and `end_line` must be >= `start_line`.

**Handler:** `./workspace/toolbox/file_read.py`
**Execution mode:** immediate

**Behavior:**
1. Validate path is within `./workspace/`
2. Read file — full contents or specified line range
3. Return `{ "success": true, "data": { "content": "<contents>", "lines_returned": <n> } }`

---

### FILE_WRITE

Creates a new file or fully overwrites an existing file in `./workspace/`.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| path | string | yes | — |
| content | string | yes | — |

Path is relative to `./workspace/`. Creates intermediate directories if needed. Overwrites without warning if file exists.

**Handler:** `./workspace/toolbox/file_write.py`
**Execution mode:** immediate

**Behavior:**
1. Validate path is within `./workspace/`
2. Create parent directories if missing
3. Write full content to file
4. Return `{ "success": true, "data": { "path": "<resolved path>", "bytes": <n> } }`

---

### FILE_EDIT

Modifies an existing file in `./workspace/`. Fails if file does not exist. Two mutually exclusive modes.

**String replacement mode:**
| name | type | required |
|---|---|---|
| path | string | yes |
| old_str | string | yes |
| new_str | string | yes |

`old_str` must be a 100% exact match of content in the file. Fails if not found. Replaces first occurrence.

**Line replacement mode:**
| name | type | required |
|---|---|---|
| path | string | yes |
| start_line | integer | yes |
| end_line | integer | yes |
| content | string | yes |

Replaces the specified line range with `content`. `end_line` must be >= `start_line`.

Mixing parameters across modes is an error.

**Handler:** `./workspace/toolbox/file_edit.py`
**Execution mode:** immediate

**Behavior:**
1. Validate path is within `./workspace/` and file exists — fail if not
2. Determine mode from parameters — fail if mixed or incomplete
3. Apply edit
4. Return `{ "success": true, "data": { "path": "<resolved path>" } }`

---

### FILE_APPEND

Appends content to a file in `./workspace/`. Creates file if it does not exist.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| path | string | yes | — |
| content | string | yes | — |

**Handler:** `./workspace/toolbox/file_append.py`
**Execution mode:** immediate

**Behavior:**
1. Validate path is within `./workspace/`
2. Open file in append mode (create if missing)
3. Write content
4. Return `{ "success": true, "data": { "path": "<resolved path>", "bytes_appended": <n> } }`

---

### FILE_MOVE

Moves or renames a file within `./workspace/`.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| src | string | yes | — |
| dest | string | yes | — |

Both paths relative to `./workspace/`. Both must remain within `./workspace/`.

**Handler:** `./workspace/toolbox/file_move.py`
**Execution mode:** immediate

**Behavior:**
1. Validate both paths are within `./workspace/`
2. Create destination parent directories if needed
3. Move file
4. Return `{ "success": true, "data": { "src": "...", "dest": "..." } }`

---

### FILE_RECYCLE

Moves a file to `./workspace/.recycle/` prefixed with `OLD_`. User empties the recycle folder manually.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| path | string | yes | — |

**Handler:** `./workspace/toolbox/file_recycle.py`
**Execution mode:** immediate

**Behavior:**
1. Validate path is within `./workspace/`
2. Create `./workspace/.recycle/` if it does not exist
3. Move file to `./workspace/.recycle/OLD_<original_filename>`
4. Return `{ "success": true, "data": { "recycled_to": "<path in .recycle>" } }`

---

### DIR_CREATE

Creates a directory within `./workspace/`.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| path | string | yes | — |

**Handler:** `./workspace/toolbox/dir_create.py`
**Execution mode:** immediate

**Behavior:**
1. Validate path is within `./workspace/`
2. Create directory and all intermediate directories
3. Return `{ "success": true, "data": { "path": "<resolved path>" } }`

---

### DIR_LIST

Lists contents of a directory within `./workspace/`.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| path | string | yes | — |

**Handler:** `./workspace/toolbox/dir_list.py`
**Execution mode:** immediate

**Behavior:**
1. Validate path is within `./workspace/`
2. List directory entries
3. Return `{ "success": true, "data": { "entries": [ { "name": "...", "type": "file|directory", "size": <bytes>, "modified": "<timestamp>" } ] } }`

---

### TERMINAL

> **⚠️ Post-MVP:** TERMINAL requires the Podman sandbox for isolation. Without sandbox isolation, the agent has unrestricted host access. This tag is seeded as `enabled=FALSE` and will only be functional when the Podman sandbox is running.

Executes a command on the host via `sh -c`, scoped to `./workspace/` and its subdirectories.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| cmd | string | yes | — |

**Handler:** system
**Execution mode:** immediate

**Rust behavior:**
1. Execute: `sh -c "<cmd>"` with working directory set to `./workspace/`
2. Capture stdout and stderr
3. Return:
```json
{
  "success": true,
  "data": {
    "stdout": "...",
    "stderr": "...",
    "exit_code": 0
  }
}
```
Non-zero exit code sets `success=false`. Full stdout/stderr still returned.

---

### REGISTER_ACTION

Registers a new action tag backed by a Python script in `./workspace/toolbox/`.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| tag | string | yes | — |
| description | string | yes | — |
| parameters | string | yes | — |
| script | string | yes | — |

`tag` must be uppercase with underscores only. `parameters` must be a valid JSON array matching the schema in §3. `script` is relative to `./workspace/toolbox/`.

**Handler:** system
**Execution mode:** immediate

**Rust behavior:**
1. Validate `tag` format and uniqueness against `action_registry`
2. Validate `parameters` is valid JSON
3. Validate script path exists within `./workspace/toolbox/`
4. INSERT into `action_registry`: `tag`, `description`, `parameters`, `handler='./workspace/toolbox/<script>'`, `execution_mode='immediate'`, `enabled=TRUE`
5. New tag is available immediately in the same session
6. Return `{ "success": true, "data": { "tag": "<tag name>", "id": <registry id> } }`

**Python script interface contract:**
- Input: JSON string passed as `sys.argv[1]`
- Output: JSON string printed to stdout matching result structure in §5
- Must handle its own exceptions and return error JSON rather than raising

---

### PERSONALITY_UPDATE

Updates a named section in the `personality` table.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| section | string | yes | — |
| content | string | yes | — |

`section` must exactly match an existing `personality.section` value. New sections cannot be created via this tag.

**Handler:** system
**Execution mode:** immediate

**Rust behavior:**
1. Validate `section` exists in `personality` table
2. UPDATE `personality SET content=?, updated_at=? WHERE section=?`
3. Return `{ "success": true, "data": { "section": "<section>" } }`

---

### REGENERATE

Agent-initiated re-entry into the generation loop.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| delay | integer | yes | — |
| msg | string | no | null |

`delay` is in seconds. Use `0` for immediate re-entry.

**Handler:** system
**Execution mode:** immediate

**Rust behavior:**
1. If `msg` is set: send status message to UI via WebSocket
2. Wait `delay` seconds
3. Trigger next generation with current context
4. If multiple REGENERATE tags parsed in one generation: only the last one executes, all prior are discarded

---

### REINDEX

Re-embeds rows that lack embeddings, or re-embeds everything when forced. Used after embedding model changes or to catch up after embedding service downtime.

**Parameters:**
| name | type | required | default |
|---|---|---|---|
| source | string | no | "all" |
| force | boolean | no | false |

Valid `source` values: `"memories"`, `"chat_logs"`, `"all"`.

**Handler:** system
**Execution mode:** immediate

**Rust behavior:**
1. If `force=TRUE`: delete all existing embeddings for the specified `source_type`(s)
2. Query target table for rows without embeddings (or all rows if `force=TRUE`)
3. For each row: call `POST /embedding` with the row's text content, INSERT into `embeddings` with appropriate `source_type` and `source_id`
4. Return `{ "success": true, "data": { "rows_processed": <n>, "source": "<source>" } }`
5. If embedding service is unavailable: return `{ "success": false, "error": "Embedding service not running" }`

---

## 9. Execution Mode Definitions

| mode | MVP status | behavior |
|---|---|---|
| `immediate` | Implemented | Rust awaits result before proceeding |
| `async` | Defined, not implemented | Operation starts; result retrieved later |
| `persistent` | Defined, not implemented | Long-running process; multi-turn interaction |
