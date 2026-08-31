# Ermete OS: Architettura del Kernel e Livello di Sicurezza

Benvenuti nella documentazione ufficiale dell'architettura di base di Ermete OS, con focus specifico sul Kernel e il livello di Sicurezza. Questo documento è rivolto agli ingegneri di sistema, agli sviluppatori e agli appassionati che desiderano comprendere i principi fondamentali che guidano il nostro approccio alla costruzione di un sistema operativo moderno, robusto e resiliente.

In Ermete OS, crediamo che la sicurezza e le prestazioni non debbano essere mutuamente esclusive. Tuttavia, riconosciamo anche che nessun sistema è perfetto. Le tecnologie e i paradigmi descritti in questa pagina non sono soluzioni magiche che garantiscono l'infallibilità, ma piuttosto strumenti rigorosi adottati per mitigare i rischi, ridurre al minimo il margine di errore umano e garantire un degrado aggraziato in condizioni estreme.

---

## 1. Il Sistema Nervoso Autonomo: eBPF e Networking Zero-Overhead

Il networking nei sistemi operativi tradizionali spesso soffre di colli di bottiglia dovuti ai continui cambi di contesto tra user-space e kernel-space e alle innumerevoli copie di buffer. In Ermete OS, abbiamo riprogettato il percorso dei pacchetti adottando un approccio "Zero-Overhead".

### AF_XDP e Umem

Per le applicazioni ad altissime prestazioni e per il routing interno del mesh, Ermete OS sfrutta intensamente le tecnologie **eBPF (Extended Berkeley Packet Filter)** in combinazione con **AF_XDP (XDP Address Family)**.

Invece di processare i pacchetti attraverso il normale stack TCP/IP del kernel linux, utilizziamo socket `AF_XDP`. Questo ci permette di reindirizzare i pacchetti di rete direttamente dalle code della scheda di rete (NIC) allo user-space.
Il vero salto prestazionale è garantito dall'utilizzo di **Umem**, una regione di memoria condivisa tra lo user-space e il kernel. Quando un pacchetto arriva, viene scritto direttamente nell'Umem. L'applicazione in user-space riceve un descrittore che punta a questa memoria, permettendo di leggere il pacchetto **senza alcuna copia (zero-copy)**.

Questo approccio ci consente di saturare link a 100 Gbps utilizzando una frazione delle risorse della CPU, riducendo le latenze a livelli di microsecondi. Non eliminiamo la complessità del networking, ma la spostiamo dove può essere gestita in modo più efficiente, riducendo l'impatto sul resto del sistema.

### Scheduling eBPF In-Kernel Deterministico (Zero-AI)

Oltre al networking, eBPF � il cuore pulsante del nostro sistema di telemetria e scheduling. In Ermete OS, le decisioni di scheduling si basano su solide e rigorose euristiche deterministiche, allontanandosi dalle pericolose allucinazioni dei modelli di Intelligenza Artificiale locale.

Abbiamo rimosso ogni traccia di inferenza instabile (NPU/GPU/candle-core) a favore di uno **Static Log Rules Engine**. Il demone `ermete-ebpf-sched` analizza in tempo reale i pattern di carico estratti dalle sonde eBPF (memoria, I/O) e applica pesi di esecuzione precisi, fallendo in modalit� *closed* se i dati non sono disponibili.
I pesi vengono iniettati nel kernel via mappe eBPF (`bpf_map`), influenzando le code di scheduling in modo dinamico e adattivo ma **matematicamente predicibile**.

---

## 2. Il Livello Hypervisor: KVM, CVMs e l'EnclaveManager

In un'ottica Cloud-Native e Zero-Trust, l'isolamento è fondamentale. Ermete OS adotta un approccio in cui le applicazioni non fidate o i servizi critici non condividono lo stesso ambiente del sistema host.

Sfruttando **KVM (Kernel-based Virtual Machine)** e framework come `crosvm`, eseguiamo MicroVM (Micro Virtual Machines) con accelerazione hardware. Quando l'hardware lo supporta, utilizziamo **CVM (Confidential Virtual Machines)** (es. AMD SEV-SNP o Intel TDX) per garantire che nemmeno l'hypervisor stesso possa leggere la memoria dell'ospite.

Il ciclo di vita di queste micro-istanze è gestito dal nostro **`EnclaveManager`**.
L'`EnclaveManager` è un componente scritto in Rust, progettato per orchestrare le MicroVM con tempi di avvio nell'ordine dei millisecondi. Esso assegna le risorse (vCPU, memoria, dispositivi virtio) e stabilisce i canali di comunicazione sicuri (VSOCK) tra l'host e l'enclave. L'obiettivo è confinare eventuali compromissioni: se un'applicazione all'interno di una MicroVM subisce una violazione, l'attaccante rimane intrappolato nell'enclave, isolato dal Ring-0 dell'host.

---

## 3. Gatekeeper Zero-Trust: Isolamento I/O e di Rete

Il paradigma Zero-Trust in Ermete OS impone che nessuna entità (utente, processo o servizio) sia considerata affidabile di default, nemmeno se in possesso di privilegi elevati.

### fanotify per l'Auditing I/O

Il nostro **Gatekeeper** utilizza pesantemente l'API **`fanotify`** del kernel Linux per intercettare e autorizzare in tempo reale gli accessi al filesystem. Ogni tentativo di apertura, esecuzione o modifica di un file critico viene sospeso finché il demone in user-space (il Gatekeeper) non lo analizza e lo autorizza in base alle policy immutabili del sistema. Questo approccio previene attivamente l'esecuzione di binari non firmati o la modifica di directory di sistema, agendo come un solido strato di difesa in profondità.

### nftables per il Micro-segmentamento di Rete

Di pari passo con l'isolamento del filesystem, implementiamo rigide policy di rete a livello di kernel utilizzando **`nftables`**. Le policy non si limitano al classico concetto di "firewall", ma implementano un micro-segmentamento dinamico. Ogni servizio o MicroVM dispone di namespace di rete dedicati e regole `nftables` generate dichiarativamente che permettono solo il traffico strettamente necessario. Nessun servizio può aprire porte arbitrarie verso l'esterno o contattare lateralmente altri servizi senza un'esplicita dichiarazione di intenti.

---

## 4. Verifica Formale Continua con Kani

Costruire un sistema operativo complesso in Rust elimina intere classi di bug legati alla sicurezza della memoria (come use-after-free o buffer overflow). Tuttavia, Rust non previene errori di logica, deadlock o panic imprevisti (ad esempio tramite l'abuso di `.unwrap()`).

Per garantire un livello di affidabilità enterprise, Ermete OS adotta la **Verifica Formale** continua del codice critico.
Utilizziamo **`cargo kani`** (un model checker basato su bounded model checking) all'interno delle nostre pipeline CI/CD per dimostrare matematicamente l'assenza di determinate classi di errori in moduli chiave.

Sottoponiamo a verifica formale:
- I parser dei protocolli di rete.
- Le implementazioni dei lock e i moduli concorrenti (`RwLock`, `Mutex`).
- Il codice di validazione crittografica.

Con Kani, non ci limitiamo a "testare" il codice con casi d'uso noti, ma dimostriamo formalmente che, entro determinati confini, il codice non andrà mai in panic, non produrrà overflow aritmetici e rispetterà le invarianti definite. È un processo computazionalmente costoso, ma vitale per componenti eseguiti in Ring-0 o come demoni di sistema.

---

## Conclusione

L'architettura del Kernel e della Sicurezza di Ermete OS è il risultato di scelte ingegneristiche deliberate. Uniamo tecnologie all'avanguardia come eBPF, Scheduling eBPF deterministico, MicroVM confidenziali e verifica formale matematica, il tutto orchestrato in Rust.

Il nostro approccio è guidato dalla consapevolezza che il software è intrinsecamente fallibile. L'adozione di queste tecnologie non ci rende immuni da bug, ma costruisce una serie di compartimenti stagni e reti di sicurezza che arginano gli errori, mitigano gli attacchi e mantengono il sistema stabile e reattivo in ogni condizione. Invitiamo la community a studiare questo codice, a sfidare le nostre assunzioni e a contribuire a rendere Ermete OS ancora più sicuro.

