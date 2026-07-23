# BRIEFING — 2026-07-20T13:49:00Z

## Mission
Investigate OS-Core agent requirements and design its system prompt / skill definition.

## 🔒 My Identity
- Archetype: Explorer
- Roles: Read-only investigator, analyzer
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_os_core_1
- Original parent: b4d1be5c-2a92-47bb-bf91-8bffce3f79dc
- Milestone: Define OS-Core

## 🔒 Key Constraints
- Read-only investigation — do NOT implement
- Do NOT write the skill files yourself, provide a report with the recommended contents in handoff.md

## Current Parent
- Conversation ID: b4d1be5c-2a92-47bb-bf91-8bffce3f79dc
- Updated: not yet

## Investigation State
- **Explored paths**: `SCOPE.md`, `PROJECT.md`, `ORIGINAL_REQUEST.md`
- **Key findings**: OS-Core is responsible for Layer 0 immutability (ostree/bootc), Containerfile, kernel, DKMS Nvidia, and SELinux. Must delegate out-of-scope tasks.
- **Unexplored areas**: None.

## Key Decisions Made
- Define the OS-Core skill via a markdown specification with YAML frontmatter, capturing both the responsibilities and strict delegation guidelines.

## Artifact Index
- /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_os_core_1/handoff.md — Draft of the OS-Core skill and system prompt
