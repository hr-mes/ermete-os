# BRIEFING — 2026-07-20T13:54:26+02:00

## Mission
Challenge the `ermete-rust-ui` SKILL.md rules against dependency injection and package installation loopholes.

## 🔒 My Identity
- Archetype: EMPIRICAL CHALLENGER
- Roles: critic, specialist
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_challenger_rust_ui_gen2_1
- Original parent: e57f7225-89e1-4eb0-b701-23f51a1871cb
- Milestone: 2 (Define Rust-UI)
- Instance: 1 of 1

## 🔒 Key Constraints
- Review-only — do NOT modify implementation code.
- Ensure strict prevention of any package managers, arbitrary shell commands, or vendoring via `build.rs` for missing system dependencies.

## Current Parent
- Conversation ID: e57f7225-89e1-4eb0-b701-23f51a1871cb
- Updated: 2026-07-20T13:54:26+02:00

## Review Scope
- **Files to review**: `/var/home/ermete/GEMINI/ermete/.agents/skills/ermete-rust-ui/SKILL.md`
- **Interface contracts**: Rust-UI Agent role boundaries.
- **Review criteria**: Completeness and strictness of constraints regarding system dependency acquisition.

## Key Decisions Made
- Conducted adversarial analysis on Rules 4, 5, 6 in SKILL.md.
- Decided to fail the ruleset due to geographic and mechanical loopholes that allow local build bypassing.

## Attack Surface
- **Hypotheses tested**: The rules prevent all ways of obtaining missing system dependencies locally.
- **Vulnerabilities found**: 
  1. Geographic loophole: "directly into the source tree" (Rule 6) allows out-of-tree vendoring (e.g., `/tmp`).
  2. Mechanical loophole: "shell commands to download" (Rule 4) allows using `write_to_file` to write known headers/XML protocols without downloading.
- **Untested angles**: Behavior of LLM when explicitly told to prioritize the spirit of Delegation Protocol over finding workarounds.

## Loaded Skills
- [TBD]

## Artifact Index
- ORIGINAL_REQUEST.md — The original dispatch request.
- BRIEFING.md — This file.
- progress.md — Heartbeat and status.
- handoff.md — The final verification report and verdict.
