# Ermete OS: Architettura dell'Interfaccia Utente e dell'Esperienza Utente (Shell & UI)

Benvenuti nella documentazione ufficiale dell'interfaccia utente di Ermete OS. Questo documento esplora le scelte architetturali e di design che alimentano il nostro ambiente desktop, spiegando come abbiamo unito sicurezza, prestazioni e semplicità d'uso. Il nostro obiettivo è offrire un'esperienza fluida, robusta e priva di distrazioni, adatta sia all'utente quotidiano che al professionista.

## Il Compositor Wayland: Sicurezza e Tiling con `niri`

Il cuore visivo di Ermete OS è il nostro compositor Wayland, basato su [niri](https://github.com/YaLTeR/niri). Abbiamo scelto di abbandonare i server grafici tradizionali a favore di un paradigma più moderno, isolato e intrinsecamente sicuro.

### Perché Wayland e `niri`?
In un ambiente Wayland ben implementato, le applicazioni sono confinate: non possono "spiare" l'intero schermo o intercettare gli input destinati ad altre finestre senza permessi espliciti e controllati. Questo isolamento è fondamentale per la nostra filosofia Zero-Trust.

Inoltre, `niri` introduce un concetto innovativo di **TilingEngine a scorrimento** (scrollable tiling). Invece di forzare le finestre a rimpicciolirsi in una griglia statica via via che se ne aprono di nuove, `niri` le dispone su un nastro orizzontale infinito. L'utente può scorrere fluidamente tra le applicazioni, mantenendole a una dimensione leggibile e naturale (*WindowPlacement* intelligente). Questo approccio unisce l'efficienza chirurgica dei window manager a mosaico con una curva di apprendimento morbida, rendendolo naturale per chiunque.

## Il Pannello delle Impostazioni: `ermete-settings-rs`

Per gestire il sistema, abbiamo sviluppato `ermete-settings-rs`, un centro di controllo nativo scritto interamente in **Rust**, utilizzando il toolkit **GTK4** e l'elegante architettura reattiva di **Relm4**.

### Rust vs JavaScript: Efficienza e Sicurezza
Molte interfacce moderne fanno affidamento su tecnologie web (come Electron o framework JavaScript/TypeScript) per costruire la UI. Pur essendo facili da sviluppare, queste soluzioni tendono a consumare enormi quantità di RAM, impegnare intensamente la CPU e introdurre micro-latenze fastidiose.

Scegliendo Rust, GTK4 e Relm4, otteniamo benefici strutturali ineguagliabili:
- **Latenza quasi zero:** L'interfaccia risponde istantaneamente agli input dell'utente perché è compilata in codice macchina nativo e ottimizzato.
- **Sicurezza e Stabilità assolute:** Rust elimina alla radice intere categorie di bug legati alla gestione della memoria (niente *segfault*, niente *data race*). Il pannello delle impostazioni semplicemente non va in crash.
- **Rispetto delle risorse:** Meno cicli di CPU sprecati e meno RAM occupata significano un sistema che respira e batterie dei portatili che durano molto di più.

### Comunicazione con il Sistema
Coerentemente con il principio del privilegio minimo, `ermete-settings-rs` non esegue azioni di modifica del sistema in modo diretto. Agisce invece da client reattivo, dialogando con demoni isolati e di basso livello tramite bus di comunicazione sicuri:
- **D-Bus:** Per la gestione delle chiamate di sistema generali e l'integrazione desktop.
- **WirePlumber / PipeWire:** Per la gestione reattiva, sicura e standard dei dispositivi audio (sostituisce il finto AudioBus deprecato).
- **NetBus:** Per la configurazione e il monitoraggio istantaneo delle reti Wi-Fi e VPN.

Questa separazione netta assicura che il codice di interfaccia rimanga snello, mentre i processi che richiedono privilegi elevati rimangono compartimentalizzati e rigidamente validati.

## L'Esperienza dell'OS Immutabile per l'Utente

Uno dei concetti più potenti e rassicuranti di Ermete OS è la sua **immutabilità**. Per l'utente, questo termine tecnico si traduce in una garanzia molto semplice: **il computer non si rompe da solo.**

Nei sistemi operativi tradizionali, l'installazione di programmi o un aggiornamento di sistema modificano direttamente i file vitali sul disco. Se si verifica un calo di tensione, o se un aggiornamento è difettoso, la macchina può diventare inutilizzabile.
In Ermete OS, le fondamenta del sistema operativo sono in sola lettura. Gli aggiornamenti avvengono in modo **transazionale**:
1. Il sistema scarica il nuovo aggiornamento silenziosamente in background.
2. Viene preparata una "nuova immagine" parallela del sistema, senza alterare quella attualmente in uso.
3. Al riavvio successivo, il computer semplicemente passa alla nuova versione.
4. Se l'aggiornamento introduce problemi, l'utente (o il sistema stesso) può tornare istantaneamente alla versione precedente e funzionante al boot successivo.

Niente schermate di errore irrimediabili, niente "Decadimento del Sistema" nel tempo. L'utente ha sempre a disposizione un dispositivo affidabile come il primo giorno.

## Il Futuro dello Store (App Center)

Al momento, la gestione e l'installazione delle applicazioni utente avvengono tramite strumenti containerizzati robusti (Flatpak/Podman) ma operano in modalità temporaneamente *headless* (senza un'interfaccia grafica proprietaria).

Stiamo lavorando alla realizzazione di un App Store grafico completamente nativo, anch'esso progettato in puro Rust (*pure-Rust UI*). Abbiamo scelto consapevolmente di non rilasciare soluzioni intermedie e approssimative basate sul web, preferendo attendere di poter offrire un'interfaccia che rispetti pienamente i nostri standard: zero latenza, integrazione perfetta con le API transazionali del sistema immutabile, e un'esperienza di navigazione fluida, sicura e coerente.

---
*Ermete OS: costruito con il rigore dei sistemi critici, progettato per la serenità dell'utente.*

