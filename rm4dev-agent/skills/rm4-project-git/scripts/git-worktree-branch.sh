#!/usr/bin/env bash
# Usage: git-worktree-branch.sh <repository-name> <branch-name>
set -eux
cd ~/rm4/worktrees/$(basename $1)
git worktree add -b $2 $2
echo "Branch directory to use: $(realpath $2)"
