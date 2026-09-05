#!/usr/bin/env bash
# Kernel Ermete: dai pin agli RPM, dentro builder/Containerfile.
# Specifica: docs/architecture/doc_kernel_build.md. Tutto cio' che scarica e' pinnato
# in pins.env, verificato con SOURCES/sources.sha256 e con le firme delle chiavi in
# SOURCES/keys. Ogni controllo che fallisce ferma la build. La fase manifest scarica i
# sorgenti dei pin e scrive il loro manifesto: e' cosi' che il bot di bump (bump.py)
# rigenera SOURCES/sources.sha256. Lo stadio microvm fa il prep e compila solo il
# kernel guest delle MicroVM (sezione 9); build compila entrambi i kernel.
set -euo pipefail

usage() { echo "uso: ${0##*/} --stage manifest|prep|microvm|build --out DIR" >&2; exit 2; }
STAGE='' OUT=''
while [[ $# -gt 0 ]]; do
  case $1 in
    --stage) STAGE=${2:?}; shift 2 ;;
    --out) OUT=${2:?}; shift 2 ;;
    *) usage ;;
  esac
done
[[ ( $STAGE == manifest || $STAGE == prep || $STAGE == microvm || $STAGE == build ) && -n $OUT ]] || usage

die() { echo "build.sh: $*" >&2; exit 1; }
step() { echo; echo ">>> $*"; }

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=pins.env
source "$HERE/pins.env"
CACHE=${ERMETE_KERNEL_CACHE:-/var/cache/ermete-kernel}
TOP=$HOME/rpmbuild
SRC=$TOP/SOURCES
mkdir -p "$CACHE" "$OUT" "$TOP"
# Area di lavoro nuova a ogni esecuzione: il repo git del merge non sopporta un secondo
# giro sullo stesso indice (le patch risultano gia' applicate).
WORK=$(mktemp -d "$TOP/ermete.XXXXXX")

# Nomi e URL derivati dai pin.
SRPM=kernel-$FEDORA_KERNEL_NVR.src.rpm
KOJI=https://kojipkgs.fedoraproject.org/packages/kernel/${FEDORA_KERNEL_NVR%%-*}/${FEDORA_KERNEL_NVR#*-}
# koji pota le copie firmate delle build non piu' recenti ma conserva l'header di
# firma in data/sigcache: ricucito sul SRPM dalla libreria koji da' il file firmato.
SRPM_URL=$KOJI/src/$SRPM
SRPM_SIG_URL=$KOJI/data/sigcache/${FEDORA_KEY_FPR: -8}/src/$SRPM.sig
FEDORA_KEY=/etc/pki/rpm-gpg/RPM-GPG-KEY-fedora-${FEDORA_KERNEL_NVR##*.fc}-primary
KVER=${CACHYOS_RELEASE#cachyos-}; KVER=${KVER%-*}     # cachyos-7.1.8-1 -> 7.1.8
SERIES=$(cut -d. -f1,2 <<< "$KVER")                     # 7.1
CACHY_TAR=$CACHYOS_RELEASE.tar.gz
CACHY_URL=https://github.com/CachyOS/linux/releases/download/$CACHYOS_RELEASE/$CACHY_TAR
VANILLA_TAR=linux-$KVER.tar.xz
VANILLA_SIGN=linux-$KVER.tar.sign
VANILLA_URL=https://cdn.kernel.org/pub/linux/kernel/v${KVER%%.*}.x
CACHY_CONFIG=cachyos-config-${CACHYOS_CONFIG_COMMIT:0:12}
CACHY_CONFIG_URL=https://raw.githubusercontent.com/CachyOS/linux-cachyos/$CACHYOS_CONFIG_COMMIT/linux-cachyos/config
PATCHES_URL=https://raw.githubusercontent.com/CachyOS/kernel-patches/$CACHYOS_PATCHES_COMMIT/$SERIES
mapfile -t PATCHES < <(grep -vE '^\s*(#|$)' "$HERE/patches.list")
mapfile -t ERMETE_PATCHES < <(find "$HERE/patches" -name '*.patch' | sort)
# In cache le patch CachyOS portano il prefisso del commit: lo stesso nome di file torna
# a ogni commit di kernel-patches con contenuto diverso, e la cache e' persistente.
patch_file() { echo "${CACHYOS_PATCHES_COMMIT:0:12}-${1##*/}"; }
mapfile -t FEDORA_WINS < <(grep -vE '^\s*(#|$)' "$HERE/fedora-wins.list")
[[ -z $(printf '%s\n' "${PATCHES[@]##*/}" | sort | uniq -d) ]] || die "patches.list: nomi di file duplicati"

# Le stesse scelte per dnf builddep (--define) e per rpmbuild (--with/--without).
# clang_lto resta acceso anche con LTO spento in kernel-local: e' l'unico bcond con
# cui kernel.spec passa HOSTCC=clang CC=clang LLVM=1 a process_configs.sh, senza il
# quale il config verrebbe valutato con gcc e kCFI sparirebbe.
WITH=(toolchain_clang clang_lto)
WITHOUT=(debug tools perf libperf bpftool ynl selftests doc)
BCONDS=() DEFINES=()
for x in "${WITH[@]}"; do BCONDS+=(--with "$x"); DEFINES+=(--define "_with_$x 1"); done
for x in "${WITHOUT[@]}"; do BCONDS+=(--without "$x"); DEFINES+=(--define "_without_$x 1"); done
MAKE_OPTS=(HOSTCC=clang CC=clang LLVM=1 LLVM_IAS=1)      # %{clang_make_opts} di kernel.spec

# --- sorgenti -----------------------------------------------------------------------

fetch() { # fetch FILE URL: nella cache, una volta sola
  [[ -f $CACHE/$1 ]] && return
  echo "scarico $1"
  curl -fsSL --retry 3 -o "$CACHE/$1.part" "$2" && mv "$CACHE/$1.part" "$CACHE/$1"
}

step "sorgenti pinnate (pins.env)"
fetch "$SRPM" "$SRPM_URL"
fetch "$SRPM.sig" "$SRPM_SIG_URL"
fetch "$CACHY_TAR" "$CACHY_URL"
fetch "$CACHY_TAR.asc" "$CACHY_URL.asc"
fetch "$VANILLA_TAR" "$VANILLA_URL/$VANILLA_TAR"
fetch "$VANILLA_SIGN" "$VANILLA_URL/$VANILLA_SIGN"
fetch "$CACHY_CONFIG" "$CACHY_CONFIG_URL"
for p in "${PATCHES[@]}"; do fetch "$(patch_file "$p")" "$PATCHES_URL/$p"; done

if [[ $STAGE == manifest ]]; then
  # Il manifesto di questi pin, dai file appena scaricati. Le firme le verifica prep,
  # che il bot esegue subito dopo sullo stesso manifesto.
  files=("$SRPM" "$SRPM.sig" "$CACHY_TAR" "$CACHY_TAR.asc" "$VANILLA_TAR" "$VANILLA_SIGN" "$CACHY_CONFIG")
  for p in "${PATCHES[@]}"; do files+=("$(patch_file "$p")"); done
  (cd "$CACHE" && sha256sum "${files[@]}") > "$OUT/sources.sha256"
  echo "manifesto di ${#files[@]} file in $OUT/sources.sha256"
  exit 0
fi

step "hash (SOURCES/sources.sha256)"
(cd "$CACHE" && sha256sum --check --quiet --strict "$HERE/SOURCES/sources.sha256")

verify_gpg() { # verify_gpg KEYDIR SIGNATURE DATA: buona firma di una delle chiavi vendorizzate
  local home
  home=$(mktemp -d)
  gpg --homedir "$home" --batch --quiet --import "$HERE/SOURCES/keys/$1"/*.asc
  gpg --homedir "$home" --batch --status-fd 1 --verify "$2" "$3" 2>/dev/null \
    | grep '^\[GNUPG:\] GOODSIG ' > /dev/null
}

step "firme"
verify_gpg cachyos "$CACHE/$CACHY_TAR.asc" "$CACHE/$CACHY_TAR" || die "firma CachyOS non valida: $CACHY_TAR"
xz -dc "$CACHE/$VANILLA_TAR" | verify_gpg kernel.org "$CACHE/$VANILLA_SIGN" - || die "firma kernel.org non valida: $VANILLA_TAR"
SIGNED_SRPM=$WORK/$SRPM
python3 -c 'import sys, koji; koji.splice_rpm_sighdr(open(sys.argv[1], "rb").read(), sys.argv[2], sys.argv[3])' "$CACHE/$SRPM.sig" "$CACHE/$SRPM" "$SIGNED_SRPM"
rpmkeys --import "$FEDORA_KEY"
rpmkeys --checksig --verbose "$SIGNED_SRPM" | grep "signature, key fingerprint: $FEDORA_KEY_FPR: OK" > /dev/null \
  || die "SRPM non firmato dalla chiave Fedora $FEDORA_KEY_FPR: $SRPM"

# --- albero -------------------------------------------------------------------------

step "kernel.spec e sorgenti Fedora in $TOP"
printf '%%_topdir %s\n%%buildid .ermete\n' "$TOP" > "$HOME/.rpmmacros"
rpm -i "$SIGNED_SRPM"

# Prima della derivazione del config: listnewconfig deve vedere la stessa toolchain di
# rpmbuild (rust-src, bindgen, pahole), altrimenti RUST_IS_AVAILABLE e le opzioni che
# ne dipendono cambiano tra il pre-pass e il gate di Fedora.
step "BuildRequires di kernel.spec"
dnf -y builddep "${DEFINES[@]}" "$TOP/SPECS/kernel.spec"

step "base CachyOS: merge a tre vie tra vanilla, CachyOS e la patch Red Hat, poi patches.list e patches/"
tar -C "$WORK" -xf "$CACHE/$VANILLA_TAR" && mv "$WORK/linux-$KVER" "$WORK/a"
tar -C "$WORK" -xzf "$CACHE/$CACHY_TAR" && mv "$WORK/$CACHYOS_RELEASE" "$WORK/b"
tar -C "$WORK" -xf "$SRC/$VANILLA_TAR" && mv "$WORK/linux-$KVER" "$WORK/fedora-vanilla"
REDHAT_PATCH=("$SRC"/patch-*-redhat.patch)
[[ ${#REDHAT_PATCH[@]} -eq 1 ]] || die "patch Red Hat: attesa una, trovate ${#REDHAT_PATCH[@]}"
# kernel.spec applica la patch Red Hat e poi linux-kernel-test.patch (Patch999999) con
# `git --work-tree=. apply`: il test patch e' il diff tra l'albero Fedora e il merge a
# tre vie (base vanilla) della base CachyOS su quell'albero, cosi' entra per costruzione
# e le aggiunte identiche (backport presenti in entrambi) si fondono da sole. Solo
# plumbing git: l'indice fa da area di lavoro, nessun checkout.
export GIT_AUTHOR_NAME=ermete GIT_AUTHOR_EMAIL=kernel@ermete.os GIT_COMMITTER_NAME=ermete GIT_COMMITTER_EMAIL=kernel@ermete.os
g() { git -C "$WORK/a" "$@"; }
g init -q
g add -Af . && VANILLA=$(g commit-tree -m vanilla "$(g write-tree)")
# Fedora rigenera il tarball con git archive: byte diversi dall'upstream firmato,
# contenuto che deve essere identico. Lo stesso albero git lo prova.
git --git-dir="$WORK/a/.git" -C "$WORK/fedora-vanilla" add -Af .
[[ $(g write-tree) == $(g rev-parse "$VANILLA^{tree}") ]] || die "il tarball nel SRPM non ha il contenuto del vanilla firmato $VANILLA_TAR"
git --git-dir="$WORK/a/.git" -C "$WORK/b" add -Af . && CACHY=$(g commit-tree -p "$VANILLA" -m cachyos "$(g write-tree)")
g read-tree "$VANILLA" && g apply --cached "${REDHAT_PATCH[0]}" && FEDORA=$(g commit-tree -p "$VANILLA" -m redhat "$(g write-tree)")
# shellcheck disable=SC2053  # il confronto con pattern e' voluto
fedora_wins() { local pat; for pat in "${FEDORA_WINS[@]}"; do [[ $1 == $pat ]] && return; done; false; }
if MERGED=$(g merge-tree --write-tree --name-only --no-messages "$FEDORA" "$CACHY"); then
  g read-tree "$MERGED"
else
  mapfile -t CONFLICTS < <(tail -n +2 <<< "$MERGED" | grep -v "^$")
  for path in "${CONFLICTS[@]}"; do
    fedora_wins "$path" || die "conflitto tra base CachyOS e patch Red Hat fuori da fedora-wins.list: $path"
  done
  g read-tree "${MERGED%%$'\n'*}"
  g restore --staged --source="$FEDORA" -- "${CONFLICTS[@]}"
  echo "conflitti risolti con l'albero Fedora (fedora-wins.list): ${CONFLICTS[*]}"
fi
for p in "${PATCHES[@]}"; do
  g apply --cached "$CACHE/$(patch_file "$p")"
  (cd "$WORK/b" && git apply "$CACHE/$(patch_file "$p")")     # l'albero CachyOS serve alla derivazione del config
done
# Le patch di Ermete (patches/), in ordine di nome, dopo quelle di CachyOS.
for p in "${ERMETE_PATCHES[@]}"; do
  g apply --cached "$p"
  (cd "$WORK/b" && git apply "$p")
done
g diff --binary "$FEDORA" "$(g write-tree)" -- . ':!.github' > "$SRC/linux-kernel-test.patch"

# --- config -------------------------------------------------------------------------

step "kernel-local: delta Ermete, poi le opzioni nuove dell'albero con i valori CachyOS"
# Stessa fusione che fa kernel.spec: config Fedora x86_64, frammenti clang e clang_lto,
# kernel-local. Su quel config listnewconfig elenca le opzioni che l'albero introduce.
merged=$WORK/merged.config
cp "$SRC/kernel-x86_64-fedora.config" "$merged"
for snip in "$SRC/partial-clang-snip.config" "$SRC/partial-clang_lto-x86_64-snip.config" "$HERE/kernel-local"; do
  python3 "$SRC/merge.py" "$snip" "$merged" > "$merged.tmp" && mv "$merged.tmp" "$merged"
done
derived=$WORK/derived.config
: > "$derived"
for _ in 1 2 3 4 5; do
  make -s -C "$WORK/b" ARCH=x86_64 "${MAKE_OPTS[@]}" KCONFIG_CONFIG="$merged" listnewconfig > "$WORK/listnew"
  grep '^CONFIG_' "$WORK/listnew" > "$WORK/new" || break
  while IFS= read -r line; do
    name=${line%%=*}
    grep -E "^($name=|# $name is not set)" "$CACHE/$CACHY_CONFIG" || echo "$line"
  done < "$WORK/new" | sed -E 's/^(CONFIG_\w+)=n$/# \1 is not set/' >> "$derived"
  python3 "$SRC/merge.py" "$derived" "$merged" > "$merged.tmp" && mv "$merged.tmp" "$merged"
done
[[ ! -s $WORK/new ]] || die "derivazione del config non convergente: $(head -3 "$WORK/new" | tr '\n' ' ')"
echo "opzioni derivate: $(wc -l < "$derived")"; cat "$derived"
{
  cat "$HERE/kernel-local"
  echo
  echo "# Opzioni introdotte dall'albero, con i valori di $CACHY_CONFIG (derivate da build.sh)."
  cat "$derived"
} > "$SRC/kernel-local"

step "solo x86_64: gli altri config Fedora diventano '# EMPTY' e process_configs.sh li salta"
for f in "$SRC"/kernel-*-fedora.config; do
  [[ $f == */kernel-x86_64-fedora.config ]] || printf '# EMPTY\n' > "$f"
done

check_delta() { # check_delta CONFIG FRAGMENT: ogni riga del frammento deve valere nel config
  local bad=0 line name
  while IFS= read -r line; do
    name=$(grep -oE 'CONFIG_\w+' <<< "$line")
    if [[ $line == CONFIG_* ]]; then
      grep -qxF "$line" "$1" || { echo "  richiesto $line, generato: $(grep -E "^(# )?${name}[= ]" "$1" || echo assente)"; bad=1; }
    else
      ! grep -qE "^$name=" "$1" || { echo "  richiesto $line, generato: $(grep -E "^$name=" "$1")"; bad=1; }
    fi
  done < <(grep -E '^(CONFIG_\w+=|# CONFIG_\w+ is not set)' "$2")
  [[ $bad -eq 0 ]] || die "il config generato non rispetta ${2#"$HERE"/}"
}

# --- rpmbuild -----------------------------------------------------------------------

export KBUILD_BUILD_USER=ermete KBUILD_BUILD_HOST=forge

step "rpmbuild -bp: patch e gate del config (process_configs.sh -w -n -c)"
rpmbuild -bp --target x86_64 "${BCONDS[@]}" "$TOP/SPECS/kernel.spec"
CONFIG=$(find "$TOP/BUILD" -path '*/configs/kernel-*-x86_64.config' -print -quit)
[[ -n $CONFIG ]] || die "config generato non trovato sotto $TOP/BUILD"
check_delta "$CONFIG" "$HERE/kernel-local"
cp "$CONFIG" "$SRC/kernel-local" "$OUT/"

# --- kernel guest MicroVM (sezione 9) ---------------------------------------------------

# Stessa sorgente (l'albero preparato da rpmbuild -bp) e stesso pin, secondo config:
# x86_64_defconfig + kvm_guest.config + microvm/kernel-local, in una directory oggetto
# separata (O=), cosi' l'albero resta pulito per rpmbuild -bb. Il gate del frammento
# (check_delta) gira in ogni stadio, la compilazione solo in microvm e build.
TREE=$(dirname "$(dirname "$CONFIG")")
[[ -f $TREE/Makefile ]] || die "albero kernel non trovato accanto a $CONFIG"
MICROVM_OBJ=$WORK/microvm
step "config MicroVM: x86_64_defconfig + kvm_guest.config + microvm/kernel-local"
# rpmbuild -bp lascia in albero include/config e include/generated (process_configs.sh)
# e O= pretende un albero pulito; mrproper e' anche il primo passo di InitBuildVars in
# kernel.spec (da cui BuildKernel riparte con configs/), quindi per il kernel
# principale non cambia nulla.
make -s -C "$TREE" mrproper
mkdir -p "$MICROVM_OBJ"
make -s -C "$TREE" O="$MICROVM_OBJ" "${MAKE_OPTS[@]}" x86_64_defconfig kvm_guest.config
"$TREE/scripts/kconfig/merge_config.sh" -m -O "$MICROVM_OBJ" "$MICROVM_OBJ/.config" "$HERE/microvm/kernel-local"
make -s -C "$TREE" O="$MICROVM_OBJ" "${MAKE_OPTS[@]}" olddefconfig
check_delta "$MICROVM_OBJ/.config" "$HERE/microvm/kernel-local"
cp "$MICROVM_OBJ/.config" "$OUT/microvm.config"
echo "config MicroVM: $(grep -c '=y$' "$MICROVM_OBJ/.config") opzioni built-in, $(grep -c '=m$' "$MICROVM_OBJ/.config") moduli"

if [[ $STAGE == microvm || $STAGE == build ]]; then
  NVR=$(bash "$HERE/nvr.sh")
  step "kernel MicroVM: rpmbuild -bb microvm/ermete-kernel-microvm.spec (O=$MICROVM_OBJ)"
  # Lo stesso SOURCE_DATE_EPOCH del kernel principale, che rpm prende dalla changelog di
  # kernel.spec: lo spec del guest non ne ha una, e senza epoch ne' i timestamp del
  # pacchetto ne' KBUILD_BUILD_TIMESTAMP sarebbero riproducibili.
  epoch=$(rpmspec -q --srpm --qf '[%{changelogtime} ]' "$TOP/SPECS/kernel.spec" | cut -d' ' -f1)
  [[ $epoch =~ ^[0-9]+$ ]] || die "changelog di kernel.spec senza data"
  SOURCE_DATE_EPOCH=$epoch rpmbuild -bb --target x86_64 --define "source_date_epoch_from_changelog 0" --define "use_source_date_epoch_as_buildtime 1" \
    --define "kernel_tree $TREE" --define "objdir $MICROVM_OBJ" \
    --define "kversion ${NVR%%-*}" --define "krelease ${NVR#*-}" \
    --define "make_opts ${MAKE_OPTS[*]}" "$HERE/microvm/ermete-kernel-microvm.spec"
  mkdir -p "$OUT/microvm"
  mv "$TOP"/RPMS/x86_64/ermete-kernel-microvm-*.rpm "$OUT/microvm/"
fi

if [[ $STAGE == build ]]; then
  step "rpmbuild -bb"
  rpmbuild -bb --noprep --target x86_64 "${BCONDS[@]}" "$TOP/SPECS/kernel.spec"
  # Tre OCI distinti (kernel-build.yml): binari, devel per i kmod esterni, debuginfo
  # con la sua retention. La classificazione e' per nome di pacchetto.
  mkdir -p "$OUT/kernel" "$OUT/devel" "$OUT/debuginfo"
  for rpm in "$TOP"/RPMS/x86_64/*.rpm; do
    case ${rpm##*/} in
      *debuginfo*) cp "$rpm" "$OUT/debuginfo/" ;;
      *devel*) cp "$rpm" "$OUT/devel/" ;;
      *) cp "$rpm" "$OUT/kernel/" ;;
    esac
  done
  rpm -qp --qf '%{VERSION}-%{RELEASE}' "$TOP"/RPMS/x86_64/kernel-core-*.rpm > "$OUT/nvr"
fi

step "fatto: $(find "$OUT" -type f -printf "%P ")"
