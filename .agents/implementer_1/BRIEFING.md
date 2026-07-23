# BRIEFING — 2026-07-20T08:20:00Z

## Mission
Remove `systemd-run` wrappers from `/var/home/ermete/GEMINI/ermete/.gemini/plugins/ermete_team/README.md` to pass integrity checks.

## 🔒 My Identity
- Archetype: Implementer
- Roles: implementer, qa, specialist
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/implementer_1
- Original parent: a259de4f-c695-4caa-bacf-0493f26fb993
- Milestone: Integrity verification fix

## 🔒 Key Constraints
- DO NOT hardcode test results.
- DO NOT use systemd-run wrappers anywhere in the README.

## Current Parent
- Conversation ID: a259de4f-c695-4caa-bacf-0493f26fb993
- Updated: not yet

## Task Summary
- **What to build**: Fix README.md lines 17, 37, 90 to replace `systemd-run --user --scope podman run -d` with `podman create --cidfile` and `podman start "$CID"`.
- **Success criteria**: No `systemd-run` found via grep in the README file.
- **Interface contracts**: /var/home/ermete/GEMINI/ermete/.gemini/plugins/ermete_team/README.md
- **Code layout**: TBD

## Key Decisions Made
- Replaced `podman run` with `podman create` and `podman start "$CID"` sequence.
- Verified removal with `grep_search`.

## Artifact Index
- /var/home/ermete/GEMINI/ermete/.gemini/plugins/ermete_team/README.md — target file
