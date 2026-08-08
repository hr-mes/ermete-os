# 🔬 Ermete OS — Report Definitivo dell'Analisi Forense Totale
> Generato dallo Sciame di 4 Agenti Pro in parallelo — 2026-08-08

---

## 🚨 CRITICITÀ BLOCCANTI (Fix Immediato)

### 1. Loop Infinito CI/CD — `ermete-forge-orchestrator.yml` ↔ `kernel-build.yml`
**Gravità: 🔴 CRITICA**
Il Forge Orchestrator si innesca al completamento del Kernel Build (`workflow_run`), e il Kernel Build si innesca al completamento del Forge Orchestrator. Questo genera un **ciclo ping-pong infinito** che consuma runner e risorse GitHub Actions all'infinito.
- **Fix**: Rompere la catena. Il Forge Orchestrator deve triggerare su `push`, non su `workflow_run` del kernel.

### 2. Race Condition — Container Name su Runner Self-Hosted (Kernel Build)
**Gravità: 🔴 CRITICA**
Il job `build-rpm` ha una matrice a 3 architetture (`x86_64`, `aarch64`, `riscv64`) che girano tutte sullo **stesso** runner self-hosted (il tuo PC). Tutte e tre tentano di creare un container `--name chimera-builder`. Se le run si sovrappongono, si scontrano sul nome e vanno in crash.
- **Fix**: `--name chimera-builder-${{ matrix.target }}`

### 3. Race Condition — OCI Image Sovrascritta (Multi-Arch Kernel)
**Gravità: 🔴 CRITICA**
Tutti e 3 gli job della matrice fanno push su `ghcr.io/hr-mes/ermete-os-kernel:latest`. L'ultimo a finire sovrascrive gli altri, **eliminando il supporto multi-arch**.
- **Fix**: Usare `buildah manifest create` per generare un manifest OCI multi-arch unificato.

### 4. eBPF Sub-Workspace Invisibile al Compilatore
**Gravità: 🔴 CRITICA**
`system/ebpf/Cargo.toml` definisce i crate `ebpf-core` e `ebpf-loader` come sub-workspace indipendente. **Non sono inclusi nel workspace root** `Cargo.toml`. Il compilatore principale non li vede, non li compila, non li testa. Regressione silenziosa totale.
- **Fix**: Aggiungere `"system/ebpf/ebpf-core"` e `"system/ebpf/ebpf-loader"` ai `members` del workspace root.

---

## ⚠️ PROBLEMI ARCHITETTURALI GRAVI

### 5. Permessi GHCR Cleanup — Token Errato
**Gravità: 🟠 ALTA**
`forge-ghcr-cleanup.yml` usa il `GITHUB_TOKEN` standard per `gh api -X DELETE`. Questo token ha `packages: write` ma **NON** `delete:packages`. Il job fallisce silenziosamente in produzione da sempre.
- **Fix**: Usare un PAT con scope `delete:packages` e caricarlo come secret `FORGE_PAT`.

### 6. 9 Pacchetti in `packages.json` senza `.spec` Corrispondente
**Gravità: 🟠 ALTA**
Trovati nel manifest ma privi di spec nel filesystem:
`kernel-forge`, `mesh-bus`, `init-oracle`, `ermete-greeter`, `ermete-compositor`, `ermete-audio-bus`, `ermete-telemetry`, `cluster-mesh`, `ermete-semantic-db`
- **Fix**: Creare le spec mancanti o rimuoverle dal manifest.

### 7. Dipendenza Fantasma — `ermete-updater-rs`
**Gravità: 🟠 ALTA**
`ermete-system-config.spec` dichiara `Requires: ermete-updater-rs`, ma questo pacchetto **non esiste** nella Forgia. Il sistema non si installerà mai correttamente.
- **Fix**: Rimuovere il `Requires` o creare il crate.

### 8. Kani Formal Verification — Gap di Automazione CI
**Gravità: 🟠 ALTA**
`kani-verify.yml` esiste ma c'è duplicazione con `rust-security-audit.yml` (entrambi eseguono `cargo kani`). La verifica formale non è un gate obbligatorio pre-merge.
- **Fix**: Unificare in un unico workflow e integrarlo come gate bloccante.

---

## 🧪 SECURITY THEATER (Falsi Positivi da Eliminare)

| File | Fake Component |
|------|---------------|
| `ermete-forge-orchestrator.yml` | `visual-regression-ui-performance` → esegue `sleep 2` e stampa "All visual checks passed" |
| `kernel-build.yml` | Step BOLT con `llvm-bolt` → profili `kernel_bolt.fdata` non generati, fallisce in silenzio con `\|\| true` |
| `live-patching.yml` | `HEALTH_SCORE=98` hardcodato presentato come telemetria eBPF intelligente |
| `fuzzing.yml` | Loop `for` con `set -e` → se il primo target crasha, i restanti non vengono mai eseguiti |

---

## 📊 STATO DELLA CODEBASE RUST

| Metrica | Valore |
|---------|--------|
| **Crate totali nel workspace** | 37 |
| **Crate implementati realmente** | 36 (97%) |
| **Crate fantasma (scaffold)** | 1 (`ermete-semantic-db`) |
| **Blocchi `unsafe`** | ~36 (tutti giustificati: FFI, eBPF, IOCTL hardware) |
| **`unwrap()` non commentati** | 0 ✅ |
| **`todo!()` / `unimplemented!()` abbandonati** | 0 ✅ |
| **TODO nei commenti** | 2 (in `ermete-backup/src/main.rs` L23-24) |
| **Crate con IPC zbus/tokio** | 30+ |

---

## 🔒 CATENA ZERO TRUST — STATO SEMAFORO

| Componente | Stato | Note |
|-----------|-------|------|
| UEFI Secure Boot | 🟢 OK | `ukify` + `sbsigntools`, firma corretta |
| UKI + PCR Binding | 🟢 OK | PCR 0, 2, 7, 11 bindati per LUKS |
| TPM 2.0 Attestation | 🟢 OK | `ermete-keylime` + `cvm_manager.rs` |
| Remote Attestation | 🟢 OK | Fase 3 Keylime configurata |
| SELinux | 🟢 OK | `allow_execmem` rimosso, policy strict |
| eBPF Runtime Security | 🟢 OK | Tetragon con `sys_execve.yaml` |
| Allocatore Scudo | 🟡 Parziale | `LD_PRELOAD` globale disattivato per stabilità |
| Live Patcher IPC | 🟡 Rischio | Validazione argomenti zbus incompleta |
| Kani Formal Verification | 🟡 Gap | Non automatizzato come gate obbligatorio |
| eBPF Boundary Checks | 🟡 Rischio | `unsafe { (*ipv4hdr) }` senza bound check espliciti |

---

## 📦 FORGE — GOD PACKAGES

I seguenti pacchetti sono dipendenze trasversali critiche. Un bug in uno di loro blocca l'intera Forgia:

1. **`ermete-style`** — Tema GTK4 condiviso da ogni componente UI
2. **`ermete-niri-ipc`** — IPC del compositor, usato da ogni componente shell
3. **`niri`** — Il compositor Wayland. God Node UI assoluto
4. **`ermete-daemon-rs`** — Il daemon centrale. Bus condiviso da tutti i servizi

---

## 🎯 Piano di Remediazione Prioritizzata

```
P0 — Fix Immediato (bloccano la CI):
  [ ] Rompere loop infinito: forge-orchestrator trigger su push, non workflow_run del kernel
  [ ] Fix container name race: --name chimera-builder-${{ matrix.target }}
  [ ] Fix OCI multi-arch: buildah manifest invece di push diretto su :latest

P1 — Fix Entro la Settimana (integrità del sistema):
  [ ] Aggiungere system/ebpf/ebpf-core e ebpf-loader al workspace root Cargo.toml
  [ ] Rimuovere ermete-updater-rs da Requires in ermete-system-config
  [ ] Configurare PAT con delete:packages per GHCR cleanup

P2 — Qualità (entro il mese):
  [ ] Implementare ermete-semantic-db (unico crate fantasma rimasto)
  [ ] Unificare rust-security-audit.yml + kani-verify.yml
  [ ] Boundary check in ebpf-core per pacchetti malformati (ipv4hdr)
  [ ] Eliminare Security Theater: visual regression, BOLT fake, HEALTH_SCORE finto
  [ ] Dinamicizzare forge-util-update-specs.yml (lista pacchetti non hardcoded)
  [ ] Fix fuzzing.yml: raccogliere fallimenti senza short-circuit

P3 — Eccellenza (roadmap):
  [ ] Kani come gate obbligatorio pre-merge
  [ ] forge-ghcr-cleanup con token corretto (delete:packages)
  [ ] Spec mancanti per 9 pacchetti in packages.json
```

---
*Report generato con: 4 agenti Pro paralleli, CodeGraph, analisi statica Python, grep semantico su 37 crate Rust, audit YAML CI/CD, audit Zero Trust security*
