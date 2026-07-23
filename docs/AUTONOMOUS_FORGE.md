# Ermete OS: Autonomous Kernel Forge

Questo documento descrive l'architettura all'avanguardia progettata per l'auto-mantenimento del Kernel ibrido di Ermete OS. L'obiettivo primario di questa infrastruttura è garantire che il sistema operativo rimanga perennemente aggiornato con gli ultimi upstream (Fedora ARK) e le massime ottimizzazioni (CachyOS/Clear Linux), senza alcun intervento umano per la risoluzione dei conflitti C.

## 1. La Visione Architetturale

Mantenere un Kernel proprietario altamente competitivo (che rivaleggi con le Big-Tech in prestazioni, stabilità e sicurezza) richiede la fusione costante di patch esterne. Solitamente, quando una patch non si applica pulitamente (`git apply` o `patch` fallisce a causa di derive nel codice sorgente upstream), il processo di build si interrompe e richiede un Kernel Maintainer umano per risolvere il conflitto sintattico e logico.

L'**Autonomous Forge** elimina questa necessità. Trasforma la postazione di sviluppo in un'Intelligenza Artificiale attiva capace di manipolare, riparare e committare codice C in tempo reale.

## 2. Componenti del Sistema

L'infrastruttura si compone di due pilastri principali, isolati e asincroni:

### A. Il Demone AI (`ermete-agent`)
Un servizio `systemd` locale (`ermete-agent.service`) in ascolto sulla porta `8000`. 
- **Cuore Logico:** Utilizza un LLM ad alte prestazioni (es. `Qwen3-14B`) in esecuzione su hardware locale (via `llama-server`).
- **Endpoint:** Espone la rotta POST `/v1/kernel/resolve-conflict`.
- **Nessun Limite:** L'interfaccia di connessione Python (`urllib`) è configurata con `timeout=None`. Il modello ha tutto il tempo necessario (anche ore) per processare interi file sorgente C senza rischiare interruzioni della connessione di rete interna. In caso di crash fatale dell'LLM, il socket del sistema operativo garantisce un `ConnectionResetError` sicuro.

### B. Il Client Autonomo (`sync-forge.sh`)
Lo script bash eseguito nel repository `ermete-kernel-source`. 
Le sue responsabilità includono:
1. Sincronizzare la storia Git pura di Fedora ARK.
2. Sincronizzare il repository `kernel-patches` di CachyOS.
3. Tentare un *merge* standard (`patch -p1`). Se il merge ha successo, esegue il commit.
4. **Intercettazione Errore:** Se la patch fallisce, lo script applica forzatamente la patch per estrarre i file di scarto (`.rej`).
5. **Autoguarigione:** Impacchetta il codice C originale e il frammento scartato (`.rej`) in un payload JSON e lo invia al demone `ermete-agent`.
6. **Auto-Iniezione:** Ricevuta la risposta dall'LLM, lo script sovrascrive istantaneamente il file sorgente del Kernel con la soluzione generata, pulisce i `.rej` e committa il risultato nel ramo di rilascio.

## 3. Sicurezza ed Efficienza

- **Dry-Run & Isolamento:** Ogni tentativo di patching avviene su branch Git isolati (`sync/chimera-*`). Il ramo principale (`main`) non viene mai sporcato da esperimenti falliti.
- **Risoluzione Mirata:** Invece di inviare al modello l'intera codebase (impossibile a causa del limite di Token), il sistema invia solo l'intestazione della patch, il file target specifico che ha subito il conflitto, e il differenziale `.rej`. Questo focalizza l'attenzione dell'IA esclusivamente sul blocco logico danneggiato.
- **Fall-back Autonomo:** Se l'LLM, dopo tutti i tentativi, dovesse restituire codice incompilabile o non riuscire a rispondere (`HTTP 500`), lo script abortisce il flusso preservando l'integrità dell'infrastruttura e sollevando l'eccezione a livello umano (unica interazione manuale richiesta nell'intero ciclo di vita del Kernel).

---
*Progettato e implementato per massimizzare le prestazioni del Kernel senza scendere a compromessi.*
