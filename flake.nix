{
  description = "Ermete OS - Immutable Build Factory (Zero-Trust)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        # The hermetic development and build shell
        devShells.default = pkgs.mkShell {
          name = "ermete-forge-env";
          
          buildInputs = with pkgs; [
            # Core Build Tools
            rpm
            python311
            python311Packages.networkx
            python311Packages.pyyaml
            
            # Languages
            rustc
            cargo
            mold
            sccache
            
            # Container & OS Build Tools
            podman
            skopeo
            osbuild
            
            # Security & Attestation
            cosign
            syft
            jq
            git
          ];

          shellHook = ''
            echo "=========================================================="
            echo "🛡️ Benvenuto nella Fabbrica Ermete OS (Nix Hermetic Shell)"
            echo "=========================================================="
            echo "Tutti i compilatori, le chiavi crittografiche e le dipendenze"
            echo "sono ora bloccati in un grafo matematico immutabile."
            export ERMETE_NIX_FACTORY="1"
          '';
        };
      }
    );
}
