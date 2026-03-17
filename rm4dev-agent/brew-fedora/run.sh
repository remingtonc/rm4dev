#!/usr/bin/env bash
# user ensures running as intended user (not sure if necessary given USER is declared in Containerfile)
# uidmap and gidmap allow host user and container user to cooperate, otherwise host user maps to root.
# security-opt and device are from documentation: https://www.redhat.com/en/blog/podman-inside-container
# mount is just handy for letting container iterate on a host-shared area.
# .local is where opencode saves configs
podman run \
    --interactive \
    --tty \
    --name rm4dev-agent \
    --user podman \
    --uidmap "+1000:@$(id -u):1" \
    --gidmap "+1000:@$(id -g):1" \
    --security-opt label=disable \
    --device /dev/fuse \
    --mount type=bind,src=.,target=/home/podman/user_share \
    --volume rm4dev-agent-usr-config:/home/podman/.config:z \
    --volume rm4dev-agent-usr-local:/home/podman/.local:z \
    --cpus $((`nproc` - (`nproc` / 4))) \
    localhost/rm4dev-agent:brew-fedora
