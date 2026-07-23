# Original User Request

## Initial Request — 2026-07-20T13:44:56+02:00

# Teamwork Project Prompt — Draft

> Status: Step 4-6 — Draft Requirements & Verification
> Goal: Craft prompt → get user approval → delegate to teamwork_preview

Continuare e completare l'implementazione dell'ecosistema di agenti specializzati per Ermete OS. L'obiettivo è configurare un team di subagents per l'Antigravity CLI che lavorino in sinergia per coprire i vari domini del sistema operativo.

Working directory: /var/home/ermete/GEMINI/ermete/
Integrity mode: development

## Requirements

### R1. Architettura Multi-Agente
Basandoci sull'architettura reale del progetto, definire e configurare i seguenti 4 agenti specializzati:
- `Forge-Builder`: responsabile della pacchettizzazione RPM, macro, dipendenze, isolamento OCI e script bash (`forge/specs`, `forge/scripts`).
- `Rust-UI`: specialista Wayland/GTK4 dedicato allo stack puramente Rust (`ermete-shell-rs`, `ermete-settings-rs`, Niri IPC).
- `OS-Core`: architetto del Layer 0 immutabile (ostree/bootc), gestione Containerfile, `ermete-kernel`, DKMS Nvidia e SELinux.
- `QA-DevOps`: responsabile di orchestrazione (`Justfile`), test VM (QEMU/systemd-vmspawn), ISO generation e provisioning kickstart (`ermete-install.ks`).

### R2. Sinergia e Comunicazione
Gli agenti devono essere istruiti nei loro System Prompt su come delegare i compiti. Ad esempio, se `ermete-ui` necessita di una dipendenza di sistema, deve sapere come inoltrare la richiesta a `ermete-forge`.

### R3. Preservare il lavoro precedente
Gli agenti non devono sovrascrivere o ignorare le codebase esistenti in `forge/` ed `ermete-shell-rs/`, ma devono integrarsi con il repository attuale.

## Acceptance Criteria

### Verifica dell'Ecosistema
- [ ] Tutti i subagents richiesti sono definiti correttamente e richiamabili dall'interfaccia CLI/teamwork.
- [ ] I System Prompt di ciascun agente descrivono chiaramente i rispettivi confini (nessun overlap di responsabilità).
- [ ] Esecuzione di un task di test end-to-end (es. "Aggiungi una feature alla UI che richiede un modulo kernel") che dimostri il corretto passaggio di consegne tra gli agenti.

---
*Next: when approved → delegate via invoke_subagent (see Delegation Protocol)*

## Follow-up — 2026-07-20T11:58:25Z

# Teamwork Project Prompt — Draft

> Status: Launched
> Goal: Craft prompt → get user approval → delegate to teamwork_preview

Continuare e completare l'implementazione dell'ecosistema di agenti specializzati per Ermete OS. L'obiettivo è configurare un team di subagents per l'Antigravity CLI che lavorino in sinergia per coprire i vari domini del sistema operativo.

Working directory: /var/home/ermete/GEMINI/ermete/
Integrity mode: development

## Requirements

### R1. Architettura Multi-Agente
Basandoci sull'architettura reale del progetto, definire e configurare i seguenti 4 agenti specializzati:
- `Forge-Builder`: responsabile della pacchettizzazione RPM, macro, dipendenze, isolamento OCI e script bash (`forge/specs`, `forge/scripts`).
- `Rust-UI`: specialista Wayland/GTK4 dedicato allo stack puramente Rust (`ermete-shell-rs`, `ermete-settings-rs`, Niri IPC).
- `OS-Core`: architetto del Layer 0 immutabile (ostree/bootc), gestione Containerfile, `ermete-kernel`, DKMS Nvidia e SELinux.
- `QA-DevOps`: responsabile di orchestrazione (`Justfile`), test VM (QEMU/systemd-vmspawn), ISO generation e provisioning kickstart (`ermete-install.ks`).

### R2. Sinergia e Comunicazione
Gli agenti devono essere istruiti nei loro System Prompt su come delegare i compiti. Ad esempio, se `ermete-ui` necessita di una dipendenza di sistema, deve sapere come inoltrare la richiesta a `ermete-forge`.

### R3. Preservare il lavoro precedente
Gli agenti non devono sovrascrivere o ignorare le codebase esistenti in `forge/` ed `ermete-shell-rs/`, ma devono integrarsi con il repository attuale.

## Acceptance Criteria

### Verifica dell'Ecosistema
- [ ] Tutti i subagents richiesti sono definiti correttamente e richiamabili dall'interfaccia CLI/teamwork.
- [ ] I System Prompt di ciascun agente descrivono chiaramente i rispettivi confini (nessun overlap di responsabilità).
- [ ] Esecuzione di un task di test end-to-end (es. "Aggiungi una feature alla UI che richiede un modulo kernel") che dimostri il corretto passaggio di consegne tra gli agenti.

---
*Next: when approved → delegate via invoke_subagent (see Delegation Protocol)*
## 2026-07-20T13:58:57+02:00
Mission: Implement the Ermete OS specialized agent ecosystem.
Working directory: /var/home/ermete/GEMINI/ermete/.agents/orchestrator
Original Request: /var/home/ermete/GEMINI/ermete/.agents/ORIGINAL_REQUEST.md

Please orchestrate the implementation of the 4 specialized agents (Forge-Builder, Rust-UI, OS-Core, QA-DevOps) according to the ORIGINAL_REQUEST.md.
Configure their system prompts to handle delegation (e.g. ermete-ui to forge) and ensure they do not overwrite existing work in `forge/` and `ermete-shell-rs/`.
Create a test scenario to demonstrate handoff between agents.
Write your plans and status to progress.md and plan.md in your working directory. Use send_message to report when all milestones are complete (victory claim).
