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
        build-tools = with pkgs; [ rpm cpio buildah skopeo jq git gnutar xz curl wget rsync flex bison bc zstd perl pkg-config autoconf automake libtool ];
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

        packages = {
          just-hermetic = pkgs.just;

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

          builderImage = pkgs.dockerTools.buildLayeredImage {
            name = "ghcr.io/hr-mes/ermete-os-builder";
            tag = "latest";
            contents = [ pkgs.bashInteractive pkgs.coreutils pkgs.findutils pkgs.gnused pkgs.gawk pkgs.cacert pkgs.tzdata pkgs.shadow ] ++ security-tools ++ c-toolchain ++ rust-tools ++ build-tools ++ system-deps;
            config = {
              Cmd = [ "/bin/bash" ];
              Env = [
                "PATH=/bin:/usr/bin"
                "HOME=/root"
                "CC=clang"
                "CXX=clang++"
                "LD=ld.lld"
                "LLVM=1"
                "LLVM_IAS=1"
              ];
            };
            fakeRootCommands = ''
              mkdir -p /root
              mkdir -p /lib64 /lib /usr/lib /usr/lib64 /lib/x86_64-linux-gnu /usr/lib/x86_64-linux-gnu
              
              ln -s ${pkgs.glibc}/lib/ld-linux-x86-64.so.2 /lib64/ld-linux-x86-64.so.2 || true
              ln -s ${pkgs.glibc}/lib/ld-linux.so.2 /lib/ld-linux.so.2 || true
              ln -s ${pkgs.glibc}/lib/libc.so.6 /lib64/libc.so.6 || true
              ln -s ${pkgs.glibc}/lib/libm.so.6 /lib64/libm.so.6 || true
              ln -s ${pkgs.glibc}/lib/libpthread.so.0 /lib64/libpthread.so.0 || true
              ln -s ${pkgs.glibc}/lib/libdl.so.2 /lib64/libdl.so.2 || true
              ln -s ${pkgs.glibc}/lib/librt.so.1 /lib64/librt.so.1 || true
              ln -s ${pkgs.stdenv.cc.cc.lib}/lib/libstdc++.so.6 /usr/lib64/libstdc++.so.6 || true
              ln -s ${pkgs.stdenv.cc.cc.lib}/lib/libstdc++.so.6 /usr/lib/libstdc++.so.6 || true
              ln -s ${pkgs.stdenv.cc.cc.lib}/lib/libgcc_s.so.1 /lib64/libgcc_s.so.1 || true
              ln -s ${pkgs.stdenv.cc.cc.lib}/lib/libgcc_s.so.1 /usr/lib64/libgcc_s.so.1 || true

              ln -s ${pkgs.glibc}/lib/ld-linux-x86-64.so.2 /lib/x86_64-linux-gnu/ld-linux-x86-64.so.2 || true
              ln -s ${pkgs.glibc}/lib/libc.so.6 /lib/x86_64-linux-gnu/libc.so.6 || true
              ln -s ${pkgs.glibc}/lib/libm.so.6 /lib/x86_64-linux-gnu/libm.so.6 || true
              ln -s ${pkgs.glibc}/lib/libpthread.so.0 /lib/x86_64-linux-gnu/libpthread.so.0 || true
              ln -s ${pkgs.glibc}/lib/libdl.so.2 /lib/x86_64-linux-gnu/libdl.so.2 || true
              ln -s ${pkgs.glibc}/lib/librt.so.1 /lib/x86_64-linux-gnu/librt.so.1 || true
              ln -s ${pkgs.stdenv.cc.cc.lib}/lib/libstdc++.so.6 /lib/x86_64-linux-gnu/libstdc++.so.6 || true
              ln -s ${pkgs.stdenv.cc.cc.lib}/lib/libgcc_s.so.1 /lib/x86_64-linux-gnu/libgcc_s.so.1 || true
            '';
          };
        };
      }
    );
}
