# Handoff Report: Fix Strategy for Ermete Team MAS Architecture

## 1. Observation
- **SIGKILL Leak & Race Condition:** The `README.md` (lines 16-20) currently instructs running `podman run -d --cidfile "$CID_FILE"`, followed by seeding source (`podman cp`), and then `podman wait "$CID"`. This introduces a severe race condition (container executes before source is seeded) and runs detached. The challenger notes that a SIGKILL to `monitor.sh` bypasses the bash cleanup trap, leaving the container as a zombie indefinitely.
- **Hang DoS:** The architecture blocks on container execution (`podman wait "$CID"`) without any timeout mechanism, making it vulnerable to infinite loops (e.g., `sleep infinity`).
- **Disk Exhaustion:** Artifact extraction uses `TMP_DIR=$(mktemp -d)` and `rm -rf "$TMP_DIR"` sequentially (lines 23-26). If `podman cp` fails, bash's `set -e` aborts the script, skipping the cleanup and leaking the `/tmp` directory.
- **CID File Leak:** The trap snippet in `README.md` (lines 60-64) runs `podman rm -v -f "$CID"` but forgets to delete the actual `$CID_FILE`. Subsequent runs crash because `podman create --cidfile` expects the CID file to not exist.
- **Critical Symlink Traversal:** `podman cp` extracts the artifact to `$TMP_DIR/out.json`. If the container created a symlink to `/etc/shadow`, `head -c` will read through the symlink on the host filesystem, leaking the host's `/etc/shadow` into the state file.

## 2. Logic Chain
1. **To fix SIGKILL Leaks and the Seeding Race Condition:** We must separate container creation from execution, seed the source while stopped, and then run it synchronously tied to the systemd service. Using `podman create --cidfile`, followed by `podman cp` to seed the source, and then `systemd-run --user --scope --property=BindsTo=ermete-mas.service podman start --attach "$CID"` binds the container's execution to the monitor's systemd scope. If the monitor is killed, the scope closes and systemd kills the attached podman process.
2. **To fix Hang DoS:** Wrapping the foreground `podman start --attach` command in a `timeout` utility enforces a maximum execution window, preventing malicious recipes from halting the pipeline forever.
3. **To fix Disk Exhaustion:** Extracting the artifact needs resilient cleanup. By checking the exit status of `podman cp` and explicitly cleaning up on failure (e.g., `|| { rm -rf "$TMP_DIR"; exit 1; }`), we guarantee that a failure cleans up its temporary environment before `set -e` takes over.
4. **To fix the CID File Leak:** Appending `rm -f "$CID_FILE"` to the trap handler directly eliminates the orphaned file state, allowing subsequent loop iterations to proceed cleanly.
5. **To fix Symlink Traversal:** Explicitly asserting that the extracted artifact is a regular file (`[ -f ]`) and not a symlink (`[ ! -L ]`) blocks symlink evaluation by `head -c` and forces an abort before host file leakage can occur.

## 3. Caveats
- Relying on `podman start --attach` within `systemd-run` relies on podman properly forwarding signals when the scope is killed. If conmon ignores the signal from systemd, zombie containers could still occur on SIGKILL. For ultimate robustness, `ExecStopPost=` inside the parent systemd service (e.g., `ermete-mas.service`) should also explicitly run `podman rm -v -f` for all known CIDs.
- The `timeout` command will exit with code `124` on timeout. The pipeline script must be prepared to catch this and properly mark the pipeline state as `FAILED` rather than silently crashing and restarting the loop abruptly.

## 4. Conclusion
The architecture must be patched with the following strategies:
1. **Execution Sequence:** Change from `podman run -d` to `podman create --cidfile ...` -> `podman cp` (seed source) -> `timeout 300s systemd-run --user --scope --property=BindsTo=ermete-mas.service podman start --attach "$CID"`.
2. **CID Cleanup Trap:** Update the trap definition to include `rm -f "$CID_FILE"`.
3. **Extraction Safety:** Implement error-resilient temporary directory management: `podman cp ... || { rm -rf "$TMP_DIR"; exit 1; }`.
4. **Symlink Defense:** Add an explicit file type check before reading the extracted file: `if [ -L "$TMP_DIR/out.json" ] || ! [ -f "$TMP_DIR/out.json" ]; then rm -rf "$TMP_DIR"; exit 1; fi`.

## 5. Verification Method
1. **Race Condition & SIGKILL Test:** Run the patched loop, inject `kill -9 $$` into the parent shell while `podman start --attach` is running. Check `podman ps` to confirm the container was terminated.
2. **Timeout Test:** Inject `sleep 600` into the container payload. Verify `timeout` kills it after 300s and the script recovers.
3. **TMP Leak Test:** Force `podman cp` to fail (e.g., missing artifact file) and verify `/tmp` does not contain orphaned directories.
4. **Symlink Test:** Inside a test payload, run `ln -s /etc/passwd /workspace/artifact.json`. Verify the extraction script rejects the file and aborts gracefully without reading `/etc/passwd`.
5. **CID Leak Test:** Artificially crash the script post-execution and verify `$CID_FILE` is deleted by the trap.
