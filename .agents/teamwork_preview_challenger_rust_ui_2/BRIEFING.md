# BRIEFING — 2026-07-20T13:50:57+02:00

## Mission
Stress-test ermete-rust-ui SKILL.md to see if rules strictly prevent the agent from using tools like dnf, bash, or direct commands to bypass delegation for missing dependencies like Wayland protocol headers.

## 🔒 My Identity
- Archetype: Empirical Challenger
- Roles: critic, specialist
- Working directory: /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_challenger_rust_ui_2
- Original parent: e57f7225-89e1-4eb0-b701-23f51a1871cb
- Milestone: Milestone 2: Define Rust-UI
- Instance: 1 of 1

## 🔒 Key Constraints
- Review-only — do NOT modify implementation code.
- Find empirical evidence of loopholes in the skill's wording.
- Actively look for failure modes, edge cases, and incorrect assumptions.

## Attack Surface
- **Hypotheses tested**: 
  - Hypothesis 1: The prohibition "DO NOT try to install them via dnf" leaves loopholes for `rpm-ostree`, `curl`/source builds, or `cargo build.rs` downloads.
  - Hypothesis 2: "bash scripts (forge/)" implies bash scripts outside that folder (or using `sh`) are allowed.
- **Vulnerabilities found**: [TBD]
- **Untested angles**: [TBD]

## Loaded Skills
- ermete-rust-ui (Local copy: none, read directly)
- Core methodology: Wayland/GTK4 Rust stack specialist, strict delegation for system dependencies.
