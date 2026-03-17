<!-- Copyright (C) 2026 RM4 LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# rm4dev
`rm4dev` is a CLI for managing local software development with AI agents (Agent). At its core, this is a wrapper of Podman. Rootless containers will launch in the foreground of the terminal with `tmux` and `opencode` as an interface. **Only Linux has been tested.**

## Background
Current Agent workflows seem to operate in your regular userspace and, while controls are present, the blast radius of misbehavior in operation or access is profound. This tool's mitigation is to utilize Podman for rootless containerization enabling the Agent to operate more autonomously, atomically, with more focused context, and greater system control.

This approach does not mitigate malicious behaviors. It only provides the Agent a more limited scope of access to your host system and data, with more freedom to accomplish work.

## Example Workflow
An example workflow for a new agent container on a particular git repo:
```bash
~/Development/q 
❯ git clone git@github.com:remingtonc/stalwart.git
Cloning into 'stalwart'...
...

~/Development/q 
❯ rm4dev agent new stalwart stalwart/:/work/stalwart
# tmux with opencode is launched
[exited]

~/Development/q 
❯ rm4dev agent list
NAME                     IMAGE                              STATUS
rm4dev-agent-rm4dev-cli  localhost/rm4dev-agent:nix-fedora  Up 4 hours
rm4dev-agent-stalwart    localhost/rm4dev-agent:nix-fedora  Exited (0) 9 seconds ago
```

## Technical Details
* Largely AI-generated code.
* Depends on Podman.
  * Expects podman-in-podman (rootful in rootless) to enable Agent container development workflows. Does not support podman-in-podman-in-podman-...
  * `root` user is used in container to simplify mount/file sharing, and eases Agent privileges.
  * Privileged containers are used to enable podman-in-podman. Rootful-in-rootless does not seem possible without privileged. The container is still rootless, but will have your user capabilities.
  * Discussion: https://github.com/containers/podman/discussions/28307
* Fedora-based image with nix for userspace packages. `brew` was originally used but `nix` enables some useful capabilities for the Agent in troubleshooting, investigation, etc. without being beholden to NixOS.
* Opens tmux with OpenCode, caches OpenCode auth.json for non-API key logins.
* Developed in Rust.

## Quickstart

```text
cd rm4dev
cargo install --path .
```

## Commands

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

## Operational Behavior
- Names are normalized to `rm4dev-agent-<word>`. Passing `alpha` becomes `rm4dev-agent-alpha`.
- `new` always creates a new container. If no name is provided, it generates one from the current Unix timestamp.
- `start` resumes an existing container when it can resolve one unambiguously; otherwise it creates a new container.
- `start` treats `--no-shared-auth` and mount arguments as create-only signals when no matching container already exists.
- `list` shows all discovered `rm4dev-agent-*` containers, including stopped containers, with image and status columns.
- `enter` opens `/bin/bash -l` inside a running container by default. Override the shell path with `RM4DEV_ENTER_SHELL`.

## Host Effects
- New containers run `podman run --privileged` and mount tmpfs at `/tmp` and `/run`.
- Shared OpenCode auth is enabled by default. `rm4dev` creates `~/.cache/rm4dev/opencode-auth.json` on demand and bind-mounts it into `/root/.local/share/opencode/auth.json`.
- Additional mounts are bind mounts. Host paths are canonicalized and must already exist.
- Embedded image builds unpack into `XDG_CACHE_HOME/rm4dev/images/nix-fedora` or `~/.cache/rm4dev/images/nix-fedora`.

See `docs/security.md` for the security and compliance implications of these defaults.

## Image Behavior
By default, `rm4dev` uses `localhost/rm4dev-agent:nix-fedora`.

```bash
RM4DEV_IMAGE=my-image ./target/release/rm4dev agent new
```

- `rm4dev image build` builds the default image.
- `rm4dev image build localhost/custom-agent:dev` builds the same embedded context under a custom tag.
- `rm4dev image ensure` only builds when the target image is missing locally.
- `rm4dev agent new` and `rm4dev agent start` automatically ensure the default image exists before creating a new container.
- Setting `RM4DEV_IMAGE` disables implicit image management; the referenced image becomes user-managed.

The embedded image build still relies on network access for upstream container layers and package downloads.

## Repository Layout
- `src/` - CLI parsing, container lifecycle, image management, and process helpers
- `rm4dev-agent/nix-fedora/` - default image context embedded into the Rust binary
- `rm4dev-agent/brew-fedora/` - legacy or experimental image variant for local-only workflows
- `build.sh` - convenience build for the bundled image definitions and release binary

## License
`rm4dev` is licensed under the GNU General Public License, version 3 or any later version. See `COPYING`.

Unless a file says otherwise, the project-authored materials in this repository are copyright RM4 LLC. Third-party dependencies and packages installed into generated images remain under their own licenses.
