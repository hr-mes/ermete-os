# Handoff Report: Rust-UI Domain Skill Design

## 1. Observation
- `PROJECT.md` defines four main agents, including `Rust-UI`, which is the "Wayland/GTK4 Rust stack" specialist (`ermete-shell-rs`, `ermete-settings-rs`, Niri IPC).
- `PROJECT.md` mandates Interface Contracts: "Agents must delegate out-of-scope tasks by explicitly identifying the required domain and forwarding the request to the Orchestrator or the respective agent via the communication protocol."
- `SCOPE.md` reiterates the requirement to create the domain skill in `.agents/skills/ermete-rust-ui/` and emphasizes delegation of system dependencies to `Forge-Builder`.
- The repository root currently does not contain the `ermete-shell-rs` or `ermete-settings-rs` folders, suggesting this agent may also be responsible for bootstrapping them, or they will be created subsequently.

## 2. Logic Chain
1. Since the `Rust-UI` agent focuses strictly on the Rust user space and Wayland/GTK4 stack, it must have clear rules bounding it to code inside `ermete-shell-rs` and `ermete-settings-rs` (or similar Rust-specific directories).
2. Because it lacks OS-level packaging permissions, any missing system dependency (e.g., C/C++ libraries, `gtk4-devel`, Wayland protocols at the OS level) must trigger a delegation workflow to `Forge-Builder` or `OS-Core`.
3. To meet the constraints of not implementing directly but providing a design, we must define the proposed `SKILL.md` content block by block so an implementer agent can reliably write it.

## 3. Caveats
- The exact frameworks for GTK4 (e.g., `gtk4-rs`, `relm4`) and IPC (e.g., Niri's exact socket format) aren't deeply specified in the root `PROJECT.md`. The design assumes standard `gtk-rs` and standard Rust Wayland practices.
- The `ermete-shell-rs` folder does not exist yet; the skill should assume the agent might need to initialize these Rust workspaces using `cargo`.

## 4. Conclusion
The domain skill for `Rust-UI` should be located at `.agents/skills/ermete-rust-ui/SKILL.md`. It must firmly establish the agent as the Rust GUI specialist while explicitly restricting it from touching RPMs, `Containerfile`, or system-wide configurations, enforcing a strict handoff protocol to `Forge-Builder` for those tasks.

### Proposed SKILL.md Structure and Rules:
```markdown
---
name: rust-ui
description: Wayland/GTK4 Rust specialist for ermete-shell-rs, ermete-settings-rs, Niri IPC
---

# Rust-UI Domain Specialist

You are the Rust-UI agent. Your exclusive domain is the Rust user space GUI stack for Ermete OS.

## Domain Boundaries
- **IN SCOPE**: Writing Rust code for `ermete-shell-rs` and `ermete-settings-rs`.
- **IN SCOPE**: Wayland protocol interactions and GTK4 (`gtk-rs`) UI development.
- **IN SCOPE**: Niri IPC integration (Unix sockets / JSON).
- **OUT OF SCOPE**: RPM packaging, `Containerfile` modifications, OS tree generation, bash scripts for system configurations.

## Delegation Protocol (Crucial)
If your Rust projects require system-level dependencies (e.g., missing `-devel` packages, system headers, or library `.so` files not present in the current container):
1. **DO NOT** try to install them via `dnf` or modify `Containerfile`.
2. Document the exact dependencies required in a `handoff.md` file.
3. Message your Orchestrator or the `Forge-Builder` agent, stating: "I am blocked on system dependencies. Please package/install [dependency list] and return control to me when done."

## Coding Guidelines
- **Rust**: Use `cargo` for workspace management. Enforce `clippy` and `rustfmt`. Use `anyhow`/`thiserror` for error handling.
- **GTK4/Wayland**: Follow idiomatic `gtk-rs` patterns. Keep UI state and logic strictly separated.
- **Niri IPC**: Handle async socket communication cleanly (e.g., using `tokio`).
```

## 5. Verification Method
- Inspect the proposed design in this document.
- The implementer agent should read this `handoff.md`, generate the target `.agents/skills/ermete-rust-ui/SKILL.md`, and the Orchestrator should verify that the delegation protocol is explicitly included.
