# BRIEFING — 2026-07-20T13:49:00Z

## Mission
Design the system prompt and skill definition for the OS-Core agent (`.agents/skills/ermete-core`) based on project documentation, without modifying the files directly.

## 🔒 My Identity
- Archetype: Teamwork explorer
- Roles: Read-only investigation, analysis, reporting
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_os_core_2
- Original parent: [TBD]
- Milestone: Define OS-Core

## 🔒 Key Constraints
- Read-only investigation — do NOT implement.
- Do NOT write the skill files yourself.
- Provide a report with the recommended contents in handoff.md.

## Current Parent
- Conversation ID: [TBD]
- Updated: 2026-07-20T13:49:00Z

## Investigation State
- **Explored paths**: SCOPE.md, PROJECT.md, ORIGINAL_REQUEST.md
- **Key findings**: OS-Core domain covers ostree/bootc, Containerfile, kernel, DKMS Nvidia, SELinux. Delegation requirements specify handing off RPM/scripts to Forge-Builder, UI/Wayland to Rust-UI, and QA/testing/ISO to QA-DevOps.
- **Unexplored areas**: N/A

## Key Decisions Made
- Proceeding to write the handoff.md report with the drafted skill definition.

## Artifact Index
- /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_os_core_2/ORIGINAL_REQUEST.md — Initial user request
- /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_os_core_2/handoff.md — Final report
