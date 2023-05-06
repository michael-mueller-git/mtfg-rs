{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        name = "mtfg-rs";
        release-version = "0.0.2"; # must match version in Cargo.toml
        release-rev = "00d5e1bdf9328123ecf0c4c18da31ac14f437803";
        git = "https://github.com/michael-mueller-git/mtfg-rs";
        rust-version = "1.65.0";

        cargoToml = (builtins.fromTOML (builtins.readFile ./Cargo.toml));
        cargo-name = "${cargoToml.package.name}";

        overlays = [
          rust-overlay.overlays.default
          (self: super: {
            opencv = super.opencv.overrideAttrs (old: rec {
              buildInputs = old.buildInputs ++ [ pkgs.qt6.full ];
              cmakeFlags = old.cmakeFlags ++ [ "-DWITH_QT=6" ];
            });
          })
        ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        craneLib = crane.lib.${system};

        pkgsMingw = pkgs.pkgsCross.mingwW64;

        opencv-win = pkgsMingw.callPackage ./opencv-win.nix {
          pthreads = pkgsMingw.windows.mingw_w64_pthreads;
        };

        buildWindowsPlatformInputs = with pkgs; [
          (rust-bin.stable.${rust-version}.default.override {
            extensions = [ "rust-src" "llvm-tools-preview" "rust-analysis" ];
            targets = [ "x86_64-pc-windows-gnu" ];
          })
          rust-analyzer
          dbus
          xorg.libxcb
          opencv-win
          pkgsMingw.windows.mcfgthreads
        ];

        wineLibPaths = (builtins.map (a: ''${a};'') [
          "${pkgsMingw.stdenv.cc.cc}/x86_64-w64-mingw32/lib/"
          "${pkgsMingw.windows.mcfgthreads}/bin/"
        ]) ++ [ "${opencv-win}/bin/" ];

        winePath = builtins.foldl' (x: y: x + y) "" wineLibPaths;

        mtfg-rs-release-artifact = craneLib.downloadCargoPackageFromGit {
          inherit git;
          rev = "${release-rev}";
        };

        mtfg-rs-build-args = {
          buildInputs = [ pkgs.opencv ];
          nativeBuildInputs = [ pkgs.pkg-config pkgs.clang ];
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
        };

        mtfg-rs-linux-latest = craneLib.buildPackage (mtfg-rs-build-args // {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
        });

        mtfg-rs-linux-release = craneLib.buildPackage (mtfg-rs-build-args // {
          inherit name;
          version = "${release-version}";
          src = craneLib.cleanCargoSource (craneLib.path "${mtfg-rs-release-artifact}/${name}-${release-version}");
        });

      in
      {
        formatter = nixpkgs.legacyPackages.x86_64-linux.nixpkgs-fmt;

        packages.default = mtfg-rs-linux-latest;
        packages.latest = mtfg-rs-linux-latest;
        packages.release = mtfg-rs-linux-release;

        packages.opencv-win = opencv-win;

        devShells.build-windows = pkgsMingw.mkShell {
          packages = buildWindowsPlatformInputs;
          buildInputs = buildWindowsPlatformInputs;
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
            export BUILD_RESULT_CODE=$?
            mkdir -p target/x86_64-pc-windows-gnu/release
            cp -fv ${opencv-win}/bin/*.dll target/x86_64-pc-windows-gnu/release
            cp -fv ${pkgsMingw.stdenv.cc.cc}/x86_64-w64-mingw32/lib/*.dll target/x86_64-pc-windows-gnu/release
            cp -fv ${pkgsMingw.windows.mcfgthreads}/bin/*.dll target/x86_64-pc-windows-gnu/release
            exit $BUILD_RESULT_CODE
          '';
        };
      });
}
