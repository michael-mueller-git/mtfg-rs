# mtfg-rs

mtfg-rs is a rust rewrite of the Motion Tracking Funscript Generator (MTFG) Add-On from [Python-Funscript-Editor](https://github.com/michael-mueller-git/Python-Funscript-Editor). mtfg-rs is a motion tracking program to partially automate the generation of funscripts.

## Why does this project exists?

- A simple project to learn the basics of the blazing fast programming language rust and [asynchronous tokio runtime](https://tokio.rs/).
- To evaluate the speed advantage of rust compared to python for the MTFG part.

## What is the current goal of this project?

- Implement only the most necessary functions needed to create a funscript with a opencv video tracker.

## Usage

I recommend to use the nix flake from this repository to use the application. Use the wrapper script `mtfg-rs.sh` to start the application. During the first setup the application may need to compile some libraries, which may take a few minutes (depending on the speed of your computer).

## Compile

### Linux native

The package can also compiled with standalone cargo. Keep in mind that the application use [opencv-rust](https://github.com/twistedfall/opencv-rust) witch requires the OpenCV package installed on the system when compiles on system.

```bash
cargo build --release
```

### Windows (Cross Comping)

```bash
nix develop ".#build-windows"
```
