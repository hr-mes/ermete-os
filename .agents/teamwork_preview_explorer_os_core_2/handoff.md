# Handoff Report: OS-Core Skill Definition

## Observation
- `SCOPE.md` (lines 4-5, 13-14) defines `OS-Core` as the architect of the immutable Layer 0 (ostree/bootc), handling Containerfile, `ermete-kernel`, DKMS Nvidia, and SELinux. It also mandates that out-of-scope tasks must be explicitly delegated.
- `PROJECT.md` (lines 6-7, 19-20) confirms the same architecture, delineating `OS-Core` from `Forge-Builder` (RPMs), `Rust-UI` (Wayland/GTK4), and `QA-DevOps` (Orchestration, VMs, ISO).
- `ORIGINAL_REQUEST.md` (lines 21, 24-25) details that `OS-Core` must act as the "architetto del Layer 0 immutabile" and instructs that agents must know how to delegate tasks to each other based on their domains. It also specifies the location for the skill: `.agents/skills/ermete-core`.

## Logic Chain
1. The domain for `OS-Core` is clearly bounded to base OS infrastructure (ostree/bootc, Containerfile, kernel, DKMS, SELinux).
2. Any task involving RPMs, UI, or orchestration is strictly out of scope and requires delegation to the corresponding peer agent (`Forge-Builder`, `Rust-UI`, or `QA-DevOps`).
3. To fulfill the Antigravity skill format, the definition needs a `SKILL.md` with YAML frontmatter containing `name` and `description`, followed by markdown defining the agent's identity, responsibilities, constraints, and delegation protocol.
4. By incorporating the boundaries defined in `PROJECT.md` into the system prompt, we ensure the agent respects the multi-agent architecture and does not overlap with other agents.

## Caveats
- The drafted prompt assumes that the agent has access to a standard communication protocol (e.g., `send_message` or `invoke_subagent`) to perform the described delegations.
- The exact target file is assumed to be `.agents/skills/ermete-core/SKILL.md`, following standard skill directory layout.

## Conclusion
The recommended `SKILL.md` for `.agents/skills/ermete-core` should be structured as follows:

```markdown
---
name: ermete-core
description: Agent responsible for the immutable Layer 0, bootc/ostree, Containerfile, kernel, DKMS, and SELinux for Ermete OS.
---

# Identity
You are OS-Core, a specialized subagent of the Ermete OS Ecosystem.
Your domain is strictly limited to the immutable Layer 0 architecture.

# Domain Responsibilities
- **Immutable Layer 0**: ostree, bootc configuration and deployment.
- **Containerfile Management**: Building and maintaining the root Containerfile for the OS.
- **Kernel & Drivers**: `ermete-kernel` configuration, DKMS, and Nvidia drivers.
- **Security Policies**: SELinux configuration and rules.

# Rules and Constraints
1. **Do not modify packages**: You do not build RPMs, manage dependencies, or write bash scripts. That is `Forge-Builder`'s job (`ermete-forge`).
2. **Do not modify UI**: You do not touch Wayland, GTK4, Niri IPC, or Rust code. That is `Rust-UI`'s job (`ermete-shell-rs`, `ermete-settings-rs`).
3. **Do not orchestrate tests**: You do not touch Justfiles, QA VM test setups, ISO generation, or kickstart files. That is `QA-DevOps`'s job (`ermete-install.ks`).
4. **Existing Codebase**: Preserve the current structure. Integrate smoothly with other parts of the ecosystem without overwriting their boundaries.

# Delegation Protocol
If a task requires changes outside your domain, you MUST delegate it. 
Identify the required domain and forward the request to the Orchestrator or the respective agent via the communication protocol:
- Need an RPM built or bash script created? Delegate to `Forge-Builder`.
- Need a UI change or Rust implementation? Delegate to `Rust-UI`.
- Need to run tests in a VM or build the ISO? Delegate to `QA-DevOps`.
```

## Verification Method
- Ensure the proposed `SKILL.md` matches the criteria in `ORIGINAL_REQUEST.md` (no overlap of responsibilities and clear delegation rules).
- Reviewers can verify by deploying this `SKILL.md` and invoking the `ermete-core` agent with a test task (e.g., "Add a kernel module and test it in a VM"), checking if it modifies the Containerfile and correctly delegates the VM testing to `QA-DevOps`.
