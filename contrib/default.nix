{ pkgs, nixpkgs, system, makeRustPlatform, rust-overlay }:
let
  rustPkgs = import nixpkgs {
    inherit system;
    overlays = [ (import rust-overlay) ];
  };

  rustVersion = "1.65.0";

  wasmTarget = "x86_64-unknown-linux-gnu";

  rustWithWasmTarget = rustPkgs.rust-bin.stable.${rustVersion}.default.override {
    targets = [ wasmTarget ];
  };

  rustPlatformWasm = makeRustPlatform {
    cargo = rustWithWasmTarget;
    rustc = rustWithWasmTarget;
  };

  common = {
    version = "0.0.1";

    nativeBuildInputs = [ pkgs.pkg-config pkgs.clang pkgs.libcxx ];
    PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
    LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
    OPENCV_INCLUDE_PATHS = "${pkgs.opencv}/include/opencv4";
    OPENCV_LINK_PATHS = "${pkgs.opencv}/lib";
    OPENCV_LINK_LIBS = "opencv_highgui470,opencv_videoio470,opencv_video470,opencv_imgcodecs470,opencv_imgproc470,opencv_core470,opencv_tracking470,opencv_img_hash470,opencv_bioinspired470,opencv_line_descriptor470";
    OPENCV_DISABLE_PROBES = "vcpkg_cmake,vcpkg,cmake";
  };
in {
  app = pkgs.rustPlatform.buildRustPackage (common // {
    pname = "mtfg-rs";
    src = ../.;

    cargoLock = {
      lockFile = ../Cargo.lock;
    };
  });
 }
