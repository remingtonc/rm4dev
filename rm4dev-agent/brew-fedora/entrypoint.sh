#!/usr/bin/env bash
# Copyright (C) 2026 RM4 LLC
# SPDX-License-Identifier: GPL-3.0-or-later

set -e

SESSION=rm4dev-agent
WEB_PORT=${RM4DEV_OPENCODE_WEB_PORT:-}
CONFIG_DIR=/home/podman/.config
CONFIG_BACKUP_DIR=/home/podman/.config.bak
CONFIG_SENTINEL_FILE="$CONFIG_DIR/.rm4dev-initialized"
LOCAL_DIR=/home/podman/.local
LOCAL_BACKUP_DIR=/home/podman/.local.bak
LOCAL_SENTINEL_FILE="$LOCAL_DIR/.rm4dev-initialized"

mkdir -p "$CONFIG_DIR"

if [[ ! -f "$CONFIG_SENTINEL_FILE" ]]; then
    cp -a "$CONFIG_BACKUP_DIR"/. "$CONFIG_DIR"/
    touch "$CONFIG_SENTINEL_FILE"
fi

mkdir -p "$LOCAL_BACKUP_DIR"
if [[ ! -f "$LOCAL_SENTINEL_FILE" ]]; then
    cp -a "$LOCAL_BACKUP_DIR"/. "$LOCAL_DIR"/
    touch "$LOCAL_SENTINEL_FILE"
fi

# Create session if it doesn't exist
if ! tmux has-session -t "$SESSION" 2>/dev/null; then
    if [[ -n "$WEB_PORT" ]]; then
        tmux new-session -d -s "$SESSION" -n web "opencode web --hostname 0.0.0.0 --port $WEB_PORT"
        tmux split-window -t "$SESSION":0 -h bash -lc "until : >/dev/tcp/127.0.0.1/$WEB_PORT 2>/dev/null; do sleep 0.2; done; exec opencode attach http://127.0.0.1:$WEB_PORT"
        tmux select-layout -t "$SESSION":0 even-horizontal
        tmux select-pane -t "$SESSION":0.1
    else
        tmux new-session -d -s "$SESSION" "opencode"
    fi
fi

# Attach to the session
exec tmux attach-session -t "$SESSION"
