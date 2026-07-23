## 2026-07-20T11:48:41Z
Read /var/home/ermete/GEMINI/ermete/.agents/sub_orch_m3/SCOPE.md, /var/home/ermete/GEMINI/ermete/PROJECT.md, and /var/home/ermete/GEMINI/ermete/.agents/ORIGINAL_REQUEST.md. 
We need to design the system prompt and skill definition for the OS-Core agent (`.agents/skills/ermete-core`). 
The OS-Core agent is responsible for the immutable Layer 0 (ostree/bootc), Containerfile management, `ermete-kernel`, DKMS Nvidia, and SELinux.
It must also delegate tasks out of its scope properly.
Investigate the project structure and create a draft of the OS-Core skill definition and system prompt.
Do NOT write the skill files yourself, provide a report with the recommended contents.
Your working directory is /var/home/ermete/GEMINI/ermete/.agents/teamwork_preview_explorer_os_core_3.
Write your report to handoff.md in your working directory and notify me when done.
