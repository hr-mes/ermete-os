# Handoff Report: Rust-UI Domain Skill Design

## 1. Observation
- `PROJECT.md` and `SCOPE.md` define `Rust-UI` as the Wayland/GTK4 Rust stack specialist, managing `ermete-shell-rs`, `ermete-settings-rs`, and Niri IPC.
- `ORIGINAL_REQUEST.md` (R3) dictates that agents must not overwrite or ignore existing codebase (e.g., `ermete-shell-rs/`), but integrate with it.
- `PROJECT.md` and `ORIGINAL_REQUEST.md` (R2) require an Interface Contract where out-of-scope tasks (e.g., system dependencies) must be explicitly delegated to the appropriate domain agent (like `Forge-Builder` for RPM packaging/dependencies).
- `SCOPE.md` specifies the target path for the skill as `.agents/skills/ermete-rust-ui/SKILL.md`.

## 2. Logic Chain
- To fulfill the agent specialization without overlap, the Rust-UI skill must explicitly define its domain boundaries, limiting its scope to Rust code, GTK4 UI, Wayland protocols, and Niri IPC.
- To enforce the integration rule, the skill must instruct the agent to analyze the existing `ermete-shell-rs/` and `ermete-settings-rs/` directories before proposing or implementing changes.
- To enforce the Interface Contract, the skill must explicitly list the other domains (`Forge-Builder`, `OS-Core`, `QA-DevOps`) and provide concrete examples of when and how to delegate tasks. E.g., if a new Rust crate requires a system-level C library to build, Rust-UI cannot install it; it must delegate to `Forge-Builder`.

## 3. Caveats
- The exact communication mechanism (e.g., `send_message` to a specific subagent ID vs. to an orchestrator) depends on the broader multi-agent setup. The skill design assumes the agent will send a structured request to its parent/orchestrator or directly to the named peer agent.
- We assume `ermete-shell-rs` and `ermete-settings-rs` are or will be Rust Cargo workspaces.

## 4. Conclusion
The proposed design for the Rust-UI skill should be written to `.agents/skills/ermete-rust-ui/SKILL.md`. 
It should contain the following structure:

```markdown
---
name: ermete-rust-ui
description: Domain skill for the Rust-UI agent. Specialist in Wayland/GTK4 Rust stack (ermete-shell-rs, ermete-settings-rs, Niri IPC).
---

# Rust-UI Domain Expert

You are the Rust-UI agent for Ermete OS. Your sole domain is the Wayland/GTK4 Rust stack.

## Core Responsibilities
- Develop and maintain `ermete-shell-rs` and `ermete-settings-rs`.
- Handle Wayland protocols and GTK4 UI components in Rust.
- Manage IPC communication with Niri.

## Strict Boundaries & Existing Code
- **DO NOT** overwrite or ignore existing code. Always investigate existing structures in `ermete-shell-rs/` before adding new features.
- **DO NOT** handle RPM packaging, Containerfiles, kernel modules, system-level configurations, or test orchestration.

## Delegation Protocol (Interface Contract)
You must delegate out-of-scope tasks. When blocked by a requirement outside your domain, explicitly request assistance from the appropriate agent:
- **System Dependencies (C libs, RPMs, isolated builds):** Delegate to `Forge-Builder`. (Example: "I need `libadwaita-devel` added to the build environment.")
- **Layer 0 / Kernel / DKMS:** Delegate to `OS-Core`.
- **Test VMs / Orchestration (Justfile) / ISOs:** Delegate to `QA-DevOps`.

**How to delegate:** Formulate a concise message identifying the target domain and the specific requirement, then send it to the orchestrator or directly to the respective agent via the communication protocol.
```

## 5. Verification Method
- **Inspection:** Verify that `.agents/skills/ermete-rust-ui/SKILL.md` contains the exact content proposed.
- **Dry Run:** Create a dummy task "Add a feature to core-shell-rs that depends on libfoo-devel". Ensure the Rust-UI agent reads the skill and emits a delegation request to `Forge-Builder` rather than attempting to write an RPM spec or run `dnf install` itself.
