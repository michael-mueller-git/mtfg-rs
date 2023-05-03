{
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs/nixos-unstable";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
  };
  outputs = { nixpkgs, flake-utils, ... }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
      };
      mtfg-rs = (with pkgs; stdenv.mkDerivation {
          pname = "mtfg-rs";
          version = "0.0.1";
          src = fetchgit {
            url = "https://github.com/michael-mueller-git/mtfg-rs.git";
            rev = "77dcce345985e8f62fe606915536519ca48cbdd7";
            sha256 = "sha256-067V0XolutU9UfSXaWd9jbu3WgqUPUBDMtQWWlOVxnM=";
            fetchSubmodules = true;
          };
          nativeBuildInputs = [
            clang
            cargo
            qt6.full
            opencv
          ];
          buildPhase = "cargo build  --release";
          installPhase = ''
            mkdir -p $out/bin
            mv target/release/mtfg-rs mtfg-rs $out/bin
          '';
        }
      );
    in rec {
      defaultApp = flake-utils.lib.mkApp {
        drv = defaultPackage;
      };
      defaultPackage = mtfg-rs;
      devShell = pkgs.mkShell {
        buildInputs = [
          mtfg-rs
          pkgs.qt6.full
          pkgs.opencv
        ];
      };
    }
  );
}
