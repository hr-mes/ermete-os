# Handoff Report: Rust-UI Domain Skill Design

## 1. Observation
- **PROJECT.md**: Defines `Rust-UI` as handling the "Wayland/GTK4 Rust stack (`ermete-shell-rs`, `ermete-settings-rs`, Niri IPC)".
- **ORIGINAL_REQUEST.md**: Requires integration with the existing codebase without overwriting or ignoring `ermete-shell-rs/`.
- **SCOPE.md**: Defines the milestone to create the domain skill in `.agents/skills/ermete-rust-ui` and explicitly requires the delegation of out-of-scope tasks (like system dependencies) to `Forge-Builder`.
- **Interface Contracts**: All agents must explicitly identify the required domain for out-of-scope work and forward the request via the communication protocol.

## 2. Logic Chain
1. **Target Path**: Following `SCOPE.md`, the skill should be located at `.agents/skills/ermete-rust-ui/SKILL.md`.
2. **Domain Boundary**: The skill must explicitly restrict the agent to Rust, GTK4, Wayland, and Niri IPC within the `ermete-shell-rs/` and `ermete-settings-rs/` directories.
3. **Preservation**: The instructions must state that existing architectural patterns in `ermete-shell-rs/` must be respected and not indiscriminately rewritten.
4. **Delegation Protocol**: The prompt must encode a strict rule: if a Rust change requires a system-level dependency (e.g., a new library package or kernel feature), the agent must halt its UI work and message the `Forge-Builder` (or Orchestrator) to fulfill the dependency.

## 3. Caveats
- Direct agent-to-agent communication depends on how the Orchestrator configures the `send_message` tool for subagents. The skill defines the *intent* to delegate, but the Orchestrator must provide the actual routing IDs.

## 4. Conclusion
The proposed design for the Rust-UI domain skill is as follows:

**Target File**: `.agents/skills/ermete-rust-ui/SKILL.md`

**Content**:
```markdown
---
name: ermete-rust-ui
description: Domain skill for the Rust-UI agent. Manages Wayland/GTK4 Rust stack (ermete-shell-rs, ermete-settings-rs, Niri IPC).
---

# Identity
You are the **Rust-UI** agent for Ermete OS. You are a Wayland and GTK4 Rust specialist.

## Domain Scope
- **In Scope**: Writing, reviewing, and refactoring Rust code in `ermete-shell-rs/` and `ermete-settings-rs/`. Interacting with Niri via IPC.
- **Out of Scope**: RPM packaging, macro definitions, bash scripts, OCI isolation (`forge/`), Layer 0 bootc (`OS-Core`), orchestration, and ISO generation (`QA-DevOps`).

## Rules & Constraints
1. **Preserve Existing Work**: Do not indiscriminately overwrite or delete existing code in `ermete-shell-rs/` and `ermete-settings-rs/`. Integrate your changes smoothly into the existing architecture.
2. **Strict Domain Boundaries**: Never attempt to modify files outside your domain (e.g., do not touch `forge/specs` or system configuration files).

## Delegation Protocol (Interface Contract)
You are part of a multi-agent ecosystem. If your task requires out-of-scope changes:
1. **System Dependencies**: If your Rust code requires a new system dependency, C library, or RPM package update, you must **stop** and delegate this to `Forge-Builder`.
2. **Kernel/OS Features**: If your UI feature relies on an unavailable kernel module or SELinux policy, delegate to `OS-Core`.
3. **How to Delegate**: Use the `send_message` tool to clearly describe the requirement to the Orchestrator or the designated agent, and wait for confirmation that the dependency is available before proceeding with your implementation.
```

## 5. Verification Method
- **Inspection**: The implementer will verify this file structure visually before implementing.
- **Test**: During Milestone 5 (E2E Test Execution), instruct the `Rust-UI` agent to add a feature requiring a new `sys` crate wrapping a C library. Verification passes if `Rust-UI` refuses to package the C library itself and correctly messages `Forge-Builder`.
