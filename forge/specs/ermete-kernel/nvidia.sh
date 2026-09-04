#!/usr/bin/env bash
# Moduli kernel NVIDIA per il kernel Ermete (docs/architecture/doc_kernel_build.md,
# sezione 10, fase K4). Gira nell'immagine nvidia/Containerfile. Due rami:
#   open    i moduli aperti (Turing e successive) dal repo GitHub
#           NVIDIA/open-gpu-kernel-modules, al commit pinnato in pins.env;
#   legacy  i moduli proprietari del ramo 580 (Maxwell, Pascal, Volta) dal .run di
#           download.nvidia.com, pinnato per sha256 in nvidia/sources.sha256.
# Compila contro l'albero kernel-devel con la toolchain del kernel (clang, LLVM): Kbuild
# applica da solo i flag del kernel ai moduli; la parte RM dei moduli aperti, che NVIDIA
# compila fuori da Kbuild, riceve gli stessi flag (kCFI, retpoline, return thunk, SLS,
# IBT) tramite EXTRA_CFLAGS: objtool non deve trovare ret nudi ne' chiamate senza retpoline
# nel codice C (cosi' si vede un flag mancante); il resto lo conta. Nel ramo legacy la
# parte RM e' il blob nv-kernel.o_binary di NVIDIA, senza kCFI ne' return thunk: le
# chiamate verso di esso sono un rischio noto, verificabile solo su hardware.
#
# Uso: nvidia.sh build --driver open|legacy --devel DIR --out DIR
#      nvidia.sh sign  --key FILE --cert FILE --devel DIR --out DIR
#   --devel    directory con kernel-devel-*.rpm (l'out/devel di build.sh o l'immagine)
#   --out      build: i .ko in OUT/<driver>/lib/modules/<kver>/extra/nvidia/, il layout
#              che l'immagine di sistema copia e che syft cataloga, con `version` e
#              `kver` in OUT/<driver>/; sign: firma ogni .ko sotto OUT con sign-file del
#              kernel-devel e lo verifica con modinfo
#   --key/--cert  chiave privata e certificato (PEM o DER) della MOK: in CI quella di
#              progetto dall'environment `signing`, in locale una effimera
set -euo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
CACHE=${ERMETE_KERNEL_CACHE:-/var/cache/ermete-kernel}
STAGE=${1:-}
[[ $# -gt 0 ]] && shift
DRIVER='' DEVEL='' OUT='' KEY='' CERT=''
while [[ $# -gt 0 ]]; do
  case $1 in
    --driver) DRIVER=$2; shift 2 ;;
    --devel) DEVEL=$2; shift 2 ;;
    --out) OUT=$2; shift 2 ;;
    --key) KEY=$2; shift 2 ;;
    --cert) CERT=$2; shift 2 ;;
    *) echo "argomento sconosciuto: $1" >&2; exit 2 ;;
  esac
done
usage() {
  echo "uso: nvidia.sh build --driver open|legacy --devel DIR --out DIR" >&2
  echo "     nvidia.sh sign --key FILE --cert FILE --devel DIR --out DIR" >&2
  exit 2
}
[[ $DEVEL && $OUT ]] || usage
case $STAGE in
  build) [[ $DRIVER == open || $DRIVER == legacy ]] || usage ;;
  sign) [[ $KEY && $CERT ]] || usage ;;
  *) usage ;;
esac

die() { echo "errore: $*" >&2; exit 1; }
step() { echo; echo "== $*"; }
fetch() { # fetch URL: nella cache, se manca
  local url=$1 name=${1##*/}
  [[ -s $CACHE/$name ]] || { mkdir -p "$CACHE"; curl -fsSL --retry 3 -o "$CACHE/$name.part" "$url" && mv "$CACHE/$name.part" "$CACHE/$name"; }
}
# shellcheck source=pins.env
. "$HERE/pins.env"

# L'albero kernel-devel estratto dal RPM: Kbuild per i moduli esterni non ha bisogno
# dell'installazione, cosi' non servono ne' rete ne' dnf.
mapfile -t DEVEL_RPM < <(find "$DEVEL" -name 'kernel-devel-[0-9]*.rpm')
[[ ${#DEVEL_RPM[@]} -eq 1 ]] || die "atteso un solo kernel-devel-*.rpm in $DEVEL, trovati ${#DEVEL_RPM[@]}"
KVER=$(rpm -qp --qf '%{VERSION}-%{RELEASE}.%{ARCH}' "${DEVEL_RPM[0]}")
WORK=$(mktemp -d)
mkdir -p "$WORK/devel" && (cd "$WORK/devel" && rpm2cpio "${DEVEL_RPM[0]}" | cpio -idm --quiet)
SYSSRC="$WORK/devel/usr/src/kernels/$KVER"
[[ -f $SYSSRC/Makefile && -f $SYSSRC/.config ]] || die "albero kernel-devel non trovato per $KVER"
mkdir -p "$OUT"
echo "kernel $KVER"

cfg() { grep -q "^$1=y" "$SYSSRC/.config"; } # cfg CONFIG_X: l'opzione e' accesa nel kernel
cfg_val() { sed -n "s/^$1=//p" "$SYSSRC/.config"; } # cfg_val CONFIG_X: il valore dell'opzione
cc_option() { # cc_option FLAG...: i flag se clang li accetta, come cc-option di Kbuild
  if clang -Werror "$@" -c -x c /dev/null -o /dev/null 2> /dev/null; then echo " $*"; fi
}

build() {
  local src kodir version flags rm_targets=()
  grep -q '^CONFIG_CC_VERSION_TEXT="clang' "$SYSSRC/.config" || die "il kernel non e' compilato con clang: i moduli devono usare la stessa toolchain"
  cfg CONFIG_CFI || die "il kernel non ha CONFIG_CFI"
  # I flag con cui Kbuild compila i moduli, letti da .config nella grafia di clang
  # (Makefile e arch/x86/Makefile del kernel), per il codice che NVIDIA compila fuori da
  # Kbuild: i suoi Makefile provano solo le grafie gcc, che clang scarta in silenzio.
  flags="-fsanitize=kcfi"
  cfg CONFIG_CFI_ICALL_NORMALIZE_INTEGERS && flags+=" -fsanitize-cfi-icall-experimental-normalize-integers"
  cfg CONFIG_FINEIBT_BHI && flags+=" -fsanitize-kcfi-arity"
  cfg CONFIG_MITIGATION_RETPOLINE && flags+=" -mretpoline-external-thunk$(cc_option -mindirect-branch-cs-prefix)"
  cfg CONFIG_MITIGATION_RETHUNK && flags+=" -mfunction-return=thunk-extern"
  cfg CONFIG_MITIGATION_SLS && flags+=" -mharden-sls=all"
  cfg CONFIG_X86_KERNEL_IBT && flags+=" -fcf-protection=branch -fno-jump-tables"
  # CALL_PADDING: il preambolo kCFI sta 16 byte prima dell'entry (movl dell'hash e 11 nop)
  # e i chiamanti leggono l'hash a -15. Senza lo stesso padding il kernel, applicando
  # FineIBT/kCFI al caricamento, non trova gli hash e il modulo non carica.
  cfg CONFIG_CALL_PADDING && flags+=" -fpatchable-function-entry=$(cfg_val CONFIG_FUNCTION_PADDING_BYTES),$(cfg_val CONFIG_FUNCTION_PADDING_BYTES)"
  local align alignflag
  align=$(cfg_val CONFIG_FUNCTION_ALIGNMENT)
  if [[ ${align:-0} -gt 0 ]]; then # come il Makefile del kernel: -fmin-function-alignment se c'e', altrimenti -falign-functions
    alignflag=$(cc_option -fmin-function-alignment="$align"); flags+="${alignflag:- -falign-functions=$align}"
  fi

  case $DRIVER in
    open)
      step "moduli aperti $NVIDIA_OPEN_VERSION, commit $NVIDIA_OPEN_COMMIT"
      git -c advice.detachedHead=false clone -q --depth 1 --branch "$NVIDIA_OPEN_VERSION" \
        https://github.com/NVIDIA/open-gpu-kernel-modules "$WORK/src" 2> /dev/null
      [[ $(git -C "$WORK/src" rev-parse HEAD) == "$NVIDIA_OPEN_COMMIT" ]] \
        || die "il tag $NVIDIA_OPEN_VERSION non punta al commit pinnato $NVIDIA_OPEN_COMMIT"
      src="$WORK/src"; kodir="$src/kernel-open"; version=$NVIDIA_OPEN_VERSION
      # La parte RM (nv-kernel.o, nv-modeset-kernel.o), compilata da NVIDIA fuori da Kbuild:
      # EXTRA_CFLAGS e' il gancio di utils.mk, e qui passano i flag del kernel.
      rm_targets=(kernel-open/nvidia/nv-kernel.o_binary kernel-open/nvidia-modeset/nv-modeset-kernel.o_binary)
      step "parte RM con clang e i flag del kernel ($flags)"
      make -C "$src" -j"$(nproc)" -Otarget CC=clang CXX=clang++ LD=ld.lld AR=llvm-ar EXTRA_CFLAGS="$flags" "${rm_targets[@]}" \
        > "$OUT/$DRIVER-rm.log" 2>&1 || { tail -n 30 "$OUT/$DRIVER-rm.log"; die "parte RM fallita, log in $OUT/$DRIVER-rm.log"; }
      ;;
    legacy)
      local run="NVIDIA-Linux-x86_64-$NVIDIA_LEGACY_VERSION-no-compat32.run"
      step "moduli proprietari $NVIDIA_LEGACY_VERSION dal .run"
      fetch "https://download.nvidia.com/XFree86/Linux-x86_64/$NVIDIA_LEGACY_VERSION/$run"
      (cd "$CACHE" && grep " $run\$" "$HERE/nvidia/sources.sha256" | sha256sum --check --quiet --strict)
      sh "$CACHE/$run" --extract-only --target "$WORK/run" > /dev/null
      src="$WORK/run/kernel"; kodir="$src"; version=$NVIDIA_LEGACY_VERSION
      ;;
  esac

  step "moduli contro $KVER con Kbuild (clang, LLVM, kCFI dal kernel)"
  # IGNORE_CC_MISMATCH: il conftest di NVIDIA vuole la stessa stringa di versione del
  # compilatore, e il clang di Fedora si aggiorna tra la build del kernel e questa.
  # Basta che sia clang: gli hash kCFI dipendono dai tipi, non dalla versione.
  # -Otarget: l'output di ogni oggetto in blocco, cosi' gli avvisi di objtool restano attribuibili.
  make -C "$src" -j"$(nproc)" -Otarget modules SYSSRC="$SYSSRC" CC=clang LD=ld.lld LLVM=1 LLVM_IAS=1 IGNORE_CC_MISMATCH=1 \
    > "$OUT/$DRIVER-build.log" 2>&1 || { tail -n 40 "$OUT/$DRIVER-build.log"; die "compilazione fallita, log in $OUT/$DRIVER-build.log"; }
  local objtool unmitigated
  objtool=$(awk '/warning: objtool:/ { n++ } END { print n + 0 }' "$OUT/$DRIVER-build.log")
  if [[ $DRIVER == open ]]; then
    # Nel ramo aperto tutto il codice e' compilato qui con i flag del kernel, e un flag che
    # manca si vede in objtool come ret nudo o chiamata/salto indiretto senza retpoline in
    # una funzione C. Il resto e' proprieta' del codice NVIDIA, non dei flag, e viene solo
    # contato: clang non estende kCFI alle chiamate virtuali ne' i return thunk ai thunk
    # (nomi _Z) del C++ di DisplayPort in nvidia-modeset.o, e nel RM restano code di
    # funzione irraggiungibili ("falls through").
    unmitigated=$(awk '/warning: objtool:/ && /MITIGATION_(RETHUNK|RETPOLINE) build/ && !/objtool: _Z/' "$OUT/$DRIVER-build.log")
    [[ -z $unmitigated ]] || { head -n 20 <<< "$unmitigated"; die "objtool trova codice C senza return thunk o retpoline nel ramo open, log in $OUT/$DRIVER-build.log"; }
    echo "objtool: $objtool avvisi, nessuno per flag mancanti (C++ di DisplayPort e code irraggiungibili del RM)"
  else
    echo "objtool: $objtool avvisi, dal blob RM di NVIDIA e dal C++ di DisplayPort (attesi nel ramo legacy)"
  fi

  local ko dest="$OUT/$DRIVER/lib/modules/$KVER/extra/nvidia"
  mkdir -p "$dest"
  for ko in "$kodir"/*.ko; do
    [[ $(modinfo -F vermagic "$ko") == "$KVER "* ]] || die "${ko##*/}: vermagic $(modinfo -F vermagic "$ko") non e' del kernel $KVER"
    # I preamboli kCFI (__cfi_<funzione>) provano che i flag del kernel sono arrivati.
    # Non `nm | grep -q`: con pipefail, grep chiude la pipe al primo match e nm muore di
    # SIGPIPE sui moduli grandi, e il controllo fallirebbe a caso.
    grep -q ' __cfi_' <(nm "$ko") || die "${ko##*/}: nessun preambolo kCFI"
    install -m 644 "$ko" "$dest/"
  done
  echo "$version" > "$OUT/$DRIVER/version"
  echo "$KVER" > "$OUT/$DRIVER/kver"
  echo "moduli $DRIVER $version: $(find "$OUT/$DRIVER" -name '*.ko' -printf '%f ')"
}

cert_cn() { # cert_cn FILE: il CN del certificato, PEM o DER
  local subject
  subject=$(openssl x509 -in "$1" -noout -subject -nameopt RFC2253 2> /dev/null \
    || openssl x509 -in "$1" -inform DER -noout -subject -nameopt RFC2253)
  subject=${subject#subject=}; subject=${subject#CN=}; echo "${subject%%,*}"
}

sign() {
  local hash cn ko signer
  hash=$(sed -n 's/^CONFIG_MODULE_SIG_HASH="\(.*\)"$/\1/p' "$SYSSRC/.config")
  [[ $hash ]] || die "CONFIG_MODULE_SIG_HASH assente nel config"
  cn=$(cert_cn "$CERT")
  step "firma con $hash, certificato \"$cn\""
  mapfile -t KOS < <(find "$OUT" -path '*/lib/modules/*' -name '*.ko' | sort)
  [[ ${#KOS[@]} -gt 0 ]] || die "nessun modulo sotto $OUT/*/lib/modules/"
  for ko in "${KOS[@]}"; do
    "$SYSSRC/scripts/sign-file" "$hash" "$KEY" "$CERT" "$ko"
    signer=$(modinfo -F signer "$ko")
    [[ $signer == "$cn" ]] || die "${ko##*/}: firmatario \"$signer\", atteso \"$cn\""
    echo "${ko#"$OUT"/}: firmato da \"$signer\", $(modinfo -F sig_hashalgo "$ko"), chiave $(modinfo -F sig_key "$ko" | cut -c1-23)..."
  done
}

case $STAGE in build) build ;; sign) sign ;; esac
step "fatto: $(find "$OUT" -type f -name '*.ko' -printf '%P ')"
