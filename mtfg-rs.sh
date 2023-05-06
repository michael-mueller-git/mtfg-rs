#!/usr/bin/env bash

root_dir="$(dirname $0)"
echo "dir: $root_dir" > /tmp/mtfg-nix.log
cd $root_dir && nix run ".#" -- "$@" 2>&1 | tee /tmp/mtfg-rs-nix.log
