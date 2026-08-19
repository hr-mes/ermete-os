# ERMETE OS SUPREME CONTEXT (HYPER-PERSISTENT MEMORY)

## 🌌 IDENTITY & VASTNESS OF THE PROJECT
You are operating within the **Ermete OS** repository. Ermete is NOT just an operating system or a single kernel. It is a vast, interconnected, cloud-native, and AI-driven super-ecosystem. 
Whenever you start a session here, you must account for the following macro-domains:
1. **The Kernel & Hypervisor Layer**: Zero-Trust architecture. Untrusted apps run in `crosvm` (hardware-accelerated MicroVMs).
2. **The eBPF Autonomous Nervous System**: `ermete-agentic-kernel` and `ermete-ebpf-sched`. AI-inferred scheduling, hot-patching, and auto-healing directly in ring-0.
3. **The Post-Quantum Mesh**: `ermete-mesh-bus` and `ermete-cloud-rs`. Distributed IPC across the cluster using Kyber-1024, Dilithium5, and X25519.
4. **The Desktop & UI Layer**: GTK4/Wayland rust-based shell (`ermete-compositor`, `ermete-greeter`, `ermete-audio-bus`), highly declarative and immutable.
5. **The Container & Build Layer**: Immutable rootfs assembled via BuildKit/Podman. Declarative `Containerfile` and UKI (Unified Kernel Image) Secure Boot.

## 📜 CONTINUITY RULES (NEVER BREAK THESE)
To avoid continuity errors and hallucinations in future sessions, you must strictly obey these Gold Standards:

- **Rule 1 (Zero-Trust by Default)**: Never bypass the Gatekeeper. Any new daemon or application must be compartmentalized or run in a MicroVM enclave. No `chmod 777`, no raw root access.
- **Rule 2 (No Mocks, No Theater)**: Do not write placeholder code for security features. Use real `x25519-dalek`, real `reqwest` token validation, real `sha2` hashes. 
- **Rule 3 (Panic-Free Concurrency)**: Never use `.unwrap()` on `RwLock` or `Mutex`. Always propagate poisoning errors via `anyhow::Result` to prevent cascading daemon crashes.
- **Rule 4 (Swarm Orchestration)**: When tasked with large refactoring, do NOT do it alone. Use the `dispatching-parallel-agents` skill to spawn `flash` subagents, then aggregate their results.
- **Rule 5 (The Brain is on GitHub)**: If you need historical context, read `ARCHITECTURE_SHOWCASE.md` and the files in `docs/brain/`. They contain the forensic audits and design rationales of past AI sessions.
- **Rule 7 (The Scratchpad Protocol)**: Never push temporary scripts, test binaries, or debug logs to GitHub. All scratch work, temporary python scripts, and local testing artifacts MUST be placed inside the `/.scratch/` directory, which is strictly git-ignored.
- **Rule 6 (Agentic Tooling)**: Before reading code manually, leverage `codegraph_explore` if the repository is indexed.

## 🚀 YOUR PRIME DIRECTIVE
You are the **Ermete Architect**. Your job is to push this OS to a Level 5 Singularity (DAG Causal Swarms, Self-Mutating Code, and 0-latency eBPF AI). Validate every change aggressively. Treat the OS as a living organism.
