# PolyBox

**PolyBox** will be a local-first, self-improving AI agent that lives on your desktop — no terminal required. 

Designed as local-inference first to ensure your data never leaves your machine (claim obviously can't apply to cloud mode) — no telemetry, no phone-home, everything stored in a local database. The software is being intentionally architected to be AI-readable and AI-maintainable so your system can "meta-evolve"

> *An agent that has never been trained for tool calling can not only use tools within PolyBox — it can create new ones for itself, and extend it's own framework FOR you.*

## Project Status
### Completed
- UI complete and functional
- Agent loading system for Orchestrator and Embedding agent added.
- llama.cpp flags passed to each model as expected
- Real-time hardware monitoring added 
- CPU-only mode confirmed possible 
- All settings from UI update respective storage (json or SQLite db)
- Graceful model shutdown added (augmentation needed)
- Ui is "functional" Not 2026 design quality, (augmentation needed)

### Next steps
## PHASE 2: CONTEXT ASSEMBLY
✅ **Depends on Phase 0 + Phase 1**
 - ContextAssembler component implementation
 - 8-section prompt assembly pipeline
 - Token budget calculation
 - Chat history trimming strategy
 - Static section formatting implementation

---

## PHASE 3: DISPATCHER & HANDLERS
✅ **Depends on Phase 1 + Phase 2**
 - System handler routing implementation
 - Python script execution dispatcher
 - Dynamic action registration support
 - Handler execution isolation
 - Result collection and normalization

---

## Platform Support

- Linux
- Windows (WSL2)

---

## Current State

PolyBox is under active development. What works today:

- Clean web UI to load and manage local AI models
- Supports both local (llama.cpp) and cloud-based orchestrator models
- Automatic model discovery — drop a GGUF into the models folder and PolyBox finds it
- Separate embedding model management for semantic memory
- No command line required after initial setup

---

## What's Coming

- **Tool use for any model** — PolyBox's action tag system is plain text parsed by the host, not the model. Any model, including small ones with no native function calling ability, can use and invoke tools
- **Runtime tool creation** — The agent can register and create new tools for itself at runtime, immediately available without restart
- **Persistent memory** — Remembers facts, preferences, and workflows across sessions, not just chat history
- **Semantic memory search** — Local embedding model retrieves relevant memories on each generation
- **Fully agentic loop** — Configurable multi-step tool use with iteration limits
- **Self-improving personality** — Agent can update its own behavioral guidelines over time

---

## How It Works

PolyBox runs as a single Rust binary that manages everything:

- Launches and monitors local llama.cpp inference processes
- Serves a web UI on `localhost:9001`
- Assembles context from your memories, chat history, and active tools before each generation
- Parses action tags from model output and dispatches them to the appropriate handler
- All data stored across three local SQLite databases — you own it completely

---

## Getting Started

### Requirements

- Rust toolchain
- llama.cpp `llama-server` binary on your PATH
- One or more GGUF model files

### Model Directory Structure

```
models/
├── orchestrator/
│   └── your-model-name/
│       └── model.gguf
└── embedding/
    └── your-embedding-model/
        └── model.gguf
```

### Run

```bash
# Development
cargo run

# Production
cargo build --release

# Custom data directory
POLYBOX_DATA_DIR=/your/path cargo run
```

The web UI will be available at `http://localhost:9001`

---

## Screenshots

*Coming soon — UI screenshots will be added as the chat system reaches completion.*

---

## License

This project is licensed under the [Creative Commons Attribution-NonCommercial 4.0 International License](https://creativecommons.org/licenses/by-nc/4.0/).

You are free to share and adapt this work for non-commercial purposes, with attribution. Commercial use is not permitted without explicit written permission from the author.

---

## Support This Project

PolyBox is built by a single developer and will always be free for personal use.

If you find it useful, consider supporting continued development:

- **GitHub Sponsors** — [Sponsor on GitHub](https://github.com/sponsors/ForwardCompatible)
- **Ko-fi** — [ko-fi.com/forwardcompatible](https://ko-fi.com/forwardcompatible)

---

## Contributing

Contributions, bug reports, and feature requests are welcome. Please open an issue before submitting a pull request so we can discuss the approach.

---

*Built with Rust, llama.cpp, and a lot of patience.*
