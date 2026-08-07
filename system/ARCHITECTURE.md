# 🏛️ Ermete OS — System Architecture & 4 God Nodes Specification

> **Component:** `system/` Core Subsystem  
> **Status:** Production / Formal Verified (Kani + Clippy Strict)  
> **Updated:** August 2026  

---

## 🦅 Executive Architecture Overview

The `system` directory represents the runtime bedrock and OCI image build pipeline for **Ermete OS**. It unifies hardware security, post-quantum P2P networking, eBPF AI-driven kernel scheduling, and declarative application orchestration.

The system architecture is anchored around **4 God Nodes**:

```mermaid
graph TD
    subgraph God_Nodes ["🏛️ THE 4 GOD NODES OF ERMETE OS"]
        GN1["🧠 God Node 1: Kernel AI Scheduler\n(system/ermete-ebpf-sched)"]
        GN2["🛡️ God Node 2: Micro-Hypervisor Enclave\n(system/ermete-hypervisor-daemon)"]
        GN3["⚡ God Node 3: Mesh PQC\n(system/ermete-mesh-bus)"]
        GN4["🏛️ God Node 4: Flatpak Declarative Orchestrator\n(system/ermete-store)"]
    end

    subgraph Hardware_Kernel ["Ring-0 & Hardware Bedrock"]
        K=["Ermete Chimera Kernel + sched_ext"]
        ENCLAVE["AMD SEV-SNP / Intel TDX Memory"]
    end

    subgraph Networking_Security ["Wire-Speed PQC & Bus"]
        WG["WireGuard P2P Mesh"]
        ZBUS["ZBus Pure Rust IPC"]
    end

    GN1 -->|eBPF sys_execve & cgroup v2| K
    GN2 -->|KVM / vmm-sys-util| ENCLAVE
    GN3 -->|Dilithium5 + ML-KEM-1024| WG
    GN3 -->|Post-Quantum Bus| ZBUS
    GN4 -->|SLSA L4 + Cosign OCI| ZBUS
```

---

## 📑 Detailed God Node Specifications

### 1. 🧠 Kernel AI Scheduler (`ermete-ebpf-sched`)
*Source Location:* [`system/ermete-ebpf-sched`](file:///var/home/ermete/GEMINI/ermete-os/system/ermete-ebpf-sched)

- **Purpose:** Zero-latency kernel process prioritization bridging AI inference with Ring-0 scheduling.
- **Mechanism:**
  1. Intercepts process execution events (`sys_execve` tracepoints) via `aya` eBPF.
  2. Queries `AiDaemonBridge` on the local NPU for task classification and priority weight scoring.
  3. Dispatches task policies to `sched_ext` (Extensible Scheduler Class) with strict microsecond slice targets:
     - `RealtimeNpu`: **100 μs** target slice
     - `InteractiveUi`: **500 μs** target slice
     - `BatchCompute`: **5 ms** target slice
     - `IdleBackground`: **20 ms** target slice
  4. Dynamically adjusts Linux cgroup v2 `cpu.weight` parameters.

---

### 2. 🛡️ Micro-Hypervisor Enclave (`ermete-hypervisor-daemon`)
*Source Location:* [`system/ermete-hypervisor-daemon`](file:///var/home/ermete/GEMINI/ermete-os/system/ermete-hypervisor-daemon)

- **Purpose:** Zero-Trust Hardware Confidential Enclave Orchestration.
- **Mechanism:**
  1. Manages hardware-encrypted memory domains via AMD SEV-SNP and Intel TDX extensions.
  2. Leverages `vmm-sys-util`, KVM `ioctl` calls, and `ring` cryptographic hashes to instantiate isolated micro-enclaves.
  3. Verifies remote attestation and seals Ring-0 secrets against unauthorized host introspection.

---

### 3. ⚡ Mesh PQC (`ermete-mesh-bus`)
*Source Location:* [`system/ermete-mesh-bus`](file:///var/home/ermete/GEMINI/ermete-os/system/ermete-mesh-bus)

- **Purpose:** Post-Quantum Cryptography Wire-Speed Mesh Bus.
- **Mechanism:**
  1. Secures inter-device and daemon communication over WireGuard P2P tunnels using post-quantum primitives.
  2. **Key Encapsulation:** Employs **ML-KEM-1024 (Kyber1024)** for quantum-resistant session key negotiation.
  3. **Digital Signatures:** Uses **Dilithium5 (ML-DSA-87)** for quantum-proof payload authentication and identity verification.

---

### 4. 🏛️ Flatpak Declarative Orchestrator (`ermete-store`)
*Source Location:* [`system/ermete-store`](file:///var/home/ermete/GEMINI/ermete-os/system/ermete-store)

- **Purpose:** Declarative, Air-Gapped App Container Engine.
- **Mechanism:**
  1. Completely disconnects non-verified public repositories (`disconnect_flathub()`).
  2. Pulls immutable OCI application bundles strictly from GHCR SLSA Level 4 supply-chain registries (`ghcr.io/hr-mes/ermete-store`).
  3. Enforces Sigstore **Cosign** signature verification against public keys in `/etc/ermete/keys/cosign.pub` before granting execution or installation access.

---

## 🛠️ System Build & OCI Deployment Pipeline

1. **Layering Strategy:** 4-Tier Pyramid Caching inside `Containerfile` (`Tier 0` Bedrock hardware to `Tier 3` User Shell).
2. **Bedrock Diet:** Strips -1.1 GB of non-consumer firmware, build tools, and cache files.
3. **Immutability:** Bootable OCI container deployed and updated atomically via `bootc switch`.
