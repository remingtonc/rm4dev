<!-- Copyright (C) 2026 RM4 LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Troubleshooting

## `rm4dev agent precheck` fails

- Confirm `podman` is installed and on `PATH`
- Run `podman info` directly to inspect daemon, storage, or permission problems
- Resolve Podman issues first; `rm4dev` depends on the local Podman CLI working normally

## Image build fails

- Check network access for base-image and package downloads
- Run `rm4dev image build` directly to reproduce the failure without container creation
- Remove the cached build context under `~/.cache/rm4dev/images/nix-fedora` if you suspect stale local materialization

## `multiple rm4dev agent containers exist; specify a name`

Pass an explicit container name:

```text
rm4dev agent start demo
rm4dev agent stop demo
rm4dev agent enter demo
```

Use `rm4dev agent list` to see the available names and statuses.

## Mount parsing errors

- verify the host path exists before calling `rm4dev`
- make sure the container path is absolute
- remember that the first positional argument containing `:` is parsed as a mount, not a name

## Auth cache problems

- if you do not want host auth persistence, recreate the container with `--no-shared-auth`
- if the auth file becomes corrupted, fix or remove `~/.cache/rm4dev/opencode-auth.json` and create a new container

## `agent enter` says the container is not running

`agent enter` only works for running containers. Start or resume the container first:

```text
rm4dev agent start demo
```

## Manual inspection commands

These Podman commands are often helpful during debugging:

```text
podman ps --all
podman inspect rm4dev-agent-demo
podman logs rm4dev-agent-demo
podman image exists localhost/rm4dev-agent:nix-fedora
```
