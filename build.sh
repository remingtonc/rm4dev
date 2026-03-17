#!/usr/bin/env bash
# Copyright (C) 2026 RM4 LLC
# SPDX-License-Identifier: GPL-3.0-or-later

pushd rm4dev-agent && ./build.sh && popd
cargo build --release --bin rm4dev
