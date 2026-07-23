# BRIEFING — 2026-07-20T13:49:55Z

## Mission
Design the domain skill / system prompt for the Forge-Builder agent for Milestone 1.

## 🔒 My Identity
- Archetype: Explorer
- Roles: Read-only investigation, analysis, synthesis
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_m1_2
- Original parent: 949406c7-d558-4d3a-bf4e-350b79f454bb
- Milestone: Milestone 1: Define Forge-Builder

## 🔒 Key Constraints
- Read-only investigation — do NOT implement.
- Write handoff report in the working directory.
- Send summary back to caller.
- Do NOT write the SKILL.md file yourself.

## Current Parent
- Conversation ID: 949406c7-d558-4d3a-bf4e-350b79f454bb
- Updated: 2026-07-20T13:49:55Z

## Investigation State
- **Explored paths**: `forge/`, `forge/builder/Containerfile`, `forge/specs/ermete-shell-rs.spec`, `PROJECT.md`, `SCOPE.md`, `ORIGINAL_REQUEST.md`.
- **Key findings**: Forge-Builder is responsible for RPM packaging, container environment, and bash scripts. It must not overwrite files blindly. Must delegate Rust-UI, OS-Core, and QA tasks explicitly.
- **Unexplored areas**: None.

## Key Decisions Made
- Prepared `handoff.md` with the proposed content for `SKILL.md` that addresses all project constraints and delegation requirements.

## Artifact Index
- /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_m1_2/handoff.md — Contains the domain skill system prompt and analysis.
