# 🧠 DIRETTIVA LAYER 1: Ermete OS (User Experience & Hardening)

## 🦅 Identità Architetturale del Layer 1
Sei nel repository `ermete os`. Il tuo unico obiettivo è incapsulare l'immagine base del Layer 0 con un ecosistema utente Wayland-First puro, focalizzato sulla performance estrema, un'estetica premium (Catppuccin/Glassmorphism) e una sicurezza brutalmente paranoica.

**IL DOGMA DELLO SPAZIO UTENTE:** 
Non sei autorizzato a modificare Kernel, Initramfs o driver NVIDIA hardware. L'obiettivo matematico di questo modulo è lo strato di *Sessione Grafica* (Niri, Ironbar, Starship) e lo strato di *Applicazioni Condotte* (Nix e Flatpak).

---

## 🛠️ Tecniche OCI e Hack Architetturali

1. **Il Miracolo del Demone Nix su OSTree (Blind-Mount Bypass)**: Dato che il package manager Nix esige testardamente il controllo della root directory `/nix`, e dato che le immagini OCI annientano ogni directory `/var` statica al momento del deploy, la semplice installazione di Nix fallisce disastrosamente. L'eventuale mount di `/var/opt/nix` su `/nix` oscura tutto lo *store* iniziale.
   * **Soluzione e Procedura Obbligatoria**: Durante la build, installa Nix tramite DNF, estrai i file generati nel rootfs e "salvali" in `/usr/share/nix-initial-state/`. Al primissimo avvio fisico, un demone asincrono oneshot (`ermete-nix-restore.service`) intercetta il boot, attende il corretto isolamento in mount di `/var/opt/nix`, preleva chirurgicamente il backup dallo `/usr/share` e lo infonde sulla directory dinamica, garantendo un'operatività immutabile a tolleranza d'errore zero. I percorsi utente vengono iniettati tramite `profile.d` verso l'alias di mount `/nix/var/nix/profiles/default/...`.
2. **Asincronia First-Boot Resiliente (Provisioning Blind-Safe)**: Il demone Flatpak (`Ermete-firstboot.service`) applica installazioni massicce in background puro. Sfrutta probe `curl` con timeout rigidi per disinnescarsi elegantemente di fronte a Captive Portal o reti disconnesse, azzerando totalmente qualsiasi loop di latenza al boot. Applica marker di stato su `/var/lib/` per mantenere la sua esecuzione perfettamente idempotente.
3. **Build Transienti ("Full-Rust" Stack)**: L'OS vive del linguaggio Rust. Nonostante questo, è tabù che la toolchain Cargo, GCC o Cmake inquini l'artefatto finale. La compilazione di estensioni o strumenti avviene estraendo da tarball firmati con `tar --no-same-owner` per poi polverizzare ogni scoria compiler nello stesso layer OCI (strutturazione a transazione unica in RUN).

---

## 🔒 L'Equilibrio Tra Paranoia e UX (Regole Letali)

La sicurezza esagerata non deve mai collassare in un Denial of Service dell'Operatore Umano.
1. **Sincronia Niri-Systemd**: Nessuna utility di sessione o daemon background (`swaybg`, `ironbar`, `lxpolkit`) si impone al Wayland Compositor. Al contrario Niri agisce nativamente come portale verso DBus, esportando le proprie coordinate d'ambiente `WAYLAND_DISPLAY`, e lancia la `niri-session.target`. Da quel millisecondo, Systemd-User domina tutto.
2. **Isolamento Skel (Zero-Trust Privacy)**: `/etc/skel` applica logiche di protezione feudali. Ogni configurazione ereditata al momento della genesi utente è bloccata a livello filemode `0700` per i percorsi dir, isolando la privacy contro ispezioni laterali da terzi. L'esecuzione di script locali viene iniettata a monte con espliciti flag `chmod +x` che bypassano difetti di versioning di repository Host mal configurati.
3. **Cecità di Rete Controllata (Firewalld & NetworkManager)**:
   * **Firewalld**: Implementa un Drop Network radicale azzerando qualsiasi scoperta portscan locale o esterna. Parallelamente, trapana policy chirurgiche per il layer mDNS `5353/UDP` onde evitare di castrare i servizi Home/Domotica locali.
   * **NetworkManager**: Mantiene la Mac Randomization inchiodata su `stable` permettendo la compatibilità persistente coi DHCP server aziendali pur oscurando l'hardware originario. Eredita il DNS-over-TLS bloccato in `opportunistic` mitigando firewall ISP ottusi che deviano la porta 853, mantenendo l'infrastruttura di connettività assolutamente resiliente.
