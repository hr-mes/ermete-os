#!/bin/bash
set -euo pipefail
# Ermete OS: The Ultimate Chimera Kernel Bedrock Builder (Fedora Upstream Zero-Trust)

# --- BEDROCK MANIFEST (PINNED COMMITS) ---
# Matrice Dominante Pura: CachyOS (Scheduler BORE) + ClearLinux (Math/CPU/Memory).
# WARNING: HEAD is unpinned — should be pinned to a specific commit hash
# Current: CACHYOS_COMMIT="HEAD"
CACHYOS_COMMIT="ea739d734ec179864b21446856315bc49f7c52fa"
# -----------------------------------------

MODE="full"
if [[ "${1:-}" == "--meta" ]]; then
  MODE="meta"
fi




WORKSPACE_DIR="$HOME/rpmbuild"
echo ">>> Pulizia profonda del workspace per evitare conflitti con vecchie build..."
mkdir -p "$WORKSPACE_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
cd "$WORKSPACE_DIR"

echo "========================================================="
echo " FASE 1: RISOLUZIONE DINAMICA KERNEL E PATCH (con NVIDIA Shield)"
echo "========================================================="

fetch_pinned() {
  local REPO=$1
  local TARGET=$2
  local BRANCH_TAG=$3
  local COMMIT=$4
  
  echo ">>> Fetching $TARGET (Commit: $COMMIT)..."
  rm -rf "$TARGET"
  if [ "$COMMIT" = "HEAD" ]; then
      git clone --depth 1 $BRANCH_TAG "$REPO" "$TARGET" || { echo "FATAL: Clone fallito per $REPO"; exit 1; }
  else
      git clone --depth 500 $BRANCH_TAG "$REPO" "$TARGET" || { echo "FATAL: Clone fallito per $REPO"; exit 1; }
      git -C "$TARGET" checkout -q "$COMMIT" || { echo "FATAL: Checkout fallito per $COMMIT"; exit 1; }
  fi
}

fetch_pinned "https://github.com/CachyOS/kernel-patches.git" "/tmp/cachyos-patches" "" "$CACHYOS_COMMIT"

# ClearLinux necessita di storia profonda (Time-Travel) per cercare i vecchi commit di kernel passati!
echo ">>> Fetching /tmp/clearlinux-patches (Depth 1000 per Time-Travel)..."
rm -rf /tmp/clearlinux-patches
git clone --depth 1000 https://github.com/clearlinux-pkgs/linux.git /tmp/clearlinux-patches || { echo "FATAL: Clone fallito per ClearLinux"; exit 1; }

echo ">>> [BEDROCK SECURE] Calcolo dinamico dello Scudo NVIDIA (Dynamic Ceiling)..."
curl -sLo /etc/yum.repos.d/fedora-nvidia.repo https://negativo17.org/repos/fedora-nvidia.repo || true
EXPECTED_SHA="9126880310a20437de6ba1a83d299ee9a2119f8a1ef1e40de601676054320fc5"
if [ -f /etc/yum.repos.d/fedora-nvidia.repo ]; then
    echo "$EXPECTED_SHA  /etc/yum.repos.d/fedora-nvidia.repo" | sha256sum -c - || { echo "FATAL: Checksum mismatch per fedora-nvidia.repo"; exit 1; }
fi
# TODO: Add SHA256 checksum verification for fedora-nvidia.repo
NVIDIA_VER=$(dnf repoquery --qf '%{VERSION}\n' akmod-nvidia 2>/dev/null | sort -V | tail -n 1 | awk -F. '{print $1}' || true)
MAX_KERNEL="6.18" # Default
if [[ -n "$NVIDIA_VER" ]]; then
    if [[ "$NVIDIA_VER" -ge 615 ]]; then MAX_KERNEL="6.20"; fi
    if [[ "$NVIDIA_VER" -ge 620 ]]; then MAX_KERNEL="7.0"; fi
    if [[ "$NVIDIA_VER" -ge 630 ]]; then MAX_KERNEL="7.2"; fi
fi
echo ">>> NVIDIA Driver rilevato: Serie ${NVIDIA_VER}.xx -> Massima versione kernel consentita: $MAX_KERNEL"

echo ">>> Ricerca della migliore versione kernel supportata (Fedora -> NVIDIA Shield -> CachyOS -> ClearLinux)..."
TARGET_RELEASEVER=""
TARGET_KERNEL_VER=""

source /etc/os-release
CURRENT_FVER=$VERSION_ID
MIN_FVER=$((CURRENT_FVER - 4))

for (( ver=$CURRENT_FVER; ver>=$MIN_FVER; ver-- )); do
    echo ">>> Analisi Fedora $ver..."
    
    URL=$(dnf download --source kernel --releasever=$ver --url 2>/dev/null | grep -E '\.src\.rpm' | head -n 1 || true)
    if [ -z "$URL" ]; then
        echo "    Nessun kernel sorgente trovato nei repo per Fedora $ver."
        continue
    fi
    
    # Estraiamo la versione major.minor (es. 6.14 da kernel-6.14.5-100.fc43.src.rpm)
    F_VER=$(basename "$URL" | sed -E 's/^kernel-([0-9]+\.[0-9]+).*/\1/')
    echo "    Kernel in Fedora $ver: $F_VER"
    
    # [NVIDIA SHIELD DINAMICO]
    if [[ $(printf "%s\n%s" "$F_VER" "$MAX_KERNEL" | sort -V | tail -n 1) != "$MAX_KERNEL" && "$F_VER" != "$MAX_KERNEL" ]]; then
        echo "    [SHIELD] Kernel $F_VER supera il tetto NVIDIA ($MAX_KERNEL). Passo al precedente..."
        continue
    fi
    
    # 1. Controllo CachyOS
    if [ ! -d "/tmp/cachyos-patches/$F_VER/all" ]; then
        echo "    CachyOS NON supporta $F_VER. Passo al precedente..."
        continue
    fi
    
    # 2. Controllo Clear Linux
    pushd /tmp/clearlinux-patches > /dev/null
    F_VER_ESC="${F_VER//./\\.}"
    CLEAR_COMMIT=$(git log --grep="update.*${F_VER_ESC}\\b" -n 1 --format="%H" || true)
    popd > /dev/null
    
    if [ -z "$CLEAR_COMMIT" ]; then
        echo "    Clear Linux NON ha patch per $F_VER. Passo al precedente..."
        continue
    fi
    
    echo ">>> MATCH PERFETTO! Fedora $ver fornisce kernel $F_VER, pienamente supportato da CachyOS e ClearLinux."
    TARGET_RELEASEVER=$ver
    TARGET_KERNEL_VER=$F_VER
    break
done

if [ -z "$TARGET_RELEASEVER" ]; then
    echo "ERRORE FATALE: Nessun kernel compatibile trovato incrociando Fedora, NVIDIA Shield, CachyOS e Clear Linux." >&2
    exit 1
fi

if [[ "$MODE" == "meta" ]]; then
    # Checkout ClearLinux per hashare i file esatti
    pushd /tmp/clearlinux-patches > /dev/null
    git checkout -q "$CLEAR_COMMIT"
    popd > /dev/null

    # Hashiamo SOLO i file patch che verranno fusi nel kernel
    CACHY_PATCH_HASH=$(find "/tmp/cachyos-patches/$TARGET_KERNEL_VER/" -type f -name "*.patch" -exec sha256sum {} + | sort | sha256sum | awk '{print $1}')
    
    CLEAR_PATCH_HASH=""
    for patch_name in \
        "0001-sched-migrate.patch" \
        "0001-sched-numa-Initialise-numa_migrate_retry.patch" \
        "0001-mm-memcontrol-add-some-branch-hints-based-on-gcov-an.patch" \
        "0002-sched-core-add-some-branch-hints-based-on-gcov-analy.patch" \
        "0170-sched-Add-unlikey-branch-hints-to-several-system-cal.patch"; do
        if [ -f "/tmp/clearlinux-patches/$patch_name" ]; then
            CLEAR_PATCH_HASH+=$(sha256sum "/tmp/clearlinux-patches/$patch_name" | awk '{print $1}')
        fi
    done
    CLEAR_PATCH_HASH=$(echo "$CLEAR_PATCH_HASH" | sha256sum | awk '{print $1}')

    # Output deterministic fingerprint data and exit
    echo "META_KERNEL_VER=$TARGET_KERNEL_VER"
    echo "META_RELEASE_VER=$TARGET_RELEASEVER"
    echo "META_CACHY_PATCHES=$CACHY_PATCH_HASH"
    echo "META_CLEAR_PATCHES=$CLEAR_PATCH_HASH"
    exit 0
fi

echo "========================================================="
echo " FASE 2: LE FONDAMENTA (Fedora Upstream Zero-Trust)"
echo "========================================================="
echo ">>> Scaricamento kernel.src.rpm puro (Releasever: $TARGET_RELEASEVER)..."
dnf download --source kernel --releasever=$TARGET_RELEASEVER
rpm -ivh kernel-*.src.rpm
KERNEL_SRPM=$(ls kernel-*.src.rpm | head -n 1)
KERNEL_VER=$(rpm -qp --qf '%{VERSION}' "$KERNEL_SRPM" | cut -d. -f1,2)
rm -f kernel-*.src.rpm

CACHY_PATCH_DIR="/tmp/cachyos-patches/$KERNEL_VER"
if [ ! -d "$CACHY_PATCH_DIR" ]; then
    echo "ERRORE FATALE: Discrepanza dinamica. Trovato $KERNEL_VER ma mancano le patch CachyOS!"
    exit 1
fi

# [BEDROCK] Universal Domain Router Ridotto (Matrice Dominante Pura)
route_patch() {
    local patch="$1"
    local source="$2"
    local lower_patch="${patch,,}"
    local domain="99"
    local priority="9"
    
    # Riconosciamo solo CachyOS e ClearLinux per garantire zero deadlock semantici.
    if [[ "$lower_patch" =~ (bore|sched|eevdf|cfs|cpu|topology) ]]; then
        domain="02"
        case "$source" in cachyos) priority="1" ;; clear) priority="2" ;; *) priority="5" ;; esac
    elif [[ "$lower_patch" =~ (bbr|tcp|net|wireguard|bpf) ]]; then
        domain="04"
        case "$source" in cachyos) priority="1" ;; clear) priority="2" ;; *) priority="5" ;; esac
    elif [[ "$lower_patch" =~ (mglru|mm|lru|zswap|zram|page|memory|vm) ]]; then
        domain="03"
        case "$source" in clear) priority="1" ;; cachyos) priority="2" ;; *) priority="5" ;; esac
    elif [[ "$lower_patch" =~ (fs|ext4|btrfs|xfs|zfs|io|block|nvme) ]]; then
        domain="05"
        case "$source" in cachyos) priority="1" ;; clear) priority="2" ;; *) priority="5" ;; esac
    else
        domain="99"
        case "$source" in cachyos) priority="1" ;; clear) priority="2" ;; *) priority="5" ;; esac
    fi
    local clean_patch=$(echo "$patch" | sed -E 's/^[0-9]+-//')
    echo "bedrock-${domain}_${priority}_${source}_${clean_patch}"
}

echo ">>> Scansione e smistamento delle patch in SOURCES/ con Matrice Dominante..."
if [ -d "$CACHY_PATCH_DIR" ]; then
    find "$CACHY_PATCH_DIR" -type f -name "*.patch" | while read -r patch; do
        cp "$patch" "SOURCES/$(route_patch "$(basename "$patch")" "cachyos")"
    done
fi

echo ">>> Sincronizzazione dinamica Clear Linux con Kernel $KERNEL_VER..."
pushd /tmp/clearlinux-patches > /dev/null
KERNEL_VER_ESC="${KERNEL_VER//./\\.}"
CLEAR_COMMIT=$(git log --grep="update.*${KERNEL_VER_ESC}\\b" -n 1 --format="%H" || true)
if [ -n "$CLEAR_COMMIT" ]; then
    echo ">>> Allineamento Clear Linux al commit: $CLEAR_COMMIT"
    git checkout -q "$CLEAR_COMMIT"
else
    echo ">>> ATTENZIONE: Nessun commit specifico trovato per $KERNEL_VER. Utilizzo l'head di main."
fi
popd > /dev/null

echo ">>> Pulizia patch obsolete (ntsync è upstream in 6.14)..."
rm -f SOURCES/*ntsync*.patch || true

echo ">>> Aggiunta patch chirurgiche Clear Linux..."
for patch_name in \
    "0001-sched-migrate.patch" \
    "0001-sched-numa-Initialise-numa_migrate_retry.patch" \
    "0001-mm-memcontrol-add-some-branch-hints-based-on-gcov-an.patch" \
    "0002-sched-core-add-some-branch-hints-based-on-gcov-analy.patch" \
    "0170-sched-Add-unlikey-branch-hints-to-several-system-cal.patch"; do
    
    if [ -f "/tmp/clearlinux-patches/$patch_name" ]; then
        cp "/tmp/clearlinux-patches/$patch_name" "SOURCES/$(route_patch "$patch_name" "clear")"
    fi
done

echo ">>> Normalizzazione kernel.spec con LLVM=1 LLVM_IAS=1..."
sed -i 's/%make_build/%make_build LLVM=1 LLVM_IAS=1/g' SPECS/kernel.spec
sed -i 's/make -s/make -s LLVM=1 LLVM_IAS=1/g' SPECS/kernel.spec
sed -i 's/\(.*\)make ARCH/\1make LLVM=1 LLVM_IAS=1 ARCH/g' SPECS/kernel.spec

echo "========================================================="
echo " FASE 3: TUNING KCONFIG (Bedrock Kbuild Merge_Config)"
echo "========================================================="
# [BEDROCK FIX] Utilizzo di Kconfig Fragment formale invece di `sed` per validare le dipendenze
cat << 'BEDROCK_CFG' > SOURCES/ermete-bedrock.cfg
# --- ERMETE FORGE: ZEN/LIQUORIX TUNING ---
CONFIG_HZ_1000=y
CONFIG_HZ=1000
# CONFIG_HZ_300 is not set
# CONFIG_HZ_250 is not set
# CONFIG_HZ_100 is not set

# Full Preemption for lowest latency
CONFIG_PREEMPT=y
CONFIG_PREEMPT_BUILD=y
# CONFIG_PREEMPT_VOLUNTARY is not set
# CONFIG_PREEMPT_NONE is not set

# Eliminate Debug Overhead
# CONFIG_DEBUG_KERNEL is not set

CONFIG_DEFAULT_BBR=y
CONFIG_TCP_CONG_BBR=y
# CONFIG_DEFAULT_CUBIC is not set

CONFIG_SCHED_BORE=y

CONFIG_MODULE_COMPRESS_ZSTD=y
# CONFIG_MODULE_COMPRESS_XZ is not set

CONFIG_LRU_GEN=y
CONFIG_LRU_GEN_ENABLED=y

# CONFIG_GENERIC_CPU is not set
# [BEDROCK DECISION] Frankenstein O3+LTO: Il Capo Ingegnere comanda.
CONFIG_CC_OPTIMIZE_FOR_PERFORMANCE_O3=y
# CONFIG_CC_OPTIMIZE_FOR_PERFORMANCE is not set

# [BEDROCK FIX] ThinLTO Riattivato in modalità Furia.
CONFIG_LTO_CLANG_THIN=y
CONFIG_LTO_CLANG=y
CONFIG_LTO=y

CONFIG_AUTOFDO_CLANG=y

# [FRANKENSTEIN UNCHAINED]
# Nessuna scusa. Togliamo di mezzo l'Ispettore (Objtool) e i Warning fatali.
CONFIG_WERROR=n
CONFIG_STACK_VALIDATION=n
CONFIG_OBJTOOL=n
CONFIG_UNWINDER_ORC=n
CONFIG_UNWINDER_FRAME_POINTER=y
CONFIG_UNWINDER_GUESS=y

CONFIG_DEBUG_INFO_NONE=y

# [BEDROCK FIX] Massacro di Rete (Scelta A): Rimozione moduli Datacenter per compatibilità ThinLTO
# CONFIG_NET_VENDOR_MELLANOX is not set
# CONFIG_NET_VENDOR_SOLARFLARE is not set
# CONFIG_NET_VENDOR_CHELSIO is not set
# CONFIG_NET_VENDOR_CISCO is not set
# CONFIG_NET_VENDOR_QLOGIC is not set
# CONFIG_NET_VENDOR_PENSANDO is not set
# CONFIG_NET_VENDOR_AMAZON is not set
# CONFIG_NET_VENDOR_GOOGLE is not set
# CONFIG_NET_VENDOR_HUAWEI is not set
# CONFIG_NET_VENDOR_NETRONOME is not set
# CONFIG_NET_VENDOR_CAVIUM is not set
# CONFIG_NET_VENDOR_MICROCHIP is not set

CONFIG_NTSYNC=y
CONFIG_RUST=y

# --- ERMETE FORGE: PGO QEMU 9PFS BOOT ---
CONFIG_VIRTIO_PCI=y
CONFIG_VIRTIO_CONSOLE=y
CONFIG_NET_9P=y
CONFIG_NET_9P_VIRTIO=y
CONFIG_9P_FS=y
# CONFIG_DRM_NOUVEAU is not set

# --- ERMETE FORGE: 64 PILASTRI KSPP HARDENING ---
# [BEDROCK DISARM] Rimosse le feature draconiane (INIT_ON_ALLOC, RANDOM_FREELIST) per liberare CPU/RAM.
CONFIG_FORTIFY_SOURCE=y
CONFIG_RANDOMIZE_BASE=y
CONFIG_RANDOMIZE_MEMORY=y
CONFIG_PAGE_TABLE_ISOLATION=y
CONFIG_BPF_UNPRIV_DEFAULT_OFF=y
CONFIG_SECURITY_DMESG_RESTRICT=y
CONFIG_LEGACY_VSYSCALL_NONE=y
CONFIG_STRICT_DEVMEM=y
CONFIG_IO_STRICT_DEVMEM=y
CONFIG_BUG_ON_DATA_CORRUPTION=y
CONFIG_SCHED_STACK_END_CHECK=y
CONFIG_PANIC_ON_OOPS=y
CONFIG_SECURITY_YAMA=y
# Rimosso LOCKDOWN invasivo per compatibilità massima con tool avanzati di power-user.
BEDROCK_CFG

for conf in SOURCES/kernel-x86_64*.config; do
    cat SOURCES/ermete-bedrock.cfg >> "$conf"
done


echo ">>> Generazione ~/.rpmmacros locale esclusivo per KERNEL..."
if [ -f ../../config/rpmmacros ]; then
    cat ../../config/rpmmacros > ~/.rpmmacros
elif [ -f config/rpmmacros ]; then
    cat config/rpmmacros > ~/.rpmmacros
fi

cat << 'EOF' >> ~/.rpmmacros
%_with_vanilla 1
%buildid .chimera
%toolchain clang
%__make /usr/bin/make LLVM=1 LLVM_IAS=1
%__cc clang
%__cxx clang++
%_build_cc clang
%_build_cxx clang++
%_host_cc clang
%_host_cxx clang++
%_ld ld.lld
%_ldflags -Wl,-O2 -Wl,--as-needed -Wl,--sort-common -Wl,-z,now -Wl,-z,relro
%optflags %{__global_compiler_flags} -O3 -march=x86-64-v3 -pipe -Wno-error -Wno-unknown-warning-option
%kcflags -O3 -march=x86-64-v3 -pipe -Wno-error -Wno-unknown-warning-option

%_without_selftests 1
%_without_tools 1
%_without_perf 1
%_without_libperf 1
%_without_ynl 1
%_without_bpftool 1
%_without_debug 1
%_without_debuginfo 1
%_without_doc 1
%_binary_payload w1.zstdio
%_source_payload w1.zstdio
EOF

echo ">>> Esecuzione rpmbuild -bp per scompattare, applicare patch e validare l'albero..."
spectool -g -R SPECS/kernel.spec
dnf builddep -y SPECS/kernel.spec
export LLVM=1
export MAKEFLAGS="LLVM=1 LLVM_IAS=1"
export KCFLAGS="-Wno-unknown-warning-option"
export KBUILD_CFLAGS="-Wno-unknown-warning-option"
rpmbuild -bp SPECS/kernel.spec --target x86_64

echo ">>> Rilevamento della directory di build del kernel preparata..."
KERNEL_BUILD_DIR=$(find "$WORKSPACE_DIR/BUILD" -maxdepth 6 -name "Makefile" -exec grep -l "^VERSION =" {} + 2>/dev/null | head -n 1 | xargs -r dirname)
if [ -z "$KERNEL_BUILD_DIR" ]; then
    echo "ERRORE FATALE: Directory di build del kernel non trovata dopo rpmbuild -bp!"
    exit 1
fi
REL_DIR=$(realpath --relative-to="$WORKSPACE_DIR/BUILD" "$KERNEL_BUILD_DIR")
echo "$REL_DIR" > "$WORKSPACE_DIR/BUILD/.kernel_version"
echo ">>> Albero del kernel preparato e registrato in BUILD/.kernel_version: $REL_DIR"



echo "========================================================="
echo " PREPARAZIONE CHIMERA COMPLETATA."
echo "========================================================="
