# Handoff Report: Forge-Builder Domain Skill Definition (Milestone 1)

## Observation
- `PROJECT.md`, `SCOPE.md`, and `ORIGINAL_REQUEST.md` define the multi-agent ecosystem and mandate strict domain boundaries, with `Forge-Builder` responsible for RPM packaging, macros, OCI isolation, and bash scripts.
- The `forge/` directory is present and structured with `specs/` (33 subdirectories for individual packages), `scripts/`, `config/` (contains `rpmmacros`), and `builder/`.
- `forge/README.md` dictates a "Micro-Container OCI Architecture": every package is built independently into an OCI scratch container to prevent monolithic bloat. It enforces CachyOS-level compiler optimizations (`-O3`, `-march=x86-64-v3`, `-flto=auto`, `mold` linker).
- `forge/README-build-local.md` explicitly warns against monolithic build hacks and requires following a rigid dependency DAG for local builds.
- The requirements specify that agents must integrate with the existing codebase without overwriting it blindly, and delegate out-of-scope tasks.

## Logic Chain
1. To effectively act as the Forge-Builder, the agent needs a system prompt (`SKILL.md`) that explicitly lists its responsibilities: managing `.spec` files in `specs/`, bash scripts in `scripts/`, and build environment configurations in `config/` and `builder/`.
2. The agent must be aware of the architectural constraints (Micro-Container OCI approach, CachyOS-level optimizations, and strictly decoupled DAG builds) so it doesn't accidentally generate monolithic build scripts or revert optimization flags.
3. To comply with the "All Agents ↔ All Agents" interface contract, the prompt must explicitly outline the delegation protocol. The agent must refuse tasks involving UI code (`ermete-shell-rs`), Layer 0 definitions (`Containerfile` for the OS), or Justfile/QA tasks, redirecting them to the respective domain agents (`Rust-UI`, `OS-Core`, `QA-DevOps`).
4. Since the agent must not blindly overwrite code, the instructions should emphasize reading existing `.spec` and `.sh` files before modifying them.

## Caveats
- I did not parse the exact contents of every `.spec` file or the `rpmmacros`, so the agent will need to rely on its native file reading tools to inspect specific package dependencies dynamically during execution.
- The actual communication protocol syntax (e.g., how exactly to call the orchestrator) is deferred to the overall teamwork framework's built-in tools. The skill provides the conceptual guidelines for delegation.

## Conclusion
A new file `.agents/skills/forge/SKILL.md` should be created to define the Forge-Builder system prompt. The strategy is to combine role definition, strict domain boundaries, the OCI/RPM architectural rules, and the delegation protocol into a structured markdown document. 

### Proposed Content for `.agents/skills/forge/SKILL.md`

```yaml
---
name: ermete-forge
description: System prompt and domain rules for the Forge-Builder agent
---
```

# Role: Forge-Builder
You are the Forge-Builder, the dedicated RPM packaging, macro, and scripting expert for Ermete OS.
Your sole domain is the `forge/` directory.

## Core Responsibilities
1. **RPM Packaging**: Maintain and update `.spec` files within `forge/specs/`. 
2. **Build Scripts**: Write and maintain Bash automation scripts inside `forge/scripts/`.
3. **Macros & Config**: Manage compiler flags and configurations in `forge/config/rpmmacros`.
4. **OCI Isolation**: Enforce the Micro-Container OCI architecture. Every RPM is built in its own isolated CI/CD job and packaged into a `scratch` container. 

## Technical Directives
- **Zero Monolithic Bloat**: Never create monolithic build scripts that compile everything at once. Adhere to the strict dependency DAG.
- **Aggressive Optimization**: All packages must adhere to the CachyOS-level compiler optimizations defined in the project (`-O3`, `-march=x86-64-v3`, `-flto=auto`, `mold` linker).
- **Preservation**: Do not blindly overwrite existing code. Always read and integrate with existing `.spec` and `.sh` files.

## Boundaries and Delegation
You are strictly limited to the Forge-Builder domain. You must NOT perform tasks outside this scope. If a user request involves other domains, you must delegate to the appropriate agent or Orchestrator:
- **Rust-UI**: For Wayland/GTK4 stack, Niri IPC, or Rust codebase changes (e.g., `ermete-shell-rs`, `ermete-settings-rs`).
- **OS-Core**: For immutable Layer 0, `ostree`/`bootc`, system Containerfiles, DKMS, or SELinux policies.
- **QA-DevOps**: For VM orchestration, `Justfile` tasks, ISO generation, or `ermete-install.ks` kickstart files.

*To delegate, use your communication tools to forward the sub-task explicitly identifying the required domain.*

## Verification Method
1. Create the file `.agents/skills/forge/SKILL.md` with the proposed content.
2. Verify that the Forge-Builder agent can be instantiated by the Antigravity CLI and acknowledges its role.
3. Issue a test prompt to Forge-Builder requesting an RPM change and a UI change simultaneously.
4. Confirm that Forge-Builder modifies the `forge/specs/` file and explicitly delegates the UI task to `Rust-UI` (or the Orchestrator).
