# 🏗️ Ingegneria Architetturale: Ermete Base NVIDIA (`ermete-base-nvidia`)
**Status Documento**: IMPLEMENTATO ED OPERATIVO (RFC-01)
**Dominio**: Layer 0 (Base OCI Bootable)

Questo documento definisce l'architettura formale e la filosofia di design utilizzate per svincolare **Ermete OS** da dipendenze upstream non verificate, ottenendo la piena sovranità sulla catena di approvvigionamento (Supply Chain).

---

## 🎯 Paradigma Operativo
1. **Sovranità Zero-Trust**: Nessun binario opaco o rootfs pre-compilato da terze parti (es. RakuOS). L'infrastruttura di base è generata in un ambiente pulito (clean-room) partendo direttamente dai sorgenti Fedora Atomic e dai repository COPR ufficiali tramite GitHub Actions auditable.
2. **Determinismo Hardware**: Iniezione a basso livello dello scheduler `BORE` e delle istruzioni `x86-64-v3` (Kernel CachyOS), unita alla compilazione *in-situ* dei moduli DKMS NVIDIA.
3. **Isolamento della Complessità**: Mantenimento di un confine invalicabile tra il Layer 0 (Kernel/Driver) e il Layer 1 (UI/UX). Il Layer 0 deve ignorare completamente l'esistenza di Wayland, Flatpak o configurazioni utente.

---

## 🗺️ Topologia dell'Implementazione

### Fase 1: Scaffold dell'Infrastruttura (Completata)
Il repository parallelo `ermete-base-nvidia` fornisce l'ambiente isolato. Utilizza un pattern `scratch AS ctx` nel Containerfile per importare gli script di build senza contaminare i layer del filesystem dell'immagine finale.

### Fase 2: Iniezione del Kernel e Risoluzione DKMS (Completata)
Il processo di build risolve asimmetrie critiche dell'ambiente OCI:
1. **Drop & Replace**: Sradicamento di `kernel`, `kernel-core` e `zram-generator-defaults` originali, sostituiti atomicamente dal ramo `kernel-cachyos`.
2. **Workaround Systemd/OCI**: I moduli `dkms-nvidia` sono installati con `--setopt=tsflags=noscripts` per prevenire errori letali di systemd all'interno del container engine non privilegiato.
3. **Compensazione Utenze Fantasma**: A causa dell'inattivazione degli scriptlet RPM (Punto 2), l'utente vitale `nvidia-persistenced` viene generato ed iniettato a mano prima del lockdown dell'immagine, impedendo al demone di crasciare in fase di power management della GPU.
4. **Compilazione Linker**: La compilazione DKMS bypassa i bug del linker `gold` forzando l'uso di `ld.bfd` (`LD=ld.bfd dkms install ...`).
5. **Initramfs Deterministico**: Invocazione manuale di `dracut` con compressione `zstd`. Al fine di garantire la funzionalità *Early KMS* essenziale per Wayland (in assenza degli scriptlet DNF), l'immagine inietta a forza i moduli video (`--force-add "nvidia nvidia_modeset nvidia_uvm nvidia_drm"`).

### Fase 3: CI/CD e Sicurezza Crittografica (Completata)
L'automazione GitHub Actions garantisce l'immutabilità della release:
- Costruzione su base schedulata e su commit.
- Chiusura dell'immagine tramite firma crittografica **Sigstore (Cosign) Keyless OIDC**. Il certificato crittografico garantisce che l'immagine derivi esclusivamente dai workflow di GitHub, senza esporre mai chiavi private statiche.
- Protezione da derive inter-layer (State Drift) blindando i file in `/etc/dnf/protected.d/`.
- Linting formale dei layer finali tramite `bootc container lint`.

### Fase 4: Integrazione "Layer 1" (Completata)
L'immagine risultante (`ghcr.io/patapem/ermete-base-nvidia:latest`) è stata configurata come `FROM` nel Containerfile di Ermete OS.
L'architettura ha raggiunto lo stato **Enterprise**: ogni singolo byte in esecuzione sul sistema host (dal ring 0 al ring 3) è tracciato, compilato e firmato da Ermete OS.
