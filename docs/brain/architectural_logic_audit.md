# 🕵️‍♂️ Ermete OS: Deep Architectural & Logic Audit

Questo report aggrega le criticità logiche, architetturali e topologiche riscontrate dallo sciame specializzato (Graph Topology Analyst, Logic & Redundancy Auditor, Architecture Critic) nel codice proprietario di Ermete OS.

## 1. 🕸️ Topologia & "God Nodes" (High Coupling)
- **`SystemEventBus` & `ProxyRegistry` (Service Locator Anti-Pattern):** Situati in `ermete-shell-rs/src/ipc/`, fungono da giganteschi colli di bottiglia. Tutto converge nel `SystemEventBus` e nel `ProxyRegistry`, che maschera le dipendenze forzando il casting dinamico via `as_any().downcast_ref()`. Questo causa un fortissimo accoppiamento globale.
- **`NetworkController` Monolitico:** Si occupa contemporaneamente di Wi-Fi, Ethernet, Mock states, connessioni D-Bus, e scansione sincrona del filesystem Linux, violando il Single Responsibility Principle.
- **Accoppiamento Semantico UI/IPC:** I listener in `ipc/system_proxies.rs` istanziano e manipolano direttamente widget GTK (es. `show_control_center_popover()`), legando indissolubilmente lo strato di trasporto (IPC) a quello di presentazione (Relm4/GTK).

## 2. 🧠 Errori Logici & I/O Bloccante
- **Micro-Stuttering (I/O Sincrono):** I metodi `get_live_state()` e `get_cached_network_status()` eseguono letture sincrone (es. `fs::read_to_string("/proc/meminfo")` e `fs::read_dir("/sys/class/net")`) direttamente sul thread GTK principale (all'interno di `TopbarInput::TickSecond`). Un banale lag del kernel gela l'intera interfaccia utente.
- **UI Reset Loop (File Watcher):** Il drag-and-drop in `desktop_widgets.rs` salva i dati sul file `widgets.json`. Il `FileMonitor` in ascolto su quel file ricarica istantaneamente la UI durante il trascinamento, distruggendo attivamente il widget che l'utente sta muovendo.
- **Wi-Fi Race Condition:** L'azione di accensione/spegnimento lancia `toggle_wifi()` e `set_wifi_powered(state)` in parallelo, invertendo due volte lo stato. In più, l'attivazione NetworkManager tenta di utilizzare `/` come ObjectPath, causando un fallimento garantito su sistemi multi-interfaccia.
- **Overhead D-Bus (Socket Leaks):** In `ermete-daemon-rs` (`bedrock.rs` e `voiceover.rs`), ogni singola chiamata (es. cambio volume) instanzia una nuova `zbus::Connection::session().await` invece di riutilizzarne una globale.

## 3. 🏛️ Discrepanze Architetturali
- **Immutabilità Violata (`bootc` vs `dnf`):** Il pacchetto `ermete-system-config.spec` inietta subdolamente `/etc/yum.repos.d/ermete-forge.repo` per abilitare aggiornamenti DNF rolling "live", andando in diretto conflitto con il paradigma OCI atomico/immutabile ostentato.
- **LD_PRELOAD Instabile (`ermete-scudo`):** `libscudo.so` viene forzato globalmente via `/etc/ld.so.preload`. Questo causa crash in demoni complessi (`greetd`, `ermete-llm`) e genera race condition durante la build dell'immagine container.
- **Demoni Ridondanti (Violazione Ponytail):** L'ecosistema è esploso in troppi micro-demoni Rust frammentati (`ermete-livepatch` vs `ermete-live-patcher`; funzioni sovrapposte tra `ermete-daemon-rs`, `xdg-desktop-portal-ermete` e `ermete-shell-rs`).
- **Build RPM Non-Ermetica (`ermete-tetragon`):** Viene usato `curl` nella fase `%build` del pacchetto RPM, annullando la riproducibilità offline.
- **SELinux Fantasma (`ermete-selinux`):** I moduli `.pp` vengono copiati ma mai attivati (manca `%post` compile), rendendo le policy inattive.
- **Collo di Bottiglia Computazionale (`opt-level = "z"`):** L'ossessione per il binary size impone un flag globale `"z"` che paralizza le performance dei servizi matematici pesanti come `ermete-ai-daemon` (Candle ML).

---
*Report redatto dallo sciame di auditing (Graph Topology Analyst, Logic & Redundancy Auditor, Architecture Critic).*
