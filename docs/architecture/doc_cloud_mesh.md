# 🌐 Network Architecture & Synchronization Specification: `ermete-mesh-sync` & `ermete-cloud-rs`

## 1. General Network Architecture

Ermete OS manages peer-to-peer (P2P) mesh connectivity, encrypted wire transport, and system-wide continuity synchronization (Universal Clipboard, Cloud Mount) through two dedicated Rust daemons:

1. **`ermete-mesh-sync`**: User-space mesh network daemon responsible for establishing and orchestrating encrypted WireGuard tunnels utilizing X25519 key exchange.
2. **`ermete-cloud-rs`**: Continuity daemon managing local P2P discovery (mDNS/UDP broadcast), encrypted universal clipboard streaming (TCP/Noise, Wayland `wl-clipboard` integration), and remote storage orchestration via FUSE (`rclone`).

```mermaid
graph TD
    subgraph DBus ["D-Bus Session & System Bus"]
        DBusMesh["org.ermete.MeshSync (/org/ermete/MeshSync)"]
        DBusCloudSync["os.ermete.CloudSync (/os/ermete/CloudSync)"]
        DBusCloud["os.ermete.Cloud (/os/ermete/Cloud)"]
    end

    subgraph MeshEngine ["ermete-mesh-sync (User-Space Mesh WG)"]
        WG_Engine["WireGuard Engine (boringtun 0.6)"]
        X25519_Keys["X25519 Keypair (x25519-dalek 2.0)"]
        UDP_WG["Listener UDP:51820 (Mesh WG Traffic)"]
        WG_Engine --- X25519_Keys
        WG_Engine --- UDP_WG
    end

    subgraph CloudEngine ["ermete-cloud-rs (Continuity & Sync)"]
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

## 2. In-Depth Component Analysis: `ermete-mesh-sync`

### 2.1 Dependencies & Module Specification
- **Crate Path:** `forge/specs/ermete-mesh-sync/ermete-mesh-sync-1.0.0/`
- **Language/Framework:** Rust 2021, Tokio 1.37 (full async runtime).
- **Allocation & Cryptography:**
  - `x25519-dalek = "=2.0.0-rc.3"`
  - `boringtun = "0.6"` (Cloudflare user-space WireGuard implementation)
  - `rand_core = "0.6"` (`OsRng`)
  - `zbus = "4.0"`

### 2.2 Cryptographic Algorithms & Key Management
- **Key Exchange (KEX):** Elliptic-curve **X25519** key exchange (Curve25519 Diffie-Hellman).
  - Generation of ephemeral private keys via hardware-backed CSPRNG (`EphemeralSecret::random_from_rng(OsRng)`).
  - Derivation of corresponding public keys (`PublicKey::from(&secret)`).
  - Encoding of public keys in **Base64** format for peer negotiation and Cloudflare WARP endpoint handshakes.
- **Symmetric Encryption & Tunneling (WireGuard Standard):**
  - Symmetric cipher: **ChaCha20-Poly1305** AEAD (handled by `boringtun`).
  - Hashing and MAC: **BLAKE2s**.

### 2.3 UDP Socket & Tunneling Details
- **UDP Socket:** Asynchronous non-blocking listener via `tokio::net::UdpSocket` bound to `0.0.0.0:51820`.
- **Routable Interface:** Interfaces with Linux TUN virtual devices (`wg-ermete`) wrapped by `boringtun::device::DeviceHandle` for user-space IP routing.
- **D-Bus Interface:**
  - Bus Name: `org.ermete.MeshSync` at `/org/ermete/MeshSync`
  - Methods:
    - `status() -> &str`: Queries daemon state (`"Mesh Sync is running (Async WireGuard)"`).
    - `get_public_key() -> String`: Exports node's public X25519 key.

---

## 3. In-Depth Component Analysis: `ermete-cloud-rs`

### 3.1 Components, Allocator & Systemd Sandboxing
- **Crate Path:** `forge/specs/ermete-cloud-rs/ermete-cloud-rs-1.0.0/`
- **Language/Framework:** Rust 2021, Tokio 1.36 (async full), `zbus` 4.4.0.
- **Global Allocator:** `mimalloc` 0.1 to eliminate heap fragmentation and accelerate network packet allocations.
- **Service Sandboxing (`ermete-cloud-rs.service`):**
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

### 3.2 Exposed D-Bus Interfaces (`zbus`)
1. **`os.ermete.CloudSync`** (`/os/ermete/CloudSync`):
   - `authenticate_oauth(provider: String, token: String) -> Result<String>`: OAuth token validation handler.
   - `mount_fuse(remote: String, mountpoint: String) -> Result<String>`: Spawns background process `rclone mount <remote> <mountpoint> --vfs-cache-mode full`.
2. **`os.ermete.Cloud`** (`/os/ermete/Cloud`):
   - `push_clipboard(content: String) -> Result<String>`: Broadcasts active clipboard content to verified network peers.

---

## 4. Network Protocol Specifications (`ermete-cloud-rs`)

### 4.1 Peer Discovery Protocol (UDP Broadcast)
- **Listening Port:** UDP `9090` (`0.0.0.0:9090`).
- **Broadcast Emitter:** Transmits broadcast packet every 5 seconds to IPv4 address `255.255.255.255:9090`.
- **Discovery Payload:** `ERMETE_HELLO` (UTF-8 magic string).
- **Peer Registry & TTL Eviction:**
  - Active nodes are stored in `Arc<Mutex<HashMap<String, Instant>>>`.
  - **Eviction Strategy:** To prevent memory leaks during long-running sessions under dynamic DHCP leases, stale peer cleanup is executed prior to clipboard transmission:
    ```rust
    p.retain(|_, time| time.elapsed() < Duration::from_secs(60));
    ```
    Peers undetected for >60 seconds are purged automatically from the registry.

### 4.2 Universal Clipboard Synchronization Protocol (TCP/Noise Protocol)
- **TCP Listening Port:** TCP `9091` (`0.0.0.0:9091`).
- **Maximum Payload Cap:** `1 MB` (`take(1024 * 1024)`).
- **4-Stage Zero-Trust Verification Pipeline:**
  1. **IP Verification (Untrusted IP Rejection):** Connected client IP on TCP 9091 is checked against `known_peers`. If the IP was not pre-validated via UDP Discovery (`ERMETE_HELLO`), the connection is instantly dropped.
  2. **Security Tunnel / Auth Check:** If the TLS/Noise session is unestablished and `auth_token` is missing (`None`), inbound payload is rejected (`TLS/Noise tunnel not established`).
  3. **Authentication Framing:**
     Transmitted payload must conform strictly to:
     ```text
     AUTH:<auth_token>\n<payload_content>
     ```
     The header line is parsed and matched against the receiver's configured `auth_token`.
  4. **Sanitization & Wayland Injection:**
     - Empty payloads or payloads containing null bytes (`\0`) are discarded.
     - Upon successful authentication, payload is piped directly to Wayland `wl-copy` stdin:
       ```rust
       tokio::process::Command::new("wl-copy")
           .stdin(std::process::Stdio::piped())
           .spawn()
       ```

---

## 5. Security & Network Port Matrix

| Crate | Protocol | Port / Transport | Cipher & Algorithms | Authentication Mechanism | Target Output |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **`ermete-mesh-sync`** | WireGuard Mesh | UDP `51820` | X25519 (x25519-dalek), ChaCha20-Poly1305, BLAKE2s | Ephemeral X25519 Key Pair Exchange | TUN device `wg-ermete` / Cloudflare WARP |
| **`ermete-cloud-rs`** | Peer Discovery | UDP `9090` (Broadcast) | Plaintext UTF-8 Magic String | IP Discovered (`ERMETE_HELLO`) | Memory Registry (`HashMap<IP, Instant>`) |
| **`ermete-cloud-rs`** | Universal Clipboard | TCP `9091` | Frame `AUTH:<token>\n<payload>` (TLS/Noise tunnel required) | Peer IP Verification + Auth Token Matching | Wayland Clipboard (`wl-copy` stdin) |
| **`ermete-cloud-rs`** | Cloud FUSE Mount | System IPC / Subprocess | OpenSSL / HTTPS via rclone | OAuth Tokens (`os.ermete.CloudSync`) | VFS Mount (`rclone mount --vfs-cache-mode full`) |
