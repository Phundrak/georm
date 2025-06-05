{ pkgs, nixpkgs, rust-overlay, ... }:
let
  overlays = [ (import rust-overlay) ];
  system = pkgs.stdenv.system;
  rustPkgs = import nixpkgs { inherit system overlays; };
  rustVersion = (rustPkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);
in {
  dotenv.enable = true;

  packages = with rustPkgs; [
    bacon
    cargo-deny
    just
    postgresql
    sqls
    sqlx-cli
    (rustVersion.override {
      extensions = [
        "rust-src"
        "rustfmt"
        "clippy"
        "rust-analyzer"
      ];
    })
  ];

  services.postgres = {
    enable = true;
    listen_addresses = "localhost";
    initialScript = ''
      CREATE USER georm WITH PASSWORD 'georm' SUPERUSER;
      CREATE DATABASE georm OWNER georm;
      GRANT ALL PRIVILEGES ON DATABASE georm TO georm;
    '';
  };
}
