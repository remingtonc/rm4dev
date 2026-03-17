<!-- Copyright (C) 2026 RM4 LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# brew-fedora

`brew-fedora` is an alternative `rm4dev-agent` image definition that installs developer tooling through Homebrew on top of Fedora.

## Status

This variant remains in the repository for local experimentation and reference. The Rust `rm4dev` CLI does not embed it, auto-build it, or auto-select it by default.

## Local usage

- `./build.sh` builds `localhost/rm4dev-agent:brew-fedora`
- `./run.sh` starts the image with the manual runtime settings defined in that script

## Comparison with `nix-fedora`

- `nix-fedora` is the supported default used by `rm4dev`
- `brew-fedora` is useful when you specifically want a Homebrew-managed toolchain inside the container
