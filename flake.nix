{
  description = "Ermete OS - Singularity Level 5 (Nix Hermetic Build Factory)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-toolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "x86_64-unknown-linux-gnu" ];
        };
        
        security-tools = with pkgs; [ syft cosign ];
        c-toolchain = with pkgs; [ gcc gnumake cmake mold llvmPackages_latest.llvm llvmPackages_latest.clang llvmPackages_latest.lld ccache bpf-linker pahole elfutils ];
        rust-tools = with pkgs; [ rust-toolchain sccache clippy rustfmt cargo-deny cargo-vet cargo-fuzz ];
        # Toolset POSIX di base incluso: rpmbuild, %autosetup e i Makefile upstream
        # danno per scontati grep, diff, patch, gzip, file, which, m4, gettext.
        build-tools = with pkgs; [ rpm cpio buildah skopeo jq git gnutar xz curl wget rsync flex bison bc zstd perl pkg-config autoconf automake libtool gnugrep diffutils patch which file gzip bzip2 unzip m4 gettext python3 ];
        system-deps = with pkgs; [ zlib openssl policycoreutils spdlog systemd nodejs_22 nlohmann_json fmt speechd gnupg ipxe ncurses iproute2 fio gtk4 pango cairo gtk4-layer-shell glib pkg-config ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-toolchain just jq python3 pkg-config openssl bcachefs-tools
          ];
          shellHook = ''
            echo "========================================================"
            echo " ERMETE OS: Nix Hermetic Build Environment Activated "
            echo "========================================================"
          '';
        };

        packages = rec {
          just-hermetic = pkgs.just;

          ermete-telemetry-rpm = pkgs.runCommand "ermete-telemetry-rpm" {
            nativeBuildInputs = [ pkgs.nfpm ];
            # Dipende matematicamente dalla compilazione Rust pura
            src = ermete-core;
          } ''
            mkdir -p $out/RPMS
            cat > nfpm.yaml <<EOF
name: "ermete-telemetry"
arch: "x86_64"
platform: "linux"
version: "1.0.0"
section: "default"
priority: "extra"
maintainer: "Ermete OS"
description: "Ermete Telemetry Daemon"
vendor: "Ermete OS"
license: "MIT"
contents:
  - src: "$src/bin/ermete-telemetry"
    dst: "/usr/bin/ermete-telemetry"
EOF
            # Infallibilit�: Genera l'RPM senza root e senza dnf!
            nfpm pkg --packager rpm --target $out/RPMS/ermete-telemetry.rpm
          '';

          ermete-core = (pkgs.makeRustPlatform {
            cargo = rust-toolchain;
            rustc = rust-toolchain;
          }).buildRustPackage {
            pname = "ermete-os-core";
            version = "1.0.0";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [ openssl glib gtk4 wayland wayland-protocols ];
            doCheck = false;
          };

          # Compatibilità FHS del builder: directory àncora e symlink verso la glibc e le
          # librerie di runtime di gcc di nixpkgs, come contenuto immutabile dell'immagine.
          # È una derivazione ordinaria, costruibile e ispezionabile da sola:
          #   nix build .#builder-fhs-compat
          builder-fhs-compat = pkgs.runCommand "ermete-builder-fhs-compat" { } ''
            mkdir -p $out/lib64 $out/lib/x86_64-linux-gnu $out/usr/lib $out/usr/lib64 $out/usr/lib/x86_64-linux-gnu

            for lib in ld-linux-x86-64.so.2 libc.so.6 libm.so.6 libpthread.so.0 libdl.so.2 librt.so.1; do
              ln -s ${pkgs.glibc}/lib/$lib $out/lib64/$lib
              ln -s ${pkgs.glibc}/lib/$lib $out/lib/x86_64-linux-gnu/$lib
            done

            for lib in libstdc++.so.6 libgcc_s.so.1; do
              ln -s ${pkgs.stdenv.cc.cc.lib}/lib/$lib $out/usr/lib64/$lib
              ln -s ${pkgs.stdenv.cc.cc.lib}/lib/$lib $out/lib/x86_64-linux-gnu/$lib
            done
            ln -s ${pkgs.stdenv.cc.cc.lib}/lib/libstdc++.so.6 $out/usr/lib/libstdc++.so.6
            ln -s ${pkgs.stdenv.cc.cc.lib}/lib/libgcc_s.so.1 $out/lib64/libgcc_s.so.1
          '';

          # Macro RPM di systemd (%_unitdir, %systemd_post, …). In Fedora le fornisce
          # systemd-rpm-macros; l'rpm di nixpkgs non le ha, e rpmbuild lascerebbe
          # `%systemd_post` come testo letterale negli scriptlet. Rese dal template
          # ufficiale di systemd con i percorsi del sistema di destinazione (Fedora,
          # rootprefix /usr), non del builder: gli scriptlet girano sull'immagine finale.
          # Il file segue il layout di rpm (macros.d/) e viene fuso nella sua config dir
          # da builder-rpm-configdir, esposta con RPM_CONFIGDIR.
          builder-rpm-macros-systemd =
            let
              targetPaths = {
                LIBEXECDIR = "/usr/lib/systemd";
                SYSTEMD_UPDATE_HELPER_PATH = "/usr/lib/systemd/systemd-update-helper";
                SYSTEM_DATA_UNIT_DIR = "/usr/lib/systemd/system";
                USER_DATA_UNIT_DIR = "/usr/lib/systemd/user";
                SYSTEM_PRESET_DIR = "/usr/lib/systemd/system-preset";
                USER_PRESET_DIR = "/usr/lib/systemd/user-preset";
                SYSTEM_GENERATOR_DIR = "/usr/lib/systemd/system-generators";
                USER_GENERATOR_DIR = "/usr/lib/systemd/user-generators";
                SYSTEM_ENV_GENERATOR_DIR = "/usr/lib/systemd/system-environment-generators";
                USER_ENV_GENERATOR_DIR = "/usr/lib/systemd/user-environment-generators";
                SYSTEMD_CATALOG_DIR = "/usr/lib/systemd/catalog";
                UDEV_HWDB_DIR = "/usr/lib/udev/hwdb.d";
                UDEV_RULES_DIR = "/usr/lib/udev/rules.d";
                KERNEL_INSTALL_DIR = "/usr/lib/kernel/install.d";
                BINFMT_DIR = "/usr/lib/binfmt.d";
                SYSCTL_DIR = "/usr/lib/sysctl.d";
                SYSUSERS_DIR = "/usr/lib/sysusers.d";
                TMPFILES_DIR = "/usr/lib/tmpfiles.d";
                USER_TMPFILES_DIR = "/usr/share/user-tmpfiles.d";
                ENVIRONMENT_DIR = "/usr/lib/environment.d";
                MODULESLOAD_DIR = "/usr/lib/modules-load.d";
                MODPROBE_DIR = "/usr/lib/modprobe.d";
              };
              substitutions = pkgs.lib.concatStringsSep " "
                (pkgs.lib.mapAttrsToList (name: path: "-e 's|{{${name}}}|${path}|g'") targetPaths);
            in
            pkgs.runCommand "ermete-builder-rpm-macros-systemd" { } ''
              mkdir -p $out/macros.d
              sed ${substitutions} ${pkgs.systemd.src}/src/rpm/macros.systemd.in > $out/macros.d/macros.systemd
              if grep -q '{{' $out/macros.d/macros.systemd; then
                echo "macros.systemd: variabili del template non sostituite:" >&2
                grep -o '{{[A-Za-z_]*}}' $out/macros.d/macros.systemd | sort -u >&2
                exit 1
              fi
            '';

          # Config dir di rpm del builder: quella di nixpkgs più le macro di systemd.
          # rpm la trova tramite RPM_CONFIGDIR (config.Env dell'immagine); l'rpm di
          # nixpkgs non consulta /etc/rpm.
          builder-rpm-configdir = pkgs.symlinkJoin {
            name = "ermete-builder-rpm-configdir";
            paths = [ "${pkgs.rpm}/lib/rpm" builder-rpm-macros-systemd ];
          };

          builderImage = pkgs.dockerTools.buildLayeredImage {
            name = "ghcr.io/hr-mes/ermete-os-builder";
            tag = "latest";
            contents = [ builder-fhs-compat pkgs.bashInteractive pkgs.coreutils pkgs.findutils pkgs.gnused pkgs.gawk pkgs.cacert pkgs.tzdata pkgs.shadow ] ++ security-tools ++ c-toolchain ++ rust-tools ++ build-tools ++ system-deps;
            config = {
              Cmd = [ "/bin/bash" ];
              Env = [
                "PATH=/bin:/usr/bin"
                "HOME=/root"
                # Bundle CA di pkgs.cacert: senza questa variabile curl, cargo e git
                # di nixpkgs non verificano alcun certificato TLS (idioma dockerTools).
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
                "RPM_CONFIGDIR=${builder-rpm-configdir}"
                "CC=clang"
                "CXX=clang++"
                "LD=ld.lld"
                "LLVM=1"
                "LLVM_IAS=1"
              ];
            };
            # Directory mutabili dell'immagine. nixpkgs esegue extraCommands nella radice del
            # layer di personalizzazione con percorsi relativi (manuale: `mkdir -m 1777 tmp`).
            # /root serve a rpmbuild (HOME=/root); /tmp e /var/tmp (%_tmppath) non esistono
            # in un'immagine buildLayeredImage se nessuno li crea.
            extraCommands = ''
              mkdir -m 0700 -p root
              mkdir -m 1777 -p tmp var/tmp
            '';
          };
        };
      }
    );
}
