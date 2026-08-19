# Ermete OS: Architettura Build System & Mesh Layer

Questo documento descrive formalmente il livello costruttivo (Build System) e il livello di interconnessione (Mesh Layer) di Ermete OS.
L'obiettivo di questi sottosistemi è fornire un'infrastruttura deterministica, immutabile e scalabile, orientata all'affidabilità mission-critical per ambienti enterprise e alla totale trasparenza operativa per l'utente finale.

## 1. Il Paradigma di Immutabilità (Immutable OS)

Ermete OS abbraccia una filosofia architetturale in cui il sistema operativo base non è modificabile a runtime tramite mutazioni non tracciate. Adottiamo un modello di distribuzione tramite container OCI, integrando:

*   **Fedora bootc & OSTree**: Il file system radice è fornito come immagine OCI e montato in sola lettura. Gli aggiornamenti avvengono in modo transazionale e atomico ("A/B patching"), azzerando il rischio di corruzione del sistema durante i processi di update.
*   **Riproducibilità Deterministica**: La generazione del rootfs avviene tramite *BuildKit* (e *Podman*), assemblando le immagini a partire da un `Containerfile` dichiarativo e terminando con una Unified Kernel Image (UKI) per garantire l'integrità del Secure Boot.
*   **Vantaggio Enterprise**: Le macchine si comportano in modo identico, le regressioni sono immediatamente reversibili tramite rollback atomico, limitando l'intervento sistemistico (Zero-Touch Provisioning).

## 2. DAG Orchestrator e Caching Distribuito

Per gestire la risoluzione delle dipendenze, sia di sistema (RPM) sia applicative (Flatpak), Ermete OS utilizza un DAG (Directed Acyclic Graph) Orchestrator scritto in Python.

*   **Costruzione del Grafo delle Dipendenze**: L'orchestrazione converte i requisiti dei pacchetti in un DAG, parallelizzando in modo aggressivo il fetch e l'installazione dei componenti che non presentano dipendenze incrociate.
*   **Caching via Redis e Resilienza Locale**: Per velocizzare le operazioni su scala cluster, l'Orchestrator impiega un sistema di caching basato su **Redis**. I layer precedentemente scaricati, i metadati e i delta di aggiornamento sono condivisi tra i nodi. Tuttavia, nel rispetto dei principi di fault tolerance, in caso di indisponibilità della rete o del cluster Redis, il sistema esegue un fallback trasparente sulla directory `.cache/` locale, assicurando la continuità operativa senza bloccare l'installazione o il boot.
*   **Isolamento delle Dipendenze**: Questa astrazione previene la "Dependency Hell", analizzando preventivamente i conflitti e sfruttando il DAG per verificare i vincoli di coerenza prima che avvenga l'applicazione transazionale dell'aggiornamento.

## 3. ZeroCopyRingBuffer: L'IPC ad Altissime Prestazioni

Al centro della comunicazione intra-processo di Ermete OS risiede lo `ZeroCopyRingBuffer`, il nodo più interconnesso ed essenziale per il bus di sistema locale.

*   **Zero-Copy Memory Mapping**: I processi cooperanti (es. daemon di sistema, layer UI) comunicano condividendo memoria attraverso ring buffer mmap-ati, permettendo al produttore di scrivere e al consumatore di leggere senza transizioni user-to-kernel per copiare i payload.
*   **Minimizzazione della Latenza**: Progettato per operare in lock-free o wait-free semantics ove possibile, elimina il context switching.
*   **Impatto**: L'efficienza energetica e la reattività dell'OS aumentano in modo misurabile. Le applicazioni critiche che muovono grandi quantità di dati (audio, video, metriche ad alta frequenza) non saturano i cicli di CPU per il marshalling/unmarshalling.

## 4. Mesh Bus: L'Ecosistema Distribuito Sicuro

Per espandere l'infrastruttura locale su una scala cloud-native distribuita, Ermete OS adotta il **Mesh Bus**, che parifica i nodi locali e remoti permettendo orchestrazioni swarm.

*   **PeerManager e MeshTunnel**: Il `PeerManager` coordina la scoperta e lo stato (health check, liveness) dei nodi sulla rete (LAN e WAN). Ogni comunicazione transita attraverso un `MeshTunnel`.
*   **Crittografia Post-Quantum e X25519**: Il tunnel implementa la crittografia basata su WireGuard, impiegando lo scambio di chiavi **X25519** (e supportando crittografia ibrida con primitive post-quantum per la segretezza in avanti). Le connessioni sono Zero-Trust; nessun nodo è intrinsecamente fidato senza verifica di identità tramite policy rigorose.
*   **Sincronizzazione di Stato**: Tramite questo bus sicuro, i servizi dell'OS possono sincronizzare lo stato applicativo, distribuire carichi e applicare policy di autoguarigione (Auto-healing) all'interno del cluster. Il risultato finale è un ambiente che appare all'utente e agli amministratori IT come un singolo computer resiliente.

---
*Architettura validata secondo gli standard Ermete OS Gold.*
