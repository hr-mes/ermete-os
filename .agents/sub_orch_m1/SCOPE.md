# Scope: Forge-Builder

## Architecture
- `Forge-Builder`: RPM packaging, macros, OCI isolation, bash scripts (`forge/specs`, `forge/scripts`).

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Define Forge-Builder | Create domain skill / system prompt for Forge-Builder | none | DONE |

## Interface Contracts
### Forge-Builder ↔ All Agents
- Agents must delegate out-of-scope tasks by explicitly identifying the required domain and forwarding the request to the Orchestrator or the respective agent via the communication protocol.
- `Forge-Builder` handles RPM packaging, macros, dependencies, OCI isolation, and bash scripts.
- Must integrate with existing codebase in `forge/` and not overwrite it blindly.
