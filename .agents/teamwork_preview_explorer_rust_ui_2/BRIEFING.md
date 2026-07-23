# BRIEFING — 2026-07-20T13:49:00Z

## Mission
Analyze and propose a design for the Rust-UI agent's domain skill (SKILL.md) ensuring strict boundaries and delegation to other agents like Forge-Builder.

## 🔒 My Identity
- Archetype: Explorer
- Roles: Read-only investigator, analyzer, synthesizer
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_rust_ui_2
- Original parent: e57f7225-89e1-4eb0-b701-23f51a1871cb
- Milestone: Milestone 2: Define Rust-UI

## 🔒 Key Constraints
- Read-only investigation — do NOT implement
- Ensure the design specifies exact file paths and structure/rules for the SKILL
- Produce a handoff.md and send_message to parent

## Current Parent
- Conversation ID: e57f7225-89e1-4eb0-b701-23f51a1871cb
- Updated: 2026-07-20T13:49:00Z

## Investigation State
- **Explored paths**: PROJECT.md, ORIGINAL_REQUEST.md, SCOPE.md
- **Key findings**: The Rust-UI agent handles `ermete-shell-rs`, `ermete-settings-rs`, and Niri IPC (Wayland/GTK4 Rust stack). It must delegate OS-level tasks (RPMs) to Forge-Builder.
- **Unexplored areas**: Existing Rust codebase in `ermete-shell-rs` or `ermete-settings-rs` if present (to tailor the skill).

## Key Decisions Made
- Will structure the skill around clear domain boundaries and explicit delegation instructions.

## Artifact Index
- /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_rust_ui_2/handoff.md — Final report (to be created)
