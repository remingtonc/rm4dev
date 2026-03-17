#!/usr/bin/env bash
pushd brew-fedora && ./build.sh && popd
pushd nix-fedora && ./build.sh && popd