# Fase 3 Vanguard: Sostituiamo il .spec RPM con una derivazione pura Nix.
{ pkgs ? import <nixpkgs> {} }:

pkgs.stdenv.mkDerivation {
  pname = "ermete-just";
  version = "1.34.0";

  src = pkgs.fetchFromGitHub {
    owner = "casey";
    repo = "just";
    rev = "1.34.0";
    sha256 = "sha256-nix-hash-placeholder-qui-inseriremo-l-hash-reale";
  };

  buildInputs = [ pkgs.cargo pkgs.rustc ];

  buildPhase = ''
    cargo build --release --locked
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp target/release/just $out/bin/
  '';

  meta = with pkgs.lib; {
    description = "Command runner nativo e sigillato crittograficamente per Ermete OS";
    homepage = "https://github.com/hr-mes/ermete-os";
    license = licenses.mit;
  };
}
