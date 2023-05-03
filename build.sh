
RUSTC_LOG="rustc_codegen_ssa::back::link=info" cargo build --target x86_64-pc-windows-gnu -vv 2>&1 | tee host_build.txt
