{
  description = "Ermete OS - The Chimera Bedrock Environment";

  inputs = {
    # Usiamo il branch stabile di nixpkgs per massima riproducibilità
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    # Rust overlay per avere le versioni esatte (se serve in futuro)
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }: 
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      
      # Strumenti Crittografici, Sicurezza e SBOM (Sostituiscono i `curl` bash)
      security-tools = with pkgs; [
        syft             # Generazione SBOM
        cosign           # Firma OCI / Zero-Trust Attestation
        cargo-kani       # Kani Verifier per prove formali
      ];

      # Toolchain C/C++, eBPF e LLVM/BOLT (La Chimera)
      c-toolchain = with pkgs; [
        gcc
        gnumake
        cmake
        mold
        llvmPackages_latest.llvm
        llvmPackages_latest.clang
        llvmPackages_latest.lld
        ccache
        bpf-linker       # Linker eBPF (Niente più .tar.zst corrotti)
        pahole           # BTF generation per eBPF (dwarves in Fedora)
        elfutils         # libelf
      ];

      # Toolchain Rust
      rust-toolchain = with pkgs; [
        rustc
        cargo
        rustfmt
        clippy
        rust-bindgen
        sccache
      ];

      # Strumenti di Build e Packaging (Sostituiscono dnf/rpm-build)
      build-tools = with pkgs; [
        rpm
        cpio
        buildah
        skopeo
        jq
        git
        gnutar
        xz
        curl
        wget
        rsync
        flex
        bison
        bc
        zstd
        perl
        pkg-config
        autoconf
        automake
        libtool
      ];

      # Dipendenze di libreria del sistema e Kernel
      system-deps = with pkgs; [
        zlib
        openssl
        policycoreutils
        spdlog
        systemd
        nlohmann_json
        fmt
        speechd
        gnupg
        ipxe
        ncurses
        iproute2
        fio
      ];

    in {
      # Immagine OCI per il Builder (Sostituisce il Containerfile)
      packages.${system}.builderImage = pkgs.dockerTools.buildLayeredImage {
        name = "ghcr.io/hr-mes/ermete-os-builder";
        tag = "latest";
        contents = [ pkgs.bashInteractive pkgs.coreutils pkgs.findutils pkgs.gnused pkgs.gawk pkgs.cacert pkgs.tzdata pkgs.shadow ] ++ security-tools ++ c-toolchain ++ rust-toolchain ++ build-tools ++ system-deps;
        config = {
          Cmd = [ "/bin/bash" ];
          Env = [
            "PATH=/bin:/usr/bin"
            "CC=clang"
            "CXX=clang++"
            "LD=ld.lld"
            "LLVM=1"
            "LLVM_IAS=1"
          ];
        };
      };

      # L'ambiente di sviluppo nativo (nix develop) e Builder Environment
      devShells.${system}.default = pkgs.mkShell {
        name = "ermete-os-bedrock-builder";
        
        buildInputs = security-tools ++ c-toolchain ++ rust-toolchain ++ build-tools ++ system-deps;

        # Variabili d'ambiente essenziali iniettate al volo
        shellHook = ''
          echo "========================================================="
          echo " 🌋 Ermete OS - Nix Bedrock Builder Attivato"
          echo "========================================================="
          echo "[*] LLVM / Clang        : $(clang --version | head -n 1)"
          echo "[*] Rust Toolchain      : $(rustc --version)"
          echo "[*] Security Tools      : Cosign $(cosign version 2>&1 | grep GitVersion | awk '{print $2}') | Syft $(syft --version | awk '{print $2}')"
          echo "[*] eBPF Linker         : $(bpf-linker --version 2>/dev/null || echo 'Installed')"
          echo "========================================================="
          
          # Forziamo LLVM e Clang come default al posto di GCC per la Chimera Build
          export CC=clang
          export CXX=clang++
          export LD=ld.lld
          export LLVM=1
          export LLVM_IAS=1
          
          # Setup sicuro di sccache e ccache
          export SCCACHE_DIR="$PWD/.sccache"
          export CCACHE_DIR="$PWD/.ccache"
        '';
      };
    };
}
