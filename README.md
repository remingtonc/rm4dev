<!-- Copyright (C) 2026 RM4 LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# rm4dev

`rm4dev` is a Rust CLI for creating, resuming, and inspecting Podman-based agent development containers.

It is built around a default `nix-fedora` image that bundles a developer toolchain, tmux, and OpenCode. The CLI can build that image from embedded source, create `rm4dev-agent-*` containers, and reattach to them later.

## Quick start

```text
cargo build --release
./target/release/rm4dev agent precheck
./target/release/rm4dev agent new demo
```

## Command surface

```text
rm4dev agent precheck
rm4dev agent list
rm4dev agent new [--no-shared-auth] [name] [host_path:container_path ...]
rm4dev agent start [--no-shared-auth] [name] [host_path:container_path ...]
rm4dev agent stop [name]
rm4dev agent rm [name]
rm4dev agent attach [name]
rm4dev agent enter [name]
rm4dev image build [image]
rm4dev image ensure [image]
```

## Operational behavior

- Names are normalized to `rm4dev-agent-<word>`. Passing `alpha` becomes `rm4dev-agent-alpha`.
- `new` always creates a new container. If no name is provided, it generates one from the current Unix timestamp.
- `start` resumes an existing container when it can resolve one unambiguously; otherwise it creates a new container.
- `start` treats `--no-shared-auth` and mount arguments as create-only signals when no matching container already exists.
- `list` shows all discovered `rm4dev-agent-*` containers, including stopped containers, with image and status columns.
- `enter` opens `/bin/bash -l` inside a running container by default. Override the shell path with `RM4DEV_ENTER_SHELL`.

## Host side effects

- New containers run `podman run --privileged` and mount tmpfs at `/tmp` and `/run`.
- Shared OpenCode auth is enabled by default. `rm4dev` creates `~/.cache/rm4dev/opencode-auth.json` on demand and bind-mounts it into `/root/.local/share/opencode/auth.json`.
- Additional mounts are bind mounts. Host paths are canonicalized and must already exist.
- Embedded image builds unpack into `XDG_CACHE_HOME/rm4dev/images/nix-fedora` or `~/.cache/rm4dev/images/nix-fedora`.

See `docs/security.md` for the security and compliance implications of these defaults.

## Image behavior

By default, `rm4dev` uses `localhost/rm4dev-agent:nix-fedora`.

```text
RM4DEV_IMAGE=my-image ./target/release/rm4dev agent new
```

- `rm4dev image build` builds the default image.
- `rm4dev image build localhost/custom-agent:dev` builds the same embedded context under a custom tag.
- `rm4dev image ensure` only builds when the target image is missing locally.
- `rm4dev agent new` and `rm4dev agent start` automatically ensure the default image exists before creating a new container.
- Setting `RM4DEV_IMAGE` disables implicit image management; the referenced image becomes user-managed.

The embedded image build still relies on network access for upstream container layers and package downloads.

## Documentation map

- `docs/install.md` - prerequisites, build, installation, and host requirements
- `docs/usage.md` - common workflows, environment variables, and command examples
- `docs/security.md` - privileged container model, bind mounts, and operational safeguards
- `docs/architecture.md` - CLI/module layout and embedded image build design
- `docs/troubleshooting.md` - common failures and recovery steps
- `CONTRIBUTING.md` - development workflow and contribution expectations
- `COPYRIGHT.md` - project copyright position and third-party material notes

## Repository layout

- `src/` - CLI parsing, container lifecycle, image management, and process helpers
- `rm4dev-agent/nix-fedora/` - default image context embedded into the Rust binary
- `rm4dev-agent/brew-fedora/` - legacy or experimental image variant for local-only workflows
- `build.sh` - convenience build for the bundled image definitions and release binary

## License

`rm4dev` is licensed under the GNU General Public License, version 3 or any later version. See `COPYING`.

Unless a file says otherwise, the project-authored materials in this repository are copyright RM4 LLC. Third-party dependencies and packages installed into generated images remain under their own licenses.
