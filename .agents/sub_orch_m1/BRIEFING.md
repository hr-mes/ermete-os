# BRIEFING — 2026-07-20T13:48:31+02:00

## Mission
Define Forge-Builder agent: design and create the system prompt / skill definition for the Forge-Builder agent in `.agents/skills/ermete-forge`.

## 🔒 My Identity
- Archetype: sub-orchestrator
- Roles: orchestrator, user_liaison, human_reporter, successor
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/sub_orch_m1
- Original parent: top-level
- Original parent conversation ID: 949406c7-d558-4d3a-bf4e-350b79f454bb

## 🔒 My Workflow
- **Pattern**: Project
- **Scope document**: /var/home/ermete/GEMINI/ermete/.agents/sub_orch_m1/SCOPE.md
1. **Decompose**: We are running the Iteration Loop directly because this is a single milestone (Define Forge-Builder).
2. **Dispatch & Execute**:
   - **Direct (iteration loop)**: Explorer -> Worker -> Reviewer -> Challenger -> Auditor
3. **On failure**:
   - Retry: nudge stuck agent or re-send task
   - Replace: spawn fresh agent with partial progress
   - Skip: proceed without (only if non-critical)
   - Redistribute: split stuck agent's remaining work
   - Redesign: re-partition decomposition
   - Escalate: report to parent (sub-orchestrators only, last resort)
4. **Succession**: at 16 spawns, write handoff.md, spawn successor
- **Work items**:
  1. Iteration 1 [in-progress]
- **Current phase**: 2
- **Current focus**: Iteration 1

## 🔒 Key Constraints
- NEVER write, modify, or create source code files directly.
- Never reuse a subagent after it has delivered its handoff — always spawn fresh

## Current Parent
- Conversation ID: 949406c7-d558-4d3a-bf4e-350b79f454bb
- Updated: 2026-07-20T13:48:31+02:00

## Key Decisions Made
- Iteration Loop started.

## Team Roster
| Agent | Type | Work Item | Status | Conv ID |
|-------|------|-----------|--------|---------|
| Explorer 1 | teamwork_preview_explorer | Design Forge-Builder skill | completed | 5df23ed1-42c7-45dd-b9e0-a7eba1e12dd4 |
| Explorer 2 | teamwork_preview_explorer | Design Forge-Builder skill | completed | a9a32257-3e5f-4689-a21b-b9a09b6106e7 |
| Explorer 3 | teamwork_preview_explorer | Design Forge-Builder skill | completed | 3556c32b-29fa-43c3-91d3-d938d539905d |
| Worker | teamwork_preview_worker | Implement Forge-Builder skill | completed | 184ad3a6-6a8e-48c8-8837-919881d23e1f |
| Reviewer 1 | teamwork_preview_reviewer | Review SKILL.md | pending | a012db7d-89b6-42b5-9bfe-f3fda0b3bfda |
| Reviewer 2 | teamwork_preview_reviewer | Review SKILL.md | pending | 460b8310-5d9d-4f50-8659-6c7b4fff1ebe |
| Challenger 1 | teamwork_preview_challenger | Challenge SKILL.md | pending | 66b32743-2ace-48af-aa0a-e8b7effea50e |
| Challenger 2 | teamwork_preview_challenger | Challenge SKILL.md | pending | 77c54974-be0e-41bc-881b-8ea71bc20059 |
| Auditor | teamwork_preview_auditor | Audit SKILL.md | pending | fe61a953-46b5-4df4-863b-edf15f8eabac |

## Succession Status
- Succession required: no
- Spawn count: 0 / 16
- Pending subagents: none
- Predecessor: none
- Successor: not yet spawned

## Active Timers
- Heartbeat cron: not started
- Safety timer: none
- On succession: kill all timers before spawning successor
- On context truncation: run `manage_task(Action="list")` — re-create if missing

## Artifact Index
- /var/home/ermete/GEMINI/ermete/.agents/sub_orch_m1/SCOPE.md — Milestone 1 scope
