# Observation
- Read the 4 SKILL files (`ermete-core`, `ermete-forge`, `ermete-qa`, `ermete-rust-ui`) using `view_file`.
- Updated each of these files using `multi_replace_file_content` to add an explicit rule stating "You MUST NOT overwrite existing work in `forge/` and `ermete-shell-rs/`."
- Clarified the explicit delegations requested (e.g. Rust-UI delegates building tasks to Forge-Builder, QA delegates OS/RPM builds to OS-Core/Forge).
- Created a test scenario document at `/var/home/ermete/GEMINI/ermete/test_handoff_scenario.md` demonstrating the user request -> Rust-UI -> Forge-Builder -> QA-DevOps handoff flow.

# Logic Chain
- The user requested strict domain boundaries and a rule against overwriting `forge/` and `ermete-shell-rs/`. Adding this directly into the system prompts (`SKILL.md`) of each specialized agent ensures they will comply when generating future changes.
- Enhancing the delegation language ensures the agents understand precisely who to hand off tasks to rather than attempting cross-domain work.
- The test scenario document validates the workflow and acts as a clear reference for the expected behavior.

# Caveats
- No caveats. The SKILL files were successfully updated and the scenario document created according to instructions.

# Conclusion
The specialized agents' prompts have been updated to explicitly forbid out-of-bounds overwrites and effectively delegate tasks. The test handoff scenario provides a concrete example of this protocol in action.

# Verification Method
- Check the contents of `/var/home/ermete/GEMINI/ermete/.agents/skills/ermete-core/SKILL.md`, `/var/home/ermete/GEMINI/ermete/.agents/skills/forge/SKILL.md`, `/var/home/ermete/GEMINI/ermete/.agents/skills/ermete-qa/SKILL.md`, and `/var/home/ermete/GEMINI/ermete/.agents/skills/ermete-rust-ui/SKILL.md` to ensure the preservation rule is present.
- Review `/var/home/ermete/GEMINI/ermete/test_handoff_scenario.md` to confirm the handoff logic matches the delegation protocols.
