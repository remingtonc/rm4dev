<!-- Copyright (C) 2026 RM4 LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Architecture

## High-level flow

`rm4dev` is a small Rust CLI that coordinates three concerns:

1. Parse CLI input into explicit container or image actions.
2. Resolve local Podman state and decide whether to create, resume, inspect, or remove a container.
3. Materialize and build the bundled default image context when needed.

## Rust modules

- `src/cli.rs` parses commands and normalizes user input into internal command enums
- `src/agent.rs` handles container lifecycle decisions, Podman argument construction, and auth mount setup
- `src/image.rs` embeds the default image context, writes it to a cache directory, and runs `podman build`
- `src/mounts.rs` validates and canonicalizes bind-mount arguments
- `src/naming.rs` normalizes `rm4dev-agent-*` container names
- `src/process.rs` wraps subprocess execution and error rendering
- `src/error.rs` defines the CLI's error model and exit-code behavior

## Embedded image context

The default `nix-fedora` image under `rm4dev-agent/nix-fedora/` is embedded into the compiled binary with `include_dir`.

At runtime, `src/image.rs` hashes that embedded directory, writes it into a cache directory, and uses Podman to build from the cached materialized context. This keeps the binary self-contained while still allowing the image definition to live in normal source files.

## Container runtime model

- container names are normalized under the `rm4dev-agent-` prefix
- `agent start` uses a decision plan to either resume an existing container or create a new one
- newly created containers receive privileged Podman settings, tmpfs mounts, and optional auth or user-supplied bind mounts
- image auto-build happens only for the default image path and only when `RM4DEV_IMAGE` is not set

## Image variants in this repository

- `rm4dev-agent/nix-fedora/` is the supported default runtime and the only image embedded into the Rust binary
- `rm4dev-agent/brew-fedora/` remains in the repository as an alternative local image definition but is not auto-managed by the CLI
