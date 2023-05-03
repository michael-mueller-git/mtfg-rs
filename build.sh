
OPENCV_LINK_LIBS=opencv_core455,opencv_imgproc455 OPENCV_LINK_PATHS=$PWD/OpenCV-MinGW-Build-OpenCV-4.5.5-x64/x64/mingw/lib OPENCV_INCLUDE_PATHS=$PWD/OpenCV-MinGW-Build-OpenCV-4.5.5-x64/include RUSTC_LOG="rustc_codegen_ssa::back::link=info" cargo build --target x86_64-pc-windows-gnu -vv 2>&1 | tee host_build.txt
