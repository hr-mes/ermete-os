## 2026-07-20T11:52:23Z
You are an Explorer for Milestone 2: Define Rust-UI.
Your working directory is /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_rust_ui_gen2_3.
Read /var/home/ermete/GEMINI/ermete/PROJECT.md, /var/home/ermete/GEMINI/ermete/.agents/ORIGINAL_REQUEST.md, and /var/home/ermete/GEMINI/ermete/.agents/sub_orch_rust_ui/SCOPE.md.
Also read the Challenger's failure report from the previous iteration at `/var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_challenger_rust_ui_1/handoff.md`.
Your task is to analyze and propose a revised design for the Rust-UI agent's domain skill (`.agents/skills/ermete-rust-ui/SKILL.md`) that explicitly addresses the loopholes found by the Challenger (forbidding ANY package manager command, forbidding vendoring system headers via curl/build.rs, and forbidding mutating the system environment via arbitrary shell commands).
Recommend the target file path and the exact content structure / rules for the skill.
Do NOT implement. Produce a `handoff.md` in your working directory with your proposed design strategy, and report back to me when done.
