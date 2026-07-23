## Challenge Summary

**Overall risk assessment**: CRITICAL

**Verdict**: FAIL

## Challenges

### [Critical] Challenge 1: `tmpfs` Volatility and Inter-Container Isolation
- **Assumption challenged**: Data written to a container's `tmpfs` mount persists after the container exits, and separate containers can share an anonymous `tmpfs` implicitly.
- **Attack scenario**: The prep-container clones the repository into its `tmpfs` and exits. The data is instantly lost. The agent container runs, expecting the repository in its own new `tmpfs`, but finds it empty. The agent writes its artifact to its `tmpfs` and exits. The daemon attempts `podman cp` from the exited container, but the `tmpfs` is already unmounted and destroyed, leading to missing artifacts.
- **Blast radius**: The entire split-entry pipeline is broken. No code is validated, and no artifacts are passed. Complete denial of service by design.
- **Mitigation**: If limiting disk usage is the goal, use Podman named volumes backed by `tmpfs` or size-limited host bind mounts (e.g., creating a loopback filesystem or explicitly mounting a `tmpfs` on the *host* and bind-mounting it into the containers). Data must reside on a host-managed location to persist across containers and after exit.

### [High] Challenge 2: False "Atomic Updates" with `cp -T`
- **Assumption challenged**: `cp -T` provides atomic filesystem updates.
- **Attack scenario**: The host daemon extracts the artifact and runs `cp -T "$TMP_FILE" "integration/state/forge_status.json"`. The `cp` command opens the target file and truncates it. In that exact millisecond, the `OS-Agent` pipeline or another consumer reads the state file, receiving a 0-byte or partially written JSON file, leading to a parsing crash or pipeline failure.
- **Blast radius**: Pipeline unreliability and race conditions. State corruption.
- **Mitigation**: Revert to `mv -T` (or `mv`). `mv` across the same filesystem uses the `rename(2)` system call, which is truly atomic. To prevent directory overwrite evasion, ensure the source is a file and the target is not a directory before executing the `mv`, or use strict validation on the extracted artifact.

### [Medium] Challenge 3: Incoherent Host Cleanup
- **Assumption challenged**: `podman unshare rm -rf "$TMP_WORKSPACE"` on the host cleans up the container's `tmpfs`.
- **Attack scenario**: `$TMP_WORKSPACE` is an absolute path intended for the container (e.g., `/workspace`). Running `rm -rf /workspace` on the host attempts to delete a host directory, which either fails (harmless) or deletes unintended host files if the variable resolves to a sensitive path.
- **Blast radius**: Potential accidental deletion of host files or useless operations.
- **Mitigation**: Remove the host-side `rm -rf` if utilizing true container-managed `tmpfs`, or properly clean up the host-side `tmpfs` if the mitigation in Challenge 1 is adopted.

## Stress Test Results
- **Scenario**: `podman cp` from `tmpfs` of exited container → **Expected**: artifact retrieved → **Predicted**: File not found (tmpfs unmounted on exit) → **FAIL**
- **Scenario**: Atomicity of `cp -T` → **Expected**: Reader never sees partial file → **Predicted**: Reader sees partial file (`O_TRUNC` behavior) → **FAIL**
