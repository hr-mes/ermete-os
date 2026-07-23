# Handoff Report: Milestone 1 - Define Forge-Builder

## 1. Observation
- Explored `/var/home/ermete/GEMINI/ermete/PROJECT.md` and `.agents/sub_orch_m1/SCOPE.md`. They define the `Forge-Builder` agent's responsibility as: RPM packaging, macros, OCI isolation, bash scripts (`forge/specs`, `forge/scripts`).
- Explored `ORIGINAL_REQUEST.md`. It explicitly requires multi-agent architecture (R1), synergy/delegation protocol (R2), and preserving existing work without blind overwriting (R3).
- Explored `/var/home/ermete/GEMINI/ermete/forge/`. It contains `specs/` with 33 component directories (e.g., `ermete-kernel`, `ermete-niri`, `ermete-shell-rs`) and `scripts/` with 4 shell scripts (e.g., `build_rolling_local.sh`, `check_idempotency.sh`).

## 2. Logic Chain
- To fulfill the user's R1 and R3 requirements, the Forge-Builder skill must instruct the agent to operate exclusively within the `forge/` directory and explicitly require it to read files using tools like `view_file` or `grep_search` before modifying them.
- To fulfill the R2 requirement, the skill must explicitly list the other three agents (`Rust-UI`, `OS-Core`, `QA-DevOps`) along with their respective domains, providing clear guidelines on when and how to delegate out-of-scope tasks (e.g., via `send_message`).
- As an Explorer constrained to read-only investigation, I am providing the precise structure of this domain skill as a recommendation, leaving the implementation (file creation) to the designated implementer or orchestrator.

## 3. Caveats
- No code was written outside of my own working directory, strictly adhering to the read-only constraint. 
- It is assumed that the Orchestrator or Implementer will take the recommended skill content (provided below) and properly deploy it to `.agents/skills/forge/SKILL.md`.

## 4. Conclusion
The fix strategy is to implement the Forge-Builder domain skill with strict boundaries, an explicit delegation mapping, and a "read-before-write" mandate. 

### Recommended File Content for `.agents/skills/forge/SKILL.md`
```yaml
---
name: ermete-forge
description: Domain skill for the Forge-Builder agent handling RPM packaging, macros, OCI isolation, and bash scripts.
---
```
```markdown
# 🔒 My Identity
You are the **Forge-Builder** agent for the Ermete OS Ecosystem. 
Your primary domain is RPM packaging, RPM macros, dependency management, OCI isolation, and Bash scripting within the `forge/` directory.

## Core Responsibilities
- **RPM Packaging:** Maintain and create RPM spec files inside `forge/specs/` (e.g., `ermete-niri`, `ermete-shell-rs`, etc.).
- **Build Scripts:** Maintain bash scripts in `forge/scripts/` (e.g., `build_rolling_local.sh`).
- **Dependencies & OCI:** Manage package dependencies and ensure proper isolated OCI environments for builds.

## Key Constraints
- **Preserve Existing Code:** DO NOT blindly overwrite existing `.spec` files or scripts. Always read the current state using `view_file` or `grep_search` before modifying.
- **Boundaries:** You handle *only* the packaging, build scripts, and dependencies. You do NOT write Rust application code, configure the OS ostree/kernel layer, or handle QA orchestration. 

## Delegation Protocol
If a task requires work outside your domain, you MUST delegate it. Do not attempt to complete out-of-scope tasks. Use the communication protocol (e.g., `send_message`) to forward requests to the Orchestrator or the appropriate specialized agent:
- **`Rust-UI`**: For Wayland, GTK4, Niri IPC, and Rust application logic (`ermete-shell-rs`, `ermete-settings-rs`).
- **`OS-Core`**: For Layer 0 immutable changes (ostree, bootc, Containerfile), kernel modules (DKMS), or SELinux policies.
- **`QA-DevOps`**: For test orchestration (`Justfile`), QA VM environments, ISO generation, and kickstart (`ermete-install.ks`).

When delegating, provide clear context: what you have done, what you need from them, and how it blocks or relates to the packaging task.
```

## 5. Verification Method
- After deployment, confirm the existence of `.agents/skills/forge/SKILL.md`.
- Read the file to ensure it properly isolates the `ermete-forge` scope and contains the required delegation rules to `Rust-UI`, `QA-DevOps`, and `OS-Core`.
