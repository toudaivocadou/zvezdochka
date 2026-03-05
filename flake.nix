{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
          config.allowUnfree = true;
        };
        rustbin = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
          toolchain.default.override {
            extensions = ["rust-src"];
          });
      in {
        formatter = pkgs.alejandra;

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "toudaivocadou";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };

        nixosModules.default = {
          config,
          options,
          ...
        }: {
          # TODO: Systemd Service Module
        };

        devShells.default = pkgs.mkShell {
          packages =
            [
              rustbin
            ]
            ++ (with pkgs; [
              llvmPackages.libclang.lib
              llvmPackages.clang
              pkg-config
              cmake
              vcpkg
              rustPlatform.bindgenHook
              rustup
              simple-http-server
              http-server
              esbuild
              lldb
              qrrs
              qrencode
              sqlx-cli
            ]);

          env.RUST_SRC_PATH = "${rustbin}/lib/rustlib/src/rust/library";
          env.LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

          shellHook = ''
            echo "WONDERHOOOOOY!!!!"
          '';
        };
      }
    );
}
