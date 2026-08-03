import sys

with open(".github/workflows/ermete-forge-orchestrator.yml", "r") as f:
    lines = f.readlines()

out = []
for idx, line in enumerate(lines):
    # Fix sccache keys
    if "key: sccache-${{ matrix.package }}-${{ github.sha }}" in line:
        line = line.replace("key: sccache-${{ matrix.package }}-${{ github.sha }}", "key: sccache-${{ matrix.package }}-${{ runner.os }}-${{ hashFiles('**/Cargo.lock', 'specs/**') }}")
    
    # Remove --pull=always
    if "--pull=always" in line:
        line = line.replace("--pull=always", "")
        
    # Remove STORAGE_DRIVER=vfs
    if "export STORAGE_DRIVER=vfs" in line:
        continue # drop the line
        
    # Remove CCACHE_DISABLE and SCCACHE_DISABLE in build-nvidia
    if "export CCACHE_DISABLE=1 SCCACHE_DISABLE=1 " in line:
        line = line.replace("export CCACHE_DISABLE=1 SCCACHE_DISABLE=1 ", "")

    out.append(line)

# Add cache step to upstream-packages
for i in range(len(out)):
    if "name: Check Idempotency (Content Hash)" in out[i] and "upstream-packages" in "".join(out[max(0, i-20):i]):
        for j in range(i+1, len(out)):
            if "- name: Install Dependencies and Build" in out[j]:
                cache_step = """      - name: Cache sccache and cargo
        if: steps.check_idempotency.outputs.skip != 'true'
        uses: actions/cache@v4
        with:
          path: |
            .sccache
            .cargo-cache
          key: sccache-${{ matrix.package }}-${{ runner.os }}-${{ hashFiles('**/Cargo.lock', 'specs/**') }}
          restore-keys: |
            sccache-${{ matrix.package }}-${{ runner.os }}-
            sccache-${{ matrix.package }}-
"""
                out.insert(j, cache_step)
                
                # Add mounts for sccache and cargo
                for k in range(j+1, len(out)):
                    if "podman run" in out[k]:
                        out.insert(k+1, "            -v ${{ github.workspace }}/.sccache:/root/.cache/sccache \\\n            -v ${{ github.workspace }}/.cargo-cache:/root/.cargo \\\n")
                        break
                
                # mkdir for the caches
                for k in range(j+1, len(out)):
                    if "mkdir -p RPMS" in out[k]:
                        out.insert(k+1, "          mkdir -p ${{ github.workspace }}/.sccache ${{ github.workspace }}/.cargo-cache\n")
                        break
                break
        break

with open(".github/workflows/ermete-forge-orchestrator.yml", "w") as f:
    f.writelines(out)

