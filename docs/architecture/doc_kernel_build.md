# Ermete OS: Specifica del Kernel (costruzione, pin, manutenzione automatica)

Stato: **approvata il 2026-09-03** (serie `stable` 7.x, `-O3` acceso, debuginfo
come OCI separato con retention di due versioni; dal 2026-09-04 Rust acceso,
ThinLTO e `RANDSTRUCT` spenti, sezione 13). Sostituisce il
README "Testo Sacro" di `forge/specs/ermete-kernel/` e lo script
`prepare-chimera.sh`. Il livello funzionale del kernel (eBPF, KVM, Gatekeeper) è
descritto in [doc_kernel_layer.md](doc_kernel_layer.md): questo documento dice
**come il kernel viene costruito, pinnato, firmato e mantenuto**, e quali garanzie
deve dare a quel livello.

Decisioni già prese con il maintainer:

- il kernel è custom e viene prima della v0; la v0 riparte con questo kernel;
- "rolling": segue Fedora stable da vicino, non necessariamente l'ultima minor;
- compatibile con i PC x86-64 dal 2014 in poi, GPU AMD, Intel e NVIDIA;
- massimo tecnico possibile con manutenzione e debito tecnico minimi: il
  sistema si auto-mantiene, l'umano interviene solo quando qualcosa è rosso.

## 1. Principi

1. **Nessuno spec forkato.** Lo spec del kernel Fedora (kernel-ark) ha due ganci
   nativi per esattamente questo uso: `Patch999999: linux-kernel-test.patch`,
   applicato con `git apply` dopo la patch Red Hat, e `Source3001: kernel-local`,
   fuso nei config da `merge.py` con il controllo di coerenza di
   `process_configs.sh`. Ermete fornisce quei due file e i bcond. Lo spec non
   viene mai modificato con `sed`.
2. **Tutto pinnato, tutto verificato.** Un solo file di pin (`pins.env`) e un
   manifest `sources.sha256`. Il SRPM Fedora è verificato con la firma GPG di
   Fedora e con l'hash; il tarball CachyOS con la firma dei suoi maintainer e con
   l'hash; le patch singole con l'hash. Nessuna risoluzione "dinamica" a build
   time: la scelta della versione avviene in una PR, mai nel job di build.
3. **Le ottimizzazioni sono opzioni di prima classe.** Ogni scelta è un'opzione
   Kconfig o una patch che upstream o CachyOS mantengono. Niente `sed` sui
   Makefile, niente `-Wno-error`, objtool acceso. Una patch che non si applica
   fa fallire la build (`git apply` è senza fuzz), non viene "saltata".
4. **I gate falliscono forte.** Config non onorato, patch non applicata, boot
   fallito, kmod NVIDIA che non compila: ognuno è un fallimento della PR di
   bump, con il messaggio esatto. Nessun `|| true`.
5. **Un solo punto di verità per versione.** `pins.env` + `KERNEL.md` generato
   dal bot. Il config effettivo è leggibile sulla macchina accesa
   (`/proc/config.gz`).

## 2. Sorgenti e pin

Directory `forge/specs/ermete-kernel/` dopo il blocco:

| File                     | Ruolo                                                                                                                                                                               |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `pins.env`               | `FEDORA_KERNEL_NVR` (es. `7.1.12-100.fc43`), `FEDORA_SOURCE_RELEASE` (43), `CACHYOS_RELEASE` (es. `cachyos-7.1.8-1`), `CACHYOS_PATCHES_COMMIT`, `KERNEL_CHANNEL` (`stable` o `lts`) |
| `SOURCES/sources.sha256` | hash del SRPM, del tarball CachyOS, delle patch singole                                                                                                                             |
| `kernel-local`           | frammento di config, una riga di motivazione per opzione                                                                                                                            |
| `patches.list`           | patch di `CachyOS/kernel-patches` da accodare dopo la base, in ordine                                                                                                               |
| `fedora-wins.list`       | percorsi in cui un conflitto del merge tra base CachyOS e patch Red Hat si risolve con l'albero Fedora; ogni altro conflitto ferma la build                                        |
| `cmdline`                | riga di comando del kernel, firmata nella UKI (sezione 6)                                                                                                                           |
| `builder/Containerfile`  | ambiente Fedora 43 (esiste già), base pinnata per digest                                                                                                                            |
| `build.sh`               | l'intera build, riproducibile in locale e in CI                                                                                                                                     |
| `microvm/`               | config e spec del kernel guest (sezione 9)                                                                                                                                          |
| `KERNEL.md`              | cosa c'è nella directory, uso locale, bump a mano; il bot (K5) ne aggiorna la parte di versione                                                                                    |

Spariscono: `prepare-chimera.sh`, `build-local.sh`, `cachyos-patches/` (1031
file, 7,4 milioni di righe), `patches/0001-acs-override.patch` (rompe
l'isolamento IOMMU, incompatibile con zero-trust), `fedora-nvidia.repo` (vive già
in `forge/assets/repos/`), `README.md`.

**Sorgente 1, Fedora.** SRPM da koji all'NVR pinnato
(`https://kojipkgs.fedoraproject.org/packages/kernel/<ver>/<rel>/src/kernel-<nvr>.src.rpm`).
Quel file non porta firma: koji conserva le copie firmate
(`data/signed/<chiave>/`) solo per le build più recenti e le pota dopo, ma tiene
per sempre l'header di firma in `data/sigcache/<chiave>/src/<nvr>.src.rpm.sig`.
`build.sh` scarica entrambi (hash nel manifest), li ricuce con
`koji.splice_rpm_sighdr` (la stessa funzione con cui koji produce le copie
firmate: il risultato è byte per byte quello del mirror) e verifica con
`rpmkeys --checksig` contro la chiave Fedora della release, la cui impronta è in
`pins.env`. Il SRPM contiene il tarball vanilla, la patch Red Hat, i config,
`merge.py`, `process_configs.sh`: tutto il macchinario di packaging.

**Sorgente 2, CachyOS.** Dal 7.x CachyOS non pubblica più una patch "base" ma il
proprio albero come release firmata su GitHub
(`cachyos-X.Y.Z-N.tar.gz` + `.asc`, chiavi `E18447AC…` e `E8B9AA39…`), che
contiene BBRv3, l'opzione `-O3`, i tuning di scheduler e memoria, `more-uarches`.
La base Ermete è il merge a tre vie, deterministico, di quell'albero sull'albero
Fedora (vanilla + patch Red Hat) con base il tarball vanilla della stessa `X.Y.Z`
(kernel.org, firma PGP): generato in build da input hashati, non conservato nel
repo. Serve il merge, non un `diff` applicato: la patch Red Hat porta backport
(ISP4 AMD in 7.1) che CachyOS ha già, e un diff li aggiungerebbe due volte; il
merge fonde le aggiunte identiche; su quelle divergenti vince Fedora solo per i
percorsi elencati in `fedora-wins.list` (in 7.1: `MAINTAINERS` e `isp4/Kconfig`),
altrove la build si ferma. BORE e
le altre patch scelte arrivano da
`CachyOS/kernel-patches` al commit pinnato, elencate in `patches.list`
(`sched/0001-bore-cachy.patch` è la prima). Il tutto, in ordine, diventa
`linux-kernel-test.patch`.

**Serie del kernel.** CachyOS segue l'ultima stable (oggi 7.2) e la LTS (6.18);
Fedora 43 è sulla 7.1 e verrà ribasata. La coppia va presa **a parità di patch
level**: il diff base è calcolato sul vanilla `X.Y.Z` e entra solo in un albero
`X.Y.Z` (provato in K1: la base 7.1.8 non entra nel 7.1.12 di Fedora, che ha già
il backport ISP4 AMD e altri hunk divergenti). Regola del bot: la release CachyOS
più recente della serie `X.Y`, e l'NVR Fedora stable con lo stesso `X.Y.Z`,
cercato prima nella release Fedora di base (43) e poi nella successiva (i kernel
Fedora sono autonomi: un SRPM di Fedora 44 si ricostruisce e gira su una rootfs
43). Koji conserva per sempre SRPM e header di firma, quindi un NVR non più
"latest" resta pinnabile. Se la coppia non esiste, la PR è rossa e decide
l'umano. `KERNEL_CHANNEL=lts` sposta la stessa logica sulla 6.18.

## 3. La build (`build.sh`)

Gira nel container `builder/Containerfile` sul runner self-hosted (16 core) e
identica in locale. Passi, tutti senza rete tranne i download verificati:

1. scarica nella cache e verifica: hash di ogni file contro il manifest, firma
   PGP del tarball CachyOS e di quello vanilla contro le chiavi vendorizzate,
   firma RPM del SRPM ricucito;
2. scrive `~/.rpmmacros` con `%_topdir` e `%buildid .ermete`; `rpm -i` del SRPM;
   `dnf builddep -y SPECS/kernel.spec` con gli stessi bcond di rpmbuild, subito,
   perché la derivazione del config deve vedere la toolchain vera (rust-src,
   bindgen, pahole: `RUST_IS_AVAILABLE` e le opzioni che ne dipendono);
3. genera `linux-kernel-test.patch`: repo git temporaneo con tre commit (vanilla,
   CachyOS, vanilla + patch Red Hat), `git merge-tree --write-tree` dei due rami
   sopra il vanilla, `patches.list` applicate sull'indice, diff dal commit
   Fedora al risultato. Le stesse patch vanno anche sull'albero CachyOS
   estratto, che serve al passo 4;
4. genera il `kernel-local` completo: il delta Ermete committato, più le opzioni
   che l'albero introduce (`make listnewconfig` sul config Fedora fuso con i
   frammenti clang e con il delta, iterato fino a convergenza) con il valore del
   config CachyOS pinnato o, se assente lì, il default Kconfig. Così il gate
   `-n` di `process_configs.sh` non trova opzioni senza decisione;
5. riduce lo spec a x86_64: gli altri `kernel-*-fedora.config` diventano
   `# EMPTY`, il valore che `process_configs.sh` salta per contratto;
6. `rpmbuild -bp --with toolchain_clang --with clang_lto --without debug
   --without tools --without perf --without libperf --without bpftool --without
   ynl --without selftests --without doc`: patch e `process_configs.sh -w -n -c`.
   Poi il gate di Ermete: ogni riga del delta committato deve valere nel config
   generato (Fedora segnala i mismatch solo sulle opzioni presenti nel
   risultato, un'opzione caduta per dipendenza non soddisfatta passerebbe in
   silenzio). Config e `kernel-local` finiscono nell'artefatto;
7. stadio `build`: `rpmbuild -bb --noprep` sullo stesso albero. Il debuginfo si
   costruisce e si pubblica a parte: serve a `perf`, `crash`, a un futuro AutoFDO
   e non entra nell'immagine;
8. riproducibilità: `SOURCE_DATE_EPOCH` dalla changelog, `KBUILD_BUILD_USER=ermete`,
   `KBUILD_BUILD_HOST=forge`, `KBUILD_BUILD_TIMESTAMP` derivato. Un job
   settimanale ricostruisce lo stesso pin su un runner diverso e confronta
   `vmlinuz`, moduli e `config`: la differenza è un bug da aprire;
7. ccache su directory persistente del runner (non `actions/cache`): tra due
   patch level cambiano pochi file, la LTO finale no;
8. pubblicazione: `ghcr.io/hr-mes/ermete-os-kernel:<nvr>` (RPM binari),
   `:<nvr>-debuginfo`, `:<nvr>-devel`; firma cosign keyless, SBOM, attestazione
   SLSA con commit, pin e digest del builder. `:latest` si muove solo al merge di
   una PR di bump.

Il job del kernel è un workflow proprio (`kernel-build.yml`), attivato da cambi in
`forge/specs/ermete-kernel/**` e a mano, con hash di idempotenza sugli input:
l'orchestratore non lo ricompila a ogni run, l'immagine di sistema consuma il tag
pinnato in `pins.env`.

## 4. Config (`kernel-local`)

Il config Fedora 43 x86_64 porta già: `SCHED_CLASS_EXT`, `DEBUG_INFO_BTF`,
`BPF_LSM`, `IMA` con `IMA_APPRAISE`, `DM_VERITY_VERIFY_ROOTHASH_SIG`, `FS_VERITY`,
`EROFS`, `INIT_ON_ALLOC_DEFAULT_ON`, `X86_KERNEL_IBT`, `HZ_1000`, `PREEMPT_DYNAMIC`,
`LRU_GEN_ENABLED`, `NTSYNC`, `RUST` (che il delta spegne, sezione 5), `WIREGUARD`,
CAKE e FQ, e la lista LSM
`lockdown,yama,integrity,selinux,bpf,landlock,ipe`. Il frammento Ermete è il
delta, e resta corto:

| Opzione                                               | Valore | Perché                                                                                                        |
| ----------------------------------------------------- | ------ | ------------------------------------------------------------------------------------------------------------- |
| `SCHED_BORE`                                          | y      | patch CachyOS, responsività desktop                                                                           |
| `CC_OPTIMIZE_FOR_PERFORMANCE_O3`                      | y      | opzione della base CachyOS, come nei loro kernel; resta finché il benchmark (sezione 7) non dice il contrario |
| `LTO_NONE`                                            | y      | ThinLTO spento: con `DEBUG_INFO_BTF`, `RUST` richiede `!LTO`; il bcond `clang_lto` resta per il toolchain (sezione 5) |
| `RUST`                                                | y      | come Fedora: la porta ai driver che nascono in Rust; con kCFI seleziona `CFI_ICALL_NORMALIZE_INTEGERS`        |
| `CFI`                                                 | y      | kCFI, richiede clang; con IBT già attivo                                                                      |
| `ZERO_CALL_USED_REGS`                                 | y      | hardening a costo trascurabile                                                                                |
| `RANDSTRUCT_NONE`                                     | y      | come Fedora: `RUST` dipende da `!RANDSTRUCT`, e il layout randomizzato costa in cache                         |
| `MODULE_SIG_FORCE`                                    | y      | ogni modulo firmato: chiave effimera di build per l'albero, MOK per i kmod esterni                            |
| `DEFAULT_TCP_CONG`                                    | "bbr3" | BBRv3 dalla base CachyOS                                                                                      |
| `DEFAULT_FQ`                                          | y      | BBR richiede pacing: FQ come qdisc di default                                                                 |
| `ZSWAP_COMPRESSOR_DEFAULT_ZSTD`, `ZRAM_DEF_COMP_ZSTD` | y      | compressione memoria zstd di default                                                                          |
| `IKCONFIG`, `IKCONFIG_PROC`                           | y      | config verificabile a runtime, usato dall'attestazione                                                        |
| `EROFS_FS`                                            | y      | built-in: la rootfs composefs non deve dipendere da un modulo nell'initrd                                     |

Non si toccano, e il documento lo dice perché il passato li ha toccati:
`OBJTOOL`, `WERROR`, `STACK_VALIDATION`, `DEBUG_INFO_*` (senza DWARF non c'è BTF
e senza BTF non c'è il nervo eBPF), `IOMMU_DEFAULT_PASSTHROUGH`, `X86_NATIVE_CPU`,
la lista LSM, i driver. Opzioni inesistenti (`UKSM`, `ACPI_CUSTOM_METHOD`,
`BCACHEFS_FS` fuori dall'albero dal 6.17) non entrano.

**Livello ISA.** `CONFIG_X86_64_VERSION` non esiste in 7.1 upstream; il kernel
Fedora compila con `-march=x86-64` e `-mtune=generic`. Resta così: nel kernel
v2/v3 non danno nulla di misurabile (il codice generico non usa SIMD, i percorsi
caldi scelgono l'implementazione a runtime) e il baseline è la compatibilità
massima. La decisione v2 riguarda la userland (`forge/config/rpmmacros`), fuori
da questo blocco.

**Il controllo di coerenza** è quello di Fedora: `process_configs.sh` con
`with_configchecks` acceso fallisce se un'opzione del frammento viene scartata
da kconfig. Non serve uno script Ermete.

## 5. Toolchain

clang, lld e llvm di Fedora 43 dal Containerfile, con la base
`registry.fedoraproject.org/fedora:43@sha256:…` pinnata per digest e aggiornata
dal bot. `LLVM=1` arriva dal bcond dello spec (`clang_make_opts`). Rust acceso come
in Fedora: in 7.1 `RUST` dipende da `!RANDSTRUCT` e, con `DEBUG_INFO_BTF` acceso
(l'eBPF di Ermete non può rinunciarvi), da `!LTO`, perché pahole non regge i DWARF
fusi da LTO con unità Rust. Quindi ThinLTO e `RANDSTRUCT` restano spenti nel delta,
verificato dal gate. Il bcond `--with clang_lto` resta comunque acceso: è l'unico
con cui kernel.spec passa `HOSTCC=clang CC=clang LLVM=1` a `process_configs.sh`,
senza il quale il config verrebbe valutato con gcc e kCFI sparirebbe; il
frammento LTO che porta con sé è sovrascritto da `kernel-local`. Quando upstream
toglierà il vincolo `!LTO`, ThinLTO si riaccende con due righe del delta; non si
patchano i Makefile per forzarlo.

## 6. Firma e catena di avvio

- **Moduli in albero**: firmati dalla chiave effimera generata dallo spec, il
  cui certificato è dentro il kernel. `MODULE_SIG_FORCE` rende il rifiuto un
  comportamento di compilazione, non di riga di comando.
- **Moduli esterni** (NVIDIA): firmati con la MOK del progetto (segreti
  `MOK_PRIVATE_KEY`/`MOK_PUBLIC_DER` già esistenti), in un job separato che non
  vede altro.
- **UKI**: kernel, initrd, `cmdline` e microcode early in un'unica immagine
  firmata con la MOK dietro lo shim Fedora; la produce la fase system-image,
  perché l'initrd dipende dall'immagine, non dal kernel. Lo spec Fedora fornisce
  già le stringhe SBAT (`kernel.sbat`, `uki.sbat`).
- **Primo avvio**: arruolamento guidato della MOK (`mokutil --import`), unica
  interazione richiesta per avere Secure Boot acceso su un PC qualsiasi.
- **`cmdline`** committata: `lockdown=integrity mitigations=auto init_on_alloc=1
randomize_kstack_offset=on page_alloc.shuffle=1 vsyscall=none preempt=full
amd_pstate=active zswap.enabled=1`. Niente `iommu=pt`, niente `mitigations=off`.
- **Rootfs**: dm-verity con roothash firmato dalla stessa chiave del progetto,
  fs-verity per composefs, TPM 2.0 per LUKS (`systemd-cryptenroll`) con fallback
  a passphrase sui PC 2014–2016 senza TPM 2.0. Le opzioni kernel ci sono già;
  la parte immagine è del blocco system-image.

## 7. Gate

Ogni PR di bump e ogni cambio in `forge/specs/ermete-kernel/**` passa:

1. **build** sul runner self-hosted (60 min misurati su 16 core con ThinLTO);
2. **config**: `process_configs.sh` con controlli accesi;
3. **boot matrix** su runner GitHub-hosted con KVM, QEMU:
   `-cpu Nehalem` (prova che nessuna istruzione oltre il baseline è entrata),
   `-cpu host`, UEFI OVMF con Secure Boot e MOK arruolata sulla UKI firmata,
   BIOS legacy. Asserzioni: `uname -r` atteso, `/sys/kernel/btf/vmlinux`,
   `bpftool feature`, `/sys/kernel/sched_ext`, lista misure IMA, stato lockdown,
   `tcp_congestion_control=bbr`, `dmesg` senza `BUG`/`WARNING`/`taint`;
4. **kmod NVIDIA**: `nvidia-open` (610) e ramo legacy 580 compilano contro
   `kernel-devel` e si firmano con la MOK;
5. **benchmark di tendenza** (non bloccante): hackbench, schbench, fio null,
   netperf loopback per cinque minuti, risultati come artefatto e grafico nel
   summary. È il numero che decide `-O3` e ogni futura opzione;
6. **riproducibilità** settimanale (sezione 3).

## 8. Auto-manutenzione: il bot di bump

Workflow `kernel-bump.yml`, giornaliero, su runner GitHub-hosted:

1. legge `pins.env`; interroga Bodhi per l'NVR stable più recente della serie
   consentita (43, poi 44), le release GitHub di `CachyOS/linux` per la stessa
   `X.Y`, e `CachyOS/kernel-patches` per il commit di testa di `X.Y`;
2. se nulla è cambiato, esce. Altrimenti, nel container Fedora: scarica e
   verifica tutto, genera il diff base, prova `git apply --check` di ogni patch
   sull'albero preparato (`rpmbuild -bp`, minuti, senza compilare), esegue
   `make listnewconfig` e raccoglie le opzioni nuove da quando c'è il pin;
3. apre una PR con `pins.env`, `sources.sha256`, `KERNEL.md` rigenerati e nel
   corpo: NVR, release CachyOS, commit patch, esito di ogni patch, opzioni
   Kconfig nuove (perché un umano veda i pomelli che Fedora ha aggiunto);
4. la PR fa partire i gate della sezione 7; con tutto verde va in **auto-merge**.
   Rosso: resta aperta con il log del gate fallito. È l'unico momento in cui
   serve una persona, e sa già dove guardare.

Il bot aggiorna anche il digest dell'immagine base del Containerfile e verifica
che le chiavi PGP pinnate firmino ancora le release (una rotazione di chiave è
una PR rossa, mai un'accettazione silenziosa). Il cambio di release Fedora della
rootfs (43→44) e il cambio di `KERNEL_CHANNEL` restano PR umane.

## 9. Kernel guest per le MicroVM

Stessa sorgente e stesso pin, secondo config: `make x86_64_defconfig` +
`kvm_guest.config` + frammento `microvm/kernel-local` (virtio, 9p/virtiofs,
EROFS, dm-verity, BPF, nessun driver fisico, nessun modulo). Spec minimo
`microvm/ermete-kernel-microvm.spec` (~100 righe, non ha bisogno del packaging
Fedora: produce `vmlinux` e `bzImage`), pochi minuti di build nello stesso job,
pubblicato in `ermete-os-kernel:<nvr>-microvm`. È il kernel che
`hypervisor-daemon` avvia in Firecracker o cloud-hypervisor; SEV/TDX guest
restano opzioni del frammento per gli host che li hanno.

## 10. NVIDIA, AMD, Intel

AMD e Intel sono in-tree (`amdgpu`, `radeon`, `i915`, `xe`) con `linux-firmware`
spacchettato per vendor nell'immagine: nessun lavoro nel kernel oltre a non
toglierli. NVIDIA, in un workflow proprio (`nvidia-kmod.yml`) che parte dopo il
kernel:

| Livello         | GPU                              | Meccanismo                                                                                                                                                         |
| --------------- | -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| default         | tutte                            | `nouveau` in-tree, firmware GSP; NVK in Mesa                                                                                                                       |
| `nvidia-open`   | Turing 2018+                     | moduli aperti 610.x compilati nel container Fedora contro `kernel-devel`, clang e kCFI coerenti, firmati MOK; le patch `misc/nvidia/*` di CachyOS si applicano qui |
| `nvidia-legacy` | Maxwell, Pascal, Volta 2014–2018 | ramo 580, stesso meccanismo                                                                                                                                        |

Pubblicazione `ermete-os-nvidia:<kernel-nvr>-<driver>`; le varianti
dell'immagine (`-nvidia`, `-nvidia-legacy`) le consumano. Le versioni del driver
sono pin in `pins.env`, alzate dal bot solo se il kmod compila.

## 11. Fuori dal kernel

In `system-tweaks` (sysctl.d, modprobe.d): `net.core.default_qdisc=cake` sul
desktop, tunable BORE, `vm.max_map_count`, `kernel.split_lock_mitigate`. In
`forge/config/rpmmacros`: baseline userland v2 e glibc-hwcaps per le librerie
che guadagnano da v3. AutoFDO: `CONFIG_AUTOFDO_CLANG=y` entra quando esiste la
pipeline di profilazione sul kernel Ermete (perf con branch sampling sul
5800X3D, `create_llvm_prof`, profilo committato con hash); nessun profilo
altrui. Variante `PREEMPT_RT` (mainline dal 6.12) e patchset `hardened` di
CachyOS: candidati da valutare con il benchmark, non default.

## 12. Ordine di esecuzione

| Fase | Contenuto                                                                                                                              | Gate                                               |
| ---- | -------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------- |
| K1   | `pins.env`, manifest, `build.sh`, `kernel-local`, `patches.list`; rimozione di script, vendoring e README; workflow `kernel-build.yml` | RPM prodotti in locale e in CI, config onorato     |
| K2   | pubblicazione OCI con cosign, SBOM, SLSA; debuginfo separato                                                                           | `cosign verify` sul tag                            |
| K3   | boot matrix in QEMU con le asserzioni della sezione 7                                                                                  | verde su Nehalem, host, UEFI+SB, BIOS              |
| K4   | `nvidia-kmod.yml` con i due rami, firma MOK                                                                                            | kmod compilati e firmati                           |
| K5   | bot di bump con auto-merge                                                                                                             | una PR di bump verde end-to-end                    |
| K6   | kernel guest MicroVM                                                                                                                   | `vmlinux` avvia in Firecracker con rootfs di prova |
| K7   | riproducibilità settimanale, benchmark di tendenza, `-O3` deciso dai numeri                                                            | primo report                                       |

Poi la v0 riprende con `ermete-os-kernel:<nvr>` nell'immagine. Ogni fase è un
insieme di commit verificabili da soli; la specifica si aggiorna se
l'implementazione scopre che un gancio Fedora non è come descritto.

## 13. Decisioni del maintainer (2026-09-03)

1. Serie: `stable` (segue 7.x con CachyOS, bump frequenti); `lts` (6.18) resta
   un valore possibile di `KERNEL_CHANNEL`.
2. `-O3` acceso di default come CachyOS; il benchmark di K7 lo conferma o lo spegne.
3. `RANDSTRUCT_FULL` era acceso; il 2026-09-04 la scelta è **Rust acceso, ThinLTO e
   `RANDSTRUCT` spenti**: in 7.1 `RUST` esclude entrambi (sezione 5), il kernel deve
   restare agnostico anche verso i driver futuri in Rust, ThinLTO vale pochi punti
   percentuali che `RANDSTRUCT_FULL` in parte annulla, e questa è la configurazione
   di Fedora e di CachyOS: il delta più corto da mantenere. ThinLTO tornerà quando
   upstream toglierà il vincolo; `RANDSTRUCT` resta escluso per costruzione.
4. Debuginfo pubblicato come OCI separato, retention di due versioni.
