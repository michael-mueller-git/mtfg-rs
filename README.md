# mtfg-rs

mtfg-rs is a rust rewrite of the Motion Tracking Funscript Generator (MTFG) Add-On from [Python-Funscript-Editor](https://github.com/michael-mueller-git/Python-Funscript-Editor).

## Why does this project exists?

- A simple project to learn the basics of the blazing fast programming language rust and [asynchronous tokio runtime](https://tokio.rs/).
- To evaluate the speed advantage of rust compared to python for the MTFG part.

## What is the current goal of this project?

- Implement only the most necessary functions needed to create a funscript with a OpenCV video tracker.

## Setup

Execute the installer script from `./contrib/Installer` to setup OpenFunscripter with mtfg-rs Add-On on your computer. The application need the [nix package manager](https://nixos.org/download.html) installed on your system. The first setup need to compile some libraries witch take a few minutes.

## Usage

The installer add the Extension `mtfg-rs` to your OpenFunscripter menu where you can control the application.

## Compile (Experts and Developer)

### Linux native

The package can also compiled with standalone cargo. Keep in mind that the application use [opencv-rust](https://github.com/twistedfall/opencv-rust) witch requires the OpenCV package installed on the system when compiles on system.

```bash
cargo build --release
```

### Windows (Cross Compiling)

```bash
nix develop ".#build-windows"
```

NOTE: The windows binary is currently broken. The ffmpeg process is not able to extract the frame data on windows.
