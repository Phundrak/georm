{
  description = "Georm, a simple, opiniated SQLx ORM for PostgreSQL";

  inputs = {
    nixpkgs.url      = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url  = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachSystem ["x86_64-linux"] (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustVersion = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);
      in {
        devShell = with pkgs; mkShell {
          buildInputs = [
            bacon
            cargo
            cargo-deny
            just
            rust-analyzer
            (rustVersion.override {
              extensions = [
                "rust-src"
                "rustfmt"
                "clippy"
                "rust-analyzer"
              ];
            })
            sqls
            sqlx-cli
          ];
        };
      });
}
