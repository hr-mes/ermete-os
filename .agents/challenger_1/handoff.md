# Empirical Challenge Report: Architecture Verification

## Observation

I empirically tested the 5 architecture security fixes detailed in `/var/home/ermete/GEMINI/ermete/.gemini/plugins/ermete_team/README.md`. I discovered 3 critical bypasses where the proposed bash implementations fail to prevent the vulnerabilities.

1. **Trap Failure DoS Bypass (Subuid Mapping)**: Tested the fix `chmod -R u+rwX "$TMP_WORKSPACE"` with rootless podman. The `chmod` failed with "Operation not permitted", and the subsequent `rm -rf` failed, leaving the workspace leaked.
2. **Zombie Container Bypass (Systemd Cgroup Escape)**: Tested the fix `systemd-run --user --scope podman run`. When the caller was killed with SIGKILL (or when the caller's systemd service was stopped), the systemd scope did NOT terminate. The container escaped the daemon's cgroup and continued running indefinitely as a zombie.
3. **FIFO Deadlock (Process Control Bypass)**: Tested the atomic file reading pipeline. Created a named pipe (`mkfifo`) as the artifact file. The `realpath` prefix check passed, but when the daemon attempted to read the artifact, the read process blocked indefinitely, freezing the entire monitoring loop.

## Logic Chain

1. **Trap Failure**: Rootless podman uses user namespaces. If a container creates a file owned by container `root` (UID 0), it maps to the host user (UID 1000), allowing `chmod`. However, if the container creates a file owned by a non-root container user (e.g., UID 1), it maps to a host subuid (e.g., `524288`). The host user running the `trap` does not own the file, causing `chmod u+rwX` to fail. If the attacker also sets the permissions to `000`, the host's `rm -rf` fails, allowing a malicious repository to exhaust host disk space.
2. **Zombie Containers**: `systemd-run --scope` creates a transient scope unit, but it explicitly does **not** tie the scope's lifecycle to the caller's process. The scope is placed outside the caller's cgroup. If the daemon is killed (SIGKILL, OOM) or its systemd service is restarted, systemd terminates the daemon's cgroup but ignores the scope. The container processes become orphaned zombies.
3. **FIFO Deadlock**: The architecture specifies a `realpath` prefix check to prevent Symlink Attacks, and `mv -T` to prevent Directory Overwrite Evasion. It misses validating the file type. A FIFO is neither a symlink nor a directory. When the synchronous daemon attempts to read the FIFO, the syscall blocks waiting for a writer, deadlocking the "Process Control & Deadlock Prevention" mechanism.

## Caveats
I tested these using podman rootless on Fedora with default configurations. Behavior of subuid mapping is standard across podman rootless setups.

## Conclusion

**VERDICT: VETO.** The architectural specifications for the security fixes are technically flawed and empirically bypassable. 

Required fixes:
1. **Trap Failure**: Must use podman's unshare utility (`podman unshare rm -rf "$TMP_WORKSPACE"`) to execute the deletion as the namespace root, bypassing subuid ownership issues.
2. **Zombie Containers**: Must abandon `--scope` and either use a transient service that ties to the caller (`--pty`), or rely on podman's built-in `--cidfile` with robust background cleanup, or use `StopWhenUnneeded=yes` property binding.
3. **FIFO Deadlock**: The daemon must explicitly verify the artifact is a regular file (`[ -f "$resolved" ]`) before reading or moving it.

## Verification Method

Run the following commands to independently verify the bypasses:

**1. Trap Failure (Subuid)**
```bash
WORKSPACE=$(mktemp -d)
podman run --rm -v "$WORKSPACE:$WORKSPACE:Z" alpine sh -c "mkdir \$WORKSPACE/dir && chown -R 1:1 \$WORKSPACE/dir && chmod -R 000 \$WORKSPACE/dir"
chmod -R u+rwX "$WORKSPACE"  # FAILS
rm -rf "$WORKSPACE"          # FAILS
```

**2. Zombie Containers (Cgroup Escape)**
```bash
systemd-run --user --scope podman run --rm alpine sleep 1000 &
PID=$!
sleep 5
kill -9 $PID
ps -ef | grep "[s]leep 1000" # STILL RUNNING
```

**3. FIFO Deadlock**
```bash
WORKSPACE=$(mktemp -d)
podman run --rm -v "$WORKSPACE:$WORKSPACE:Z" alpine sh -c "mkfifo \$WORKSPACE/artifact.json"
cat "$WORKSPACE/artifact.json" # HANGS FOREVER
```
