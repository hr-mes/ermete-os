# 🧠 DIRETTIVA LAYER 1: Ermete OS (User Experience & Hardening)

## 🦅 Identità Architetturale del Layer 1
Sei nel repository `ermete os`. Il tuo unico obiettivo qui è costruire un ecosistema utente Wayland-First, focalizzato sulla performance estrema e sulla sicurezza paranoica, bilanciate **matematicamente** con l'usabilità (Zero Denial of Service).

**IL DOGMA DELLO SPAZIO UTENTE:** 
Non sei autorizzato a modificare il Kernel o i driver NVIDIA in questa directory. Qualsiasi software GUI va delegato a Flatpak.

---

## 🛠️ Tecniche OCI e Pattern di Sviluppo Avanzati

1. **Il Compromesso di Nix (Hack del Symlink)**: Per via dell'architettura Fedora Atomic, l'installazione di Nix collide con il rootfs. Utilizza l'hack comprovato del `rm -f /nix` seguito dallo spostamento dei binari, che rimane il pattern offline più sicuro e resiliente rispetto a layer di terze parti instabili.
2. **Ingegneria del Caching OCI**: Non distruggere `/var/cache/dnf` all'interno delle singole ricette bash. Usa `dnf clean all` solo alla fine dell'ultimo layer di cleanup.
3. **Build Transienti ("Full-Rust" Stack)**: L'OS usa Rust (Niri, Ironbar, Anyrun, Starship). Scarica release pre-compilate firmate o esegui la build da commit pinnati in un layer transiente, eliminando immediatamente lo stack di compilazione (gcc, cargo).
4. **Provisioning Resiliente (First-Boot)**: Il demone Flatpak (`Ermete-firstboot.service`) non deve mai bloccare il boot o generare loop fatali su reti captive. Usa un `curl` con timeout, controlli non bloccanti e flag di idempotenza su `/var/lib/`.

---

## 🔒 L'Equilibrio Tra Paranoia e UX (Regole Letali)

La sicurezza estrema non deve mai tradursi in un Denial of Service per l'utente. Applica questi dogmi correttivi:

*   **Evitare l'OOMD Nuke Trap**: Delega `systemd-oomd` ai default bilanciati di Fedora. Limitazioni troppo feroci (es. kill al 90% in 5s) causano il massacro dell'intero cgroup `user@.service`, annientando il server Wayland per banali picchi di memoria.
*   **Amnesia di Rete (Breakage Evitati)**: La MAC Randomization di NetworkManager deve essere settata su `stable` (non `random`) per sopravvivere ai portali Captive di hotel e reti filtrate. Il protocollo DNS-over-TLS (DoT) deve essere `opportunistic` per evitare la paralisi totale su reti locali che bloccano la porta 853.
*   **Cecità di Rete e Isolamento Flatpak**: Se adotti la `drop` zone su Firewalld, devi iniettare un'eccezione esplicita per `mdns` o azzererai la scoperta locale di device (Chromecast, Stampanti). Non usare override Flatpak distruttivi globali (come `--nosocket=x11`); gestisci le sandbox per-app.
*   **Precisione Privacy (`/etc/skel/`)**: Quando forzi permessi sicuri per lo skeleton dir (privacy da altri utenti), usa comandi granulari (`find` a 700 per dir, 600 per file) garantendo il flag di esecuzione (`+x`) ai file bash `.sh`, per non rompere i workflow clonati.
