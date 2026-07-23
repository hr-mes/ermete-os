# Handoff Report: MAS Architecture Iteration 14 Fixes

## 1. Observation
- **Lifecycle Binding Facade**: The Failure Report (`FAILURE_REPORT_ITERATION_14.md:14-23`) notes that `podman run -d` immediately exits the CLI client, causing `systemd-run`'s scope to complete instantly and fail to bind the container's true process lifecycle (`libpod-<CID>.scope`) to `ermete-mas.service`. Furthermore, Challenger 1 (`FAILURE_REPORT_ITERATION_14.md:45`) highlights a race condition where the container starts before source code is seeded via `podman cp`.
- **Extraction Flaws**: The Failure Report (`FAILURE_REPORT_ITERATION_14.md:27-38` and `42-43`) highlights that `podman cp` executes sequentially to the host before size limits apply, permitting disk exhaustion. Additionally, Challenger 2 notes that symlinks and FIFOs inside the container can cause arbitrary host file reads or daemon hangs.
- **Inode Exhaustion**: The Failure Report (`FAILURE_REPORT_ITERATION_14.md:44`) states that `--mount tmpfs-size=10M` lacks an inode limit, allowing attackers to exhaust host inodes.

## 2. Logic Chain
1. **Binding Fix**: To bind the container correctly to `systemd-run`, the podman client process must stay alive in the foreground for the entire container lifecycle. Using `podman create` returns a static CID. We can then seed data using `podman cp`. Finally, launching the container in the foreground with `systemd-run --user --scope --property=BindsTo=ermete-mas.service podman start --attach "$CID"` ensures systemd correctly manages and scopes the running container.
2. **Safe Extraction**: Streaming the output via `podman exec ... cat` is unsafe because `cat` resolves symlinks and blocks on FIFOs from *inside* a potentially running container. Instead, the container should be stopped first (nullifying TOCTOU). Then, using `podman mount "$CID"`, the daemon can mount the container filesystem directly. The host can safely evaluate `[ -f "$MNT/path" ] && [ ! -L "$MNT/path" ] && [ ! -p "$MNT/path" ]` before streaming safely using `head -c 1048576 "$MNT/path" > "$TMP_FILE"`.
3. **Inode Limits**: Appending the exact parameter requested (`tmpfs-options=nr_inodes=1024`) directly applies the kernel's `nr_inodes` mount argument to the podman tmpfs mount.

## 3. Caveats
- `podman mount` requires the user running the command to have sufficient privileges (typically root or proper user namespace mapping in rootless). 
- If the artifact size reaches exactly 1MB, `head -c` cleanly truncates it, which may result in an invalid JSON file. Downstream components must gracefully handle malformed JSON.
- `podman start --attach` binds standard streams to systemd's journal, which might capture excessive stdout/stderr from the container unless log limits or explicit standard stream redirection are applied.

## 4. Conclusion
We recommend the following three concrete fixes:
1. **Systemd Binding without Detach**: Replace `podman run -d` with a 3-step startup sequence:
   - `CID=$(podman create --mount type=tmpfs,destination=/workspace,tmpfs-size=10M,tmpfs-options=nr_inodes=1024 agent-image)`
   - `podman cp <source> "$CID:/workspace/"`
   - `systemd-run --user --scope --property=BindsTo=ermete-mas.service podman start --attach "$CID"`
2. **Safe Extraction**: Implement extraction only *after* the container exits, using `podman mount` to avoid TOCTOU, tar extraction, and symlink/FIFO attacks:
   - `MNT=$(podman mount "$CID")`
   - Evaluate `[ -f "$MNT/workspace/artifact.json" ] && [ ! -L "$MNT/workspace/artifact.json" ] && [ ! -p "$MNT/workspace/artifact.json" ]`
   - Read safely: `head -c 1048576 "$MNT/workspace/artifact.json" > "$TMP_FILE"`
   - `podman unmount "$CID"`
3. **Explicit Inode Limit**: Ensure the `podman create` mount includes `tmpfs-options=nr_inodes=1024` as indicated above.

## 5. Verification Method
- **Verify Binding**: Run the container with the `podman start --attach` syntax under `systemd-run`, restart `ermete-mas.service`, and verify using `podman ps` that the container is terminated (no zombie process).
- **Verify Extraction**: Create an `artifact.json` as a symlink to `/etc/shadow` or as a FIFO (`mkfifo`) inside the container. Attempt extraction using the `podman mount` script. It must fail safely without hanging or reading host files.
- **Verify Inodes**: Inside the container, run a loop to touch files in `/workspace/`. It should fail with `No space left on device` after creating 1024 files, leaving host inodes unaffected.
