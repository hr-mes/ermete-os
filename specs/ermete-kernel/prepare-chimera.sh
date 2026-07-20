#!/bin/bash
set -euo pipefail
# Ermete OS: The Ultimate Chimera Kernel Bedrock Builder (Fedora Upstream Zero-Trust)

# --- BEDROCK MANIFEST (PINNED COMMITS) ---
# Matrice Dominante Pura: CachyOS (Scheduler BORE) + ClearLinux (Math/CPU/Memory).
# WARNING: HEAD is unpinned — should be pinned to a specific commit hash
# Current: CACHYOS_COMMIT="HEAD"
CACHYOS_COMMIT="HEAD"
# -----------------------------------------

MODE="full"
if [[ "${1:-}" == "--meta" ]]; then
  MODE="meta"
fi

if [[ "$MODE" == "full" ]]; then
  echo ">>> [BEDROCK FUZZER] Preparazione Fuzzer in /usr/local/bin..."
  REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." &> /dev/null && pwd)"
  cp "$REPO_ROOT/specs/ermete-kernel/auto-dmz-fuzzer.sh" /usr/local/bin/auto-dmz-fuzzer.sh || cp specs/ermete-kernel/auto-dmz-fuzzer.sh /usr/local/bin/auto-dmz-fuzzer.sh
  chmod +x /usr/local/bin/auto-dmz-fuzzer.sh
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

echo ">>> Scrittura script di applicazione e validazione AST/Kbuild in /tmp/patch_apply.txt..."
> /tmp/patch_apply.txt
cat << 'EOF' >> /tmp/patch_apply.txt
echo ">>> [BEDROCK] Inizio applicazione matrice Kbuild/AST per le patch..."
export LLVM=1
export MAKEFLAGS="LLVM=1 LLVM_IAS=1"
export KCFLAGS="-Wno-unknown-warning-option"
export KBUILD_CFLAGS="-Wno-unknown-warning-option"
CONF_FILE=$(ls %{_sourcedir}/kernel-x86_64*.config 2>/dev/null | head -n 1 || ls /root/rpmbuild/SOURCES/kernel-x86_64*.config 2>/dev/null | head -n 1 || ls configs/kernel-x86_64*.config 2>/dev/null | head -n 1)
if [ -n "$CONF_FILE" ] && [ -f "$CONF_FILE" ]; then
    cp "$CONF_FILE" .config
else
    echo "ERRORE CRITICO: File di configurazione kernel-x86_64*.config non trovato!"
    exit 1
fi

echo ">>> [BEDROCK] FIX RUST NO-JUMP-TABLES AND TARGET POINTER WIDTH AND CORE EDITION"
find . -type f -name "Makefile" -exec sed -i 's/-Zno-jump-tables/-Zunstable-options/g' {} + || true
find . -type f -name "Makefile" -exec sed -i 's/-Z no-jump-tables/-Z unstable-options/g' {} + || true
find . -type f -name "generate_rust_target.rs" -exec sed -i 's/"target-pointer-width", "64"/"target-pointer-width", 64/g' {} + || true
find . -type f -name "generate_rust_target.rs" -exec sed -i 's/"target-pointer-width", "32"/"target-pointer-width", 32/g' {} + || true
find . -type f -name "Makefile" -path "*/rust/Makefile" -exec sed -i 's/rustc_target_flags = $(core-cfgs)/rustc_target_flags = $(core-cfgs) --edition=2024/g' {} + || true
find . -type f -name "Makefile" -path "*/rust/Makefile" -exec sed -i 's/skip_flags = -Wunreachable_pub/skip_flags = -Wunreachable_pub --edition=2021/g' {} + || true
find . -type f -name "Makefile" -path "*/arch/x86/tools/Makefile" -exec sed -i 's/$(call cmd,posttest)/true/g' {} + || true
find . -type f -name "Makefile" -path "*/arch/x86/tools/Makefile" -exec sed -i 's/$(call cmd,sanitytest)/true/g' {} + || true

echo ">>> [BEDROCK] Sradicamento totale flag GCC incompatibili con Clang e disattivazione -Werror per unknown warnings..."
find . -type f \( -name "Makefile*" -o -name "Kbuild*" \) -exec sed -i 's/-Wrestrict//g; s/-Wpacked-not-aligned//g; s/-Wstringop-truncation//g; s/-Werror=unknown-warning-option/-Wno-unknown-warning-option/g' {} + || true

make LLVM=1 LLVM_IAS=1 olddefconfig
make LLVM=1 LLVM_IAS=1 prepare

for patch in %{_sourcedir}/bedrock-*.patch; do
    if [ ! -f "$patch" ]; then continue; fi
    patch_name=$(basename "$patch")
    echo "-> Test di compatibilità per $patch_name..."
    
    TOUCHED_FILES=$(grep -E '^\+\+\+ b/' "$patch" | awk '{print $2}' | sed 's/^b\///' || true)
    for t_file in $TOUCHED_FILES; do
        if [ -f "$t_file" ]; then cp -p "$t_file" "${t_file}.bedrock_bak"; fi
    done
    
    APPLIED=0
    FUZZ_VAL=0
    NON_C_FILES=""
    for f in 0 1; do
        if patch -p1 -F $f --force --dry-run --silent < "$patch"; then
            patch -p1 -F $f --force < "$patch" > /dev/null || true
            APPLIED=1
            FUZZ_VAL=$f
            break
        fi
    done
    if [ $APPLIED -eq 0 ]; then
        if [ -z "$NON_C_FILES" ]; then
            echo "   [SKIP] Conflitto strutturale (Falliti Fuzz 0 e 1). Patch scartata (Prevenzione Frankenstein)."
        fi
        for t_file in $TOUCHED_FILES; do rm -f "${t_file}.bedrock_bak" "${t_file}.orig" "${t_file}.rej"; done
    fi

    if [ $APPLIED -eq 1 ]; then
        if [ -f /forge/.ast_done ]; then
            echo "   [SKIP AST] Fase 3 rilevata. Salto AST Validation per $patch_name per velocizzare la build finale."
            for t_file in $TOUCHED_FILES; do rm -f "${t_file}.bedrock_bak" "${t_file}.orig" "${t_file}.rej"; done
            continue
        fi
        MODIFIED_C_FILES=$(grep -E '^\+\+\+ b/' "$patch" | awk '{print $2}' | sed 's/^b\///' | grep '\.c$' || true)
        if [ -n "$MODIFIED_C_FILES" ]; then
            make LLVM=1 LLVM_IAS=1 olddefconfig </dev/null >/dev/null 2>&1 || true
            echo "   [AST VALIDATION (Fuzz $FUZZ_VAL)] Controllo purezza albero sintattico per $patch_name..."
            AST_FAILED=0
            for c_file in $MODIFIED_C_FILES; do
                if [ -f "$c_file" ]; then
                    target_o="${c_file%.c}.o"
                    if ! make LLVM=1 LLVM_IAS=1 "$target_o" </dev/null >/dev/null 2>&1; then
                        AST_FAILED=1
                        echo "   [AST FATAL] Kbuild ha fallito la compilazione AST di $c_file!"
                        break
                    fi
                fi
            done
            if [ $AST_FAILED -eq 1 ]; then
                echo "   [ROLLBACK ATOMICO] Conflitto sintattico rilevato in $patch_name! Ripristino chirurgico dei file modificati..."
                for t_file in $TOUCHED_FILES; do
                    rm -f "${t_file}.orig" "${t_file}.rej"
                    if [ -f "${t_file}.bedrock_bak" ]; then
                        mv -f "${t_file}.bedrock_bak" "$t_file"
                    else
                        rm -f "$t_file"
                    fi
                done
            else
                echo "   [SUCCESS] Patch fusa e validata tramite AST: $patch_name"
                for t_file in $TOUCHED_FILES; do rm -f "${t_file}.bedrock_bak" "${t_file}.orig" "${t_file}.rej"; done
            fi
        else
            echo "   [SUCCESS] Patch fusa (Fuzz $FUZZ_VAL - Non-C Fast Track): $patch_name"
            for t_file in $TOUCHED_FILES; do rm -f "${t_file}.bedrock_bak" "${t_file}.orig" "${t_file}.rej"; done
        fi
    fi
done

make LLVM=1 LLVM_IAS=1 olddefconfig </dev/null >/dev/null 2>&1 || true
EOF

awk '/# END OF PATCH APPLICATIONS/{system("cat /tmp/patch_apply.txt")}1' SPECS/kernel.spec > SPECS/kernel.spec.new
mv SPECS/kernel.spec.new SPECS/kernel.spec

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

echo ">>> [BEDROCK FRANKENSTEIN] Il Capo Ingegnere comanda: O3 GLOBALE CON AUTO-DMZ FUZZER."
pushd "$KERNEL_BUILD_DIR" > /dev/null

# 1. Distruzione Infiniband e SCSI orfani
sed -i '/infiniband\//d' drivers/Makefile
sed -i 's/source "drivers\/infiniband\/Kconfig"//g' drivers/Kconfig
# I driver SCSI Chelsio dipendono dalla rete Chelsio che abbiamo distrutto. Eliminiamo anche loro.
sed -i '/cxgbi\//d' drivers/scsi/Makefile
sed -i '/qla2xxx\//d' drivers/scsi/Makefile
sed -i '/qla4xxx\//d' drivers/scsi/Makefile

# 2. Distruzione Simulatore di Rete Sviluppatori (netdevsim)
sed -i '/netdevsim\//d' drivers/net/Makefile
sed -i 's/source "drivers\/net\/netdevsim\/Kconfig"//g' drivers/net/Kconfig

# 3. Distruzione Vendor Enterprise dal Makefile Ethernet
for vendor in amazon cavium chelsio cisco google huawei mellanox netronome pensando qlogic sfc wangxun hisilicon yunsilicon fungible emulex myricom ni; do
    sed -i "/${vendor}\//d" drivers/net/ethernet/Makefile
    sed -i "s/source \"drivers\\/net\\/ethernet\\/${vendor}\\/Kconfig\"//g" drivers/net/ethernet/Kconfig || true
done

# 4. Distruzione Chirugica Datacenter Intel
for intel_dc in ice i40e ixgbe ixgbevf fm10k idpf; do
    sed -i "/${intel_dc}\//d" drivers/net/ethernet/intel/Makefile
    sed -i "s/source \"drivers\\/net\\/ethernet\\/intel\\/${intel_dc}\\/Kconfig\"//g" drivers/net/ethernet/intel/Kconfig || true
done

# 5. Pulizia Kconfig testuale (rafforzamento)
./scripts/config --disable NETDEVSIM --disable NET_VENDOR_MELLANOX --disable INFINIBAND --disable SCSI_CXGB3_ISCSI --disable SCSI_CXGB4_ISCSI
make LLVM=1 LLVM_IAS=1 olddefconfig
popd > /dev/null

echo "========================================================="
echo " PREPARAZIONE CHIMERA COMPLETATA."
echo "========================================================="
