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

- Add Interpolation for `skip_frames` > 0.
