# Handoff Report: QA-DevOps Skill Definition Strategy

## 1. Observation
- `PROJECT.md` defines QA-DevOps responsibilities as: "Orchestration (`Justfile`), test VMs, ISO generation, kickstart (`ermete-install.ks`)".
- `PROJECT.md` explicitly lists other components handled by distinct agents: `Forge-Builder` (RPM, macros, OCI, bash scripts), `Rust-UI` (Wayland/GTK4), and `OS-Core` (ostree/bootc, Containerfile, kernel, DKMS, SELinux).
- `.agents/ORIGINAL_REQUEST.md` (R1, R2, R3) requires creating multi-agent configurations where agents have strictly defined boundaries and are instructed in their system prompts on how to delegate tasks to other agents. They must integrate with the existing codebase (like `Justfile`, `ermete-install.ks`).
- `.agents/sub_orch_qa/SCOPE.md` dictates the creation of a skill definition `.agents/skills/ermete-qa/SKILL.md` (which is out of scope for this read-only Explorer to implement, but in scope to plan).
- A root listing shows the existence of `Justfile` and `ermete-install.ks` in the project root.

## 2. Logic Chain
1.  **Identity and Archetype**: The `ermete-qa` skill must configure the agent as the CI/CD and Testing Master for Ermete OS.
2.  **Core Responsibilities**: Based on the project specifications, the system prompt must explicitly instruct the agent to manage the following local targets:
    *   **Orchestration**: Manage targets within the `Justfile`.
    *   **Test VMs**: Use `QEMU` and `systemd-vmspawn` to configure and launch test instances.
    *   **ISO Generation**: Manage the build pipeline for converting OS images into bootable ISOs.
    *   **Provisioning**: Handle changes to `ermete-install.ks` for automated installations.
3.  **Strict Boundaries (Negative Constraints)**: To prevent domain overlap (as required by R1 and R3):
    *   The agent must NOT write or modify RPM specs or scripts in `forge/` (Forge-Builder's domain).
    *   The agent must NOT write or modify Rust code for UI components (`ermete-shell-rs/`, etc.) (Rust-UI's domain).
    *   The agent must NOT modify the base `Containerfile` or Layer 0 ostree configs (OS-Core's domain).
4.  **Interface Contracts (Delegation)**: Following R2, the system prompt must define exactly how `ermete-qa` handles requests outside its scope:
    *   If a test VM fails to boot due to a missing kernel module or SELinux issue, delegate to **OS-Core**.
    *   If a test VM fails because a package cannot be downloaded or installed, delegate to **Forge-Builder**.
    *   If an automated UI test fails, delegate to **Rust-UI**.
    *   All delegations must use the explicit communication protocol (e.g., `send_message` or `invoke_subagent` if applicable) to pass context and exact failure logs to the responsible agent.

## 3. Caveats
- I did not review the specific contents of `Justfile` or `ermete-install.ks` to map out exact command targets since the requirement is to outline the structural plan and domain rules for the SKILL.md system prompt, not to debug the current pipeline.
- The precise syntax for the teamwork framework's skill definition (e.g., YAML frontmatter) is assumed to follow standard Antigravity `.agents/skills/<name>/SKILL.md` conventions.

## 4. Conclusion
The strategy for defining `.agents/skills/ermete-qa/SKILL.md` is complete. The system prompt must establish the agent as the QA and Pipeline maintainer. It should constrain its write-access to orchestration files (`Justfile`), virtualization scripts, ISO builders, and the kickstart file (`ermete-install.ks`). It must explicitly instruct the agent to halt and delegate to `OS-Core` (for base OS/kernel issues), `Forge-Builder` (for packaging issues), or `Rust-UI` (for GUI issues) whenever a test or build fails outside of pipeline misconfiguration. 

## 5. Verification Method
- **Method**: The implementer will write `.agents/skills/ermete-qa/SKILL.md` based on this strategy.
- **Validation**:
  1. Review `SKILL.md` to ensure `Justfile`, `QEMU`/`systemd-vmspawn`, ISO generation, and `ermete-install.ks` are explicitly listed in the responsibilities.
  2. Review `SKILL.md` to ensure delegation rules to `Forge-Builder`, `Rust-UI`, and `OS-Core` are clearly defined.
  3. Run an E2E pipeline test via the Orchestrator to confirm `ermete-qa` correctly defers domain-specific fixes to peer agents.
