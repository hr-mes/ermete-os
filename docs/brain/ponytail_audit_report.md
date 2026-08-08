# Ponytail Audit: Ermete OS (Post-Refactoring)

L'architettura è stata disaccoppiata e il God Node abbattuto, ma questa transizione al paradigma 100% Actor-Model e Reattivo ha lasciato dietro di sé una scia di astrazioni, strutture e metodi sincroni ormai obsoleti. 

Non stiamo valutando la correttezza, ma l'**over-engineering** e il codice morto (Dead Code). Ecco i tagli chirurgici da effettuare per asciugare la codebase, ordinati per impatto:

- `[delete]` **Polling & State Store legacy**. L'intero `SettingsStateStore` e le struct di cache monolitiche (`SettingsState`) non servono più a nulla, la UI è completamente guidata dall'EventBus. `src/ipc/system_proxies.rs`
- `[delete]` **Metodi getter sincroni e command DBus morti**. La UI ora ascolta passivamente in ricezione, rendendo obsoleti decine di metodi getter: `get_volume()`, `get_brightness()`, `get_mpris_state()`, `get_battery_state()`, `get_last_player_command()`. Le enumerazioni DBus (`AudioCommand::GetVolume`, `DisplayCommand::GetBrightness`, `MprisCommand::RefreshMpris`) sono YAGNI. Da estirpare. `src/ipc/*.rs`
- `[shrink]` **Moduli di utilità hardware bypassati**. Funzioni come `get_ram_info()` e `get_cpu_load()` in `src/sys/stats.rs` non sono più invocate nel main loop (il publisher in background fa tutto). Possono essere ridotte o integrate diversamente. `src/sys/stats.rs`
- `[yagni]` **Motore di Animazione custom a molla**. Structs complesse come `SpringConfig` e `SpringAnimator` in `src/ui/anim.rs` (30+ righe) sono dead code o non implementate. Se GTK4/Relm4 gestiscono già le transizioni CSS, mantenere un physics engine in Rust è puro over-engineering. `src/ui/anim.rs`
- `[delete]` **Strutture Payload gonfiate**. I parametri `ssid`, `ip`, `gateway`, `dns` nel comando `ModifyWifi` di `NetworkController` non vengono mai letti; stiamo trasmettendo bytes fantasma. `src/ipc/network.rs`
- `[delete]` **Asset UI deprecati e CSS inline**. Costanti enormi come `POWERMENU_CSS` e `CLIPBOARD_CSS` sono morte e sepolte. L'inizializzatore `init_css()` ha preso il controllo centralizzato. `src/ui/*.rs`
- `[stdlib]` **Tipi vuoti per Eventi fittizi**. `SystemEvent::SystemMetricsUpdated` non viene mai generato. Se lo stream non lo usa, rimuoverlo. `src/ipc/system_proxies.rs`

**Net lines removable:** ~350 - 450 righe di codice (inclusi boilerplate DBus)
**Dependencies removable:** Nessuna dipendenza esterna da rimuovere, ma alleggerimento critico dell'eseguibile binario e dei tempi di compilazione.

*Nonostante i tagli da fare, il pattern architetturale (IPC + EventBus) è solido e in linea con le eccezioni consentite. Le dipendenze di sistema sono appropriate. Taglia il codice morto e spedisci.*
