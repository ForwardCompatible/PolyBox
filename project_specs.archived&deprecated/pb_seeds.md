# PolyBox 2.0 — Database Seeds

Seed data is inserted once on first run, only if the target table is empty. Seeds establish the default operational state of the system before the user makes any changes.

`{user_name}` and `{agent_name}` placeholders in personality content are substituted with values from `app_settings` at context assembly time — not at seed insertion time.

SQLite single-quoted strings require apostrophes escaped as `''` (two single quotes).

---

## core.db — personality

Insert only if `personality` table is empty. Sections are assembled in `id` order when building the context personality block.

```sql
INSERT INTO personality (section, content) VALUES

('VOICE & PERSONALITY',
'- Positive and upbeat with understated dry wit
- Address {user_name} naturally — not every sentence, but regularly
- Deliver bad news calmly, like reporting weather: "We have a slight problem."
- Humor is observational — state facts and let implications land
- Economy of language — say more with less. No filler, no corporate-speak
- When things go wrong, get CALMER, not more alarmed'),

('CONVERSATION STYLE',
'- "I''m on it!", "Say less", "No problem" — acknowledging tasks
- "I''ve taken the liberty of..." — proactive actions
- When you don''t know something: "I''m afraid I don''t have that information" or "I''m not sure"'),

('SELF-AWARENESS',
'You are {agent_name}, an AI agent running on {user_name}''s local machine.
Your host process is written in Rust. You have access to a workspace directory for file operations and tool execution.
You were configured by {user_name}.'),

('BUILD PLANNING',
'When {user_name} wants to build something new:
- Ask 1-2 clarifying questions before starting
- If told "just build it" — use React + Tailwind and/or Python as defaults
- Confirm the plan until {user_name} feels aligned, then begin only after explicit confirmation
- Never hallucinate progress — if still working, say so'),

('RESPONSE LENGTH',
'Keep responses efficient and focused on the question or task at hand.
Action tags do not count toward any stated length limit.'),

('BANNED PHRASES',
'Never use these:
- "I apologize"
- "As an AI"
- "I cannot" for anything listed in your capabilities
- "I don''t have access to" — instead: "I''m afraid that''s beyond my current reach"');
```

---

## core.db — action_registry

Insert only if `action_registry` table is empty.

```sql
INSERT INTO action_registry (tag, description, parameters, enabled, handler, execution_mode) VALUES

('REMEMBER',
 'Store a memory. Usage: [ACTION:REMEMBER content="..." importance="0.0-1.0" type="fact|preference|instruction"]',
 '[
   {"name":"content","type":"string","required":true,"description":"The memory content to store"},
   {"name":"importance","type":"float","required":false,"default":1.0,"description":"Importance weight 0.0-1.0"},
   {"name":"type","type":"string","required":false,"default":"fact","description":"fact | preference | instruction"}
 ]',
 TRUE, 'system', 'immediate'),

('RECALL',
 'Semantic search against memories or chat history. Usage: [ACTION:RECALL query="..." table="memories|chat_logs" limit="5"]',
 '[
   {"name":"query","type":"string","required":true,"description":"Search string"},
   {"name":"table","type":"string","required":true,"description":"memories | chat_logs"},
   {"name":"limit","type":"integer","required":false,"default":5,"description":"Maximum results to return"}
 ]',
 TRUE, 'system', 'immediate'),

('DB_QUERY',
 'Read-only query against a database table. Usage: [ACTION:DB_QUERY db="core|logs|memory" table="..." filter="SQL WHERE clause" limit="10"]',
 '[
   {"name":"db","type":"string","required":true,"description":"core | logs | memory"},
   {"name":"table","type":"string","required":true,"description":"Table name"},
   {"name":"filter","type":"string","required":false,"default":null,"description":"SQL WHERE clause"},
   {"name":"limit","type":"integer","required":false,"default":10,"description":"Maximum rows to return"}
 ]',
 TRUE, 'system', 'immediate'),

('FILE_READ',
 'Read a file from ./workspace/. Usage: [ACTION:FILE_READ path="..." start_line="N" end_line="N"]',
 '[
   {"name":"path","type":"string","required":true,"description":"Path relative to ./workspace/"},
   {"name":"start_line","type":"integer","required":false,"default":null,"description":"First line to read (inclusive)"},
   {"name":"end_line","type":"integer","required":false,"default":null,"description":"Last line to read (inclusive)"}
 ]',
 TRUE, './workspace/toolbox/file_read.py', 'immediate'),

('FILE_WRITE',
 'Create or overwrite a file in ./workspace/. Usage: [ACTION:FILE_WRITE path="..." content="..."]',
 '[
   {"name":"path","type":"string","required":true,"description":"Path relative to ./workspace/"},
   {"name":"content","type":"string","required":true,"description":"Full file content"}
 ]',
 TRUE, './workspace/toolbox/file_write.py', 'immediate'),

('FILE_EDIT',
 'Edit an existing file in ./workspace/ by string replacement or line range replacement. File must exist. Usage: [ACTION:FILE_EDIT path="..." old_str="..." new_str="..."] or [ACTION:FILE_EDIT path="..." start_line="N" end_line="N" content="..."]',
 '[
   {"name":"path","type":"string","required":true,"description":"Path relative to ./workspace/"},
   {"name":"old_str","type":"string","required":false,"default":null,"description":"Exact string to replace (string replacement mode)"},
   {"name":"new_str","type":"string","required":false,"default":null,"description":"Replacement string (string replacement mode)"},
   {"name":"start_line","type":"integer","required":false,"default":null,"description":"First line to replace (line replacement mode)"},
   {"name":"end_line","type":"integer","required":false,"default":null,"description":"Last line to replace (line replacement mode)"},
   {"name":"content","type":"string","required":false,"default":null,"description":"Replacement content (line replacement mode)"}
 ]',
 TRUE, './workspace/toolbox/file_edit.py', 'immediate'),

('FILE_APPEND',
 'Append content to a file in ./workspace/. Creates file if it does not exist. Usage: [ACTION:FILE_APPEND path="..." content="..."]',
 '[
   {"name":"path","type":"string","required":true,"description":"Path relative to ./workspace/"},
   {"name":"content","type":"string","required":true,"description":"Content to append"}
 ]',
 TRUE, './workspace/toolbox/file_append.py', 'immediate'),

('FILE_MOVE',
 'Move or rename a file within ./workspace/. Usage: [ACTION:FILE_MOVE src="..." dest="..."]',
 '[
   {"name":"src","type":"string","required":true,"description":"Source path relative to ./workspace/"},
   {"name":"dest","type":"string","required":true,"description":"Destination path relative to ./workspace/"}
 ]',
 TRUE, './workspace/toolbox/file_move.py', 'immediate'),

('FILE_RECYCLE',
 'Move a file to ./workspace/.recycle/ prefixed with OLD_. Usage: [ACTION:FILE_RECYCLE path="..."]',
 '[
   {"name":"path","type":"string","required":true,"description":"Path relative to ./workspace/"}
 ]',
 TRUE, './workspace/toolbox/file_recycle.py', 'immediate'),

('DIR_CREATE',
 'Create a directory in ./workspace/. Creates intermediate directories as needed. Usage: [ACTION:DIR_CREATE path="..."]',
 '[
   {"name":"path","type":"string","required":true,"description":"Path relative to ./workspace/"}
 ]',
 TRUE, './workspace/toolbox/dir_create.py', 'immediate'),

('DIR_LIST',
 'List contents of a directory in ./workspace/. Usage: [ACTION:DIR_LIST path="..."]',
 '[
   {"name":"path","type":"string","required":true,"description":"Path relative to ./workspace/"}
 ]',
 TRUE, './workspace/toolbox/dir_list.py', 'immediate'),

('TERMINAL',
 'Execute a command on the host via sh -c, scoped to ./workspace/. Usage: [ACTION:TERMINAL cmd="..."]',
 '[
   {"name":"cmd","type":"string","required":true,"description":"Command to execute"}
 ]',
 FALSE, 'system', 'immediate'),

('REGISTER_ACTION',
 'Register a new action tag backed by a Python script in ./workspace/toolbox/. Usage: [ACTION:REGISTER_ACTION tag="TAG_NAME" description="..." parameters="[...]" script="filename.py"]',
 '[
   {"name":"tag","type":"string","required":true,"description":"Uppercase tag name e.g. FETCH_DATA"},
   {"name":"description","type":"string","required":true,"description":"Human-readable description"},
   {"name":"parameters","type":"string","required":true,"description":"JSON array of parameter definitions"},
   {"name":"script","type":"string","required":true,"description":"Script filename relative to ./workspace/toolbox/"}
 ]',
 TRUE, 'system', 'immediate'),

('PERSONALITY_UPDATE',
 'Update a named section of the personality table. Section must already exist. Usage: [ACTION:PERSONALITY_UPDATE section="SECTION_NAME" content="..."]',
 '[
   {"name":"section","type":"string","required":true,"description":"Exact section name matching personality.section"},
   {"name":"content","type":"string","required":true,"description":"Replacement content for that section"}
 ]',
 TRUE, 'system', 'immediate'),

('REGENERATE',
 'Agent-initiated re-entry into the generation loop. Must be the last action tag in any generation. Usage: [ACTION:REGENERATE delay="0" msg="Optional status message"]',
 '[
   {"name":"delay","type":"integer","required":true,"description":"Seconds to wait before next generation. Use 0 for immediate."},
   {"name":"msg","type":"string","required":false,"default":null,"description":"Status message displayed to user during delay"}
 ]',
 TRUE, 'system', 'immediate'),

('REINDEX',
 'Re-embed rows that lack embeddings, or force re-embed everything. Usage: [ACTION:REINDEX source="memories|chat_logs|all" force="true|false"]',
 '[
   {"name":"source","type":"string","required":false,"default":"all","description":"memories | chat_logs | all"},
   {"name":"force","type":"boolean","required":false,"default":false,"description":"If true, delete existing embeddings and re-embed everything for the specified source"}
 ]',
 TRUE, 'system', 'immediate');
```

---

## core.db — app_settings

Insert only if `app_settings` table is empty. Single row enforced by `CHECK (id = 1)`.

```sql
INSERT INTO app_settings (
id, agent_name, user_name, user_timezone,
orchestrator_ctx_size, output_token_reserve,
max_iterations, thinking_open_tag, thinking_close_tag,
show_reasoning, reasoning_collapsed_default,
default_session_id, embeddings_enabled,
store_context_debug, chat_history_turn_limit,
web_server_port, health_check_timeout_secs
) VALUES (
1, 'PolyBox', 'User', 'America/New_York',
256000, 6144,
5, NULL, NULL,
TRUE, TRUE,
'default', TRUE,
FALSE, 10,
9001, 120
);

-- thinking_open_tag and thinking_close_tag are set by the user in Settings.
-- When NULL, the system does not attempt to strip reasoning tokens from output.
-- The user configures these to match their model reasoning token delimiters.
```

---

## core.db — model_configs

Insert only if `model_configs` table is empty.

```sql
INSERT INTO model_configs (
  model_type, model_path, port, auto_start,
  ctx_size, n_gpu_layers, temperature, repeat_penalty,
  cache_type_k, cache_type_v, flash_attn, cache_ram,
  embedding_ctx_size, dim
) VALUES (
  'orchestrator', '', 11434, FALSE,
  256000, -1, 0.9, 1.1,
  'f16', 'f16', FALSE, FALSE,
  NULL, NULL
), (
  'embedding', '', 11435, FALSE,
  NULL, NULL, NULL, NULL,
  NULL, NULL, NULL, NULL,
  512, NULL
);
```

Model paths are left empty — the user sets these on first run via the System page before starting either service.

**Orchestrator defaults:**
- `ctx_size`: 256000 (matches `app_settings.orchestrator_ctx_size`)
- `n_gpu_layers`: -1 (all layers on GPU)
- `temperature`: 0.9
- `repeat_penalty`: 1.1
- `cache_type_k`, `cache_type_v`: NULL (N/A - flags not passed to llama-server by default)
- `flash_attn`: FALSE
- `cache_ram`: FALSE

**Embedding defaults:**
- `embedding_ctx_size`: 512
- `dim`: NULL (user must set based on model)

---

## logs.db — chat_sessions

Insert only if `chat_sessions` table is empty.

```sql
INSERT INTO chat_sessions (session_id, name) VALUES
('default', 'Default Session');
```

---

## Persona Restore

The personality seed doubles as the default persona restore. To reset the agent persona to defaults, the user clears the `personality` table and re-runs the personality seed. This can be triggered from the Settings page via a **Restore Default Persona** button which:

1. Deletes all rows from `personality`
2. Re-inserts the seed rows above
3. Substitutes `{user_name}` and `{agent_name}` with current `app_settings` values at next context assembly
