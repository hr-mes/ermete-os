# Handoff: QA-DevOps Skill Strategy

## 1. Observation
- `PROJECT.md` (lines 7, 20): QA-DevOps is responsible for Orchestration (`Justfile`), test VMs (QEMU/systemd-vmspawn), ISO generation, kickstart (`ermete-install.ks`). It explicitly requires that agents delegate out-of-scope tasks.
- `ORIGINAL_REQUEST.md` (lines 22, 25): QA-DevOps role description matches PROJECT.md. Explicit instruction: "Gli agenti devono essere istruiti nei loro System Prompt su come delegare i compiti."
- `SCOPE.md` (lines 11-12): Mandates delegation of out-of-scope tasks (Forge-Builder, Rust-UI, OS-Core) to other agents or Orchestrator.

## 2. Logic Chain
- To fulfill the requirements, `.agents/skills/ermete-qa/SKILL.md` must be constructed with a clear definition of the agent's domain and strict boundaries.
- **Domain Definition**: The prompt must instruct the agent to act as the QA/DevOps expert, providing concrete guidelines on how to manage `Justfile` targets, configure QEMU/systemd-vmspawn for integration testing, handle ISO build toolchains, and manage Kickstart provisioning configurations.
- **Delegation Definition**: Because the agent is part of an ecosystem, the prompt must explicitly map out-of-scope domains to specific agents to ensure a clean handoff:
  - Any request modifying package macros or RPM specs must trigger a delegation to **Forge-Builder**.
  - Any test failure related to GTK4, Wayland, or Rust IPC must trigger a delegation to **Rust-UI**.
  - Any issue regarding the base bootc image, kernel panics, or SELinux denials must trigger a delegation to **OS-Core**.

## 3. Caveats
- The exact file paths for ISO generation scripts or VM test wrappers are not fully detailed in the current documentation (besides `Justfile` and `ermete-install.ks`), so the agent prompt should instruct the agent to discover these dynamically or manage them at the root/standard locations.
- The precise messaging syntax for delegation depends on the communication protocol in use, which the skill should refer to generically (e.g., "Use `send_message` to the relevant agent").

## 4. Conclusion
**Strategy and Plan for `.agents/skills/ermete-qa/SKILL.md`**:
The file should be implemented using the following structure:
1. **Frontmatter**: `name: ermete-qa`, `description: QA-DevOps specialist for Ermete OS`.
2. **Identity**: Define the agent as the master of CI/CD orchestration and testing infrastructure.
3. **Core Responsibilities**:
   - **Orchestration**: Execute and maintain the `Justfile`.
   - **Testing**: Spin up and configure test environments using QEMU and `systemd-vmspawn`.
   - **Provisioning**: Maintain the `ermete-install.ks` Kickstart file and ISO generation processes.
4. **Delegation Protocol**:
   - Delegate to **Forge-Builder** for RPM, OCI isolation, and build macros.
   - Delegate to **Rust-UI** for GTK4, Wayland, and Niri IPC development.
   - Delegate to **OS-Core** for ostree/bootc layered image changes, kernel configuration, and SELinux.
5. **Execution Rules**: Stay within domain boundaries. Never edit `forge/` or `ermete-shell-rs/` directly; always delegate.

## 5. Verification Method
- **Static Inspection**: Read the generated `.agents/skills/ermete-qa/SKILL.md` to ensure all 4 core responsibilities and 3 delegation rules are explicitly written out.
- **Behavioral Test**: Assign an E2E test task via `antigravity-cli` asking the agent to "Test the ISO and fix a missing GTK package". The agent should spin up the VM (its own domain) but must generate a delegation message to **Forge-Builder** for the package addition.
