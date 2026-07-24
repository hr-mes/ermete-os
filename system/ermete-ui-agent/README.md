# Ermete UI Agent (Generative UI)

Questo demone Python è l'implementazione pratica del flag "Generative UI" in `labs.rs`.
Si interfaccia nativamente con il Widget Engine scritto in puro Rust (`ermete-shell-rs`).

## Architettura "Big Tech" a impatto zero:
- **Zero-cost Context**: Legge passivamente Wayland/Niri per determinare cosa stai facendo.
- **Trigger-based Generation**: Non interroga l'IA continuamente, lo fa SOLO quando il tuo contesto cambia (es. apri un nuovo programma, cambia la batteria).
- **Edge AI**: Disegnato per modelli leggerissimi come `llama3.2:1b` eseguiti in locale via Ollama. Nessun dato lascia la tua macchina.
- **Hot-Reload Sync**: Scrive `widgets.json` in silenzio. Il motore GTK4 in Rust intercetta il cambiamento a livello di filesystem e muove i widget senza riavvii e senza latenza.

## Dipendenze
- Python 3.10+
- `requests`
- `ollama` in locale con il modello scaricato (`ollama run llama3.2:1b`)
