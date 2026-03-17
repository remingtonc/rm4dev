#!/bin/bash
# Copyright (C) 2026 RM4 LLC
# SPDX-License-Identifier: GPL-3.0-or-later

set -euo pipefail

podman build -t localhost/rm4dev-agent:nix-fedora -t localhost/rm4dev-agent:latest .
