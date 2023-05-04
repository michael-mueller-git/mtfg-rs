{
  description = "rust workspace";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay, ... }:
    let
      myapp = "mtfg-rs";
      rust-version = "1.65.0";
      overlays = [ rust-overlay.overlays.default ];
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system overlays; };
      pkgsMingw = pkgs.pkgsCross.mingwW64;
      lib = pkgs.lib;

      opencv-win = pkgsMingw.callPackage ./opencv-win.nix {
        pthreads = pkgsMingw.windows.mingw_w64_pthreads;
      };
      buildPlatformInputs = with pkgs; [
        (rust-bin.stable.${rust-version}.default.override {
          extensions =
            [ "rust-src" "llvm-tools-preview" "rust-analysis" ];
          targets = [ "x86_64-pc-windows-gnu" ];
        })
        rust-analyzer
        dbus
        xorg.libxcb
        opencv-win

        pkgsMingw.windows.mcfgthreads
      ];

      opencv = pkgs.opencv;
      wineLibPaths = (builtins.map (a: ''${a};'') [
        "${pkgsMingw.stdenv.cc.cc}/x86_64-w64-mingw32/lib/"
        "${pkgsMingw.windows.mcfgthreads}/bin/"
      ]) ++ [ "${opencv-win}/bin/" ];
      winePath = builtins.foldl' (x: y: x+y) "" wineLibPaths;

    in
    {
      packages.${system}.opencv-win = opencv-win;
      devShells.${system}.default = pkgsMingw.mkShell {
        packages = buildPlatformInputs;
        buildInputs = buildPlatformInputs;
        depsBuildBuild = with pkgs; [
          llvmPackages.clang
        ];
        WIN_PTHREADS = "${pkgsMingw.windows.mingw_w64_pthreads}/lib";
        RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
          pkgsMingw.windows.mcfgthreads
        ]) ++ (builtins.map (a: ''-l ${a}'') [
          "mcfgthread"
        ]);
        CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER = "${pkgs.wineWowPackages.stable}/bin/wine64";
        OPENCV_INCLUDE_PATHS = "${opencv-win}/include/opencv4";
        OPENCV_LINK_PATHS = "${opencv-win}/lib";
        OPENCV_LINK_LIBS = "opencv_highgui470,opencv_videoio470,opencv_video470,opencv_imgcodecs470,opencv_imgproc470,opencv_core470,opencv_tracking470,opencv_img_hash470,opencv_bioinspired470,opencv_line_descriptor470";
        OPENCV_DISABLE_PROBES = "vcpkg_cmake,vcpkg,cmake";
        LIBCLANG_PATH = "${pkgs.llvmPackages_11.libclang.lib}/lib";
        WINEPATH = winePath;

        shellHook = ''
          export PATH=$PATH:$HOME/.cargo/bin
          cargo build --release --target x86_64-pc-windows-gnu
          cp -fv ${opencv-win}/bin/* target/x86_64-pc-windows-gnu/release
          exit $?
        '';
      };
    };
}
