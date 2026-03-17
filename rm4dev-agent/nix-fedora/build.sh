#!/bin/bash
set -euo pipefail

podman build -t localhost/rm4dev-agent:nix-fedora -t localhost/rm4dev-agent:latest .
