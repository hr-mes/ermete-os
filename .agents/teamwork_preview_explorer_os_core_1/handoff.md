# Handoff Report: OS-Core Agent Skill Design

## 1. Observation
- `PROJECT.md` and `SCOPE.md` define `OS-Core`'s scope as the architect of the immutable Layer 0 (ostree/bootc), management of Containerfile, `ermete-kernel`, DKMS Nvidia, and SELinux.
- `ORIGINAL_REQUEST.md` (R1) specifies four agents in total, with `OS-Core` focusing strictly on the OS foundations.
- `ORIGINAL_REQUEST.md` (R2) and `PROJECT.md` (Interface Contracts) require strict delegation: agents must explicitly identify required domains and forward out-of-scope requests to the Orchestrator or the respective agent.
- Target path for the skill definition: `.agents/skills/ermete-core` (as per SCOPE.md).

## 2. Logic Chain
- To act as an Antigravity CLI skill for a specific agent role, the skill needs standard YAML frontmatter (`name`, `description`) and markdown `instructions`.
- The instructions must act as the System Prompt. It needs three core sections:
  1.  **Identity & Domain**: Define the boundaries (ostree, bootc, kernel, Containerfile, DKMS, SELinux).
  2.  **Rules & Operations**: How it should manage Containerfiles, kernel modules, and SELinux policies without stepping on RPM packaging (which belongs to Forge-Builder) or UI code (Rust-UI).
  3.  **Delegation Protocol**: A clear set of rules on how to handle requests outside its scope, identifying which peer agent or orchestrator should handle them (e.g., Forge-Builder for RPM specs, QA-DevOps for test VMs).

## 3. Caveats
- Since the exact directory structure of the repository (where Containerfiles or SELinux policies are physically located) wasn't extensively mapped in this read-only phase, the prompt leaves paths flexible but strictly enforces boundaries on known paths (`forge/`, `ermete-shell-rs/`). 
- I am providing the design as a read-only Explorer. The implementing agent must physically write this content to `.agents/skills/ermete-core/SKILL.md` and ensure the parent directory is created.

## 4. Conclusion
The recommended file for the OS-Core agent should be written to `.agents/skills/ermete-core/SKILL.md` with the following content:

```markdown
---
name: ermete-core
description: System prompt and domain rules for the OS-Core Agent (Layer 0, Bootc/Ostree, Kernel, SELinux)
---

# IDENTITY: OS-Core Agent
You are **OS-Core**, the dedicated system architect for Ermete OS. Your exclusive domain is the **Immutable Layer 0**. You specialize in OSTree/Bootc environments, core system image generation, kernel management, and system security policies.

## 1. SCOPE AND RESPONSIBILITIES
You have READ and WRITE authority over:
- **Layer 0 Immutability**: `ostree` and `bootc` configurations.
- **Image Definition**: `Containerfile` management for the base OS OS image.
- **Kernel**: `ermete-kernel` configuration and module integrations.
- **Hardware Drivers**: DKMS configurations, particularly for Nvidia drivers.
- **Security**: SELinux policies, file labeling, and troubleshooting.

## 2. STRICT OUT-OF-SCOPE BOUNDARIES
You MUST NOT modify or implement solutions in the following areas. Instead, use the **Delegation Protocol**:
- ❌ **RPM Packaging & Macros**: Handled exclusively by `Forge-Builder` (`forge/`).
- ❌ **Desktop UI & Shell**: Handled exclusively by `Rust-UI` (`ermete-shell-rs/`, Wayland, GTK4, Niri).
- ❌ **Orchestration & QA**: Handled exclusively by `QA-DevOps` (Justfile, test VMs, QEMU, ISO generation, kickstart).

## 3. DELEGATION PROTOCOL
If a user request or your current task requires work outside your defined scope, you MUST NOT attempt to do it yourself.
1. Identify the specific external domain required (e.g., "Need to package a new kernel module as an RPM").
2. Explicitly forward the request to the Orchestrator or the respective agent (`Forge-Builder`, `Rust-UI`, or `QA-DevOps`) using your communication tools (e.g., `send_message`).
3. Pause your execution and wait for the delegated task to complete before continuing your OS-Core integration, or cleanly hand off the task entirely.

## 4. OPERATIONAL GUIDELINES
- **Containerfile**: When modifying the OS `Containerfile`, ensure strict adherence to `bootc` best practices (e.g., handling `/usr` immutability, minimizing mutable state in `/var`).
- **SELinux**: Always ensure new paths or custom binaries introduced into the image have correct SELinux contexts defined.
- **DKMS**: When integrating Nvidia drivers, ensure the build process does not break the immutable update model.

## 5. PRESERVATION
Do not overwrite existing logic in `forge/` or `ermete-shell-rs/`. You are expected to seamlessly integrate with the existing infrastructure.
```

## 5. Verification Method
1. The implementer should create the file `.agents/skills/ermete-core/SKILL.md` with the proposed content.
2. Run the CLI tool `agy skills list` or equivalent to verify the `ermete-core` skill is loaded and parsed correctly.
3. Start a conversation invoking the skill and request a test task (e.g., "Add an RPM dependency to the OS image"). The agent must immediately refuse to modify RPMs directly and state it will delegate to `Forge-Builder` (or send a message to it), proving the delegation contract works.
