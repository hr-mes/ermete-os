# BRIEFING — 2026-07-20T13:48:00+02:00

## Mission
Design and create the system prompt / skill definition for the OS-Core agent in `.agents/skills/ermete-core`.

## 🔒 My Identity
- Archetype: sub_orch
- Roles: orchestrator
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/sub_orch_m3
- Original parent: 0fec212f-ec13-414f-9826-ae4ff26ed3b3
- Original parent conversation ID: 0fec212f-ec13-414f-9826-ae4ff26ed3b3

## 🔒 My Workflow
- **Pattern**: Project / Canonical / Infinite
- **Scope document**: /var/home/ermete/GEMINI/ermete/.agents/sub_orch_m3/SCOPE.md
1. **Decompose**: See SCOPE.md. We have a single milestone for creating the OS-Core skill.
2. **Dispatch & Execute**:
   - **Direct (iteration loop)**: Explorer -> Worker -> Reviewer -> Challenger -> Auditor
3. **On failure**: Retry -> Replace -> Skip -> Redistribute -> Degrade -> Escalate
4. **Succession**: Self-succeed at 16 spawns.

## 🔒 Key Constraints
- Must NOT write the files directly; delegate to a worker.
- Return a report when the milestone passes the gate.

## Current Parent
- Conversation ID: 0fec212f-ec13-414f-9826-ae4ff26ed3b3
- Updated: not yet

## Key Decisions Made
- None yet.

## Team Roster
| Agent | Type | Work Item | Status | Conv ID |
|-------|------|-----------|--------|---------|
| Explorer 1 | teamwork_preview_explorer | Draft OS-Core Skill | completed | 1778fade-126d-43d5-a2b5-d9c7ec7b0164 |
| Explorer 2 | teamwork_preview_explorer | Draft OS-Core Skill | completed | 90fc5ebd-4bb3-4920-b566-a1c714fd1eea |
| Explorer 3 | teamwork_preview_explorer | Draft OS-Core Skill | completed | 39d401d8-4591-4e95-9eaf-54e93cc0da72 |
| Worker 1 | teamwork_preview_worker | Implement OS-Core Skill | completed | 7cec3380-65c8-473c-b8b0-4f86ae2425ed |
| Reviewer 1 | teamwork_preview_reviewer | Review OS-Core Skill | completed | 3da34642-0368-4173-aebe-6a9fc77be8e4 |
| Reviewer 2 | teamwork_preview_reviewer | Review OS-Core Skill | completed | d8512e0b-0301-4b22-b915-6ed6e1a67056 |
| Challenger 1 | teamwork_preview_challenger | Challenge OS-Core Skill | in-progress | f278a0e6-3ab3-4e4b-b7a7-2492be147712 |
| Challenger 2 | teamwork_preview_challenger | Challenge OS-Core Skill | completed | 963f5312-caea-4df5-8dec-38dec5154ee3 |
| Explorer 1 Gen2 | teamwork_preview_explorer | Fix OS-Core Skill | completed | 65eb2a68-ec8a-432d-b718-8335cf397fba |
| Explorer 2 Gen2 | teamwork_preview_explorer | Fix OS-Core Skill | in-progress | 56a390f8-d313-43f6-936c-45394f207810 |
| Explorer 3 Gen2 | teamwork_preview_explorer | Fix OS-Core Skill | completed | 3b33c7ed-ae4e-4e82-ba7c-04754262dc7a |
| Worker 1 Gen2 | teamwork_preview_worker | Implement OS-Core Skill Fix | completed | 91429bc3-a825-4cca-bab7-144cc9eb4c69 |
| Reviewer 1 Gen2 | teamwork_preview_reviewer | Review OS-Core Skill Fix | completed | dfd2cba7-45ed-4186-88a8-d5e2e2c8c46d |
| Reviewer 2 Gen2 | teamwork_preview_reviewer | Review OS-Core Skill Fix | completed | 12f9dc79-e873-499c-b3ed-23f17c0a7f84 |
| Challenger 1 Gen2 | teamwork_preview_challenger | Challenge OS-Core Skill Fix | in-progress | c9964c5f-6193-4c9d-b276-4e147574d2d4 |
| Challenger 2 Gen2 | teamwork_preview_challenger | Challenge OS-Core Skill Fix | completed | a5fa2c10-b813-4f41-b0df-6f21ecd5a31a |
| Auditor 1 Gen2 | teamwork_preview_auditor | Audit OS-Core Skill Fix | completed | 71d7a401-b016-435f-9f91-cb11f17f1048 |

## Succession Status
- Succession required: no
- Spawn count: 0 / 16
- Pending subagents: none
- Predecessor: none
- Successor: not yet spawned

## Active Timers
- Heartbeat cron: not started
- Safety timer: none

## Artifact Index
- /var/home/ermete/GEMINI/ermete/.agents/sub_orch_m3/SCOPE.md — scope specific milestone decomposition
