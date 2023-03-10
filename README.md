# mtfg-rs

mtfg-rs is a rust rewrite of the Motion Tracking Funscript Generator (MTFG) Add-On from [Python-Funscript-Editor](https://github.com/michael-mueller-git/Python-Funscript-Editor). mtfg-rs is a motion tracking program to partially automate the generation of funscripts.

## Compile

```bash
cargo +nightly build --release
```

## Why does this project exists?

- A simple project to learn the basics of the blazing fast programming language rust and [asynchronous tokio runtime](https://tokio.rs/).
- To evaluate the speed advantage of rust compared to python for the MTFG part.

## What is the current goal of this project?

- Implement only the most necessary functions needed to create a funscript with a opencv video tracker.

## TODO

### Cross Compiling

- https://packages.msys2.org/package/mingw-w64-x86_64-opencv
- https://github.com/twistedfall/opencv-rust/issues/333
- https://github.com/nix-community/naersk/tree/master/examples/cross-windows
- https://git.m-labs.hk/astro/nix-scripts/commit/a808a5d0900010e1c30de0afaa912fc6597840b2#diff-189c3d27cb167dbd8ca910a72cdeec174d418885
