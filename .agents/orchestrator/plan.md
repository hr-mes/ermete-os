# Orchestration Plan: Ermete OS Agent Ecosystem

## Milestones
1. **Analyze and Refine System Prompts**: Review the 4 existing skills (`ermete-forge`, `ermete-rust-ui`, `ermete-core`, `ermete-qa`) and update their system prompts (in `SKILL.md`) to explicitly handle delegation and enforce the rule of NOT overwriting existing work in `ermete-forge/` and `ermete-shell-rs/`.
2. **Create Test Scenario**: Design and implement a test scenario (e.g., a markdown script or automated test) that demonstrates a handoff between the agents (e.g., UI to Forge to QA).
3. **Verification**: Verify that the system prompts are correctly configured and the test scenario exists and properly illustrates the ecosystem handoff.
4. **Final Reporting**: Report victory to the parent agent.
