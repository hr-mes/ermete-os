# Ermete OS: Autonomous Kernel & Spec Forge

Questo documento descrive l'architettura all'avanguardia progettata per l'auto-mantenimento del Kernel ibrido e dei pacchetti RPM di Ermete OS. L'obiettivo primario di questa infrastruttura è garantire che il sistema operativo rimanga perennemente aggiornato con gli ultimi upstream (Fedora ARK) e le massime ottimizzazioni (CachyOS/Clear Linux), in un ambiente zero-trust e altamente automatizzato.

## 1. La Visione Architetturale

Mantenere un OS immutabile altamente competitivo (che rivaleggi nelle prestazioni con kernel custom come CachyOS o Clear Linux) richiede la fusione costante di patch esterne e pacchetti ottimizzati per `x86-64-v3`.

L'**Autonomous Forge** automatizza la risoluzione delle dipendenze, il patching dei file `.spec` e l'estrazione deterministica dei sorgenti RPM tramite il sistema **Chimera Bedrock**.

## 2. Componenti del Sistema

L'infrastruttura si compone di pilastri modulari e isolati:

### A. Chimera Bedrock Builder (`prepare-chimera.sh`)
Lo script principale responsabile della preparazione del Kernel Chimera:
1. **Dynamic Ceiling (NVIDIA Shield)**: Rileva la versione dei driver NVIDIA proprietari e calcola la massima versione kernel consentita per prevenire schermate nere.
2. **Matrice Dominante (CachyOS + Clear Linux)**: Scarica e organizza in cartelle prioritari (`SOURCES/bedrock-*`) le patch dello scheduler BORE e le ottimizzazioni memory/CPU di Clear Linux.
3. **AST & Kconfig Tuning**: Inietta frammenti di configurazione kernel (`ermete-bedrock.cfg`) abilitando `CONFIG_SCHED_BORE=y`, `CONFIG_HZ_1000=y`, `CONFIG_PREEMPT=y`, `CONFIG_LTO_CLANG_THIN=y` e `-O3 -march=x86-64-v3`.

### B. Micro-Container OCI Packaging (`build_rolling_local.sh`)
- Esegue la compilazione isolata di ciascun pacchetto RPM all'interno di micro-container OCI temporanei (`scratch` or `fedora:43`).
- Isola l'ambiente di build prevenendo la contaminazione della macchina host.
- Salva deterministicamente gli RPM prodotti in `~/.rpmbuild/RPMS/`.

## 3. Sicurezza ed Efficienza

- **Isolamento OCI**: Ogni tentativo di compilazione avviene in container effimeri privi di privilegi elevati non necessari.
- **Cache Deterministica**: Gli hash dei file `.spec` e dei sorgenti prevengono la ricompilazione superfluo dei pacchetti invariati nei workflow di CI/CD.
- **Fall-back Autonomo**: Se una patch o una build fallisce, il sistema interrompe l'esecuzione e notifica l'errore nei log di compilazione per l'intervento dell'Architect.

---
*Progettato e implementato per massimizzare le prestazioni del Kernel senza scendere a compromessi.*
