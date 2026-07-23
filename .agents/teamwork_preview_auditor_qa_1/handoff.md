## Forensic Audit Report

**Work Product**: `/var/home/ermete/GEMINI/ermete/.agents/skills/ermete-qa/SKILL.md`
**Profile**: General Project
**Verdict**: CLEAN

### Phase Results
- Source Code Analysis (Hardcoded output detection): PASS — The file is a Markdown document defining agent instructions. It contains no hardcoded outputs or tests.
- Source Code Analysis (Facade detection): PASS — The file does not contain any executable code, and therefore no facade implementations.
- Source Code Analysis (Pre-populated artifact detection): PASS — No related pre-populated artifacts or output files were found for this markdown definition.
- Behavioral Verification (Build and run): PASS — Not applicable as the work product is a documentation/skill definition file.
- Behavioral Verification (Output verification): PASS — Not applicable.

### Evidence
```
$ cat /var/home/ermete/GEMINI/ermete/.agents/skills/ermete-qa/SKILL.md
---
name: ermete-qa
description: "QA-DevOps specialist for Ermete OS: manages orchestration, testing, ISO generation, and kickstart."
---

# ermete-qa

## Responsibilities
- **Orchestration**: Maintain and expand the `Justfile` to automate repository tasks (build, test, run).
- **Test VMs**: Configure and execute isolated test environments using QEMU or systemd-vmspawn.
- **ISO Generation**: Manage the process of composing bootable ISOs from the base immutable OS image.
- **Provisioning**: Maintain the kickstart configuration (`ermete-install.ks`) for unattended installations.

## Delegation Protocol
You do not write UI code, package RPMs, or configure the base OS layers. When you encounter issues outside your domain, delegate explicitly to the respective domain agent using your communication protocol:
- **To Forge-Builder**: If a test reveals missing RPM dependencies, macro errors, bash script failures in `forge/scripts`, or OCI isolation problems.
- **To Rust-UI**: If UI applications (`ermete-shell-rs`, `ermete-settings-rs`, Niri IPC) fail or crash in your test VMs.
- **To OS-Core**: If the base image (ostree/bootc) fails to build, if there are kernel/DKMS issues, or if SELinux denials prevent testing/ISO generation.

## Integrity constraints
DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task. A Forensic Auditor will independently verify your work. Integrity violations WILL be detected and your work WILL be rejected.
```

---

## 5-Component Handoff Report

### 1. Observation
I observed the content of `/var/home/ermete/GEMINI/ermete/.agents/skills/ermete-qa/SKILL.md` via `cat`. The file is a Markdown document containing natural language instructions and YAML frontmatter. There is no executable code in the file. The file itself establishes integrity constraints for the `ermete-qa` agent, explicitly stating: "DO NOT CHEAT. All implementations must be genuine. DO NOT hardcode test results, create dummy/facade implementations, or circumvent the intended task." No pre-populated result logs or artifacts were found in the directory (`/var/home/ermete/GEMINI/ermete/.agents/skills/ermete-qa/`).

### 2. Logic Chain
1. The work product specified for auditing is `/var/home/ermete/GEMINI/ermete/.agents/skills/ermete-qa/SKILL.md`.
2. I inspected the file and confirmed it is a text-based definition file (Markdown).
3. Hardcoded test results and facade implementations require executable source code to circumvent execution.
4. Since the file contains no code, it cannot technically contain a dummy facade or hardcoded logic. It only contains instructions for an agent.
5. Therefore, the work product does not contain any integrity violations.

### 3. Caveats
- The audit was explicitly limited to the file `.agents/skills/ermete-qa/SKILL.md`. Any associated artifacts in the broader repository (such as `Justfile`, test VM scripts, ISO scripts) were not requested for auditing and thus were not checked.

### 4. Conclusion
The file `/var/home/ermete/GEMINI/ermete/.agents/skills/ermete-qa/SKILL.md` is CLEAN. It authentically serves its purpose as a skill definition file and contains no integrity violations, hardcoded test results, or dummy facade implementations.

### 5. Verification Method
To independently verify:
Run `cat /var/home/ermete/GEMINI/ermete/.agents/skills/ermete-qa/SKILL.md` to confirm the contents are solely markdown instructions.
