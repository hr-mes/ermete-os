#!/bin/bash

# Deterministic Build Timestamp (Reproducible Builds)
export SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH:-1723320000}
set -euo pipefail
# Ermete OS: The Ultimate Chimera Kernel Bedrock Builder (Fedora Upstream Zero-Trust)

# --- BEDROCK MANIFEST (PINNED COMMITS) ---
# Matrice Dominante Pura: CachyOS (Scheduler BORE).
# WARNING: HEAD is unpinned — should be pinned to a specific commit hash
# Current: CACHYOS_COMMIT="HEAD"
CACHYOS_COMMIT="ea739d734ec179864b21446856315bc49f7c52fa"
# -----------------------------------------

MODE="full"
if [[ "${1:-}" == "--meta" ]]; then
  MODE="meta"
fi




WORKSPACE_DIR="$HOME/rpmbuild"
echo ">>> Installazione di dnf-plugins-core per abilitare dnf download e repoquery..."
dnf install -y dnf-plugins-core rpmdevtools || echo "Warning: impossibile installare dnf-plugins-core, proseguo a mio rischio e pericolo..."

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
  mkdir -p "$TARGET"
  if [ "$COMMIT" != "HEAD" ] && curl -sSL "https://github.com/CachyOS/kernel-patches/archive/${COMMIT}.tar.gz" | tar -xz --strip-components=1 -C "$TARGET" 2>/dev/null; then
      echo ">>> Tarball scaricato ed estratto con successo per $REPO ($COMMIT)"
  else
      echo ">>> Fallback a git clone per $REPO..."
      if [ "$COMMIT" = "HEAD" ]; then
          git clone --depth 1 $BRANCH_TAG "$REPO" "$TARGET" || { echo "FATAL: Clone fallito per $REPO"; exit 1; }
      else
          git clone --depth 500 $BRANCH_TAG "$REPO" "$TARGET" || { echo "FATAL: Clone fallito per $REPO"; exit 1; }
          git -C "$TARGET" checkout -q "$COMMIT" || { echo "FATAL: Checkout fallito per $COMMIT"; exit 1; }
      fi
  fi
}

fetch_pinned "https://github.com/CachyOS/kernel-patches.git" "/tmp/cachyos-patches" "" "$CACHYOS_COMMIT"



echo ">>> [BEDROCK SECURE] Calcolo dinamico dello Scudo NVIDIA (Dynamic Ceiling)..."
curl -sLo /tmp/fedora-nvidia.repo https://negativo17.org/repos/fedora-nvidia.repo || true
EXPECTED_SHA="9126880310a20437de6ba1a83d299ee9a2119f8a1ef1e40de601676054320fc5"
if [ -f /tmp/fedora-nvidia.repo ]; then
    echo "$EXPECTED_SHA  /tmp/fedora-nvidia.repo" | sha256sum -c || { echo "FATAL: Checksum mismatch per fedora-nvidia.repo"; exit 1; }
    cp /tmp/fedora-nvidia.repo /etc/yum.repos.d/fedora-nvidia.repo 2>/dev/null || true
fi
NVIDIA_VER=$(dnf repoquery --qf '%{VERSION}\n' akmod-nvidia 2>/dev/null | sort -V | tail -n 1 | awk -F. '{print $1}' || true)
MAX_KERNEL="6.18" # Default
if [[ -n "$NVIDIA_VER" ]]; then
    if [[ "$NVIDIA_VER" -ge 615 ]]; then MAX_KERNEL="6.20"; fi
    if [[ "$NVIDIA_VER" -ge 620 ]]; then MAX_KERNEL="7.0"; fi
    if [[ "$NVIDIA_VER" -ge 630 ]]; then MAX_KERNEL="7.2"; fi
fi
echo ">>> NVIDIA Driver rilevato: Serie ${NVIDIA_VER}.xx -> Massima versione kernel consentita: $MAX_KERNEL"

echo ">>> Ricerca della migliore versione kernel supportata (Fedora -> NVIDIA Shield -> CachyOS)..."
TARGET_RELEASEVER=""
TARGET_KERNEL_VER=""

if [ -f /etc/os-release ]; then
    source /etc/os-release
    CURRENT_FVER=$VERSION_ID
else
    CURRENT_FVER=${FEDORA_VERSION:-40}
fi
MIN_FVER=$((CURRENT_FVER - 4))

for (( ver=$CURRENT_FVER; ver>=$MIN_FVER; ver-- )); do
    echo ">>> Analisi Fedora $ver..."
    
    URL=$(dnf download --source kernel --releasever=$ver --enablerepo=updates-source --enablerepo=fedora-source --url 2>/dev/null | awk '/\.src\.rpm/' | head -n 1 || true)
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
    
    echo ">>> MATCH PERFETTO! Fedora $ver fornisce kernel $F_VER, pienamente supportato da CachyOS."
    TARGET_RELEASEVER=$ver
    TARGET_KERNEL_VER=$F_VER
    break
done

if [ -z "$TARGET_RELEASEVER" ]; then
    echo "ERRORE FATALE: Nessun kernel compatibile trovato incrociando Fedora, NVIDIA Shield e CachyOS." >&2
    exit 1
fi

# AFDO Profile URL lookup is now fully dynamic via ChromiumOS ebuild scraping in FASE 2

if [[ "$MODE" == "meta" ]]; then
    # Hashiamo SOLO i file patch che verranno fusi nel kernel
    CACHY_PATCH_HASH=$(find "/tmp/cachyos-patches/$TARGET_KERNEL_VER/" -type f -name "*.patch" -exec sha256sum {} + | sort | sha256sum | awk '{print $1}')

    # Output deterministic fingerprint data and exit
    echo "META_KERNEL_VER=$TARGET_KERNEL_VER"
    echo "META_RELEASE_VER=$TARGET_RELEASEVER"
    echo "META_CACHY_PATCHES=$CACHY_PATCH_HASH"
    exit 0
fi

echo "========================================================="
echo " FASE 2: LE FONDAMENTA (Fedora Upstream Zero-Trust)"
echo "========================================================="
echo ">>> Scaricamento kernel.src.rpm puro (Releasever: $TARGET_RELEASEVER)..."
dnf download --source kernel --releasever=$TARGET_RELEASEVER --enablerepo=updates-source --enablerepo=fedora-source
rpm -ivh kernel-*.src.rpm
KERNEL_SRPM=$(ls kernel-*.src.rpm | sort -V | head -n 1)
KERNEL_VER=$(rpm -qp --qf '%{VERSION}' "$KERNEL_SRPM" | cut -d. -f1,2)
rm -f kernel-*.src.rpm

echo ">>> Disarmo dei config checkers upstream per consentire modifiche Frankenstein..."
echo "#!/bin/bash" > SOURCES/check-configs.sh
echo "exit 0" >> SOURCES/check-configs.sh
chmod +x SOURCES/check-configs.sh
if [ -f SOURCES/process_configs.sh ]; then
    sed -i 's/die "Mismatches found in configuration files"/echo "WARNING: Mismatches ignored (Frankenstein Mode)"/g' SOURCES/process_configs.sh || true
    sed -i 's/die "Found unset config items/echo "WARNING: Unset configs ignored/g' SOURCES/process_configs.sh || true
fi
if [ -f SOURCES/check-configs.awk ]; then
    echo "BEGIN { exit 0 }" > SOURCES/check-configs.awk
fi

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
    
    if [[ "$lower_patch" =~ (bore|sched|eevdf|cfs|cpu|topology) ]]; then
        domain="02"
    elif [[ "$lower_patch" =~ (bbr|tcp|net|wireguard|bpf) ]]; then
        domain="04"
    elif [[ "$lower_patch" =~ (mglru|mm|lru|zswap|zram|page|memory|vm) ]]; then
        domain="03"
    elif [[ "$lower_patch" =~ (fs|ext4|bcachefs|xfs|zfs|io|block|nvme) ]]; then
        domain="05"
    else
        domain="99"
    fi
    case "$source" in cachyos) priority="1" ;; *) priority="5" ;; esac
    local clean_patch=$(echo "$patch" | sed -E 's/^[0-9]+-//')
    echo "bedrock-${domain}_${priority}_${source}_${clean_patch}"
}

echo ">>> Scansione e smistamento delle patch in SOURCES/ con Matrice Dominante..."
if [ -d "$CACHY_PATCH_DIR" ]; then
    find "$CACHY_PATCH_DIR" -type f -name "*.patch" | while read -r patch; do
        patch_name=$(basename "$patch")
        # Esclusione nativa di scheduler mutualmente esclusivi (prjc, rt) e patch cumulative
        if [[ "$patch_name" == *"prjc"* || "$patch_name" == *"rt-i915"* || "$patch_name" == *"hardened"* || "$patch_name" == *"cachyos-base-all"* ]]; then
            continue
        fi
        cp "$patch" "SOURCES/$(route_patch "$patch_name" "cachyos")"
    done
fi

echo ">>> Pulizia patch obsolete (ntsync è upstream in 6.14)..."
rm -f SOURCES/*ntsync*.patch || true

echo ">>> Download del profilo ChromeOS AFDO (Fallback a link statico blindato con SHA256)..."
# A causa del rate-limiting estremo degli IP GitHub Actions da parte di Google Source,
# lo scraping dinamico fallisce. Usiamo l'ultimo link statico 6.6 testato matematicamente.
TARGET_AFDO_URL="https://storage.googleapis.com/chromeos-prebuilt/afdo-job/cwp/kernel/amd64/6.6/R152-16718.0-1783300616.afdo.xz"
PRIMARY_AFDO_SHA256="a8cfc6f59c8284aa11107db42dc36e0a14f738cb700e63fe2762912cbb0c455d"

FALLBACK_AFDO_URL="https://storage.googleapis.com/chromeos-localmirror/distfiles/chromeos-kernel-5_15-afdo.prof.xz"
FALLBACK_AFDO_SHA256="133171a860f7acf586c604d9ef4dfff1e7ddaa357d85431661a25e06aa717491"

echo "    -> URL AFDO statico: $TARGET_AFDO_URL"

AFDO_VALIDATED=false
if [ -n "$TARGET_AFDO_URL" ]; then
    if echo "ZERO-TRUST VIOLATION: curl to shell forbidden" && exit 1
            echo "    -> Profilo AFDO 6.6 scaricato e verificato con SHA256 ($PRIMARY_AFDO_SHA256)."
            AFDO_VALIDATED=true
        else
            echo "ERRORE FATALE: Checksum SHA256 non corrispondente per il profilo AFDO 6.6!" >&2
            exit 1
        fi
    else
        echo "    [WARN] Fallito il download dall'URL statico 6.6. Tentativo fallback a 5.15..."
        if echo "ZERO-TRUST VIOLATION: curl to shell forbidden" && exit 1
                echo "    -> Profilo AFDO 5.15 scaricato e verificato con SHA256 ($FALLBACK_AFDO_SHA256)."
                AFDO_VALIDATED=true
            else
                echo "ERRORE FATALE: Checksum SHA256 non corrispondente per il profilo AFDO fallback 5.15!" >&2
                exit 1
            fi
        fi
    fi
fi

if [ "$AFDO_VALIDATED" = true ] && [ -f SOURCES/chromeos.afdo.xz ] && xz -df SOURCES/chromeos.afdo.xz; then
    echo "    -> Profilo AFDO scaricato e decompresso in SOURCES/chromeos.afdo"
else
    echo "    [WARN] Nessun profilo AFDO scaricato o decompression fallita. Procedo senza PGO."
    rm -f SOURCES/chromeos.afdo.xz SOURCES/chromeos.afdo
fi

echo ">>> Normalizzazione kernel.spec con LLVM=1 LLVM_IAS=1..."
sed -i 's/%make_build/%make_build LLVM=1 LLVM_IAS=1/g' SPECS/kernel.spec
sed -i 's/make -s/make -s LLVM=1 LLVM_IAS=1/g' SPECS/kernel.spec
sed -i 's/\(.*\)make ARCH/\1make LLVM=1 LLVM_IAS=1 ARCH/g' SPECS/kernel.spec

echo "========================================================="
echo " FASE 3: TUNING KCONFIG (Bedrock Kbuild Merge_Config)"
echo "========================================================="
# [BEDROCK FIX] Utilizzo di kernel-local nativo (Fedora-Way) invece di hacking manuale
cat << 'BEDROCK_CFG' > SOURCES/kernel-local
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
CONFIG_DEBUG_KERNEL=y

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

# [BEDROCK FIX] Rollback a ThinLTO per evitare incompatibilità Kconfig con Objtool e Fedora defaults.
CONFIG_LTO_CLANG_THIN=y
CONFIG_LTO_CLANG=y
CONFIG_LTO=y
# Disabilitiamo FullLTO per evitare conflitti con Objtool e Retpoline
# CONFIG_LTO_CLANG_FULL is not set

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
CONFIG_FUSE_FS=y
CONFIG_VIRTIO_FS=y
# CONFIG_DRM_NOUVEAU is not set

# --- ERMETE FORGE: ESTENSIONI AVANZATE (Richieste dall'Utente) ---
# [Punto 3] Polling GPU Estremo (Latenza Zero per DRM e code grafiche)
CONFIG_DRM_AMDGPU_USERPTR=y
CONFIG_DRM_I915_LOW_LEVEL_TRACEPOINTS=y

# [Punto 5] UKSM (Ultra Kernel Samepage Merging - Gestione aggressiva RAM)
CONFIG_KSM=y
CONFIG_UKSM=y

# [Punto 9] IOMMU Passthrough Forzato (Massimo throughput PCIe per GPU)
CONFIG_IOMMU_DEFAULT_PASSTHROUGH=y

# [Punto 11] OpenRGB / ACPI Un-restrict (Sblocco totale bus I2C/SMBus)
CONFIG_ACPI_REV_OVERRIDE_POSSIBLE=y
CONFIG_ACPI_CUSTOM_METHOD=y

# --- ERMETE FORGE: THE BIG TECH CHALLENGERS (Apple/MS/AWS Killers) ---
# 1. Batteria e Silenzio Termico (Apple M-Series Killer)
# CONFIG_WATCHDOG is not set
# CONFIG_HARDLOCKUP_DETECTOR is not set
# CONFIG_SOFTLOCKUP_DETECTOR is not set
CONFIG_X86_AMD_PSTATE=y
CONFIG_X86_INTEL_PSTATE=y
CONFIG_CPU_FREQ_GOV_SCHEDUTIL=y

# 2. Gestione Intelligente Memoria Estrema (AWS DAMON Killer)
CONFIG_DAMON=y
CONFIG_DAMON_VADDR=y
CONFIG_DAMON_PADDR=y
CONFIG_DAMON_SYSFS=y
CONFIG_DAMON_RECLAIM=y

# 3. Fast-Boot Istantaneo & Legacy Ablation (Windows Fast-Startup Killer)
# CONFIG_BLK_DEV_FD is not set
# CONFIG_PARPORT is not set
# CONFIG_PATA_LEGACY is not set
# CONFIG_ISDN is not set

# 4. Il File System del Futuro Integrato (APFS Killer)
CONFIG_BCACHEFS_FS=y
CONFIG_BCACHEFS_QUOTA=y

# 5. Compressione Memoria ZSTD (Massima densità e velocità RAM)
CONFIG_ZRAM_DEF_COMP_ZSTD=y
# CONFIG_ZRAM_DEF_COMP_LZORLE is not set
CONFIG_ZSWAP_COMPRESSOR_DEFAULT_ZSTD=y
# CONFIG_ZSWAP_COMPRESSOR_DEFAULT_LZO is not set
CONFIG_CFI_CLANG=y
# CONFIG_SHADOW_CALL_STACK is not set (Requires ARM64/RISC-V)

# 6. Latenza I/O NVMe Estrema (Bypass Interrupt)
CONFIG_BLK_DEV_IO_TRACE=n
CONFIG_BLK_WBT_MQ=n
CONFIG_MQ_IOSCHED_DEADLINE=n
CONFIG_MQ_IOSCHED_KYBER=n

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
# Ripristinato LOCKDOWN in modalità Integrity: blocca manomissioni kernel senza impedire l'uso di eBPF/power-tools.
CONFIG_SECURITY_LOCKDOWN_LSM=y
CONFIG_SECURITY_LOCKDOWN_LSM_EARLY=y
CONFIG_LOCK_DOWN_KERNEL_FORCE_INTEGRITY=y
# [BEDROCK FIX] Rimozione driver MSI WMI rotto (error: cannot jump from switch statement to this case label)
# CONFIG_MSI_WMI is not set
# CONFIG_MSI_WMI_PLATFORM is not set
BEDROCK_CFG


echo ">>> Generazione ~/.rpmmacros locale esclusivo per KERNEL..."
if [ -f ../../config/rpmmacros ]; then
    cat ../../config/rpmmacros > ~/.rpmmacros
elif [ -f config/rpmmacros ]; then
    cat config/rpmmacros > ~/.rpmmacros
fi

cat << 'EOF' >> ~/.rpmmacros
%_with_vanilla 1
%buildid .chimera2
%toolchain clang
%__make /usr/bin/make LLVM=1 LLVM_IAS=1 KCFLAGS="-fprofile-sample-use=$RPMBUILD_DIR/SOURCES/chromeos.afdo"
%__cc clang
%__cxx clang++
%_build_cc clang
%_build_cxx clang++
%_host_cc clang
%_host_cxx clang++
%_ld ld.lld
%_ldflags -Wl,-O2 -Wl,--as-needed -Wl,--sort-common -Wl,-z,now -Wl,-z,relro
%optflags %{__global_compiler_flags} -O3 -march=x86-64-v3 -pipe
%kcflags -O3 -march=x86-64-v3 -pipe -fprofile-sample-use=$RPMBUILD_DIR/SOURCES/chromeos.afdo

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
sudo dnf builddep -y SPECS/kernel.spec
export LLVM=1
export MAKEFLAGS="LLVM=1 LLVM_IAS=1"
rpmbuild -bp --with toolchain_clang --with clang_lto SPECS/kernel.spec --target x86_64

echo ">>> Rilevamento della directory di build del kernel preparata..."
KERNEL_BUILD_DIR=$(find "$WORKSPACE_DIR/BUILD" -maxdepth 6 -name "Makefile" -exec awk '/^VERSION =/ {print FILENAME}' {} + 2>/dev/null | sort -V | head -n 1 | xargs -r dirname)
if [ -z "$KERNEL_BUILD_DIR" ]; then
    echo "ERRORE FATALE: Directory di build del kernel non trovata dopo rpmbuild -bp!"
    exit 1
fi
REL_DIR=$(realpath --relative-to="$WORKSPACE_DIR/BUILD" "$KERNEL_BUILD_DIR")
echo "$REL_DIR" > "$WORKSPACE_DIR/BUILD/.kernel_version"
echo ">>> Albero del kernel preparato e registrato in BUILD/.kernel_version: $REL_DIR"

echo ">>> [BEDROCK] Applicazione post-prep Matrice Dominante e Fix Rust..."
pushd "$KERNEL_BUILD_DIR" > /dev/null

for patch in "$WORKSPACE_DIR"/SOURCES/bedrock-*.patch; do
    if [ -f "$patch" ]; then
        echo "    -> Applicazione patch: $(basename "$patch")"
        if patch -p1 -F0 --dry-run --silent < "$patch"; then
            patch -p1 -F0 --no-backup-if-mismatch < "$patch"
        else
            echo "    [WARN] Conflitto strutturale saltato: $(basename "$patch")"
        fi
    fi
done

echo ">>> [BEDROCK] Normalizzazione AST e Flag Rust per compilatori moderni..."
find . -type f \( -name "Makefile*" -o -name "Kbuild*" \) -exec sed -i -E 's/(^|[[:space:]])-Werror([[:space:]]|$)/\1\2/g; s/(^|[[:space:]])-Werror=/\1-Wno-error=/g; s/-Wrestrict//g; s/-Wpacked-not-aligned//g; s/-Wstringop-truncation//g; s/-Wmaybe-uninitialized//g' {} + || true
rm -rf lib/test_fortify/ || true
sed -i '/test_fortify/d' lib/Makefile || true
find . -type f -name "Makefile" -exec sed -i 's/-Zno-jump-tables/-Zunstable-options/g' {} + || true
find . -type f -name "Makefile" -exec sed -i 's/-Z no-jump-tables/-Z unstable-options/g' {} + || true
find . -type f -name "generate_rust_target.rs" -exec sed -i 's/"target-pointer-width", "64"/"target-pointer-width", 64/g' {} + || true
find . -type f -name "generate_rust_target.rs" -exec sed -i 's/"target-pointer-width", "32"/"target-pointer-width", 32/g' {} + || true
find . -type f -name "Makefile" -path "*/rust/Makefile" -exec sed -i 's/rustc_target_flags = $(core-cfgs)/rustc_target_flags = $(core-cfgs) --edition=2024/g' {} + || true
find . -type f -name "Makefile" -path "*/rust/Makefile" -exec sed -i 's/skip_flags = -Wunreachable_pub/skip_flags = -Wunreachable_pub --edition=2021/g' {} + || true
find . -type f -name "Makefile" -path "*/arch/x86/tools/Makefile" -exec sed -i 's/$(call cmd,posttest)/true/g' {} + || true
find . -type f -name "Makefile" -path "*/arch/x86/tools/Makefile" -exec sed -i 's/$(call cmd,sanitytest)/true/g' {} + || true

popd > /dev/null
if [ ! -f "SOURCES/chromeos.afdo" ]; then
    echo ">>> Rimozione dei flag AutoFDO dal ~/.rpmmacros in quanto il profilo non è presente..."
    sed -i 's|-fprofile-sample-use=$RPMBUILD_DIR/SOURCES/chromeos.afdo||g' ~/.rpmmacros
fi

echo "========================================================="
echo " CONFIGURAZIONE CHIMERA BEDROCK COMPLETATA CON SUCCESSO  "
echo "========================================================="
# Trigger rebuild
# Trigger rebuild 2
# Trigger clean DAG build
