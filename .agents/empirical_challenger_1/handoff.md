# Challenge Report

## Observation
I analyzed the architecture document at `/var/home/ermete/GEMINI/ermete/.gemini/plugins/ermete_team/README.md`. I specifically focused on the interaction protocol and the exact shell commands proposed for sandboxing and artifact ingestion.

I observed the following command constructions:
1. `systemd-run --user --scope --property=BindsTo=ermete-mas.service podman run -d --cidfile "$CID_FILE" --log-opt max-size=10m --mount type=tmpfs,destination=/workspace,tmpfs-size=10M agent-image`
2. `TMP_DIR=$(mktemp -d)`
   `podman cp "$CID:/workspace/artifact.json" "$TMP_DIR/out.json"`
   `head -c 1048576 "$TMP_DIR/out.json" > "$TMP_FILE"`
   `rm -rf "$TMP_DIR"`
3. `install -m 0644 "$TMP_FILE" "$FINAL_STATE_FILE" && podman rm -v -f "$CID"`

## Logic Chain
From an adversarial perspective, these constructions present several critical security flaws when interacting with an untrusted container environment.

1. **Arbitrary File Read (Host Data Exfiltration) via Symlinks**
   - **Mechanism**: The container user can create a symlink: `ln -s /etc/shadow /workspace/artifact.json`.
   - **Vulnerability**: `podman cp` copies symlinks as symlinks by default. When copied to the host, `$TMP_DIR/out.json` becomes a symlink to `/etc/shadow`.
   - **Exploitation**: The subsequent command `head -c 1048576 "$TMP_DIR/out.json" > "$TMP_FILE"` executes on the host. `head` will follow the symlink and read up to 1MB of the host's `/etc/shadow`. 
   - **Result**: The host's `/etc/shadow` is written to `$TMP_FILE` and then installed into the state directory, exposing it to the pipeline.

2. **Daemon DoS (Permanent Hang) via FIFOs**
   - **Mechanism**: The container user can create a named pipe: `mkfifo /workspace/artifact.json`.
   - **Vulnerability**: If `podman cp` copies this as a FIFO (or blocks reading it), the host daemon will hang. Specifically, if the FIFO is instantiated on the host at `$TMP_DIR/out.json`, the command `head -c 1048576 "$TMP_DIR/out.json"` will open the FIFO and block indefinitely waiting for a writer.
   - **Result**: The `monitor.sh` sequential daemon loop is permanently halted.

3. **Quota Bypass (Host Inode Exhaustion DoS)**
   - **Mechanism**: The tmpfs quota uses `--mount type=tmpfs,destination=/workspace,tmpfs-size=10M`.
   - **Vulnerability**: `tmpfs-size` restricts data volume but not file count (inodes). 
   - **Exploitation**: An attacker can create millions of 0-byte files or symlinks in `/workspace`.
   - **Result**: Exhausts host kernel memory (dentry/inode caches) or hits the host's global inode limits, causing a system-wide Denial of Service.

4. **Ineffective Zombie Container Prevention (Lifecycle Decoupling)**
   - **Mechanism**: `systemd-run --scope ... podman run -d ...`
   - **Vulnerability**: The `-d` (detach) flag means the podman client process forks the container to `conmon` and immediately exits.
   - **Result**: The `systemd-run` scope terminates immediately because its primary process (the podman client) has exited. The `BindsTo=ermete-mas.service` property becomes completely ineffective. If `ermete-mas.service` crashes after the container starts, the container continues running as an orphaned zombie.

## Caveats
- I did not test the actual `monitor.sh` script execution dynamically because my permissions for running host commands were constrained. The conclusions are derived conceptually based on standard Linux utilities, podman behaviors, and systemd semantics.
- `podman cp` behavior may vary slightly depending on the exact Podman version, but copying symlinks as symlinks is standard behavior across container runtimes (both Docker and Podman) to preserve intra-container symlink topology.

## Conclusion
**Overall Risk Assessment: CRITICAL**

The interaction protocol is highly vulnerable to container escape primitives focusing on file extraction (`podman cp`) and lifecycle management. The extraction flow assumes the file inside the container is a regular file.

**Recommended Mitigations:**
1. **Fix Artifact Extraction**: Validate file type *before* reading it on the host:
   ```bash
   podman cp ...
   if [ ! -f "$TMP_DIR/out.json" ] || [ -h "$TMP_DIR/out.json" ]; then
       # Alert and abort, it's not a regular file
   fi
   head ...
   ```
   *Alternative*: Stream directly from the container via `podman exec` or `podman run --log-driver`, avoiding `podman cp`.
2. **Fix Tmpfs Quotas**: Explicitly limit inodes to prevent host cache exhaustion.
   ```bash
   --mount type=tmpfs,destination=/workspace,tmpfs-size=10M,tmpfs-options=nr_inodes=1024
   ```
3. **Fix Lifecycle Management**: Do not use `podman run -d` with `systemd-run --scope`. Run the podman client in the foreground so the systemd scope maps to its lifespan.

## Verification Method
- **Symlink Read Test**: Inside a test container, run `ln -s /etc/passwd /workspace/artifact.json`, run the 3-step extraction, and inspect `$FINAL_STATE_FILE`.
- **FIFO Hang Test**: Inside a test container, run `mkfifo /workspace/artifact.json`, run the 3-step extraction, and observe `head` hanging.
- **Inode Exhaustion Test**: Inside a test container, run a loop creating files in `/workspace`. Monitor host memory and `df -i`.
- **Zombie Test**: Run the `systemd-run` command with `-d`, then stop `ermete-mas.service`. Check if `podman ps` still shows the container running.
