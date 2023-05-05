#!/usr/bin/env bash

docker run -t --rm -v $PWD:/src -w /src nixos/nix nix --extra-experimental-features nix-command --extra-experimental-features flakes build
