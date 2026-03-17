<!-- Copyright (C) 2026 RM4 LLC -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Security

## Container privilege model

New agent containers are launched with `--privileged`. That is a deliberate tradeoff to support the nested Podman workflow inside the development container.

Treat these containers as highly trusted local development environments, not as sandbox boundaries.

## Host mounts

`rm4dev` may mount several categories of host data into a container:

- the shared OpenCode auth file by default
- any additional bind mounts you pass on the command line
- tmpfs mounts for `/tmp` and `/run`

Because bind mounts expose host paths directly, only mount directories and files you are comfortable making available to privileged container processes.

## Auth persistence

Unless `--no-shared-auth` is used, `rm4dev` creates and mounts:

```text
~/.cache/rm4dev/opencode-auth.json
```

That makes OpenCode authentication survive container recreation, but it also means the container can read the mounted credentials file.

## Network and supply chain considerations

The default image build downloads upstream container layers and packages at build time. Review and pin external dependencies appropriately if you are using `rm4dev` in a controlled environment or redistributing derived artifacts.

## Resource behavior

`rm4dev` reserves roughly one quarter of the host's available CPU threads and assigns the remainder to the created container. On smaller systems or shared workstations, that may still be aggressive.

## Recommended safeguards

- use dedicated local development machines or trusted user accounts
- keep mounted host paths narrow and explicit
- use `--no-shared-auth` when persistent credentials are unnecessary
- inspect `rm4dev agent list` output regularly and remove containers you no longer need
- review third-party package license and security obligations before redistributing images or binaries
