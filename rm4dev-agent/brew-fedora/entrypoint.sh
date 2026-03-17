#!/usr/bin/env bash
set -e

SESSION=rm4dev-agent
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
    tmux new-session -d -s "$SESSION" "opencode"
fi

# Attach to the session
exec tmux attach-session -t "$SESSION"
