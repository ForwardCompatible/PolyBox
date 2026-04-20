# File & Module Architecture Rules
Version: 1.0
Purpose: Ensure the codebase remains highly maintainable and understandable by AI agents through strict modularity and clear boundaries.

## Core Constraints
- Every file MUST have exactly one clear, single responsibility.
- If a file ever handles two or more distinct concerns, it MUST be split into separate files immediately — regardless of file length.
- No file may exceed 300 lines (including comments and blank lines).

## Coupling & Boundaries
- Files and modules MUST be small and clearly/safely decoupled.
- An agent editing one file must be able to do so with minimal risk of affecting unrelated parts of the system.
- Every file MUST expose only a minimal, well-defined public interface.
- All internal implementation details MUST remain hidden.
- Agents MUST NOT rely on implicit knowledge of other modules — only use exposed interfaces.

## Project Structure
- Use properly organized folder structures that follow the language's natural conventions and/or separate concerns/domains clearly (e.g., by feature, layer, or business domain).
- Prefer flatter folder structures when possible to reduce navigation depth for agents.
- Avoid deep nesting unless strictly required by the language or domain.

## Shared Utilities
- Shared "utils" or "helpers" files MUST NOT grow large or accumulate unrelated functions.
- If a utility file starts mixing unrelated concerns, split it by domain or responsibility.

## Index Files (Mandatory)
- A top-level `codebase_index.md` MUST always exist and be kept up-to-date. It is the Single Source of Truth and master map.
- `codebase_index.md` MUST list every file and folder with a short, precise explanation of its responsibility and its role in the system.
- For any folder needing more detail, include a `folder_index.md` inside it for granular information.
- The top-level `codebase_index.md` MUST explicitly reference any `folder_index.md` files to maintain a clear reference chain.
- A separate `changelog.md` MUST be used for recording structural changes (splits, renames, major refactors). Keep `codebase_index.md` focused only on current responsibilities.

## Agent Responsibilities
- Before making any changes, agents MUST read the `codebase_index.md` (and relevant `folder_index.md`) for orientation.
- When adding, removing, renaming, or meaningfully changing a file's responsibility, agents MUST:
  - Update the `codebase_index.md`
  - Update the relevant `folder_index.md` if applicable
  - Choose names and locations that clearly signal the file's single purpose
- Agents MUST keep the number of public/exported items in each file as small as possible.
