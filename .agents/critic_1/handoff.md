# Challenge Summary

**Overall risk assessment**: CRITICAL

The architecture proposed in `README.md` introduces fatal logical flaws that break its own pipeline and critical vulnerabilities that allow Denial of Service (DoS) and Zombie Container Escape. The pipeline will fail on every successful run due to the volatile nature of tmpfs mounts, and the `BindsTo` zombie mitigation is entirely bypassed.

VERDICT: **FAIL**

## Challenges

### 1. [CRITICAL] Challenge 1: Tmpfs Artifact Destruction (Self-DoS / Data Loss)

- **Assumption challenged**: The host can extract the state artifact from a tmpfs volume using `podman cp` after `podman wait` blockingly waits for the container to exit.
- **Attack scenario**: Not even an attack—this is a guaranteed logic failure. The pipeline mounts `/workspace` as a `tmpfs` volume (`--mount type=tmpfs,destination=/workspace`). A tmpfs is tied to the container's active mount namespace. When the container stops, the tmpfs is destroyed. By executing `podman wait` *before* `podman cp`, the host attempts to extract `artifact.json` from a stopped container's empty rootfs overlay.
- **Blast radius**: `podman cp` will consistently throw a "file not found" error. The artifact is permanently lost. The `mv -T "$TMP_FILE"` operation will either fail or install an empty file, breaking the split-entry pipeline completely.
- **Mitigation**: Instead of `podman wait`, use a blocking pipe (`podman exec` or `podman logs`) while the container is still running, or use a bound host directory with restricted permissions rather than an internal tmpfs. Alternatively, the container itself could stream the artifact over an explicit named pipe/socket prior to exiting.

### 2. [HIGH] Challenge 2: Zombie Container Escape (BindsTo Bypass)

- **Assumption challenged**: Systemd's `BindsTo=ermete-mas.service` dependency will kill zombie containers if the daemon crashes or is stopped.
- **Attack scenario**: The README explicitly mandates: "Strictly REMOVE all references to `systemd-run --scope`" and uses raw `podman run -d`. When `podman run -d` is executed, the container runtime (conmon) typically forks into the Podman daemon's cgroup or a default `machine.slice`. It completely escapes the `ermete-mas.service` systemd cgroup. Because the container is no longer a systemd transient unit, `BindsTo` has absolutely no effect. If `monitor.sh` crashes (e.g., OOM kill, `SIGKILL`), systemd cleans up the script but leaves the detached container running as an untracked zombie.
- **Blast radius**: An attacker intentionally hanging the container can accumulate detached zombie containers if the daemon restarts, bypassing the very Gen-8 mitigation designed to stop them.
- **Mitigation**: Revert to using `systemd-run --scope --user` so the container process remains strictly bound to a systemd slice that can enforce `BindsTo` lifecycle termination, or enforce `--cgroup-parent`.

### 3. [HIGH] Challenge 3: Host Resource Exhaustion DoS (Memory, PIDs, Logs)

- **Assumption challenged**: Podman's `--timeout 300` flag alone guarantees a "hard execution bound" against DoS.
- **Attack scenario**: While the runtime is bounded by 300 seconds, the resource consumption *during* those 300 seconds is completely unbounded.
  1. **Log Exhaustion**: `podman run -d` captures stdout/stderr. An attacker can write gigabytes to stdout, exhausting the host's `/var/lib/containers` disk space.
  2. **PID/Memory Exhaustion**: The attacker can execute a fork bomb or a fast memory allocation loop. Without `--memory` or `--pids-limit`, the host kernel will OOM or run out of PIDs, locking up the host system before the 300 seconds elapse.
- **Blast radius**: Complete host system freeze or Disk/OOM DoS within milliseconds.
- **Mitigation**: Add strict limits to `podman run`: `--memory=1G`, `--pids-limit=256`, and `--log-opt max-size=10m`.

### 4. [MEDIUM] Challenge 4: Stale Lockfile Denial of Service

- **Assumption challenged**: `integration/monitor.lock` ensures exclusive execution without risking permanent lockouts.
- **Attack scenario**: If the daemon loop exits abruptly (power failure, `kill -9`, kernel panic) bypassing the bash `trap`, the `monitor.lock` file is permanently left on disk.
- **Blast radius**: When the daemon restarts, it cannot acquire the lock. It permanently refuses to run, causing a persistent Denial of Service requiring manual sysadmin intervention.
- **Mitigation**: Use `flock` on a file descriptor (e.g. `exec 200>monitor.lock; flock -n 200`) instead of simple file presence checks, as kernel file locks are automatically released upon process termination.

## Verification Method

1. **Verify Tmpfs Destruction**:
   ```bash
   # Run a detached container with a tmpfs, write a file, wait for it to exit, then attempt to copy:
   podman run -d --name tmp_test --mount type=tmpfs,destination=/workspace alpine sh -c 'echo "data" > /workspace/artifact.json'
   podman wait tmp_test
   podman cp tmp_test:/workspace/artifact.json -  # Expected: Error: file not found
   ```

2. **Verify Zombie Cgroup Escape**:
   ```bash
   # Create a systemd unit that runs `podman run -d alpine sleep 1000`.
   # Stop the systemd unit.
   # Check `podman ps` — the container will still be running.
   ```
