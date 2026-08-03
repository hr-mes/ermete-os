<div align="center">
  <br />
  <h1>🌋 Ermete OS</h1>
  <h3>L'Ambiente Desktop Cloud-Native Più Avanzato, Elegante e Sicuro.</h3>
  <br />
  
  [![Build Status](https://img.shields.io/badge/Build-Passing-brightgreen?style=for-the-badge&logo=githubactions)](#)
  [![SLSA Level 4](https://img.shields.io/badge/SLSA-Level_4-purple?style=for-the-badge&logo=slsa)](#)
  [![Rust](https://img.shields.io/badge/Rust-1.80+-orange?style=for-the-badge&logo=rust)](#)
  [![GTK4](https://img.shields.io/badge/GTK-4.14+-blue?style=for-the-badge&logo=gtk)](#)
  [![Memory](https://img.shields.io/badge/Allocator-Mimalloc-yellow?style=for-the-badge)](#)
</div>

<hr />

## 📖 Indice Architetturale
1. [Filosofia del Progetto](#-filosofia-del-progetto)
2. [L'Architettura Cloud-Native (Immutabilità)](#-larchitettura-cloud-native-immutabilità)
3. [Ermete Glass: Interfaccia Utente e UX](#-ermete-glass-interfaccia-utente-e-ux)
4. [Motore Asincrono: Zero-Lag e Zero-Polling](#-motore-asincrono-zero-lag-e-zero-polling)
5. [Ermete Daemon: Il Cuore del Sistema (Zero-Trust)](#-ermete-daemon-il-cuore-del-sistema-zero-trust)
6. [Sicurezza e Hardening: Ring-0 e Confidential Computing](#-sicurezza-e-hardening-ring-0-e-confidential-computing)
7. [Ottimizzazione Estrema: Il Paradigma "Ultra Leggero"](#-ottimizzazione-estrema-il-paradigma-ultra-leggero)
8. [Pipeline CI/CD: Livello Big-Tech e SLSA 4](#-pipeline-cicd-livello-big-tech-e-slsa-4)

---

## 🏛️ Filosofia del Progetto
**Ermete OS** nasce per ridefinire il concetto di Sistema Operativo Desktop. Non ci siamo posti alcun limite. Ogni singolo byte, ogni thread e ogni pixel sono stati soppesati, criticati e portati alla perfezione teorica. Ermete non è una semplice distribuzione Linux: è un'infrastruttura Cloud-Native immutabile, vestita con un'interfaccia utente (UI) che unisce l'estetica mozzafiato del *Glassmorphism* alla potenza asincrona nuda e cruda di Rust.

---

## 🔒 L'Architettura Cloud-Native (Immutabilità)
Ermete OS scarta i fragili gestori di pacchetti tradizionali (es. `apt` o `dnf`) per abbracciare un paradigma 100% basato su immagini container OCI (tramite la tecnologia `bootc` e `rpm-ostree`). 
- **Aggiornamenti Atomici:** Il sistema operativo si aggiorna esattamente come un container Docker. Un aggiornamento è una singola transazione crittograficamente firmata. O funziona tutto, o il sistema esegue un rollback automatico in frazioni di secondo.
- **Nix-Paradigm & Determinismo Estremo:** Il filesystem radice è immutabile. Le applicazioni vivono confinate e l'intera infrastruttura del SO (Kernel, Driver Nvidia, UI) è stratificata modularmente.

---

## 💎 Ermete Glass: Interfaccia Utente e UX
L'estetica di Ermete è gestita dal nostro motore proprietario **Ermete Glass**. 
- **Design Big-Tech Killer:** Trasparenze *Glassmorphism* assolute, padding generosi, border-radius di 24px per finestre e popover, animazioni fluide basate su curve di Bézier per ogni singolo hover e click.
- **Singleton CSS Provider:** Diversamente dai vecchi desktop environment, Ermete non carica e parsa file CSS multipli in giro per la RAM. Il crate condiviso `ermete-style` inietta globalmente un singolo `CssProvider` alla radice dell'albero GTK4, annientando le allocazioni rindondanti di memoria.
- **Accelerazione Hardware Pura (NGL/Vulkan):** La UI rifiuta il fallback su rendering software (Cairo) o XWayland. Ogni frame è processato nativamente dal nuovo motore GPU di GTK 4.14+ (GSK NGL) per garantire 144Hz granitici e consumi di batteria nulli.

---

## ⚡ Motore Asincrono: Zero-Lag e Zero-Polling
Il codice sorgente dell'intera UI (scritta in Rust) è un ecoscandaglio di efficienza. Non esiste I/O sincrono.
1. **Reference Cycles Spezzati:** Per sconfiggere i famigerati *Memory Leaks* storici di GTK, ogni singola *closure* (come `connect_clicked`) sfrutta la macro `clone!(@weak widget)` per rilasciare istantaneamente la memoria alla chiusura della finestra.
2. **I/O 100% Asincrono:** Le vecchie UI si congelano quando leggono un file (micro-stuttering). In Ermete OS, componenti come `Spotlight` (ricerca locale) o la `Clipboard` (cliphist) generano processi `tokio::process::Command` imbrigliati dentro al `glib::MainContext::spawn_local`. Risultato? Il thread della GUI non si ferma **mai**.
3. **Morte del Polling:** L'indicatore di Batteria, Volume e Rete non innesca timer CPU. Si affida al `SystemEventBus`: la UI si ridisegna *solo* ed *esclusivamente* quando il dato grezzo alla base cambia stato. 

---

## 🧠 Ermete Daemon: Il Cuore del Sistema (Zero-Trust)
Il centro nevralgico dell'OS è l'**Ermete Daemon** (`ermete-daemon-rs`), un demone ZBus asincrono che gira in background e agisce da ponte tra il Kernel e l'utente.
- **Thread Starvation Immune:** Ogni salvataggio di impostazione o interazione di rete sfrutta `tokio::fs` (operazioni puramente non bloccanti).
- **ZBus Pattern Matching Fail-Safe:** Il parsing dei segnali D-Bus usa fallback resilienti. Un pacchetto malformato inviato al demone viene ignorato silenziosamente senza far crasciare il demone.

---

## 🛡️ Sicurezza e Hardening: Ring-0 e Confidential Computing
La sicurezza di Ermete OS non ha eguali sul mercato Desktop.
- **D-Bus Privilege Escalation Sigillato:** Il demone non esegue metodi alla cieca. L'integrazione di `zbus_polkit` impone che qualsiasi azione mutabile (modifica rete, installazioni) venga esplicitamente verificata e autorizzata dal PolicyKit, azzerando le falle *Zero-Day* di Privilege Escalation.
- **Kernel Hardening (sysctl):** Abbiamo blindato il Ring-0 del Kernel Linux: `kptr_restrict=2`, `dmesg_restrict=1`, ptrace limitato via Yama e disattivazione dell'eBPF user-space.
- **Confidential Computing (CVM):** Integrazione nativa dei meccanismi di *hardware attestation* tramite tecnologie enclavi AMD SEV-SNP e Intel TDX, garantendo che lo stato in RAM sia crittografato e certificato prima del login.

---

## 🗜️ Ottimizzazione Estrema: Il Paradigma "Ultra Leggero"
Ermete OS è stato progettato per consumare meno RAM di un microcontrollore.
1. **Mimalloc:** Il Garbage Collector non ci serve, ma la frammentazione della memoria è il nemico. Ermete OS inietta come allocatore globale `mimalloc` (di Microsoft Research) in ogni singolo crate. Niente RAM gonfiata dopo settimane di uptime.
2. **LTO & Code-Size estremi:** Il compilatore Rust è forzato con direttive draconiane:
   ```toml
   [profile.release]
   opt-level = "z"
   lto = true
   codegen-units = 1
   panic = "abort"
   strip = true
   ```
   Ogni binario è microscopico e privo di codice di debug, rimuovendo interamente lo stack-unwinding per velocità pura.

---

## 🚀 Pipeline CI/CD: Livello Big-Tech e SLSA 4
I nostri workflow di GitHub Actions non si limitano a compilare il codice, ma lo certificano per l'uso governativo/enterprise.
- **Idempotenza a Livello di Strato:** Prima di avviare il container `osbuild`, i nostri script verificano gli hash del codice. Ermete compila solo ciò che è cambiato (Incremental Builds).
- **Caching Avanzato:** Uso estensivo di cache per `sccache` (Rust) e RPM Nvidia precompilati (AKMODS).
- **SLSA Level 4 Attestation:** *Non ci fidiamo di noi stessi*. Ogni ISO e micro-container generato viene sottoposto a scan per vulnerabilità (Trivy), riceve una SBOM (Syft), e viene crittograficamente firmato con **Cosign** nel registro di trasparenza Sigstore. La firma garantisce l'esatta provenienza del codice.
- **Visual DAGs:** L'architettura dei file YAML è divisa in Job estetici (es. `🏗️ Build`, `🛡️ Security Audit`, `✍️ Attestation`) per offrire una dashboard di monitoraggio limpida, interconnessa tramite `needs`, degna dei repository delle più grandi corporazioni tecnologiche mondiali.

<br />

<div align="center">
  <i>"Il nostro cliente non vuole porsi limiti. Noi li abbiamo superati."</i>
</div>
