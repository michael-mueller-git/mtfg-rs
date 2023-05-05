{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
      nixpkgs.overlays = [
        (self: super: {
          yabai = super.opencv.overrideAttrs (old: rec {
            buildInputs = old.buildInputs ++ [pkgs.qt5.full];
            cmakeFlags = old.cmakeFlags ++ ["-DWITH_QT=ON"];
          });
        })
      ];
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let


        pkgs = import nixpkgs {
          inherit system;
        };
        craneLib = crane.lib.${system};
      in
    {


      packages.default = craneLib.buildPackage {
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        buildInputs = [ pkgs.opencv ];
        nativeBuildInputs = [ pkgs.pkg-config pkgs.clang ];
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      };
    });
}
