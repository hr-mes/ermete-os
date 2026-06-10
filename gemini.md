# Gemini Context: Ermete OS

## 🦅 Informazioni sul Progetto e Filosofia
**Ermete OS** è un sistema operativo Linux "cloud-native", immutabile e atomico, basato su **Fedora** e sulle potenti tecnologie **bootc / Universal Blue**.
Non è una distribuzione classica. Segue rigorosamente un approccio **Infrastructure-as-Code (IaC)**: l'intero sistema operativo è costruito a livelli in un'immagine OCI, azzerando l'entropia del sistema locale dell'utente.
Gli obiettivi primari e non negoziabili del progetto sono: **Privacy Totale**, **Prestazioni Estreme (Zero-Bloat)** e **Affidabilità Infrangibile (Atomic Updates)**.

---

## 🛠️ Stack Tecnologico & Design Pattern
* **Base Image OCI:** `ghcr.io/rakuos/rakuos-base-nvidia:latest`. Garantisce out-of-the-box driver proprietari NVIDIA, evitando le storiche frammentazioni su Wayland.
* **Paradigma dei Pacchetti:** 
  * Il sistema base ospita unicamente pacchetti RPM installati in build-time. 
  * L'utente utilizzerà **esclusivamente Flatpak** per le GUI e **Homebrew** per la CLI (grazie ai preset di ublue-os).
* **The "Full-Rust" Stack:** L'intero livello interattivo (UX) è deliberatamente implementato in Rust per abbattere i memory leak e massimizzare i frame:
  * Compositor: **Niri** (Scrollable Tiling).
  * Status Bar: **Ironbar**.
  * Launcher: **Anyrun**.
  * Login Greeter: **Tuigreet**.
  * CLI Tools: `eza`, `bat`, `fd-find`, `ripgrep`, `nushell`, `starship`.
* **Zero-Bloat / High-Performance:** 
  * Boot asincrono fulmineo (demoni di network-wait disabilitati).
  * Nessun file di SWAP su disco; viene allocato dinamicamente lo spazio tramite **zram-generator** in RAM.
* **Privacy "Paranoica" (Privacy-First):** 
  * Lo skeleton di default per i nuovi utenti isola i file con permessi rigidi (700/600).
  * **Machine ID azzerato** ad ogni build per impedire la profilazione sui cloni.
  * **Coredump disabilitati** per evitare salvataggi non crittografati di memoria (che potrebbero contenere password o chiavi).
  * Firewall in ingresso (`firewalld`) abilitato di default.

---

## 📂 Struttura della Repository
1. `Containerfile`: Il manifesto primario che stratifica l'OS montando volumi temporanei per cache (`/var/cache/dnf`) ed eseguendo i moduli in ordine.
2. `build_files/recipes/*.sh`: Moduli Bash idempotenti che installano e preconfigurano i pacchetti RPM nel rootfs dell'immagine OCI.
3. `build_files/dot_config/`: Configurazione e dotfiles per popolare `/etc/skel`.
4. `Justfile`: Task runner per testare le build in locale (`just build`, `just run-vm-qcow2`).

---

## 🤖 INSTRUZIONI CRITICHE PER LO SVILUPPO (Per l'IA e gli Human Maintainers)
Chiunque apporti modifiche al progetto **deve** conformarsi rigidamente ai seguenti divieti e prassi:

1. **Mai usare `dnf install` standard:** Ogni nuovo pacchetto deve essere installato usando esplicitamente `dnf -y install --setopt=install_weak_deps=False <pacchetto>`. L'inserimento di "weak_deps" è considerato una violazione architetturale, poiché ingrassa l'immagine OCI senza reale necessità.
2. **Nessun salvataggio in directory persistenti (`/var`, `/home`):** Essendo il sistema un'immagine OCI per `bootc`, tutto ciò che non risiede in directory globali (`/usr`, `/etc`) non verrà distribuito o sarà sovrascritto. Per le configurazioni utente va usato SEMPRE `/etc/skel/`.
3. **Usa le Native Systemd Policies:** Non usare mai script empirici (es. `rc.local` o hack bash) per gestire servizi. Usa esclusivamente systemd presets (`/usr/lib/systemd/system-preset/99-Ermete.preset`) e unit files canonici in `/etc/systemd/`.
4. **Idempotenza Assoluta e Sicurezza:** Gli script in `recipes/*.sh` verranno iterati. Modifiche, aggiunte o creazioni di file devono usare flag sicuri (`mkdir -p`, e configurazioni `cat > file << EOF`).
5. **Non Distruggere la Build-Cache:** Non inserire comandi di cancellazione indiscriminata come `rm -rf /var/cache/dnf/*` in fase di cleanup. Il Containerfile gestisce quella directory come un bind-mount dell'host, per cui distruggerla penalizza le build locali. Basterà usare `dnf clean all`.
6. **Mantenere la Privacy Totale:** Qualsiasi demone, logger o tool diagnostico aggiunto non deve abilitare la telemetria e non deve immagazzinare dati esposti in chiaro se non strettamente legati al journal crittografato o al normale dmesg.
7. **Wayland-First:** È categoricamente vietato installare server X11 o tool incompatibili con il protocollo Wayland. Qualsiasi applicativo GUI aggiunto nel core dovrà girare nativamente su Niri.
