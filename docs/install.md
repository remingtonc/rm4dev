<!-- Copyright (C) 2026 RM4 LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Installation

## Host requirements

- A working Rust toolchain if you are building `rm4dev` from source
- A working Podman installation available on `PATH`
- Permission for your user to run `podman info` successfully
- Network access when building the default image for the first time

`rm4dev` is designed around Podman-based local development containers. The default image and runtime model assume nested container workflows and privileged Podman execution inside the created container.

## Build from source

```text
cargo build --release
```

The resulting binary is `target/release/rm4dev`.

For a quick local install into Cargo's bin directory:

```text
cargo install --path .
```

## Smoke test

Before creating a container, confirm the local runtime is healthy:

```text
rm4dev agent precheck
```

This verifies that `podman --version` and `podman info` both succeed.

## Default image build behavior

When `RM4DEV_IMAGE` is not set, the CLI manages `localhost/rm4dev-agent:nix-fedora` automatically:

- `rm4dev image build` always rebuilds it
- `rm4dev image ensure` builds it only if missing
- `rm4dev agent new` and `rm4dev agent start` automatically ensure it exists before creating a new container

The embedded build context is unpacked into:

- `XDG_CACHE_HOME/rm4dev/images/nix-fedora`, or
- `~/.cache/rm4dev/images/nix-fedora`, or
- a temporary directory if neither cache location is available

## Host files and directories created by rm4dev

- `~/.cache/rm4dev/opencode-auth.json` is created on demand unless `--no-shared-auth` is used
- the image cache directory is created on first embedded build
- Podman itself creates local image and container state under its normal storage location

## Optional image override

Set `RM4DEV_IMAGE` to use a prebuilt image instead of the bundled `nix-fedora` image:

```text
RM4DEV_IMAGE=localhost/custom-agent:dev rm4dev agent new
```

When this variable is set, `rm4dev` does not auto-build or auto-ensure the image for you.
