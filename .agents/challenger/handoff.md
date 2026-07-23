# Handoff Report

## 1. Observation
- The `README.md` states the pipeline uses "native Podman timeouts", but the execution explicitly uses `podman start "$CID" && podman wait "$CID"` with no timeout flag, and `-t 0` is only passed to `podman rm`.
- The artifact extraction mitigates "TAR corruption" using `podman cp` to the host, followed by `stat -c%s` to enforce a 1MB limit.
- `podman cp` preserves symlinks by default. `stat -c%s` without `-L` returns the length of the symlink path.
- The trap cleanup snippet only executes `podman rm -f -t 0 -- "$CID"`, omitting `$SAFE_TMP_DIR` deletion.
- The daemon loop snippet similarly omits the deletion of `$CID_FILE`.
- The architecture claims to prevent "Disk Exhaustion DoS" but `podman cp` copies the entire file to the host's `/tmp` (which could be up to 4GB due to the tmpfs limit) *before* enforcing the 1MB limit.

## 2. Logic Chain
1. **Zombie Container Bug**: Because `podman wait "$CID"` has no timeout, an infinite loop in the container will cause the script to hang indefinitely. This breaks the sequential pipeline permanently.
2. **Disk Exhaustion Limit Bypass**: Since `podman cp` extracts the full file before checking its size, an attacker can create a 4GB file, causing `podman cp` to write 4GB to the host `/tmp`, potentially causing a system-wide disk space DoS.
3. **Symlink Arbitrary File Read**: A symlink to `/etc/shadow` created in the container will be copied to the host by `podman cp` as a symlink. `stat -c%s` will evaluate the symlink's length (11 bytes), which is `< 1048576`, so the file is moved to `integration/state/final_artifact.json`. Any subsequent read from the host will resolve the symlink on the host, reading `/etc/shadow`.
4. **Trap Leaks**: Since `rm -rf "$SAFE_TMP_DIR"` is absent from the trap, pipeline interruptions leave temporary directories orphaned, causing slow disk exhaustion over time.
5. **Loop Failure**: Since `$CID_FILE` is not deleted, the second execution of `podman create --cidfile "$CID_FILE"` will fail, permanently breaking the loop.

## 3. Caveats
- I did not test the exact version of podman installed. Behavior of `podman cp` regarding symlinks could theoretically differ on very specific versions, but by default it acts like `cp -a` and preserves them.
- I assumed `/tmp` size is smaller than or equal to the container tmpfs size (4GB) or that multiple concurrent extractions could overwhelm it.

## 4. Conclusion
**Verdict: FAIL**
The architecture is fundamentally flawed. The exact mechanisms designed to provide "Generation-8 Security Patches" actively introduce critical vulnerabilities: infinite execution hangs (Zombie bug), host disk exhaustion, arbitrary host file reads, and resource leaks. The bash snippets provided must not be implemented as-is.

## 5. Verification Method
- **Zombie Bug**: Run `podman start ... && podman wait ...` with a container image executing `sleep infinity`. Observe that the script blocks forever.
- **Disk Exhaustion**: Inside a container, run `dd if=/dev/zero of=/workspace/artifact.json bs=1M count=4000`. Run the extraction script and observe the host `/tmp` space decrease by 4GB during `podman cp`.
- **Symlink Exploit**: Inside a container, run `ln -s /etc/shadow /workspace/artifact.json`. Run the extraction snippet, then `cat integration/state/final_artifact.json` on the host to observe the contents of the host's shadow file.
