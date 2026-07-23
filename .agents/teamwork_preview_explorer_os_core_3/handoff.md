# OS-Core Agent Definition Handoff

## 1. Observation
- **Scope & Project definitions**: `SCOPE.md` and `PROJECT.md` state that `OS-Core` is responsible for Immutable Layer 0 (ostree/bootc), `Containerfile`, `ermete-kernel`, DKMS Nvidia, and SELinux. 
- **Target path**: `SCOPE.md` requests the skill definition to be placed in `.agents/skills/ermete-core`.
- **Siblings**: The ecosystem contains `Forge-Builder` (RPMs, bash scripts), `Rust-UI` (Wayland/GTK4), and `QA-DevOps` (Justfile, VMs, ISOs).
- **Architecture**: `ermete-base-blueprint.md` and `Containerfile` highlight a strict separation between Layer 0 (kernel/drivers) and Layer 1 (UI), using `bootc`, `ostree`, and `dracut` for deterministic builds.
- **Constraints**: I am tasked to draft the recommended contents and NOT to write the actual skill files.

## 2. Logic Chain
1. **Identity & Scope**: The OS-Core agent is the architect of the underlying bootable system. It must manage the `Containerfile` and base layers, ensuring the system remains immutable and atomic.
2. **Boundaries & Delegation**: To adhere to `PROJECT.md`'s Interface Contracts, the agent must be explicitly told what it *cannot* do.
   - Any requirement for new RPM packages or macro modifications must trigger a delegation to `Forge-Builder`.
   - Any user interface or Wayland integration must be delegated to `Rust-UI`.
   - Any testing or ISO generation must be delegated to `QA-DevOps`.
3. **Format**: Following typical skill conventions, a `SKILL.md` file with YAML frontmatter is the optimal way to provide both the tool definition and the system prompt instructions.

## 3. Caveats
- The exact location of SELinux policies wasn't fully explored, but the domain ownership by OS-Core is established.
- It is assumed that the Orchestrator handles the actual routing of delegated messages, so the prompt instructs the agent to forward out-of-scope requests to the Orchestrator or the specific agent.

## 4. Conclusion
The `OS-Core` agent should be defined via a `SKILL.md` file in `.agents/skills/ermete-core/`. The file should contain both its operational boundaries and communication protocols.

### Recommended Content for `.agents/skills/ermete-core/SKILL.md`

```markdown
---
name: ermete-core
description: Domain skill for OS-Core agent - manages immutable Layer 0 (ostree/bootc), Containerfile, kernel, DKMS Nvidia, and SELinux.
---
# Ermete OS-Core Agent Prompt & Instructions

You are the OS-Core agent for Ermete OS.
Your primary domain is the Immutable Layer 0, the `Containerfile`, the custom `ermete-kernel`, hardware integrations (DKMS Nvidia), and SELinux policies.

## Core Responsibilities
1. **Immutable Layer 0**: Manage `ostree` and `bootc` configurations. Ensure system immutability, zero-trust integrity, and atomic updates.
2. **Containerfile**: You own the root `Containerfile` and the `system-config-container` directory. You configure the base OS image layers.
3. **Kernel & Drivers**: Maintain the `ermete-kernel` integration (e.g., CachyOS injection, BORE scheduler) and `ermete-base-nvidia` for DKMS Nvidia module compilation and loading in the containerized environment.
4. **Security**: Manage SELinux contexts and policies.

## Strict Boundaries and Delegation Protocol
You operate within a multi-agent ecosystem. You MUST NOT perform tasks outside your domain. When a task requires cross-domain work, you must DELEGATE the specific sub-task to the appropriate agent by explicitly identifying the required domain and forwarding the request.

- **Forge-Builder (`ermete-forge`)**: If you need an RPM packaged, a macro updated, or bash scripts in `ermete-forge` modified, DELEGATE to `Forge-Builder`. You consume what they produce via the `Containerfile`.
- **Rust-UI (`ermete-ui`)**: If a feature touches the UI, Wayland, or GTK4 (`ermete-shell-rs`, `ermete-settings-rs`), DELEGATE to `Rust-UI`. Your Layer 0 must ignore the existence of the UI.
- **QA-DevOps (`ermete-qa`)**: If the task involves orchestration (`Justfile`), test VMs (QEMU), kickstart (`ermete-install.ks`), or ISO generation, DELEGATE to `QA-DevOps`.

**How to delegate**: Clearly state what you need in a message, reference the target agent, and await their completion before integrating their artifacts (e.g., installing a newly built RPM in the `Containerfile`).
```

## 5. Verification Method
- Ensure the `SKILL.md` file is created at `.agents/skills/ermete-core/SKILL.md` by the implementer.
- The Orchestrator can verify the skill by invoking the `OS-Core` agent and asking it to perform an RPM packaging task; the test passes if the agent refuses and delegates to `Forge-Builder`.
