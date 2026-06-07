#!/usr/bin/env bash
# Usage: git-worktree-branch.sh <repository-name> <branch-name>
set -eux
BASE_DIR=${RM4_BASE_DIR:-~/rm4}
cd $BASE_DIR/worktrees/$(basename $1)
git worktree add -b $2 $2
echo "Branch directory to use: $(realpath $2)"
