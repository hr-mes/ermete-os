# Handoff Report: Architecture Fix Strategy for Ermete MAS

## 1. Observation
- **SIGKILL Leak & Zombie Containers**: The architecture (as per `README.md`) relies on a Bash trap and detached containers (`podman start "$CID"` followed by `podman wait "$CID"`). The Challenger's report correctly notes that a `SIGKILL` to `monitor.sh` will bypass the trap, leaving the detached container running indefinitely.
- **Hang DoS**: The execution loop uses `podman wait "$CID"` without a timeout, meaning a container executing `sleep infinity` will block the daemon forever.
- **Disk Exhaustion**: The artifact extraction snippet creates a temporary directory (`TMP_DIR=$(mktemp -d)`) and then runs `podman cp`. If `podman cp` fails and `set -e` is active, the script aborts immediately, skipping `rm -rf "$TMP_DIR"` and leaking inodes/disk space.
- **CID File Leak**: The `trap` handler in `README.md` runs `podman rm -v -f "$CID"` but forgets to delete the `$CID_FILE`. This prevents `podman create --cidfile` from working on subsequent runs.
- **Critical Symlink Traversal**: The extraction snippet copies the output with `podman cp`, then runs `head -c 1048576 "$TMP_DIR/out.json" > "$TMP_FILE"`. It performs no file type checks. A malicious symlink to a sensitive host file (e.g., `/etc/shadow`) will cause `head` to read the host's file, thereby leaking it.

## 2. Logic Chain
1. **Hang DoS Resolution**: To prevent blocking, `podman wait` must be bound by a time limit. Wrapping `podman wait` with `timeout` ensures the script regains control. If a timeout occurs, the container must be forcefully terminated via `podman kill`.
2. **SIGKILL Leak Resolution**: Bash traps are insufficient for robust lifecycle management. To guarantee container termination even if `monitor.sh` is SIGKILLed, the container must be tied to the systemd service's lifecycle. Adding an `ExecStopPost=` directive to the daemon's systemd unit to sweep and remove containers (e.g., using a specific Podman `--label`) ensures guaranteed cleanup.
3. **Disk Exhaustion Resolution**: Decoupling the extraction logic from `set -e` aborts prevents directory leaks. By wrapping `podman cp` in an `if` statement (or using a dedicated cleanup trap for the tmp dir), we guarantee `rm -rf` executes regardless of `podman cp`'s success.
4. **CID Leak Resolution**: Explicitly adding `rm -f "$CID_FILE"` inside the `trap` handler and the regular loop cleanup ensures the state is clean for the next `podman create`.
5. **Symlink Traversal Resolution**: `podman cp` can extract symlinks that point to host targets. By adding a strict `[ -L ... ] || [ ! -f ... ]` check *before* `head` reads the file, the script explicitly rejects symlinks, neutralizing the path traversal vector.

## 3. Caveats
- The fix strategy assumes `monitor.sh` is managed by a systemd service (`ermete-mas.service`) as implied by the `systemd-run` snippets in the README. If it is run manually outside systemd, the `ExecStopPost` mitigation for SIGKILLs will not apply (though the timeout fixes still hold).
- Modifying `podman wait` with `timeout` requires ensuring that the underlying timeout command sends the correct signal and doesn't just orphan the wait process while the container continues. Thus, `podman kill` must explicitly follow a timeout.

## 4. Conclusion
The architecture can be secured by implementing the following integrated fix strategy:
1. **DoS Mitigations**: Implement `timeout 600 podman wait "$CID" || podman kill "$CID"` to cap execution time.
2. **Robust Cleanup**: Update `ermete-mas.service` with `ExecStopPost=/usr/bin/podman rm -f $(podman ps -aq --filter label=mas-agent=true)` and add `--label mas-agent=true` to `podman create` to prevent zombie leaks on `SIGKILL`.
3. **Extraction Safety**: Refactor the tmpfs artifact extraction to check for symlinks (`if [ -L "$TMP_DIR/out.json" ] || [ ! -f "$TMP_DIR/out.json" ]; then exit 1; fi`) before invoking `head`, and structure the Bash logic so `rm -rf "$TMP_DIR"` executes even if `podman cp` fails.
4. **State Hygiene**: Append `rm -f "$CID_FILE"` to all trap and loop exit paths.

## 5. Verification Method
1. **Hang DoS Fix**: Inject `sleep infinity` in the agent container; observe `monitor.sh` resume and kill the container after the specified timeout.
2. **SIGKILL Fix**: Send `kill -9` to `monitor.sh` (or `systemctl kill -s SIGKILL ermete-mas.service`); run `podman ps` to confirm the container is destroyed by systemd's `ExecStopPost`.
3. **Disk Exhaustion Fix**: Force `podman cp` to fail (e.g., missing artifact); confirm `/tmp` or the specified tmpfs has no residual `$TMP_DIR` directories.
4. **CID File Fix**: Trigger a script crash; verify `$CID_FILE` is removed.
5. **Symlink Traversal Fix**: Have the container output `ln -s /etc/passwd /workspace/artifact.json`; verify the pipeline rejects it and does NOT leak `/etc/passwd`.
