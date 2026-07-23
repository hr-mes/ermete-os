## 2026-07-20T11:55:49Z
You are an Explorer for Milestone 2: Define Rust-UI.
Your working directory is /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_rust_ui_gen3_1.
Read /var/home/ermete/GEMINI/ermete/PROJECT.md, /var/home/ermete/GEMINI/ermete/.agents/ORIGINAL_REQUEST.md, and /var/home/ermete/GEMINI/ermete/.agents/sub_orch_rust_ui/SCOPE.md.
Also read the Gen2 Challenger's failure report at `/var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_challenger_rust_ui_gen2_1/handoff.md`.
Your task is to analyze and propose a revised design for the Rust-UI agent's domain skill (`.agents/skills/ermete-rust-ui/SKILL.md`) that explicitly addresses ALL loopholes found by the Challenger, including the geographic loophole (writing out of tree like `/tmp`), mechanical loopholes (using `write_to_file` to write headers directly, using custom Rust scripts or any other mechanism to download/bypass).
Recommend the target file path and the exact content structure / rules for the skill.
Do NOT implement. Produce a `handoff.md` in your working directory with your proposed design strategy, and report back to me when done.
