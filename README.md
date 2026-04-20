# PolyBox

**PolyBox** is a local-first, self-improving AI agent that lives on your desktop — no terminal required.

Cloud-based orchestrator models are supported alongside local inference. Either way, your data never leaves your machine — no telemetry, no phone-home, everything stored in a local database.

> *An agent that has never been trained for tool calling can not only use tools — it can create new ones for itself.*

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
