# BRIEFING — 2026-07-20T13:48:55+02:00

## Mission
Design the domain skill / system prompt for the Forge-Builder agent.

## 🔒 My Identity
- Archetype: Explorer
- Roles: Read-only investigator, analyzer, synthesizer
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_m1_1
- Original parent: 949406c7-d558-4d3a-bf4e-350b79f454bb
- Milestone: 1 (Define Forge-Builder)

## 🔒 Key Constraints
- Read-only investigation — do NOT implement or write the target SKILL.md file directly.
- The Forge-Builder is responsible for RPM packaging, macros, dependencies, OCI isolation, and bash scripts.
- Must know how to delegate tasks to other agents.
- Write handoff report in working directory, send message back with summary.

## Current Parent
- Conversation ID: 949406c7-d558-4d3a-bf4e-350b79f454bb
- Updated: 2026-07-20T13:48:55+02:00

## Investigation State
- **Explored paths**: `PROJECT.md`, `.agents/sub_orch_m1/SCOPE.md`, `.agents/ORIGINAL_REQUEST.md`, `forge/` directory structure.
- **Key findings**: Forge-Builder must manage 33 spec folders and 4 bash scripts. It requires strict boundaries to prevent overwriting existing code (R3) and must explicitely map out `Rust-UI`, `OS-Core`, and `QA-DevOps` for accurate delegation (R2).
- **Unexplored areas**: No caveats. Core requirements mapped perfectly.

## Key Decisions Made
- Generated a structured draft for `.agents/skills/forge/SKILL.md` detailing Identity, Core Responsibilities, Key Constraints, and Delegation Protocol.

## Artifact Index
- /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_m1_1/ORIGINAL_REQUEST.md — Original request log
- /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_m1_1/handoff.md — Final Output Handoff Report containing the proposed SKILL.md content.
