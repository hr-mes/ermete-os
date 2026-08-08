# 🔬 ERMETE OS: Report di Audit Architetturale e God Node

L'analisi incrociata tra la codebase, i dati del Knowledge Graph (CodeGraph) e i requisiti Big-Tech documentati negli audit di ieri ha rivelato la presenza di 5 colli di bottiglia critici, noti come "God Node" (strutture software onnipotenti e monolitiche). Questi blocchi impediscono l'evoluzione verso un sistema operativo reattivo, sicuro e guidato dall'AI.

---

## 1. Il God Node Primario: `SystemController` (Orchestratore Monolitico)
**Posizione:** `ermete-shell-rs/ermete-shell-rs-1.0.0/src/core/system_controller.rs` (>500 righe)

Questo struct agisce come un Facade onnipotente. Invece di delegare le responsabilità a domini isolati, centralizza la gestione di **tutto**:
- Rete (WiFi) e Bluetooth
- Audio (Volume, Mute) e Luminosità
- Riproduzione Multimediale (MPRIS / Player Commands)
- Gestione Energetica e Spegnimento

**Violazione Big-Tech:** Rompe brutalmente il _Single Responsibility Principle (SRP)_. Questa architettura monolitica impedisce le implementazioni asincrone modulari. Ad esempio, qualsiasi evento innescato dalla "Morphic Pill" costringerebbe il `SystemController` a un re-render o a una gestione di stato globale non necessaria, creando un collo di bottiglia sulle prestazioni di sistema.

---

## 2. Lo Stato Globale Centralizzato in `SettingsService`
**Posizione:** `ermete-daemon-rs-0.2.1/src/settings.rs` e `ermete-shell-rs-1.0.0/src/core/system_proxies.rs`

La struttura `SettingsState` definisce l'intero stato del sistema operativo in un'unica enorme struct monolitica (che contiene boolean per wifi, bluetooth, mute, e stringhe per l'SSID attivo).

**Violazione Big-Tech:** Questo pattern crea un _tight coupling_ estremo. Modificare un parametro del volume forza il lock del Mutex dell'intero stato del sistema. I settings dovrebbero essere entità isolate gestite da un message broker distribuito a zero-copia (es. tramite un SystemEventBus decentralizzato), essenziale per abilitare il routing del linguaggio naturale (*Natural Language Routing*) prescritto dall'audit UI.

---

## 3. L'Interfaccia Monolitica del Greeter: `greeter.rs`
**Posizione:** `ermete-shell-rs/ermete-shell-rs-1.0.0/src/greeter.rs` (780 righe)

Questo modulo è gigantesco e mescola letalmente la logica di render grafico (UI) con la complessa logica di autenticazione e sicurezza.

**Violazione Big-Tech:** L'audit UI/UX prescrive una "Sicurezza Visiva" (_Seamless Biometrics_ ed _Explainable Security_). Il greeter attuale è un blocco di codice legacy che non permette l'integrazione di un'Enclave fidata (come YubiKey o Windows Hello), né facilita il passaggio verso il *Confidential Computing* (Intel TDX) richiesto nella roadmap. L'autenticazione deve essere disaccoppiata e astratta in un servizio PAM asincrono.

---

## 4. UI Congestionata nel Dock: `ui.rs`
**Posizione:** `ermete-dock/ermete-dock-1.0.0/src/ui.rs` (715 righe)

Il file gestisce contemporaneamente il layout, l'aggiornamento visivo, l'interazione del mouse e la sincronizzazione profonda dello stato delle finestre (Niri).

**Violazione Big-Tech:** Non è predisposto per lo **Spatial Dock (Wayland Native)** né per il *Drag & Drop Universale*. Per permettere il lancio di "Contesti" anziché di app isolate (es. aprire lo spazio di lavoro "Ricerca Web"), la logica di business del dock deve essere separata dall'interfaccia grafica GTK4, spostando la logica contestuale in un gestore di stato Wayland-first.

---

## 5. Mancanza di Astrazione per l'AI e latenza RPC
**Posizione:** `topbar.rs`, `spotlight.rs`, `system_proxies.rs`

La struttura della shell è attualmente cablata ad azioni predefinite. Attualmente l'Omni-Spotlight chiama metodi di ricerca statica anziché interfacciarsi con il Local AI Daemon.
Inoltre, l'assenza di agganci *eBPF* nei proxy di sistema denota che la rete e le app usano ancora D-Bus tradizionale (es. `BlueZProxy`) con timeout di 5 secondi.

**Violazione Big-Tech:** Palese contrasto con la direttiva "Ring-0 Analytics a latenza zero". Dobbiamo rimpiazzare il polling e le chiamate DBus sincrone con notifiche push eBPF-driven per avere prestazioni istantanee.

---

## 🚀 Prossimi Passi
L'azione prioritaria per l'architettura sarà **destrutturare il `SystemController` e il `SettingsService` in micro-servizi asincroni Actor-based**, pronti per essere pilotati dall'AI Locale e per operare a latenza zero senza bloccare il thread principale dell'interfaccia utente.
