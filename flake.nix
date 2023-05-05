{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.lib.${system};
      in
    {
      packages.default = craneLib.buildPackage {
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        buildInputs = [ pkgs.opencv ];
        nativeBuildInputs = [ pkgs.pkg-config pkgs.clang ];
        OPENCV_DISABLE_PROBES = "vcpkg_cmake,vcpkg,cmake";
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      };
    });
}
