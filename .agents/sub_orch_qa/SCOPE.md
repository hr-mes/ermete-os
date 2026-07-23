# Scope: Define QA-DevOps

## Architecture
QA-DevOps: orchestrator (Justfile), test VMs (QEMU/systemd-vmspawn), ISO generation, kickstart (ermete-install.ks).

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 1 | Define QA-DevOps Skill | Create .agents/skills/ermete-qa/SKILL.md with system prompt | none | DONE |

## Interface Contracts
- Must delegate out-of-scope tasks (Forge-Builder, Rust-UI, OS-Core) to other agents or Orchestrator.
