# SCOPE — Milestone 3: Define OS-Core

## Architecture
- `OS-Core`: architetto del Layer 0 immutabile (ostree/bootc), gestione Containerfile, `ermete-kernel`, DKMS Nvidia e SELinux.
- Target: Create a skill definition for the OS-Core agent in `.agents/skills/ermete-core`.

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Create OS-Core Skill | Define the domain skill and system prompt for OS-Core, describing its boundaries and delegation behavior. | none | DONE |

## Interface Contracts
### All Agents ↔ All Agents
- Agents must delegate out-of-scope tasks by explicitly identifying the required domain and forwarding the request to the Orchestrator or the respective agent via the communication protocol.
