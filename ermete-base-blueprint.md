# 🏗️ Progetto: Ermete Base NVIDIA (`ermete-base-nvidia`)

Questo documento delinea il piano ingegneristico definitivo per svincolare **Ermete OS** da maintainer di terze parti (come RakuOS), ottenendo il controllo del 100% della catena di approvvigionamento (Supply Chain). 

L'obiettivo è creare una repository parallela che funga da fondamenta per l'OS, compilando autonomamente il **Kernel CachyOS** e i **Driver NVIDIA proprietari**.

---

## 🎯 Obiettivi Architetturali
1. **Zero-Trust Supply Chain:** Nessun codice esterno pre-compilato. Il sistema di base sarà generato direttamente dai sorgenti Fedora e dai COPR ufficiali tramite le nostre GitHub Actions.
2. **Prestazioni Intatte:** Mantenimento dello scheduler `BORE` e delle ottimizzazioni `x86-64-v3` tipiche del Kernel CachyOS.
3. **Estrema Leggerezza:** Rimozione di qualsiasi pacchetto o libreria "bloat" (es. dipendenze GNOME/KDE) che RakuOS potrebbe aver incluso per la sua utenza, mantenendo solo il layer Wayland/EGL necessario per Niri.

---

## 🗺️ Implementazione Completata (Status: OPERATIVO)

### Fase 1: Creazione dell'Infrastruttura
Il repository `ermete-base-nvidia` è stato creato e configurato correttamente. Fornisce un'infrastruttura minimalista e dedicata esclusivamente alla compilazione del kernel e dei driver.

### Fase 2: Il `Containerfile` dello Stack
Il file di build è stato riscritto per operare in autonomia:
1. **Punto di Partenza:** `FROM quay.io/fedora-ostree-desktops/base-atomic:44`
2. **Iniezione CachyOS:** Aggiunta dei repository COPR `bieszczaders` e sostituzione del kernel (`kernel-cachyos`).
3. **Compilazione NVIDIA (Akmods):** Compilazione forzata dei moduli `kmod` contro gli header del Kernel CachyOS tramite `dkms`.
4. **Ottimizzazione:** Rigorosa adozione di `dnf install --setopt=install_weak_deps=False` per prevenire il bloat e rispetto della regola Zero-Persistenza per il builder.

### Fase 3: La CI/CD Pipeline (GitHub Actions)
La pipeline di compilazione automatica è **attiva e funzionante**. 
- Configurazione di trigger sia temporizzati (`cron`) che reattivi (`push`/`pull_request`).
- Generazione puntuale dei tag OCI (`latest`, `YYYYMMDD`).
- **Sicurezza:** Firma dell'immagine tramite **Keyless OIDC** (Sigstore/Cosign), abolendo la necessità di chiavi private vulnerabili nei secret.

### Fase 4: Integrazione in Ermete OS (Completata)
L'immagine `ghcr.io/patapem/ermete-base-nvidia:latest` è ora generata con successo dal repository dedicato.
Ermete OS è stato agganciato a questa nuova immagine tramite il proprio `Containerfile`. 
Ermete OS è ora un prodotto Enterprise: l'intera Supply Chain è sviluppata, controllata e firmata al 100% da noi.
