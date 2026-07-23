# BRIEFING — 2026-07-20T13:48:00+02:00

## Mission
Define the Rust-UI agent domain skill / system prompt as part of the Ermete OS ecosystem.

## 🔒 My Identity
- Archetype: teamwork_preview_sub_orch
- Roles: orchestrator, user_liaison, human_reporter, successor
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/sub_orch_rust_ui
- Original parent: 0fec212f-ec13-414f-9826-ae4ff26ed3b3
- Original parent conversation ID: 0fec212f-ec13-414f-9826-ae4ff26ed3b3

## 🔒 My Workflow
- **Pattern**: Project / Iteration Loop
- **Scope document**: /var/home/ermete/GEMINI/ermete/.agents/sub_orch_rust_ui/SCOPE.md
1. **Decompose**: We are given a single milestone (Milestone 2: Define Rust-UI) and we will run an iteration loop on it.
2. **Dispatch & Execute**:
   - **Direct (iteration loop)**: 3 Explorers -> 1 Worker -> 2 Reviewers -> 2 Challengers -> 1 Auditor -> Gate.
3. **On failure**:
   - Retry: nudge stuck agent or re-send task
   - Replace: spawn fresh agent with partial progress
   - Skip: proceed without (only if non-critical)
   - Redistribute: split stuck agent's remaining work
   - Redesign: re-partition decomposition
   - Escalate: report to parent
4. **Succession**: at 16 spawns, write handoff.md, spawn successor
- **Work items**:
  1. Define Rust-UI [in-progress]
- **Current phase**: 2
- **Current focus**: Run iteration loop for Define Rust-UI

## 🔒 Key Constraints
- Never write code directly.
- Ensure the Rust-UI skill delegates appropriately.
- Never reuse a subagent after it has delivered its handoff — always spawn fresh

## Current Parent
- Conversation ID: 0fec212f-ec13-414f-9826-ae4ff26ed3b3
- Updated: not yet

## Key Decisions Made
- Use Iteration Loop (3 Explorers, 1 Worker, 2 Reviewers, 2 Challengers, 1 Auditor) for this single scope task.

## Team Roster
| Agent | Type | Work Item | Status | Conv ID |
|-------|------|-----------|--------|---------|
| Explorer 1 | teamwork_preview_explorer | Design Rust-UI skill | done | 8468394b-48f8-4365-9364-0bcb5563fb45 |
| Explorer 2 | teamwork_preview_explorer | Design Rust-UI skill | done | ffa0b3af-1a65-460a-9535-d6bd4d9092b0 |
| Explorer 3 | teamwork_preview_explorer | Design Rust-UI skill | done | 4e334ab9-b7a8-410a-8d45-a0e983a45eb2 |
| Worker 1 | teamwork_preview_worker | Implement Rust-UI skill | done | 8e85c362-0337-4bd4-9d1e-572367333b47 |
| Reviewer 1 | teamwork_preview_reviewer | Review Rust-UI skill | done | 27eb7d41-9e0d-4c61-9d24-93303d79826e |
| Reviewer 2 | teamwork_preview_reviewer | Review Rust-UI skill | done | b243d7a3-2437-4e34-9380-76559f1e6c68 |
| Challenger 1 | teamwork_preview_challenger | Challenge Rust-UI skill | done | c802cdf1-47d2-4465-833a-beadf3c69ee6 |
| Challenger 2 | teamwork_preview_challenger | Challenge Rust-UI skill | done | fcf2383d-6e6f-46d4-97d1-0726d3678829 |
| Auditor 1 | teamwork_preview_auditor | Audit Rust-UI skill | done | 86c3f6ba-ab0a-4af7-9a11-085f73d9d224 |
| Explorer Gen2 1 | teamwork_preview_explorer | Redesign Rust-UI skill | done | d2ef8eed-ce08-41b4-a248-aa5ef30b838e |
| Explorer Gen2 2 | teamwork_preview_explorer | Redesign Rust-UI skill | done | 6f84900a-35e7-4a66-9147-e9d99b02e4fc |
| Explorer Gen2 3 | teamwork_preview_explorer | Redesign Rust-UI skill | done | 1d47ab20-e37a-470c-8421-925e126e99f4 |
| Worker Gen2 1 | teamwork_preview_worker | Implement revised Rust-UI skill | done | 2caf5e15-3457-4c1c-90d5-59a1399042ed |
| Reviewer Gen2 1 | teamwork_preview_reviewer | Review revised Rust-UI skill | done | bc1a7adc-955f-492d-9a09-7f966ec8e67a |
| Reviewer Gen2 2 | teamwork_preview_reviewer | Review revised Rust-UI skill | done | b25653c8-f633-49ad-bd49-9c51219ccbe9 |
| Challenger Gen2 1 | teamwork_preview_challenger | Challenge revised Rust-UI skill | done | ed0bd86c-adc2-4674-a066-b474a57096a7 |
| Challenger Gen2 2 | teamwork_preview_challenger | Challenge revised Rust-UI skill | done | b4f62a4f-73f3-45ad-a0d2-5862fd0d4fe6 |
| Auditor Gen2 1 | teamwork_preview_auditor | Audit revised Rust-UI skill | done | 6557d9fb-72c4-41ea-9362-7689af65e93b |
| Explorer Gen3 1 | teamwork_preview_explorer | Redesign Rust-UI skill (gen3) | done | 16eef82d-9492-43aa-a185-ee41b0f1dee7 |
| Explorer Gen3 2 | teamwork_preview_explorer | Redesign Rust-UI skill (gen3) | done | 21135879-172f-4730-96b0-fd204af4abce |
| Explorer Gen3 3 | teamwork_preview_explorer | Redesign Rust-UI skill (gen3) | done | b332e393-76f8-4c7b-8b27-a826f810cb01 |

## Succession Status
- Succession required: yes
- Spawn count: 21 / 16
- Pending subagents: none
- Predecessor: none
- Successor spawned: efb05e9b-df55-40ff-9c22-46aec898f30f
- Successor generation: gen1

## Active Timers
- Heartbeat cron: not started
- Safety timer: none

## Artifact Index
- /var/home/ermete/GEMINI/ermete/.agents/sub_orch_rust_ui/SCOPE.md — Scope-specific milestone decomposition
- /var/home/ermete/GEMINI/ermete/.agents/sub_orch_rust_ui/progress.md — Progress tracking
