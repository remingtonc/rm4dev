<!-- Copyright (C) 2026 RM4 LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Usage

## Basic workflow

Create a new agent container:

```text
rm4dev agent new demo
```

Resume or create depending on what already exists:

```text
rm4dev agent start demo
```

List known agent containers:

```text
rm4dev agent list
```

Open a shell in a running container:

```text
rm4dev agent enter demo
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
