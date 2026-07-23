# Handoff Report: QA-DevOps Skill Strategy

## 1. Observation
- `PROJECT.md` defines QA-DevOps domain as: "Orchestration (`Justfile`), test VMs, ISO generation, kickstart (`ermete-install.ks`)."
- `ORIGINAL_REQUEST.md` (R1) specifies: "QA-DevOps: responsabile di orchestrazione (`Justfile`), test VM (QEMU/systemd-vmspawn), ISO generation e provisioning kickstart (`ermete-install.ks`)."
- `ORIGINAL_REQUEST.md` (R2) mandates that agents must know how to delegate tasks to other specialized agents.
- `.agents/sub_orch_qa/SCOPE.md` confirms the objective is to create `.agents/skills/ermete-qa/SKILL.md` with a system prompt and explicit delegation contracts.

## 2. Logic Chain
1. To fulfill the domain requirements, the `ermete-qa` skill must clearly instruct the agent to act as the pipeline and testing authority.
2. The agent must focus on CI/CD-like workflows locally: parsing/modifying `Justfile`, configuring `QEMU` or `systemd-vmspawn` for isolated testing, building bootable ISOs, and writing kickstart files.
3. Since this agent only builds, tests, and packages the final image, it relies on the outputs of the other three domains. Therefore, if a test fails or a build component is missing, it must delegate:
   - To **Forge-Builder**: For any RPM packaging issues, missing system dependencies, or OCI isolation problems detected during tests.
   - To **Rust-UI**: For any crashes or bugs related to `ermete-shell-rs`, `ermete-settings-rs`, or Niri IPC observed in the test VMs.
   - To **OS-Core**: For boot failures, kernel panics, ostree/bootc build errors, or SELinux denials preventing ISO generation or test VM booting.

## 3. Caveats
- The specific commands for ISO generation and QEMU testing are not yet defined in the codebase; the skill prompt should focus on the agent's capability to search for and use these tools appropriately or construct them in the `Justfile`.
- Implementation of the skill itself is out of scope for this Explorer phase.

## 4. Conclusion
**Proposed Structure for `.agents/skills/ermete-qa/SKILL.md`:**

```markdown
---
name: ermete-qa
description: "QA-DevOps specialist for Ermete OS: manages orchestration, testing, ISO generation, and kickstart."
---
# Ermete QA-DevOps Agent

## Role
You are the QA-DevOps specialist for Ermete OS. Your domain covers build orchestration, automated testing environments, bootable media generation, and provisioning.

## Responsibilities
- **Orchestration**: Maintain and expand the `Justfile` to automate repository tasks (build, test, run).
- **Test VMs**: Configure and execute isolated test environments using QEMU or systemd-vmspawn.
- **ISO Generation**: Manage the process of composing bootable ISOs from the base immutable OS image.
- **Provisioning**: Maintain the kickstart configuration (`ermete-install.ks`) for unattended installations.

## Delegation Protocol
You do not write UI code, package RPMs, or configure the base OS layers. When you encounter issues outside your domain, delegate immediately:
- **To Forge-Builder**: If a test reveals missing RPM dependencies, macro errors, or packaging failures.
- **To Rust-UI**: If UI applications (`ermete-shell-rs`, `ermete-settings-rs`) fail or crash in your test VMs.
- **To OS-Core**: If the base image (ostree/bootc) fails to build, if there are kernel/DKMS issues, or if SELinux denials prevent testing/ISO generation.
```

## 5. Verification Method
- **Inspection**: Verify that `.agents/skills/ermete-qa/SKILL.md` contains the exact domain responsibilities and delegation rules outlined above.
- **Execution**: During the E2E task execution (Milestone 5), trigger a test scenario (e.g., via `just test-vm`) that requires a kernel module or UI change, and verify that the QA agent correctly delegates the failure to OS-Core or Rust-UI instead of attempting to fix the code itself.
