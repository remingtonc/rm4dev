#!/usr/bin/env bash
# Copyright (C) 2026 RM4 LLC
# SPDX-License-Identifier: GPL-3.0-or-later

pushd brew-fedora && ./build.sh && popd
pushd nix-fedora && ./build.sh && popd