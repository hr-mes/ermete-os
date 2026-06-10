# 🏗️ Progetto: Ermete Base NVIDIA (`ermete-base-nvidia`)

Questo documento delinea il piano ingegneristico definitivo per svincolare **Ermete OS** da maintainer di terze parti (come RakuOS), ottenendo il controllo del 100% della catena di approvvigionamento (Supply Chain). 

L'obiettivo è creare una repository parallela che funga da fondamenta per l'OS, compilando autonomamente il **Kernel CachyOS** e i **Driver NVIDIA proprietari**.

---

## 🎯 Obiettivi Architetturali
1. **Zero-Trust Supply Chain:** Nessun codice esterno pre-compilato. Il sistema di base sarà generato direttamente dai sorgenti Fedora e dai COPR ufficiali tramite le nostre GitHub Actions.
2. **Prestazioni Intatte:** Mantenimento dello scheduler `BORE` e delle ottimizzazioni `x86-64-v3` tipiche del Kernel CachyOS.
3. **Estrema Leggerezza:** Rimozione di qualsiasi pacchetto o libreria "bloat" (es. dipendenze GNOME/KDE) che RakuOS potrebbe aver incluso per la sua utenza, mantenendo solo il layer Wayland/EGL necessario per Niri.

---

## 🗺️ Roadmap di Implementazione (Day 2)

### Fase 1: Creazione dell'Infrastruttura (Repository)
Dovremo creare un nuovo repository GitHub chiamato `ermete-base` (es. `patapem/ermete-base`).
Questo repository avrà una struttura minimalista:
* `Containerfile`: Il manifesto primario per la sostituzione del Kernel e dei driver.
* `.github/workflows/build.yml`: La pipeline CI/CD per la compilazione settimanale automatica.

### Fase 2: Il `Containerfile` del Motore (Lo Stack)
Il file di build non si baserà più su RakuOS, ma eseguirà un'operazione chirurgica partendo da Fedora nuda (tramite gli strumenti di `ublue-os`):

1. **Il Punto di Partenza:** `FROM ghcr.io/ublue-os/base-main:latest` (Fedora atomica pura).
2. **Iniezione CachyOS:** 
   - Aggiunta del repository COPR `bieszczaders/kernel-cachyos`.
   - Esecuzione del comando di scambio nucleare: `rpm-ostree override replace kernel kernel-core kernel-modules --from repo=copr:copr.fedorainfracloud.org:bieszczaders:kernel-cachyos`.
3. **Compilazione NVIDIA (Akmods):** 
   - Scaricamento dei binari chiusi NVIDIA da RPMFusion.
   - Compilazione forzata dei moduli (`kmod`) specificatamente contro gli header del nuovo Kernel CachyOS appena installato.
4. **Ottimizzazione Firmware:** Installazione di `linux-firmware` aggiornato e spegnimento definitivo dei driver `nouveau`.

### Fase 3: La CI/CD Pipeline (GitHub Actions)
La compilazione di moduli Kernel e driver NVIDIA in un container richiede risorse elevate e logica specifica.
* **Schedulazione:** Creeremo un job in GitHub Actions (con cron timer) che compila la base ogni martedì.
* **Gestione degli Errori (Fallback):** Se un aggiornamento del Kernel CachyOS "rompe" la compatibilità con NVIDIA, la pipeline deve fallire silenziosamente avvisandoci via email, mantenendo intatta su GHCR l'ultima immagine `ermete-base-nvidia:latest` funzionante. In questo modo Ermete OS non si romperà mai.

### Fase 4: Integrazione in Ermete OS (Il Varo)
Una volta che la build automatica di `ermete-base` sarà completata e rilasciata con successo nei pacchetti GitHub (`ghcr.io/patapem/ermete-base-nvidia`), la migrazione finale sarà un'operazione da 1 riga di codice:
* Nel repository principale (`ermete`), cambieremo la riga del `Containerfile`:
  `FROM ghcr.io/rakuos/rakuos-base-nvidia:latest`
  ⬇️
  `FROM ghcr.io/patapem/ermete-base-nvidia:latest`

Da quel preciso momento, Ermete OS sarà un prodotto "Enterprise", sviluppato, controllato e validato dal primo all'ultimo byte unicamente da noi.

---

## 🛠️ Requisiti per Domani
Per iniziare i lavori domani, servirà:
1. Aver creato un nuovo repository GitHub vuoto (es. `ermete-base`).
2. Configurare i permessi del nuovo repo per permettere alle GitHub Actions di pubblicare su *GitHub Container Registry* (GHCR).
