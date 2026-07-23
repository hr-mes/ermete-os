# Review Handoff Report: Rust-UI Skill Definition

## 1. Observation
- Inspected `/var/home/ermete/GEMINI/ermete/PROJECT.md` lines 3-8, establishing the architectural domains, and line 20 detailing the Interface Contracts (explicit delegation requirement).
- Inspected the original constraints in `/var/home/ermete/GEMINI/ermete/.agents/ORIGINAL_REQUEST.md`.
- Inspected the proposed `/var/home/ermete/GEMINI/ermete/.agents/skills/ermete-rust-ui/SKILL.md`.
- Observed the identity and scope in the skill correctly targets "Wayland/GTK4 Rust stack (ermete-shell-rs, ermete-settings-rs, Niri IPC)".
- Observed Rule 4: "No System Mutation" explicitly forbids `curl`, `wget`, and bypassing dependencies via shell commands.
- Observed Rule 5: "No Package Managers" explicitly forbids `dnf`, `rpm-ostree`, `microdnf`, `apt`, `pacman` and others.
- Observed Rule 6: "No Vendoring Workarounds" explicitly forbids vendoring system headers, C libraries, and Wayland XML protocols via `build.rs` or direct downloads.
- Observed Delegation Protocol explicitly requires delegating system dependencies (`libadwaita-devel`, `wayland-protocols-devel`, etc.) to `Forge-Builder`, OS/Kernel features to `OS-Core`, and testing to `QA-DevOps` via the `send_message` tool.

## 2. Logic Chain
1. The project boundaries define the Rust-UI agent as strictly focusing on Rust-based Wayland/GTK4 UI components (`ermete-shell-rs`, `ermete-settings-rs`). The skill correctly internalizes these components into the IN SCOPE boundaries.
2. The `PROJECT.md` Interface Contract mandates explicit delegation for out-of-scope tasks. The skill's Delegation Protocol section enforces this perfectly, even mapping specific domain tasks (e.g. system dependencies) to the correct agent (Forge-Builder).
3. The previous loopholes regarding environment pollution and scope escape have been targeted: 
   - Rule 4 prevents downloading binaries directly or altering the shell environment.
   - Rule 5 prevents the agent from escalating to system package managers to resolve its own missing C/system dependencies.
   - Rule 6 closes the `build.rs` vendoring vector, a common loophole for Rust agents attempting to bundle C libraries.
4. With these strict directives, if the Rust-UI agent encounters a missing Wayland XML protocol or C-library header, it must halt and issue a `send_message` to Forge-Builder, adhering strictly to the ecosystem design.

## 3. Caveats
- While `build.rs` fetch scripts are explicitly forbidden, an overly ambitious Rust-UI agent might still attempt to use raw `std::process::Command` in a `build.rs` file to invoke local compilers for inline C code if not closely monitored. However, Rule 4's phrasing "forbidding mutating the system environment via arbitrary shell commands" provides strong systemic coverage against this.
- No other structural loopholes were identified.

## 4. Conclusion
**Verdict: PASS** (APPROVE)
The `ermete-rust-ui` skill correctly defines the scope of the agent, perfectly aligns with the `PROJECT.md` interface contract, and thoroughly patches the identified loopholes related to package managers, system mutation, and vendoring. The work is complete and ready for use.

## 5. Verification Method
- File inspection: Read `/var/home/ermete/GEMINI/ermete/.agents/skills/ermete-rust-ui/SKILL.md` and verify rules 4, 5, 6 and the Delegation Protocol exist and cover the requested constraints.
- Future End-to-End Test: When the Rust-UI agent is invoked to add a feature requiring a new `wayland-protocol`, it should be observed emitting a `send_message` to Forge-Builder instead of running `dnf install` or downloading the XML directly.
