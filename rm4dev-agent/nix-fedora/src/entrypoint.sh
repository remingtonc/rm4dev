#!/usr/bin/env bash
# Copyright (C) 2026 RM4 LLC
# SPDX-License-Identifier: GPL-3.0-or-later

set -e

SESSION=rm4dev
WEB_PORT=${RM4DEV_OPENCODE_WEB_PORT:-}

# Create session if it doesn't exist
if ! tmux has-session -t "$SESSION" 2>/dev/null; then
    if [[ -n "$WEB_PORT" ]]; then
        tmux new-session -d -s "$SESSION" -n web "opencode web --hostname 0.0.0.0 --port $WEB_PORT"
        tmux split-window -t "$SESSION":0 -h bash -lc "until : >/dev/tcp/127.0.0.1/$WEB_PORT 2>/dev/null; do sleep 2; done; exec opencode attach http://127.0.0.1:$WEB_PORT"
        tmux select-layout -t "$SESSION":0 even-horizontal
        tmux select-pane -t "$SESSION":0.1
    else
        tmux new-session -d -s "$SESSION" "opencode"
    fi
fi

# Attach to the session
exec tmux attach-session -t "$SESSION"
