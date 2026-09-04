# ermete-kernel

Il kernel di Ermete OS: il pacchetto `kernel` di Fedora ricostruito con clang/ThinLTO
sopra la base CachyOS (BORE, -O3, tunable), con l'hardening in piu' di Ermete. La
specifica e' `docs/architecture/doc_kernel_build.md`; qui c'e' solo cosa sta in questa
directory e come si usa.

| File | Ruolo |
|------|-------|
| `pins.env` | i pin: NVR Fedora (stesso patch level della release CachyOS), release CachyOS, commit del config e delle patch |
| `SOURCES/sources.sha256` | hash di ogni file che build.sh scarica |
| `SOURCES/keys/{cachyos,kernel.org}/` | chiavi pubbliche che firmano i tarball CachyOS e vanilla |
| `keys/mok/` | certificato pubblico della MOK di progetto, che firma UKI e moduli esterni; la chiave privata sta nel secret `MOK_PRIVATE_KEY` dell'environment `signing` |
| `kernel-local` | delta Kconfig di Ermete sul config x86_64 di Fedora |
| `patches.list` | patch di CachyOS/kernel-patches applicate sopra la base |
| `patches/` | patch di Ermete, in formato git, applicate dopo quelle di CachyOS |
| `fedora-wins.list` | percorsi in cui un conflitto tra base CachyOS e patch Red Hat si risolve con l'albero Fedora |
| `cmdline` | la riga di comando del kernel che la UKI firma (spec, sezione 6) |
| `build.sh` | dai pin agli RPM: stadio `prep` (sorgenti, patch, gate del config) e `build` |
| `build-inputs.py` | gli input della build come JSON: predicato dell'attestazione dei pin e chiave del riuso in CI |
| `nvr.sh` | l'NVR del kernel derivato dai pin, lo stesso che rpmbuild produce e che i tag OCI usano |
| `builder/Containerfile` | l'ambiente: Fedora pinnata per digest piu' la toolchain LLVM |
| `boot.sh` | la boot matrix: dal kernel-core a quattro avvii QEMU con le asserzioni della spec |
| `boot/Containerfile`, `boot/init` | l'ambiente della boot matrix (qemu, OVMF, shim, ukify) e il PID 1 dell'initramfs di prova |
| `nvidia.sh` | i moduli kernel NVIDIA, rami `open` (610) e `legacy` (580), contro il kernel-devel: `build` e `sign` |
| `nvidia/Containerfile` | l'ambiente di nvidia.sh: la toolchain LLVM del kernel, kmod, openssl |

## Uso locale

```sh
podman build -t localhost/ermete-kernel-builder forge/specs/ermete-kernel/builder
mkdir -p "$HOME/.cache/ermete-kernel" out
podman run --rm -v "$PWD:/forge" -v "$HOME/.cache/ermete-kernel:/var/cache/ermete-kernel" \
  -w /forge localhost/ermete-kernel-builder \
  bash forge/specs/ermete-kernel/build.sh --stage prep --out /forge/out
```

`prep` dura pochi minuti e lascia in `out/` il config generato e il `kernel-local`
completo delle opzioni derivate; `build` produce gli RPM (un'ora su 16 core) in
`out/kernel`, `out/devel`, `out/debuginfo`, con l'NVR in `out/nvr`. La CI e'
`.github/workflows/kernel-build.yml`: ricompila solo se `build-inputs.py` non
coincide con l'attestazione dei pin dell'immagine `<nvr>` gia' pubblicata; altrimenti
la boot matrix gira sul kernel-core pubblicato e non si pubblica nulla.

## Boot matrix

```sh
podman build -t localhost/ermete-kernel-boot forge/specs/ermete-kernel/boot
podman run --rm --device /dev/kvm -v "$PWD:/forge" -w /forge localhost/ermete-kernel-boot bash forge/specs/ermete-kernel/boot.sh --rpms /forge/out --out /forge/boot-out
```

Quattro avvii, firmware {SeaBIOS, OVMF con Secure Boot via shim} x CPU {Nehalem,
host}, ognuno con le asserzioni di `boot/init` (uname, BTF, bpftool, sched_ext, IMA,
lockdown, BBR v3, taint, dmesg; in UEFI anche Secure Boot acceso e MOK arruolata).
Serve solo il kernel-core: `--rpms` accetta l'`out/` di build.sh o una directory con
il solo RPM. Senza `/dev/kvm` (WSL, podman machine) aggiungi `--accel tcg`: minuti
invece di secondi, e `host` diventa `max`. Log seriali e riepilogo in `boot-out/`.

## Moduli NVIDIA

```sh
podman build -t localhost/ermete-kernel-nvidia forge/specs/ermete-kernel/nvidia
podman run --rm -v "$PWD:/forge" -v "$HOME/.cache/ermete-kernel:/var/cache/ermete-kernel" \n  -w /forge localhost/ermete-kernel-nvidia \n  bash forge/specs/ermete-kernel/nvidia.sh build --driver open --devel /forge/out/devel --out /forge/nvidia-out
```

`--driver open` (610, GitHub al commit pinnato) o `legacy` (580, il `.run` nel
manifest degli hash); `--devel` e' una directory con il `kernel-devel-*.rpm` (l'`out/`
di build.sh, o l'immagine `ermete-os-kernel-devel:<nvr>`). I `.ko` finiscono in
`nvidia-out/<driver>/` con il vermagic del kernel e i preamboli kCFI, senza firma:
`nvidia.sh sign --key K --cert C --devel DIR --out DIR` li firma con sign-file del
kernel-devel, in locale con una chiave effimera, in CI con la MOK di progetto
(workflow `.github/workflows/nvidia-kmod.yml`).

## Pubblicazione

Ogni push su `main` o `iso-v0` che tocca questa directory costruisce e pubblica tre
OCI con i soli RPM dentro, tag `<nvr>` (es. `7.1.8-100.ermete.fc43`):

| Immagine | Contenuto |
|----------|-----------|
| `ghcr.io/hr-mes/ermete-os-kernel` | kernel, core, modules, modules-core/extra/internal, uki-virt |
| `ghcr.io/hr-mes/ermete-os-kernel-devel` | kernel-devel, per i kmod esterni (NVIDIA, fase K4) |
| `ghcr.io/hr-mes/ermete-os-kernel-debuginfo` | debuginfo, restano le due versioni piu' recenti |
| `ghcr.io/hr-mes/ermete-os-nvidia` | i `.ko` NVIDIA firmati, tag `<nvr>-open` e `<nvr>-legacy` (workflow `nvidia-kmod.yml`) |

Ognuna e' firmata con cosign keyless dall'identita' del workflow, porta un SBOM SPDX
e un'attestazione custom con i pin (`pins.json`: pins.env, hash del manifest, del
delta e del Containerfile, immagine base del builder); la principale ha anche la
provenance SLSA di GitHub. `:latest` si muove solo su `main`. Verifica:

```sh
cosign verify --certificate-identity-regexp '^https://github.com/hr-mes/ermete-os/' \n  --certificate-oidc-issuer https://token.actions.githubusercontent.com \n  ghcr.io/hr-mes/ermete-os-kernel:7.1.8-100.ermete.fc43
gh attestation verify oci://ghcr.io/hr-mes/ermete-os-kernel:7.1.8-100.ermete.fc43 --repo hr-mes/ermete-os
```

## Bump a mano (finche' non c'e' il bot, fase K5)

1. Aggiorna `pins.env`.
2. Lancia `prep`: scarica i file nuovi nella cache e si ferma sull'hash. Sostituisci in
   `SOURCES/sources.sha256` le righe dei file cambiati con il loro `sha256sum`.
3. `prep` verde, poi `build`. Se il gate del config elenca opzioni nuove o mancanti,
   la decisione va in `kernel-local`.
