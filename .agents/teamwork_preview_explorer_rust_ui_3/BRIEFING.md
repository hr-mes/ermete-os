# BRIEFING — 2026-07-20T13:48:30Z

## Mission
Analyze and propose a design for the Rust-UI agent's domain skill (SKILL.md) to define its role and interface contracts.

## 🔒 My Identity
- Archetype: Explorer
- Roles: Read-only investigator, Domain skill designer
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_rust_ui_3
- Original parent: e57f7225-89e1-4eb0-b701-23f51a1871cb
- Milestone: Milestone 2: Define Rust-UI

## 🔒 Key Constraints
- Read-only investigation — do NOT implement.
- Output handoff.md with design strategy.

## Current Parent
- Conversation ID: e57f7225-89e1-4eb0-b701-23f51a1871cb
- Updated: 2026-07-20T13:48:30Z

## Investigation State
- **Explored paths**: PROJECT.md, ORIGINAL_REQUEST.md, .agents/sub_orch_rust_ui/SCOPE.md
- **Key findings**: 
  - Rust-UI handles Wayland/GTK4 Rust stack (`ermete-shell-rs`, `ermete-settings-rs`, Niri IPC).
  - Must preserve work in `ermete-shell-rs`.
  - Must delegate system dependency and packaging to `Forge-Builder`.
  - Must delegate core OS changes to `OS-Core` and QA/orchestration to `QA-DevOps`.
- **Unexplored areas**: none

## Key Decisions Made
- Will define the target skill file as `.agents/skills/ermete-rust-ui/SKILL.md`.

## Artifact Index
- handoff.md — Proposed design for the Rust-UI domain skill.
