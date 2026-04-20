# Documentation & Header Standards
Version: 1.0
Purpose: Make every file and the overall codebase self-documenting so AI agents can understand purpose, responsibilities, and boundaries quickly and reliably.

## Header Requirements
- Every single file in the codebase MUST begin with a standardized header comment.
- The header MUST be the very first content in the file (before any code or imports).
- Headers MUST use consistent formatting across the entire project so agents can parse them reliably.
- Headers MUST be written in clear, concise, plain language optimized for machine readability.

## Required Header Content
Every file header MUST include:

- File name (for quick verification)
- One-sentence summary of the file’s exact purpose
- Short bullet list of the file’s main responsibilities or key functions
- High-level list of important dependencies or interfaces it relies on (external modules, key data structures, etc.)
- Date the file was created or last significantly updated (optional but recommended for traceability)

## Header Guidelines
- Keep headers focused on purpose and boundaries only — do NOT include implementation details, algorithms, or code examples.
- Headers should be short enough to fit comfortably within an agent’s context window.
- When an agent significantly changes a file’s responsibility, splits a file, or merges logic, it MUST update the header immediately to reflect the new reality.
- Use consistent comment markers (e.g., `//`, `/* */`, `#`, or `"""` depending on language) so agents can easily locate and extract the header.

## Index Files & Documentation
- The top-level `codebase_index.md` is the master map and Single Source of Truth. It MUST be kept up-to-date with current file and folder responsibilities.
- Use `folder_index.md` files inside folders when more granular explanation is needed. The top-level `codebase_index.md` MUST reference these folder indexes.
- Keep all index files concise, focused on current responsibilities, and written in clear plain English.
- Do NOT embed long explanations, tutorials, or detailed guides inside source code files.
- If additional documentation is needed for a module or feature, place it in the appropriate `folder_index.md` or a dedicated documentation file.

## Agent Responsibilities
- Before modifying any file, agents MUST read:
  - The file’s own header
  - The relevant sections of `codebase_index.md`
  - Any applicable `folder_index.md`
- Agents MUST ensure they fully understand the intended purpose and boundaries before making changes.
- After any structural change (add, remove, rename, or responsibility shift), agents MUST update the affected headers and index files.
