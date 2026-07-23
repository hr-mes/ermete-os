# Ermete OS Infrastructure Audit Report

## 1. Secrets & Defaults
- **Hardcoded Cryptographic Salt**: In `ermete-daemon-rs/src/secret_enroller.rs`, the HKDF key derivation uses a hardcoded salt (`b"ermete-secret-enroller-v3"`). This weakens the cryptographic strength of the generated key, as the salt should ideally be randomly generated and stored alongside the ciphertext.
- **Plaintext Passwords over IPC**: In `ermete-daemon-rs/src/network.rs`, network passwords (like for enterprise WPA-EAP) are handled as plain strings and passed in the clear via DBus dictionaries.

## 2. Service Misconfigurations
- **Missing Sandboxing for Root Daemons**: The systemd services for `ermete-cloud-rs.service` and `ermete-lvfs-rs.service` are executed as root DBus services. They lack modern systemd security sandboxing features such as `DynamicUser=yes`, `ProtectSystem=strict`, `ProtectHome=read-only`, and `NoNewPrivileges=true`. This unnecessarily broadens their attack surface.
- **Infinite Restart Loops**: Both `ermete-cloud-rs.service` and `ermete-lvfs-rs.service` define `Restart=always` but fail to specify a `RestartSec=` or start limit bounding. If these daemons crash immediately upon startup, they will enter an infinite, rapid restart loop that can peg CPU utilization and flood the journal logs.

## 3. Build & QA Vulnerabilities
- **Unverified Remote Downloads (Missing Checksums)**: The script `ermete-kernel/prepare-chimera.sh` uses `curl -sLo` to download `fedora-nvidia.repo` directly into the build environment without validating a SHA256 checksum, opening the door to man-in-the-middle (MITM) attacks or upstream compromises.
- **Brittle Idempotency Hashing**: The `scripts/check_idempotency.sh` script relies on a naive loop (`find | sort | while read; cat ... | sha256sum`) to compute content hashes. This approach is unsafe for filenames containing spaces or special characters, and it fails to account for file permissions or ownership changes that might impact the resulting OS image.
