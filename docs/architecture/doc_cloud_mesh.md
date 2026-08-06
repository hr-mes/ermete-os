# 🌐 Specifica di Architettura di Rete e Sincronizzazione: `ermete-mesh-sync` & `ermete-cloud-rs`

## 1. Architettura Generale di Rete (Network Architecture)

Ermete OS gestisce la connettività peer-to-peer (P2P), la crittografia di rete e la sincronizzazione di continuità (Universal Clipboard, Cloud Mount) tramite due daemon dedicati scritti in Rust:

1. **`ermete-mesh-sync`**: Daemon responsabile dell'istituzione e della gestione dei tunnel di rete mesh crittografati in user-space basati su WireGuard e X25519.
2. **`ermete-cloud-rs`**: Daemon di continuità per la sincronizzazione locale P2P (Discovery mDNS/UDP, Clipboard TCP/Noise, integrazione Wayland/`wl-clipboard` e orchestrazione FUSE tramite `rclone`).

```mermaid
graph TD
    subgraph D-Bus Session & System Bus
        DBusMesh["org.ermete.MeshSync (/org/ermete/MeshSync)"]
        DBusCloudSync["os.ermete.CloudSync (/os/ermete/CloudSync)"]
        DBusCloud["os.ermete.Cloud (/os/ermete/Cloud)"]
    end

    subgraph "ermete-mesh-sync (User-Space Mesh WG)"
        WG_Engine["WireGuard Engine (boringtun 0.6)"]
        X25519_Keys["X25519 Keypair (x25519-dalek 2.0)"]
        UDP_WG["Listener UDP:51820 (Mesh WG Traffic)"]
        WG_Engine --- X25519_Keys
        WG_Engine --- UDP_WG
    end

    subgraph "ermete-cloud-rs (Continuity & Sync)"
        Mimalloc["Global Allocator: mimalloc 0.1"]
        SyncEngine["SyncEngine Context"]
        UDP_Disc_Listen["UDP 9090 Receiver (ERMETE_HELLO)"]
        UDP_Disc_Send["UDP 255.255.255.255:9090 Announce"]
        TCP_Clip_Listen["TCP 9091 Receiver (AUTH + Payload)"]
        WlCopy["wl-clipboard (wl-copy stdin)"]

        SyncEngine --- UDP_Disc_Listen
        SyncEngine --- UDP_Disc_Send
        SyncEngine --- TCP_Clip_Listen
        TCP_Clip_Listen -->|Authenticated Payload| WlCopy
    end

    DBusMesh -->|Control & Status| WG_Engine
    DBusCloud -->|push_clipboard()| SyncEngine
    DBusCloudSync -->|mount_fuse()| RClone["rclone FUSE"]
```

---

## 2. Analisi Dettagliata: `ermete-mesh-sync`

### 2.1 Componenti e Dipendenze
- **Path Crate:** `forge/specs/ermete-mesh-sync/ermete-mesh-sync-1.0.0/`
- **Linguaggio/Framework:** Rust 2021, Tokio 1.37 (async full).
- **Allocazione e Criptografia:**
  - `x25519-dalek = "=2.0.0-rc.3"`
  - `boringtun = "0.6"` (Implementazione WireGuard in user-space sviluppata da Cloudflare)
  - `rand_core = "0.6"` (`OsRng`)
  - `zbus = "4.0"`

### 2.2 Algoritmi di Crittografia e Gestione Chiavi
- **Key Exchange (KEX):** Scambio di chiavi ellittiche **X25519** (Curve25519 Diffie-Hellman).
  - Generazione della chiave privata effimera tramite il PRNG hardware sicuro del sistema operativo (`EphemeralSecret::random_from_rng(OsRng)`).
  - Derivazione della chiave pubblica corrispondente (`PublicKey::from(&secret)`).
  - Encoding della chiave pubblica in formato **Base64** per la condivisione e negoziazione con nodi peer / endpoint Cloudflare WARP.
- **Symmetric Encryption & Tunneling (WireGuard Standard):**
  - Cifrario simmetrico: **ChaCha20-Poly1305** AEAD (gestito da `boringtun`).
  - Hashing e MAC: **BLAKE2s**.

### 2.3 Specifiche di Rete UDP e Tunneling
- **Socket UDP:** Asincrono non bloccante via `tokio::net::UdpSocket` in ascolto su `0.0.0.0:51820`.
- **Routable Interface:** Strutturato per interfacciarsi con dispositivi TUN Linux (`wg-ermete`) avvolti da `boringtun::device::DeviceHandle` per il routing utente dei pacchetti IP meshati.
- **Interfaccia D-Bus:**
  - Bus Name: `org.ermete.MeshSync` su `/org/ermete/MeshSync`
  - Metodi:
    - `status() -> &str`: Restituisce lo stato operativo (`"Mesh Sync is running (Async WireGuard)"`).
    - `get_public_key() -> String`: Esporta la chiave pubblica X25519 del nodo.

---

## 3. Analisi Dettagliata: `ermete-cloud-rs`

### 3.1 Componenti, Allocatore e Sandboxing Systemd
- **Path Crate:** `forge/specs/ermete-cloud-rs/ermete-cloud-rs-1.0.0/`
- **Linguaggio/Framework:** Rust 2021, Tokio 1.36 (async full), `zbus` 4.4.0.
- **Allocatore Globale:** `mimalloc` 0.1 per minimizzare la frammentazione della memoria e velocizzare le allocazioni di rete.
- **Hardening del Servizio (`ermete-cloud-rs.service`):**
  ```ini
  DynamicUser=yes
  ProtectSystem=strict
  ProtectHome=read-only
  NoNewPrivileges=true
  CPUWeight=50
  MemoryMax=512M
  OOMScoreAdjust=100
  Restart=always
  RestartSec=5s
  ```

### 3.2 Interfacce D-Bus esposte (`zbus`)
1. **`os.ermete.CloudSync`** (`/os/ermete/CloudSync`):
   - `authenticate_oauth(provider: String, token: String) -> Result<String>`: Gestione token OAuth.
   - `mount_fuse(remote: String, mountpoint: String) -> Result<String>`: Avvia in background un processo `rclone mount <remote> <mountpoint> --vfs-cache-mode full`.
2. **`os.ermete.Cloud`** (`/os/ermete/Cloud`):
   - `push_clipboard(content: String) -> Result<String>`: Inoltra il contenuto degli appunti ai peer fidati scoperti in rete.

---

## 4. Specifiche dei Protocolli di Rete (`ermete-cloud-rs`)

### 4.1 Peer Discovery Protocol (UDP Broadcast)
- **Porta di ascolto:** UDP `9090` (`0.0.0.0:9090`).
- **Broadcast Emitter:** Invia ogni 5 secondi un pacchetto di broadcast all'indirizzo IPv4 `255.255.255.255:9090`.
- **Payload di Discovery:** `ERMETE_HELLO` (stringa UTF-8).
- **Gestione dello Stato dei Peer e TTL Eviction:**
  - I nodi attivi sono salvati in una struttura dati `Arc<Mutex<HashMap<String, Instant>>>`.
  - **Eviction Strategy:** Per prevenire leak di memoria durante sessioni prolungate con lease DHCP dinamici, prima di ogni invio della clipboard viene eseguita la pulizia dei peer inattivi:
    ```rust
    p.retain(|_, time| time.elapsed() < Duration::from_secs(60));
    ```
    I peer non visti negli ultimi 60 secondi vengono rimossi automaticamente dalla mappa.

### 4.2 Universal Clipboard Synchronization Protocol (TCP/Noise Protocol)
- **Porta di ascolto TCP:** TCP `9091` (`0.0.0.0:9091`).
- **Dimensione Massima Payload:** `1 MB` (`take(1024 * 1024)`).
- **Flusso di Verifica e Sicurezza a 4 Livelli:**
  1. **Verifica IP (Untrusted IP Rejection):** L'indirizzo IP del client connesso su TCP 9091 viene verificato contro la mappa `known_peers`. Se l'IP non è stato precedentemente convalidato tramite UDP Discovery (`ERMETE_HELLO`), la connessione viene immediatamente rifiutata.
  2. **Security Tunnel / Auth Check:** Se la sessione sicura TLS/Noise non è stabilita e la chiave `auth_token` è assente (`None`), l'inbound clipboard viene scartato con un avviso di sicurezza (`TLS/Noise tunnel not established`).
  3. **Header di Autenticazione:**
     Il payload trasmesso via TCP deve rispettare il formato:
     ```text
     AUTH:<auth_token>\n<payload_content>
     ```
     La prima riga viene parsata e confrontata con l'`auth_token` configurato nel nodo ricevente.
  4. **Sanitizzazione Input e Iniezione Wayland:**
     - Payload vuoti o contenenti caratteri nulli (`\0`) vengono scartati.
     - Se l'autenticazione ha esito positivo, il payload viene inoltrato via pipe `stdin` al comando Wayland `wl-copy`:
       ```rust
       tokio::process::Command::new("wl-copy")
           .stdin(std::process::Stdio::piped())
           .spawn()
       ```

---

## 5. Matrice Riassuntiva della Sicurezza e Porte Rete

| Crate | Protocollo | Porta / Transport | Algoritmi & Cifratura | Meccanismo di Autenticazione | Target Output |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`ermete-mesh-sync`** | WireGuard Mesh | UDP `51820` | X25519 (x25519-dalek), ChaCha20-Poly1305, BLAKE2s | Ephemeral X25519 Key Pair Exchange | TUN device `wg-ermete` / Cloudflare WARP |
| **`ermete-cloud-rs`** | Peer Discovery | UDP `9090` (Broadcast) | Plaintext UTF-8 Magic String | IP Discovered (`ERMETE_HELLO`) | Memory Registry (`HashMap<IP, Instant>`) |
| **`ermete-cloud-rs`** | Universal Clipboard | TCP `9091` | Frame `AUTH:<token>\n<payload>` (TLS/Noise tunnel required) | Peer IP Verification + Auth Token Matching | Wayland Clipboard (`wl-copy` stdin) |
| **`ermete-cloud-rs`** | Cloud FUSE Mount | System IPC / Subprocess | OpenSSL / HTTPS via rclone | OAuth Tokens (`os.ermete.CloudSync`) | VFS Mount (`rclone mount --vfs-cache-mode full`) |
