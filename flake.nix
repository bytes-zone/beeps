{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs:
    inputs.flake-utils.lib.eachDefaultSystem (system:
      let pkgs = import inputs.nixpkgs { inherit system; };
      in {
        formatter = pkgs.nixpkgs-fmt;

        devShell = pkgs.mkShell {
          packages = [
            pkgs.rustc
            pkgs.cargo
            pkgs.cargo-edit
            pkgs.cargo-machete
            pkgs.rust-analyzer
            pkgs.clippy
            pkgs.rustfmt
            pkgs.libiconv
            pkgs.typos
            pkgs.postgresql_16

            pkgs.darwin.Security
          ];
        };
      }
    );
}
