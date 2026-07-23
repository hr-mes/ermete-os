## Challenge Summary

**Overall risk assessment**: CRITICAL

The architecture described in `README.md` contains multiple severe flaws. Despite the claims of "Generation-8 Security Patches", "DOS protection", and "Zombie Container Prevention", the documented bash snippets actively introduce infinite execution loops, disk exhaustion vulnerabilities, host arbitrary file reads via symlinks, and resource leaks.

**Verdict:** FAIL

## Challenges

### [Critical] Zombie Container Bug (Infinite Execution DoS)

- **Assumption challenged**: The architecture assumes that containers will eventually terminate and that `podman rm -f -t 0` provides execution timeouts.
- **Attack scenario**: The daemon uses a strictly sequential, blocking execution model: `podman start "$CID" && podman wait "$CID"`. There is absolutely no timeout applied to the container execution (the `-t 0` flag only applies to the grace period during `podman rm`). A malicious or broken repository can include a build script that loops infinitely (`while true; sleep 1; done`).
- **Blast radius**: `podman wait` will block indefinitely. Because `monitor.sh` is single-threaded and blocking, this causes the entire pipeline to stall permanently. The trap is never reached.
- **Mitigation**: Use `timeout` on the host side (e.g., `timeout 600 podman wait "$CID"`) or use `podman run --timeout` instead of the manual create/start/wait pattern.

### [Critical] Disk Exhaustion Limit Bypass

- **Assumption challenged**: The architecture assumes that checking the artifact file size on the host prevents disk exhaustion and "TAR corruption".
- **Attack scenario**: The size check `if [ "$(stat -c%s "$SAFE_TMP_DIR/out.json")" -le 1048576 ]` occurs **after** the file has been fully extracted to the host via `podman cp`. The container workspace tmpfs allows up to 4GB. An attacker generates a 4GB `artifact.json` file. `podman cp` streams all 4GB to the host's `/tmp` partition.
- **Blast radius**: The host's `/tmp` directory is rapidly exhausted, leading to system-wide Denial of Service, affecting other services and potentially crashing the host before the file is even checked or deleted.
- **Mitigation**: Limit the file size during extraction. Instead of `podman cp`, use `podman exec "$CID" cat /workspace/artifact.json | head -c 1048576 > "$SAFE_TMP_DIR/out.json"` and verify it wasn't truncated.

### [Critical] Symlink Arbitrary File Read (Information Disclosure)

- **Assumption challenged**: The architecture assumes `artifact.json` is a regular file and `stat -c%s` securely calculates its payload size.
- **Attack scenario**: `podman cp` preserves symlinks. A malicious container creates a symlink: `ln -s /etc/shadow /workspace/artifact.json`. `podman cp` copies the symlink to the host. `stat -c%s` without `-L` returns the length of the symlink itself (e.g., 11 bytes), bypassing the size check. `mv -T` moves the symlink to `integration/state/final_artifact.json`.
- **Blast radius**: The next agent or the host daemon will read `final_artifact.json`, inadvertently reading the host's `/etc/shadow` file (or other sensitive host files) and incorporating it into the build or leaking it in logs.
- **Mitigation**: Use `stat -L -c%s` to resolve symlinks or ensure the extracted file is a regular file using `[ -f "$SAFE_TMP_DIR/out.json" ]` and `[ ! -h "$SAFE_TMP_DIR/out.json" ]`.

### [High] Resource Leak on Trap (Temporary Directory Leak)

- **Assumption challenged**: The trap cleans up all resources reliably.
- **Attack scenario**: The shell `trap` in `monitor.sh` is explicitly documented to only clean up the container: `podman rm -f -t 0 -- "$CID"`. It fails to delete `$SAFE_TMP_DIR` (created via `mktemp -d`).
- **Blast radius**: If the daemon is restarted, killed via SIGTERM, or encounters an error while `$SAFE_TMP_DIR` exists, the temporary directory is left orphaned in `/tmp`. Over time, repeated restarts or pipeline crashes will lead to slow disk exhaustion on the host.
- **Mitigation**: Add `rm -rf "$SAFE_TMP_DIR"` to the trap handler.

### [High] Loop Failure due to Existing CID File

- **Assumption challenged**: The loop cleans up properly for the next iteration.
- **Attack scenario**: The documentation states "remove the CID file" but the strict bash snippet provided (`Container Cleanup in Trap and Loop: podman rm -f -t 0 -- "$CID"`) does not delete `$CID_FILE`.
- **Blast radius**: `podman create --cidfile "$CID_FILE"` fails if the file already exists. On the second iteration of the `while true` loop, container creation will fail, `CID` will be read from the old run, and `podman start` will fail. The daemon breaks permanently after one successful run.
- **Mitigation**: Explicitly execute `rm -f "$CID_FILE"` at the end of the loop.

## Stress Test Results

- **Scenario**: Container runs an infinite loop `while true; do sleep 1; done` → **Expected**: Container times out, pipeline recovers → **Actual**: `podman wait` hangs forever, blocking the daemon indefinitely → **Pass/Fail**: FAIL
- **Scenario**: Container outputs 4GB artifact → **Expected**: Extraction is blocked, artifact rejected → **Actual**: `podman cp` dumps 4GB to host `/tmp`, exhausting host disk before the size check triggers → **Pass/Fail**: FAIL
- **Scenario**: Container creates symlink to `/etc/shadow` → **Expected**: Rejected as invalid size or file type → **Actual**: Symlink is copied to host, `stat` measures symlink length (passes check), symlink is moved to state directory → **Pass/Fail**: FAIL
