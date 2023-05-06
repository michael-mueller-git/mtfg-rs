#!/usr/bin/env bash

root_dir="$(dirname $0)"
cd $root_dir && nix run ".#" -- "$@" 2>&1 | tee /tmp/mtfg-rs-nix.log
