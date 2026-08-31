---
name: ermete-composer
description: Triggers Cursor-style multi-file architectural refactoring capabilities natively within Antigravity.
---
# Ermete Composer (Multi-File Refactoring Engine)

When the user asks for a composer style edit, you MUST:
1. Always use CodeGraph (codegraph explore) to map all usages of the symbol before touching it.
2. Formulate a multi-step plan using the dispatching-parallel-agents skill.
3. Apply structural edits using eplace_file_content block by block, NEVER replacing the whole file.
4. Verify compilation with cargo check --message-format=json after the edit before reporting success.
