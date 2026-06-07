#!/usr/bin/env bash
# Usage: git-worktree-clone.sh <repository-url>
set -eux
BASE_DIR=${RM4_BASE_DIR:-~/rm4}
# Bare repositories
mkdir -p $BASE_DIR/bare-repos
pushd $BASE_DIR/bare-repos
git clone --bare $1
REPO_DIR=$(basename $1)
REPO_PATH=$(realpath $REPO_DIR)
REPO_NAME=$(basename $REPO_DIR .git)
popd
# Git worktrees
mkdir -p $BASE_DIR/worktrees/$REPO_NAME
WORKTREE_PATH=$(realpath $BASE_DIR/worktrees/$REPO_NAME)
cd $WORKTREE_PATH
echo "gitdir: $REPO_PATH" > .git
git config remote.origin.fetch "+refs/heads/*:refs/remotes/origin/*"
git fetch --all
# Assumes main...
git worktree add main main
echo "Bare git repository path: $REPO_PATH"
echo "Git worktree directory to use: $WORKTREE_PATH"
