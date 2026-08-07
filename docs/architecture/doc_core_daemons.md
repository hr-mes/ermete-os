# Architettura dei Demoni Centrali di Ermete OS: `ermete-daemon-rs`, `ermete-gatekeeper-rs` e Sicurezza IPC / Polkit

## 1. Panoramica Architetturale Generale

I demoni centrali di Ermete OS garantiscono l'orchestrazione dello stato del sistema, il supporto all'integrazione desktop (XDG Desktop Portals), la persistenza delle impostazioni utente e la sicurezza di esecuzione Zero-Trust a livello di kernel.

L'architettura si divide principalmente in due componenti fondamentali:
1. **`ermete-daemon-rs` (Bedrock, Settings Engine & Desktop Portals)**: Eseguito nel contesto della **D-Bus Session Bus** utente (con connessione secondaria al System Bus per i servizi di sistema). Gestisce lo stato delle impostazioni ACID, il proxy verso NetworkManager e BlueZ, la sintesi vocale e i portali XDG Desktop (Settings, ScreenCast e RemoteDesktop).
2. **`ermete-gatekeeper-rs` (Zero-Trust Security Execution Gatekeeper)**: Eseguito come servizio di sistema **Root (Systemd Service)** registrato sulla **D-Bus System Bus**. Intercetta le chiamate di esecuzione di file binari in tempo reale tramite il sottosistema del kernel Linux `fanotify`, garantendo l'isolamento in sandbox Bubblewrap (`bwrap`) per i binari non verificati o in quarantena.

Entrambi i demoni sono sviluppati in **Pure Rust** utilizzando l'allocatore ad alte prestazioni `mimalloc` e la libreria asincrona **`zbus 5.x`** per le comunicazioni IPC su D-Bus.

---

## 2. Analisi Dettagliata di `ermete-daemon-rs` (Bedrock & Desktop Services)

### 2.1 Architettura dei Moduli e Modello Actor/Channel

`ermete-daemon-rs` gestisce lo stato di sistema mediante un modello asincrono basato su Tokio e canali di comunicazione `tokio::sync` (`watch`, `mpsc`, `oneshot`).

```
                              ┌──────────────────────────────────────────────┐
                              │            ermete-daemon-rs                  │
                              │           (D-Bus Session Bus)                │
                              └──────────────────────┬───────────────────────┘
                                                     │
         ┌───────────────────┬───────────────────────┼───────────────────────┬───────────────────┐
         │                   │                       │                       │                   │
┌────────▼────────┐ ┌────────▼────────┐    ┌─────────▼─────────┐   ┌─────────▼─────────┐ ┌────────▼────────┐
│   bedrock.rs    │ │   settings.rs   │    │    network.rs     │   │   bluetooth.rs    │ │ portal_screencast│
│ os.ermete.      │ │ org.ermete.     │    │ os.ermete.Bedrock.│   │ os.ermete.Bedrock.│ │ org.freedesktop. │
│ Bedrock         │ │ Settings        │    │ Network           │   │ Bluetooth         │ │ impl.portal.*    │
└────────┬────────┘ └────────┬────────┘    └─────────┬─────────┘   └─────────┬─────────┘ └────────┬────────┘
         │                   │                       │                       │                    │
         │ (Proxy)           │ (ACID Watch/Store)    │ (System Bus Proxy)    │ (System Bus Proxy) │ (UNIX Socket /
         ▼                   ▼                       ▼                       ▼                    PipeWire)
 ┌───────────────┐   ┌───────────────┐       ┌───────────────┐       ┌───────────────┐    ┌───────────────┐
 │ AudioWorker   │   │settings.json  │       │NetworkManager │       │    BlueZ      │    │  Niri Socket  │
 │ D-Bus Service │   │(Atomic Temp)  │       │  System Bus   │       │  System Bus   │    │  / PipeWire   │
 └───────────────┘   └───────────────┘       └───────────────┘       └───────────────┘    └───────────────┘
```

#### Moduli Componenti:

1. **`bedrock.rs` (`os.ermete.Bedrock`)**:
   - **Responsabilità**: Gestione dei parametri base del sistema utente (es. volume audio principale via `AtomicU64`).
   - **Flusso IPC**: Comunica con il servizio `os.ermete.AudioWorker` tramite `AudioWorkerProxy` inviando le modifiche di volume.

2. **`settings.rs` (`org.ermete.Settings` / `os.ermete.Bedrock.Settings`)**:
   - **Decentralized Domain States**: Mantiene i micro-stati di dominio decentralizzati `AppearanceDomainState` (tema chiaro/scuro, colori di accento, wallpaper, configurazione dock, True Tone) e `VoiceOverDomainState` (VoiceOver enabled).
   - **Persistenza Atomica**: Scrive le impostazioni su file di dominio dedicati (`appearance.json`, `voiceover.json`) con rinominazione atomica su `~/.config/ermete/`.
   - **Actor Loop Asincrono**: Utilizza un canale `mpsc::channel(32)` e messaggi `SettingsCommand` con risposte `oneshot::Sender`. Quando una proprietà viene modificata, aggiorna i canali di dominio dedicati (`watch::Sender<AppearanceDomainState>`, `watch::Sender<VoiceOverDomainState>`), notificando solo i sub-servizi dipendenti dal rispettivo dominio.

3. **`network.rs` (`os.ermete.Bedrock.Network`)**:
   - **Integrazione NetworkManager**: Si connette alla **System D-Bus** e interagisce con `org.freedesktop.NetworkManager`.
   - **Scansione Concorrente AP**: Utilizza `futures_util::future::join_all` e `tokio::join!` per interrogare concorrentemente i dispositivi Wi-Fi (`device_type == 2`) e richiedere scansione ed estrazione delle proprietà degli Access Point (SSID, potenza segnale, flag di sicurezza WPA/RSN).
   - **Wi-Fi Enterprise & VPN**: Supporta la configurazione di reti 802.1x EAP (PEAP) e tunnel VPN (OpenVPN/WireGuard) costruendo dizionari varianti `zbus::zvariant::Value` inviati a `NmSettingsProxy.add_connection`.

4. **`bluetooth.rs` (`os.ermete.Bedrock.Bluetooth`)**:
   - **Integrazione BlueZ**: Si interfaccia con il servizio di sistema BlueZ (`org.bluez`) su `/org/bluez/hci0` tramite `PropertiesProxy` (lettura/scrittura della proprietà `Powered`) e `ObjectManagerProxy` su `/` per enumerare i dispositivi Bluetooth accoppiati e connessi.

5. **`portal.rs` e `portal_screencast.rs` (XDG Desktop Portal Backend)**:
   - **`org.freedesktop.impl.portal.Settings`**: Esporta i parametri di aspetto del desktop (schema colori, accento RGB) leggendoli in modalità reattiva dal `watch::Receiver<AppearanceDomainState>`.
   - **`org.freedesktop.impl.portal.ScreenCast` & `RemoteDesktop`**: Gestisce le sessioni di cattura dello schermo per il compositore Wayland **Niri**. Comunica direttamente con il socket UNIX del compositore (`$NIRI_SOCKET`) tramite `OutputDiscovery::query_niri_outputs()` per rilevare i monitor fisici e risolve dinamicamente i `node_id` di PipeWire (`PipeWireStreamManager::resolve_pipewire_node`).

6. **`voiceover.rs` (`os.ermete.VoiceOver`)**:
   - Legge lo stato dal canale `watch::Receiver<VoiceOverDomainState>`. Se l'accessibilità è abilitata, inoltra i testi da sintetizzare al servizio `os.ermete.VoiceOverWorker`.

7. **`qos.rs` (App Nap QoS Observer)**:
   - Controlla in background i PID delle applicazioni in secondo piano applicando un valore di nice elevato (`nice 19`) tramite `libc::setpriority(PRIO_PROCESS, pid, 19)` per preservare le risorse CPU del sistema.

---

## 3. Analisi Dettagliata di `ermete-gatekeeper-rs` (Zero-Trust Execution Gatekeeper)

`ermete-gatekeeper-rs` è il demone centrale di sicurezza Zero-Trust di Ermete OS. Previene l'esecuzione involontaria o malevola di binari non autorizzati o scaricati da fonti esterne.

### 3.1 Intercettazione Kernel via `fanotify`

Il demone inizializza un descrittore di file `fanotify` in modalità non bloccante:
```rust
libc::fanotify_init(
    FAN_CLASS_CONTENT | FAN_NONBLOCK,
    (libc::O_RDONLY | libc::O_LARGEFILE) as u32
)
```
Successivamente applica la marcatura di monitoraggio sui mount point del file system critici (`/var/home`, `/tmp`, `/var/tmp`, `/opt`):
```rust
libc::fanotify_mark(
    fanotify_fd,
    FAN_MARK_ADD | FAN_MARK_MOUNT,
    FAN_OPEN_EXEC_PERM,
    libc::AT_FDCWD,
    path.as_ptr()
)
```
Quando un processo tenta di eseguire un binario su uno di questi file system, il kernel blocca il processo in attesa dell'autorizzazione `FAN_ALLOW` o `FAN_DENY` da parte del demone Gatekeeper.

### 3.2 Flusso di Verifica della Quarantena e Approvazione Sandbox

```
[ Kernel Execution Request ] ──► (fanotify: FAN_OPEN_EXEC_PERM)
                                          │
                                          ▼
                         ┌─────────────────────────────────┐
                         │ Is file xattr quarantined?      │
                         │ (user.ermete.quarantine check)  │
                         └────────────────┬────────────────┘
                                          │
                   ┌──────────────────────┴──────────────────────┐
                   │ NO                                          │ YES
                   ▼                                             ▼
        [ Send FAN_ALLOW ]                     [ Freeze Execution & Register fd_id ]
       (Allow Native Exec)                                       │
                                                                 ▼
                                                  [ Emit D-Bus Signal: prompt_required ]
                                                                 │
                                                                 ▼
                                                  [ User Interaction in Gatekeeper UI ]
                                                                 │
                                                                 ▼
                                                  [ Invocation of approve_execution(fd_id) ]
                                                                 │
                                                                 ▼
                                                  [ Polkit Check: pkcheck os.ermete.gatekeeper.approve ]
                                                                 │
                                      ┌──────────────────────────┴──────────────────────────┐
                                      │ Success                                             │ Failed
                                      ▼                                                     ▼
                       [ Remove xattr user.ermete.quarantine ]                    [ Send FAN_DENY ]
                                      │                                           (Block Exec)
                                      ▼
                       [ Spawn binary inside Bubblewrap ]
                       (bwrap --unshare-all ...)
                                      │
                                      ▼
                       [ Send FAN_DENY to unsandboxed original ]
```

#### Passaggi Dettagliati dell'Algoritmo:

1. **Rilevamento Evento**: L'event loop asincrono basato su `tokio::io::unix::AsyncFd` legge la struttura `fanotify_event_metadata`.
2. **Controllo dell'Attributo Esteso (TOCTOU-Safe)**: Risolve il percorso tramite `/proc/self/fd/<fd>` ed esegue un controllo non bloccante via `tokio::task::spawn_blocking` per verificare la presenza dell'attributo esteso `user.ermete.quarantine`.
3. **Gestione dei Binari Non Quarantenati**: Se l'attributo non è presente, risponde immediatamente al kernel con `FAN_ALLOW` e chiude il descrittore.
4. **Intercettazione e Prompt UI**: Se il binario è quarantenato:
   - Assegna un `fd_id` univoco e memorizza il file descriptor in una mappa thread-safe `pending_events`.
   - Invia un segnale D-Bus `prompt_required(fd_id, app_name)` sulla **System D-Bus**.
   - L'interfaccia utente (`gatekeeper-ui`) mostra un avviso grafico richiedendo l'approvazione dell'utente.
5. **Approvazione e Sandboxing Bubblewrap**:
   - L'interfaccia o l'utente invoca il metodo D-Bus `approve_execution(fd_id)`.
   - **Verifica Polkit**: Il demone esegue `pkcheck --system-bus-name <sender> --action-id os.ermete.gatekeeper.approve`.
   - Se autorizzato, rimuove l'attributo di quarantena `user.ermete.quarantine`.
   - Avvia l'applicazione all'interno di una sandbox isolata **Bubblewrap (`bwrap`)** con flag restrittivi: `--unshare-all`, `--share-net`, `--ro-bind` per `/usr`, `/lib`, `/lib64`, `/etc`, e `--proc /proc`.
   - Risponde al kernel con **`FAN_DENY`** per l'esecuzione originale non protetta, delegando la gestione dell'applicazione esclusivamente al processo sandboxed figlio appena creato.

---

## 4. Sicurezza IPC, Polkit e D-Bus Policy

### 4.1 Identificatori Polkit e Interfacce D-Bus

| Demone | Bus D-Bus | Interfaccia D-Bus | Azione Polkit (`action-id`) | Descrizione |
| :--- | :--- | :--- | :--- | :--- |
| `ermete-gatekeeper-rs` | **System Bus** | `os.ermete.Gatekeeper` | `os.ermete.gatekeeper.approve` | Approvazione ed esecuzione sandboxed di binari in quarantena |
| `ermete-gatekeeper-rs` | **System Bus** | `os.ermete.Gatekeeper` | `os.ermete.gatekeeper.root` | Richiesta elevazione privilegi root (con fallback FIDO2) |
| `ermete-daemon-rs` | **Session Bus** | `org.ermete.Settings` | N/A (Mocked check) | Modifica impostazioni utente e aspetto desktop |
| `ermete-daemon-rs` | **Session Bus** | `os.ermete.Bedrock` | N/A (Mocked check) | Regolazione parametri audio/volume di sistema |

### 4.2 Analisi dei Rischi di Sicurezza e Vulnerabilità Rilevate

Durante l'audit del codice sorgente sono stati identificati i seguenti punti di attenzione sulla sicurezza IPC e la concorrenza:

1. **Mock Check di Polkit in `settings.rs` e `bedrock.rs`**:
   - Nel file `settings.rs` (linea 20) e `bedrock.rs` (linea 6), la funzione `check_polkit_auth()` restituisce in modo hardcoded `true` senza effettuare un'autenticazione reale via `pkcheck` o D-Bus Authority.
   - *Raccomandazione*: Sostituire le funzioni stub con chiamate asincrone a `zbus::fdo::AuthorityProxy` o invoche `pkcheck` reali prima di accettare la mutazione delle impostazioni di sistema.

2. **Rischio TOCTOU e Enumerazione `fd_id` in `ermete-gatekeeper-rs`**:
   - Gli identificatori `fd_id` vengono generati tramite un semplice contatore incrementale `next_id += 1`.
   - *Raccomandazione*: Associare ogni `fd_id` al D-Bus unique sender che ha innescato l'evento ed estendere l'uso di UUID v4 casuali per evitare tentativi di indovinamento o hijacking delle chiamate `approve_execution`.

3. **Iniezione di Comandi Unsafe via Shellout**:
   - `settings.rs` esegue comandi di sistema per l'applicazione dei temi (`dconf`, `matugen`, `wlsunset`, `swww`, `spd-say`).
   - *Raccomandazione*: Validare ed igienizzare rigorosamente tutti i parametri di stringa provenienti dai messaggi D-Bus prima di passarli ai comandi di sistema.

4. **Gestione delle Eccezioni ed Eliminazione dei Panici**:
   - In alcuni punti di `network.rs`, `bluetooth.rs` e `portal_screencast.rs` sono presenti utilizzi di `unwrap()` e `expect()` durante la deserializzazione delle varianti D-Bus. Un payload malformato potrebbe provocare il crash imprevisto del demone.
   - *Raccomandazione*: Convertire tutti gli `unwrap()` in gestione esplicita con `match` o `?` ritornando errori `zbus::fdo::Error::Failed`.

---

## 5. Matrice Architetturale e Mappatura Dipendenze (CodeGraph)

```mermaid
graph TD
    subgraph Kernel Space
        KERN[Linux Kernel fanotify]
    end

    subgraph User Space System Daemons (Root)
        GK[ermete-gatekeeper-rs]
        PK[Polkit Authority / pkcheck]
        BWRAP[Bubblewrap Sandbox Engine]
        NM[NetworkManager Service]
        BZ[BlueZ Bluetooth Daemon]
    end

    subgraph User Space Session Daemons (User Session)
        DM[ermete-daemon-rs]
        NIRI[Niri Compositor / $NIRI_SOCKET]
        PW[PipeWire Audio/Video Server]
        UI[Gatekeeper UI Prompt]
    end

    KERN -- FAN_OPEN_EXEC_PERM --> GK
    GK -- Check xattr user.ermete.quarantine --> KERN
    GK -- D-Bus Signal prompt_required --> UI
    UI -- Call approve_execution --> GK
    GK -- Autenticazione pkcheck --> PK
    GK -- Remove xattr & Launch --> BWRAP
    GK -- Send FAN_DENY original exec --> KERN

    DM -- System Bus Proxy --> NM
    DM -- System Bus Proxy --> BZ
    DM -- UNIX Socket Query --> NIRI
    DM -- Stream Node Resolution --> PW
```

---

### Conclusioni e Prossimi Passi

L'architettura dei demoni centrali di Ermete OS dimostra un'eccellente separazione delle responsabilità tra la gestione dello stato e delle interfacce desktop (`ermete-daemon-rs`) e l'enforcement della sicurezza a livello kernel (`ermete-gatekeeper-rs`). L'integrazione di `fanotify` con le sandbox `bwrap` fornisce una difesa Zero-Trust di alto livello.

L'implementazione delle raccomandazioni di hardening (rimozione dei mock Polkit, sanitizzazione input e gestione difensiva degli errori senza `unwrap`) consentirà di raggiungere il massimo livello di affidabilità e sicurezza enterprise.
