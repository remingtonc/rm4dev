#!/usr/bin/env bash
pushd rm4dev-agent && ./build.sh && popd
cargo build --release --bin rm4dev
