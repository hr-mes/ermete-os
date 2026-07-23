# BRIEFING — 2026-07-20T13:48:40+02:00

## Mission
Analyze and propose a design for the Rust-UI agent's domain skill (a SKILL.md file) ensuring delegation to other agents per the Interface Contract.

## 🔒 My Identity
- Archetype: Explorer
- Roles: Read-only investigation, analysis, structured reporting
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_rust_ui_1
- Original parent: e57f7225-89e1-4eb0-b701-23f51a1871cb (Orchestrator)
- Milestone: Milestone 2: Define Rust-UI

## 🔒 Key Constraints
- Read-only investigation — do NOT implement.
- Must produce a `handoff.md` with the proposed design strategy.

## Current Parent
- Conversation ID: e57f7225-89e1-4eb0-b701-23f51a1871cb
- Updated: 2026-07-20T13:48:40+02:00

## Investigation State
- **Explored paths**: PROJECT.md, ORIGINAL_REQUEST.md, .agents/sub_orch_rust_ui/SCOPE.md
- **Key findings**: 
  - Rust-UI handles Wayland/GTK4 Rust stack (`ermete-shell-rs`, `ermete-settings-rs`, Niri IPC).
  - Out of scope tasks, like system dependencies, must be delegated (e.g. to Forge-Builder).
  - The target path for the skill should be `.agents/skills/ermete-rust-ui/SKILL.md`.
- **Unexplored areas**: None.

## Key Decisions Made
- Define the exact structure and rules for `.agents/skills/ermete-rust-ui/SKILL.md` inside `handoff.md`.

## Artifact Index
- handoff.md — Proposed design strategy.
