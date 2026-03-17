#!/usr/bin/env bash
set -e

SESSION=rm4dev

# Create session if it doesn't exist
if ! tmux has-session -t "$SESSION" 2>/dev/null; then
    tmux new-session -d -s "$SESSION" "opencode"
fi

# Attach to the session
exec tmux attach-session -t "$SESSION"
