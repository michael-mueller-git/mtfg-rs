app = pkgs.rustPlatform.buildRustPackage {
  pname = "mtfg-rs";
  version = "0.0.1";
  src = ../.;
  cargoBuildFlags = "";

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  nativeBuildInputs = [ pkgs.pkg-config ];
  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
  OPENCV_INCLUDE_PATHS = "${pkgs.opencv}/include/opencv4";
  OPENCV_LINK_PATHS = "${pkgs.opencv}/lib";
  OPENCV_LINK_LIBS = "opencv_highgui470,opencv_videoio470,opencv_video470,opencv_imgcodecs470,opencv_imgproc470,opencv_core470,opencv_tracking470,opencv_img_hash470,opencv_bioinspired470,opencv_line_descriptor470";
  OPENCV_DISABLE_PROBES = "vcpkg_cmake,vcpkg,cmake";
};
