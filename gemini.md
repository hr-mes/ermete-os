# 🧠 DIRETTIVA LAYER 1: Ermete OS (User Experience & Hardening)

## 🦅 Identità Architetturale del Layer 1
Sei nel repository `ermete os`. Il tuo unico obiettivo qui è costruire un ecosistema utente Wayland-First, focalizzato sulla performance estrema e sulla sicurezza paranoica, partendo dalla base inespugnabile di `ermete-base-nvidia`.

**IL DOGMA DELLO SPAZIO UTENTE:** 
Non sei autorizzato a modificare il Kernel CachyOS o i driver NVIDIA in questa directory. Qualsiasi software GUI va delegato a Flatpak al First-Boot. Nessuna applicazione grafica (GUI) può esistere nei layer RPM dell'OS.

---

## 🛠️ Tecniche OCI e Pattern di Sviluppo Avanzati

Ogni modifica che effettui alle `recipes/*.sh` o al `Containerfile` deve rispettare questi teoremi:

1. **Ingegneria del Caching OCI**: Le ricette Bash sono eseguite in sequenza nel `Containerfile`. Non distruggere mai indiscriminatamente `/var/cache/dnf` all'interno delle ricette, poiché è un bind-mount dell'host vitale per accelerare le build locali. Usa `dnf clean all` solo alla fine dell'ultimo layer di cleanup.
2. **Build Transienti per il "Full-Rust" Stack**: L'OS è dominato da tool Rust (Niri, Ironbar, Anyrun, Tuigreet, Starship). Quando compili dai sorgenti via `cargo install`, sei obbligato a installare le dipendenze (gcc, gtk-devel, clang), completare la build, muovere il binario in `/usr/bin/` e **rimuovere (dnf remove) l'intero stack di compilazione nello stesso script**. L'immagine atomica finale non deve mai ospitare compilatori.
3. **Provisioning Resiliente (First-Boot)**: Il demone di installazione Flatpak al primo avvio (`Ermete-firstboot.service`) non deve bloccare il boot. Usa sempre override sicuri (`--nosocket=x11`, `--nofilesystem=home`) e scrivi loop di rete intelligenti (che prevengano falsi positivi su captive portal, verificando `NetworkManager` prima di eseguire blind curl).
4. **Auto-Ripristino e Greenboot**: Se aggiungi un servizio critico, aggiungi un check `greenboot`. Non testare mai la rete con ping su IP esterni (potrebbero fallire su voli o reti isolate causando rollback indesiderati); controlla solo lo stato del demone locale (`systemctl is-active`).

---

## 🔒 Tuning del Sistema (Il Livello Estremo)

Sei il custode del performance tuning. Rispetta queste baseline:
*   **Networking Ottimale**: Assicurati che il kernel sia spinto verso `bbr` e `fq_pie` per la minima latenza TCP.
*   **ZRAM Aggressiva**: Applica la ZRAM tramite `zram-generator` forzando la compressione `zstd` e permettendo uno `zram-fraction` del 100%. Assicurati che il kernel preferisca fortemente il memory offload prima del collasso (es. `vm.swappiness=150`).
*   **Protezione Wayland (OOMD)**: Il demone `systemd-oomd` deve essere impostato con soglie feroci (90%) per assassinare i processi utente *prima* che il compositor (Niri) si blocchi, garantendo frame-rate perpetui.
*   **Privacy dell'Utente (`/etc/skel/`)**: Quando trasferisci file di configurazione `dot_config` nel template skeleton, sigilla la directory assicurandoti che abbia permessi `chown root:root` e `chmod -R go-rwx`. Nessun altro utente del sistema deve poter leggere configurazioni altrui.
