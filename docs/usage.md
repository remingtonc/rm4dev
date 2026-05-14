<!-- Copyright (C) 2026 RM4 LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Usage

## Basic workflow

Create a new agent container:

```text
rm4dev agent new demo
```

Disable the web interface for a new container:

```text
rm4dev agent new --no-web demo
```

Resume or create depending on what already exists:

```text
rm4dev agent start demo
```

You can also skip web when creating a new container via `start`:

```text
rm4dev agent start --no-web demo
```

List known agent containers:

```text
rm4dev agent list
```

Open a shell in a running container:

```text
rm4dev agent enter demo
```

Open the web UI in your desktop browser for a running container:

```text
rm4dev agent attach demo
```

Force the terminal UI instead:

```text
rm4dev agent attach --no-web demo
```

## Naming rules

- `alpha` becomes `rm4dev-agent-alpha`
- already-prefixed names are preserved
- valid characters are letters, numbers, `.`, `_`, and `-`

## `new` versus `start`

- `agent new` always creates a fresh container
- `agent start` resumes an existing container when it can identify one unambiguously
- `agent start` creates a new container when:
  - the requested name does not exist
  - no agent containers exist yet
  - create-only options are supplied without naming an existing container
- `--no-web` disables OpenCode web for a newly created container

If multiple agent containers already exist and you do not specify a name, `agent start`, `agent stop`, `agent rm`, `agent attach`, and `agent enter` require a target name.

## Mounts

Mount syntax is:

```text
host_path:container_path
```

Example:

```text
rm4dev agent new demo "$PWD:/work"
```

Mount behavior:

- host paths must already exist
- host paths are canonicalized before use
- container paths must be absolute
- if the first positional argument contains `:`, it is treated as a mount, not a container name

## Shared auth

By default, newly created containers receive this bind mount:

```text
~/.cache/rm4dev/opencode-auth.json:/root/.local/share/opencode/auth.json
```

Disable it for a new container with:

```text
rm4dev agent new --no-shared-auth demo
```

## Web interface

New containers publish OpenCode web on a static host port starting at `35080`.

- the chosen port is stored in the container config
- the port is published on `127.0.0.1`
- the container starts `opencode web` in a `web` tmux window and `opencode attach` in a `tui` tmux window
- `rm4dev agent list` shows the web port and password in its output
- each web-enabled container uses the container name suffix as its OpenCode server password
- `agent attach` opens `http://127.0.0.1:<port>` in the host browser when a web port is available
- `agent attach --no-web` always uses the TUI attach

## Entering a running container

`agent enter` opens `/bin/bash -l` inside a running container by default.

Override the shell path if needed:

```text
RM4DEV_ENTER_SHELL=/usr/bin/zsh rm4dev agent enter demo
```

The override should point to a shell that accepts `-l`.

## Image commands

Build the default bundled image:

```text
rm4dev image build
```

Build the bundled context under a different tag:

```text
rm4dev image build localhost/custom-agent:dev
```

Only build when missing:

```text
rm4dev image ensure
```

## Cleanup

Stop a container:

```text
rm4dev agent stop demo
```

Remove a stopped container:

```text
rm4dev agent rm demo
```

If you need to recover disk space, remove unused Podman images and cached build contexts separately.
