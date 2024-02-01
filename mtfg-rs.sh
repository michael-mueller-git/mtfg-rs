#!/usr/bin/env bash
unset LD_LIBRARY_PATH
root_dir="$(dirname $0)"
cd $root_dir && nix run ".#" -- "$@" 2>&1 | tee /tmp/mtfg-rs-nix.log
