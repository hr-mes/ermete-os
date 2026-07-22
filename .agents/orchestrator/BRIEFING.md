# BRIEFING — 2026-07-20T14:01:10+02:00

## Mission
Implement the Ermete OS specialized agent ecosystem (Forge-Builder, Rust-UI, OS-Core, QA-DevOps), configure their system prompts for delegation/safety, and create a test scenario.

## 🔒 My Identity
- Archetype: Project Orchestrator
- Roles: orchestrator, user_liaison, human_reporter, successor
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/orchestrator
- Original parent: c9431b99-ab0f-470f-9d7a-bc3507e187bc
- Original parent conversation ID: c9431b99-ab0f-470f-9d7a-bc3507e187bc

## 🔒 My Workflow
- **Pattern**: Project
- **Scope document**: /var/home/ermete/GEMINI/ermete/.agents/orchestrator/PROJECT.md
1. **Decompose**: Split into Agent Prompt Refinement and Test Scenario Creation.
2. **Dispatch & Execute**: Single worker strategy used due to simplicity of markdown edits.
3. **On failure**: Retry -> Replace -> Skip -> Redistribute -> Redesign -> Escalate.
4. **Succession**: At 16 spawns, write handoff.md, spawn successor.
- **Work items**:
  1. System Prompt Refinement [done]
  2. Test Scenario Creation [done]
- **Current phase**: 4
- **Current focus**: Final reporting

## 🔒 Key Constraints
- NEVER write, modify, or create source code files directly.
- Ensure subagents do not overwrite existing work in `ermete-forge/` and `ermete-shell-rs/`.
- Use send_message to report when all milestones are complete.

## Current Parent
- Conversation ID: c9431b99-ab0f-470f-9d7a-bc3507e187bc
- Updated: not yet

## Team Roster
| Agent | Type | Work Item | Status | Conv ID |
|-------|------|-----------|--------|---------|
| Ecosystem Implementation Worker | teamwork_preview_worker | M1 & M2 | completed | 86b4d718-6442-4f9f-ba95-eac5460acd45 |

## Succession Status
- Succession required: no
- Spawn count: 1 / 16
- Pending subagents: none
- Predecessor: none
- Successor: not yet spawned

## Specialist Agent Registry

### Forge Specialists (10)
- forge-build-analyst: Build failure analysis
- forge-dep-watchdog: Dependency breakage detection
- forge-patch-compat: Kernel patch compatibility
- forge-nvidia-watch: NVIDIA driver/kernel compatibility
- forge-opt-guard: Compiler optimization monitoring
- forge-spec-keeper: RPM spec file maintenance
- forge-cache-optimizer: Cache optimization
- forge-upstream-spy: Upstream release monitoring
- forge-vuln-scanner: Security vulnerability scanning
- forge-sign-guard: RPM signing and GPG keys

### OS Specialists (9)
- os-selinux-craft: SELinux policy development
- os-firewall-guard: Firewalld configuration
- os-firstboot-doctor: First-boot services
- os-containerfile-lint: Containerfile validation
- os-disk-builder: Disk/ISO generation
- os-vm-tester: VM testing and boot validation
- os-cosign-guard: Image signing (Cosign OIDC)
- os-supply-chain: Supply chain security
- os-perf-benchmark: Performance benchmarking

### Shared Specialists (2)
- shared-docs-sync: Documentation synchronization
- shared-ci-doctor: CI/CD pipeline health
