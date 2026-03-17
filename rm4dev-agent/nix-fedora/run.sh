#!/usr/bin/env bash
# Copyright (C) 2026 RM4 LLC
# SPDX-License-Identifier: GPL-3.0-or-later

# user ensures running as intended user (not sure if necessary given USER is declared in Containerfile)
# uidmap and gidmap allow host user and container user to cooperate, otherwise host user maps to root.
# security-opt and device are from documentation: https://www.redhat.com/en/blog/podman-inside-container
# mount is just handy for letting container iterate on a host-shared area.
# .local is where opencode saves configs
podman run \
    --interactive \
    --tty \
    --name rm4dev-agent-nix \
    --security-opt label=disable \
    --device /dev/fuse \
    --mount type=bind,src=.,target=/home/podman/user_share \
    --cpus $((`nproc` - (`nproc` / 4))) \
    localhost/rm4dev-agent:nix-fedora
