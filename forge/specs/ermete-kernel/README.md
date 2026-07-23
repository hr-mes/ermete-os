# 🦅 Ermete Chimera Kernel - Documentazione Architetturale Definitiva

Questa documentazione rappresenta il "Testo Sacro" del kernel di Ermete OS. Ogni scelta tecnica, dall'astrazione ad alto livello fino alla singola macro di compilazione (filosofia *"Vista da Corvo fino alla Bedrock"*), è esplicitata, spiegata e motivata per garantire una manutenibilità assoluta.

---

## 1. Visione Olistica e Architettura a Micro-Container
Il kernel non è un monolite statico agganciato all'OS, ma un modulo ibrido isolato. L'intera compilazione avviene tramite la pipeline `ermete-forge`, che impacchetta i binari generati (RPM) all'interno di un'immagine container `scratch` purissima inviata a GHCR (GitHub Container Registry). 
* **Vantaggio**: Zero inquinamento dell'host build. `ermete-os` può ingerire i binari del kernel pullando semplicemente i file nativi, mantenendo una cronologia individuale per la cache e il deployment.
* **Agnosticismo Driver**: Il kernel base è privo di codice proprietario. Il modulo NVIDIA (Akmods, ecc.) viene mantenuto all'esterno dell'albero del kernel, in un modulo/layer separato.

---

## 2. Il Paradigma della "Chimera" (Base Fedora + CachyOS + Clear Linux)
Al posto di ricompilare ciecamente il kernel vanilla, Ermete usa il kernel base di Fedora (Rawhide/Branched) come solida intelaiatura di sistema, su cui applichiamo dinamicamente in tempo di build un arsenale di patch ad alte prestazioni estrapolate dai migliori progetti del mondo Linux.

### CachyOS (Prestazioni Brute e Responsività)
* **Scheduler BORE (Burst-Oriented Response Enhancer)**: Sostituisce la latenza del CFS standard, bilanciando in modo aggressivo compiti interattivi (Desktop/Gaming) rispetto a processi di calcolo in background.
* **BBRv3**: L'algoritmo di congestione TCP di ultimissima generazione per massimizzare il throughput di rete.

### Clear Linux (Ottimizzazioni Branch/Scheduler)
* **Branch Hints basati su gcov**: Patch chirurgiche per aggiungere `likely/unlikely` hints in chiamate di sistema critiche e core dello scheduler, ottimizzando le predizioni di salto della CPU.

---

## 3. Gestione Difensiva delle Patch e Hashing Chirurgico (Bedrock Patching)
Le patch vengono applicate tramite lo script `prepare-chimera.sh` con una logica draconiana.
* **Hashing Dinamico del Testo**: Per evitare false positive rebuilds, l'orchestratore non calcola più l'hash dell'intero commit upstream di CachyOS, ma esegue l'hashing esclusivamente sul contenuto testuale fisico delle singole patch iniettate (es. `0001-sched-migrate.patch`). Se la patch è invariata, si ottiene un CACHE_HIT immediato (riducendo i tempi CI da 7 minuti a pochi secondi).
* **Strict Fuzz 0**: Non sono ammesse patch fuzzy con offset sballati. Ogni patch viene prima validata con `patch --dry-run`.

---

## 4. Il Santo Graal: LLVM/Clang e l'Auto-DMZ Fuzzer
Per estrarre il 100% delle prestazioni, abbiamo abbandonato definitivamente GCC e il vecchio manifesto PGO. Il kernel è ora **interamente forgiato su toolchain LLVM/Clang** sfruttando l'Architettura Darwiniana del Fuzzer Sintattico:

1. **Toolchain Pura LLVM**: Compilazione con `%toolchain clang`, linker `lld` e attivazione di **ThinLTO** (`CONFIG_LTO_CLANG_THIN=y`).
2. **Auto-DMZ Fuzzer (`auto-dmz-fuzzer.sh`)**: Invece di abbassare le ottimizzazioni globalmente per evitare Kernel Panic, forziamo il kernel in compilazione brutale (`-O3` e `-flto=auto`). 
3. **Isolamento Dinamico AST**: Se Clang si schianta (rottura dell'AST C o errore `-Werror` su codice sporco), l'Auto-DMZ Fuzzer intercetta l'errore, individua la directory del modulo fallato, gli inietta al volo uno "Scudo O2" (`-O2 -fno-lto`) nei Makefile/Kbuild, e rilancia la compilazione. Il kernel risultante è estremo (-O3) dove possibile, e mitigato (-O2) solo dove necessario.

* **Architettura `-march=x86-64-v3`**: Non supportiamo hardware obsoleto. AVX/AVX2 native per architetture moderne come Ryzen 5800X3D.

---

## 5. Decisioni di Esclusione e Ablazione Moduli (Debloat)
Per garantire stabilità "Bedrock" e dimezzare i tempi di build, intere sezioni del kernel sono state piallate:

### 5.1 Rimozione di Rust (`CONFIG_RUST is not set`)
* Per Ermete OS, l'implementazione Rust nel kernel attualmente è acerba. Rimuoverla ha annichilito i tempi di compilazione e reso l'albero LLVM titanicamente più stabile.

### 5.2 Estirpazione Tools e Documentazione
* Iniezioni RPM: `%_without_tools 1`, `%_without_perf 1`, `%_without_selftests 1`. Spegne la fase `install_doc` di Make che in precedenza schiantava l'intera pipeline.

### 5.3 Azzeramento Debug
* `%_without_debug 1`, `%_without_debuginfo 1`, `CONFIG_DEBUG_INFO_NONE=y`. Il kernel DWARF debug bloat è bruciato via per comprimere la grandezza finale.

---

## 6. Il Lifecycle RPM
Il payload RPM è impostato al volo su `w0.gzdio` (nessuna compressione reale) per evitare di consumare CPU inutilmente, dato che l'OCI (Docker/Podman) comprime già i layer in gzip.

## 7. Mantenimento Futuro (DIRETTIVE SEVERE)
* **MAI applicare "cerotti" applicativi (Workaround) per fixare la build**.
* Se la toolchain LLVM si schianta, lascia lavorare l'**Auto-DMZ Fuzzer**. Se l'errore è fatale (DMZ Propagation raggiunge la radice), il problema è strutturale.
* L'unica soluzione accettabile per un fallimento di modulo in `/tools` o sottosistemi inutili è aggiungere la corrispondente flag `%_without_[nome]` in `.rpmmacros`. Sradicare alla radice.
