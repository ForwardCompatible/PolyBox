# Maintenance & Refactoring Rules
Version: 1.0
Purpose: Enable safe, predictable, and incremental maintenance and refactoring by AI agents, especially when decomposing monolithic files into small, single-responsibility files.

## Core Principles
- Refactoring is encouraged and expected whenever a file approaches the 300-line limit or begins handling multiple concerns.
- All refactoring MUST preserve the exact original behavior unless a task explicitly requires functional changes.
- The primary refactoring pattern is monolithic file decomposition: split distinct responsibilities into new, focused files.

## Before Refactoring
- Agents MUST read the target file’s header, relevant sections of `codebase_index.md`, and any applicable `folder_index.md`.
- Agents MUST clearly identify all distinct responsibilities currently mixed in the file.
- Agents MUST create a clear split plan: define exactly what each new file will be responsible for.

## During & After Refactoring
- Each new file MUST receive its own standardized header clearly stating its single responsibility.
- The original file MUST be updated (or removed if fully decomposed) with its header revised accordingly.
- All affected index files (`codebase_index.md` and relevant `folder_index.md`) MUST be updated with the new structure and responsibility descriptions.
- Structural changes MUST be recorded in `changelog.md` (example: “Decomposed monolithic_parser.py into parser_core.py, validator.py, and utils.py”).
- When extracting code, move ONLY the code belonging to the target responsibility. Update all references and imports correctly.
- Prefer small, incremental refactors over large, risky ones. Handle one responsibility at a time when possible.

## Split Point Guidelines
- Choose logical split points based on clear domain boundaries, data flow, or distinct functions (e.g., parsing vs validation vs transformation vs I/O).
- Keep shared code minimal. If shared utilities are needed, create small, focused helper files rather than growing a single utils file.

## Agent Responsibilities
- Agents MUST verify that the refactored code behaves exactly as before (using available tests or other verification methods).
- Never leave the codebase in a broken or partially refactored state. Every refactor MUST result in a clean, compilable, and functional system.
- Favor simplicity and clarity over clever optimizations that could reduce readability for other agents.
- After any refactor, update the `codebase_index.md` (and folder indexes) before considering the task complete.
