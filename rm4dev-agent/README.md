<!-- Copyright (C) 2026 RM4 LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# rm4dev-agent

This directory holds the container image definitions used by `rm4dev`.

## Image variants

- `nix-fedora/` is the default and supported image definition. It is embedded into the Rust binary and auto-managed by `rm4dev image build`, `rm4dev image ensure`, `rm4dev agent new`, and `rm4dev agent start`.
- `brew-fedora/` is a local-only alternative image definition kept in the repository for experimentation and reference. The Rust CLI does not embed or auto-manage it.

## Scripts

- `./build.sh` builds both image variants locally
- each image directory also provides its own `build.sh` and `run.sh` helpers for manual work

## Runtime expectations

The images are designed for trusted local development workflows with nested Podman use and a tmux-based entrypoint that launches OpenCode automatically.
