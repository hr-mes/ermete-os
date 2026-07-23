# Adversarial Challenge Report: MAS Architecture (Iteration 14)

## Observation
I reviewed the updated MAS Architecture document at `/var/home/ermete/GEMINI/ermete/.gemini/plugins/ermete_team/README.md`. The design attempts to sandbox execution using `systemd-run --scope BindsTo` and enforce a 10MB quota via tmpfs. The source is seeded using `podman cp` after `podman run -d`. The extraction mechanism uses `podman cp` followed by `head -c 10M`.

## Logic Chain

### 1. Quota Bypass & Host Disk Exhaustion (Critical DoS)
**Observation**: The artifact extraction uses `podman cp "$CID:/workspace/artifact.json" "$TMP_DIR/out.json"`, followed by `head -c 10M`.
**Logic**: The `--mount type=tmpfs,tmpfs-size=10M` only limits the size of files *stored* on the tmpfs. It does not limit what `podman cp` streams out. If the untrusted payload executes `ln -s /dev/zero /workspace/artifact.json`, `podman cp` will stream infinite zeros from the container's `/dev/zero` to `$TMP_DIR/out.json`. `podman cp` does not natively enforce size limits during extraction, meaning it will run until the host's `/tmp` (or `/var/tmp`) partition is 100% full. The `head -c` command only runs *after* `podman cp` completes, which will be never. 
Similarly, an attacker can write a 100GB file to the container's unquoted overlay filesystem (e.g., `/var/tmp/huge`) and symlink `artifact.json` to it. `podman cp` will copy the entire 100GB to the host, completely bypassing the 10MB tmpfs quota.

### 2. Zombie Container Leak (The `systemd-run` vs `-d` mismatch)
**Observation**: The execution command is `systemd-run --user --scope --property=BindsTo=ermete-mas.service podman run -d ...`.
**Logic**: Using `podman run -d` places the container in the background. The client process exits almost immediately. When the client exits, the `systemd-run` scope becomes empty and systemd destroys it. The actual container process is spun up by Podman's `conmon` inside a *different* systemd scope (e.g., `libpod-$CID.scope`) which is NOT bound to `ermete-mas.service`. Therefore, if the host daemon crashes or is killed, the `BindsTo` property is completely useless, and the container will run forever as a zombie, silently consuming resources.

### 3. Container Startup Race Condition (TOCTOU)
**Observation**: The pipeline executes `podman run -d`, and then immediately after executes `podman cp /path/to/clone/. "$CID:/workspace/src/"`.
**Logic**: Because `podman run -d` starts the container execution asynchronously, the container's entrypoint will begin running *concurrently* with the host's `podman cp` operation. If the container payload attempts to compile or read the source code immediately, it will fail because `/workspace/src` will be empty or partially copied. This introduces a fatal TOCTOU race condition causing non-deterministic build failures.

## Caveats
- Since I am operating under CODE_ONLY constraints and human prompts time out, I verified these behaviors conceptually based on known `podman`, `systemd`, and Linux VFS semantics rather than empirical execution. The `podman cp` infinite stream behavior on `/dev/zero` is a well-known container extraction vulnerability.

## Conclusion
The architecture **FAILS** the security and stability stress test. 
1. **Quota Bypass**: The tmpfs quota is trivially bypassed via symlinks to `/dev/zero` or overlayfs, leading to Host Disk Exhaustion.
2. **Zombie Leak**: The `systemd-run BindsTo` strategy is totally broken due to the use of `podman run -d`, leading to persistent zombie containers.
3. **Race Condition**: The source seeding mechanism creates a TOCTOU race condition where the container executes before its source is injected.

## Verification Method
1. **Test Quota Bypass**: Run a container that creates a symlink `ln -s /dev/zero /workspace/artifact.json`, wait for it to exit, then run the host `podman cp` snippet. Watch the host disk fill up indefinitely.
2. **Test Zombie Leak**: Run the exact `systemd-run ... podman run -d` command, note the container is running. Then `systemctl stop ermete-mas.service` (or kill the daemon). The container will still be running.
3. **Fix Recommendation**: 
   - **For Quota**: Replace `podman cp` with `podman exec "$CID" cat /workspace/artifact.json | head -c 1048576 > "$TMP_FILE"`. This enforces the limit synchronously and avoids all symlink attacks.
   - **For Zombies**: Do NOT use `-d`. Run `podman run` in the foreground inside the `systemd-run` scope.
   - **For Race Condition**: Use `podman create`, then `podman cp` the source, then `podman start --attach`.
