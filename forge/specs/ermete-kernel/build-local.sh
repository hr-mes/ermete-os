#!/bin/bash
set -euo pipefail
# Ermete OS: The Ultimate Chimera Kernel Bedrock Local Builder
# Riproduce in bit-perfect il workflow di GitHub Actions all'interno di un micro-container OCI locale
# [REVISION 3.0] Architettura a Via 1: Zero-Trust, AST Validated, ThinLTO Puro (LLVM Assoluto)

echo ">>> [BEDROCK] Inizializzazione Ambiente di Build Isolato Locale (Fedora 43 OCI)"

FORGE_DIR=$(git rev-parse --show-toplevel)
CACHE_DIR="$FORGE_DIR/.ccache_local"
mkdir -p "$CACHE_DIR"

echo ">>> [BEDROCK] Immagine di build del kernel (forge/specs/ermete-kernel/builder, la stessa della CI)..."
podman build --build-arg FEDORA_VERSION=43 \
  -t localhost/ermete-kernel-builder:43 \
  "$FORGE_DIR/forge/specs/ermete-kernel/builder"

echo ">>> [BEDROCK] Esecuzione Container Fedora 43 (Privileged limitato)..."
podman run --rm -i \
  --cap-add SYS_ADMIN \
  --security-opt label=disable \
  --security-opt seccomp=unconfined \
  -v "$FORGE_DIR":/forge \
  -w /forge \
  -e GITHUB_WORKSPACE=/forge \
  localhost/ermete-kernel-builder:43 \
  /bin/bash -s << 'DOCKEREOF'
    set -euo pipefail
    echo '>>> [FASE 1] Fetch, Validazione AST e Fusione Kconfig (Universal Matrix)...'
    # Questo script scarica il kernel, esegue le patch in Fuzz 1, scarta quelle rotte e imposta il LTO/BORE
    bash forge/specs/ermete-kernel/prepare-chimera.sh
    
    KERNEL_DIR=$(cat ~/rpmbuild/BUILD/.kernel_version)
    cd ~/rpmbuild/BUILD/$KERNEL_DIR
    
    
    
    export PATH="/usr/lib64/ccache:/usr/lib/ccache:$PATH"
    export CCACHE_DIR=/forge/.ccache_local
    export CCACHE_COMPRESS=1
    export CCACHE_MAXSIZE=10G
    ccache -z
    
    echo '>>> [FASE 2] Iniezione Macro LLVM Globali (Anti-GCC per Kmods e Build)...'
    cat << 'MACRO' >> ~/.rpmmacros
%_smp_mflags -j$(nproc)
%toolchain clang
%__make /usr/bin/make LLVM=1 LLVM_IAS=1
%__cc clang
%__cxx clang++
%_build_cc clang
%_build_cxx clang++
%_host_cc clang
%_host_cxx clang++
%_ld ld.lld
%_ldflags -Wl,-O2 -Wl,--as-needed -Wl,--sort-common -Wl,-z,now -Wl,-z,relro -fuse-ld=lld
%optflags %{__global_compiler_flags} -march=x86-64-v3 -pipe -Wno-error
%kcflags -march=x86-64-v3 -pipe -Wno-error
MACRO

    cd ~/rpmbuild/SPECS
    echo ">>> [FASE 3] Lancio RPMBuild con Skip %prep (ThinLTO Assoluto e Fuzzer Reattivo)..."
    # Usando --noprep diciamo a rpmbuild di saltare il download tarball e la piallatura del codice.
    # Userà l'albero del sorgente chirurgicamente modificato da prepare-chimera.sh (Zero overhead e sicurezza totale)
    rpmbuild -bb --noprep kernel.spec \
        --target x86_64 </dev/null
    ccache -s
    
    echo '========================================================='
    echo ' BUILD LOCALE COMPLETATA CON SUCCESSO (ThinLTO Puro).'
    echo '========================================================='
    ls -lh ~/rpmbuild/RPMS/x86_64/
    mkdir -p /forge/RPMS_OUT
    cp ~/rpmbuild/RPMS/x86_64/*.rpm /forge/RPMS_OUT/
DOCKEREOF
