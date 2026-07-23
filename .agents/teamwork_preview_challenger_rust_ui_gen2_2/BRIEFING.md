# BRIEFING — 2026-07-20T13:55:24Z

## Mission
Verify ermete-rust-ui skill for strict dependency restrictions regarding missing Wayland protocol headers.

## 🔒 My Identity
- Archetype: Challenger
- Roles: critic, specialist
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_challenger_rust_ui_gen2_2
- Original parent: e57f7225-89e1-4eb0-b701-23f51a1871cb
- Milestone: Milestone 2: Define Rust-UI
- Instance: 1 of 1

## 🔒 Key Constraints
- Review-only — do NOT modify implementation code
- Network restriction: CODE_ONLY (no external websites/services)

## Current Parent
- Conversation ID: e57f7225-89e1-4eb0-b701-23f51a1871cb
- Updated: 2026-07-20T13:54:26Z

## Review Scope
- **Files to review**: `/var/home/ermete/GEMINI/ermete/.agents/skills/ermete-rust-ui/SKILL.md`
- **Review criteria**: Check if rules strictly prevent using package managers (rpm-ostree, etc.), arbitrary shell commands (curl), and vendoring via build.rs for missing system dependencies. Stress test wording for loopholes.

## Key Decisions Made
- Identified that the rule wording contains loopholes allowing for `build.rs` to download into `OUT_DIR` and agent tools to write to `/tmp`, bypassing the intended system dependency delegation.

## Artifact Index
- /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_challenger_rust_ui_gen2_2/handoff.md — Verdict report
- /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_challenger_rust_ui_gen2_2/progress.md — Progress log

## Attack Surface
- **Hypotheses tested**: Can the agent bypass "no downloading into source tree" and "no shell commands"?
- **Vulnerabilities found**: 
  1. `build.rs` with `reqwest` fetching to `OUT_DIR` (bypasses source tree restriction).
  2. `write_to_file` tool writing to `/tmp` (bypasses shell command and source tree restrictions).
- **Untested angles**: Using procedural macros to fetch data at compile time.

## Loaded Skills
- ermete-rust-ui: `/var/home/ermete/GEMINI/ermete/.agents/skills/ermete-rust-ui/SKILL.md` — Domain skill for the Rust-UI agent.
