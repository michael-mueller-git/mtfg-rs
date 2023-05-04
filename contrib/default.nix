{ lib
, naersk
, stdenv
, clangStdenv
, hostPlatform
, targetPlatform
, pkg-config
, libiconv
, rustfmt
, cargo
, rustc
, llvmPackages
, opencv
, clang
, pkgs
}:

let
  cargoToml = (builtins.fromTOML (builtins.readFile ../Cargo.toml));
in

naersk.lib."${targetPlatform.system}".buildPackage rec {
  src = ../.;

  buildInputs = [
    rustfmt
    pkg-config
    cargo
    rustc
    libiconv
    opencv
    clang
  ];
  nativeBuildInputs = with pkgs; [ pkgsStatic.stdenv.cc ];
  checkInputs = [ cargo rustc ];

  doCheck = true;
  CARGO_BUILD_INCREMENTAL = "false";
  RUST_BACKTRACE = "full";
  copyLibs = true;

  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
  OPENCV_INCLUDE_PATHS = "${opencv}/include/opencv4";
  OPENCV_LINK_PATHS = "${opencv}/lib";
  OPENCV_LINK_LIBS = "opencv_highgui470,opencv_videoio470,opencv_video470,opencv_imgcodecs470,opencv_imgproc470,opencv_core470,opencv_tracking470,opencv_img_hash470,opencv_bioinspired470,opencv_line_descriptor470";
  OPENCV_DISABLE_PROBES = "vcpkg_cmake,vcpkg,cmake";

  name = cargoToml.package.name;
  version = cargoToml.package.version;

  meta = with lib; {
    description = cargoToml.package.description;
    homepage = cargoToml.package.homepage;
    license = with licenses; [ mit ];
    maintainers = with maintainers; [ ];
  };
}

