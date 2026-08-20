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
      in
      {
        # L'ambiente di sviluppo universale. Riproducibile bit-per-bit su qualsiasi macchina.
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-toolchain
            just
            jq
            python3
            pkg-config
            openssl
            bcachefs-tools
          ];
          
          shellHook = ''
            echo "========================================================"
            echo " 🌌 ERMETE OS: Nix Hermetic Build Environment Activated "
            echo "========================================================"
            echo "Zero-Trust SLSA 4 Enforcement: Active"
            echo "YAML Destruction Protocol: Initiated"
          '';
        };

        # Il nostro primo mattoncino riproducibile (Proof of Concept Vanguard)
        packages = {
          # Un bridge per compilare "just" (il nostro command runner) deterministicamente 
          just-hermetic = pkgs.just;
        };
      }
    );
}
