#!/usr/bin/env bash
# Usage: git-worktree-clone.sh <repository-url>
set -eux
# Bare repositories
mkdir -p ~/rm4/bare-repos
pushd ~/rm4/bare-repos
git clone --bare $1
REPO_DIR=$(basename $1)
REPO_PATH=$(realpath $REPO_DIR)
REPO_NAME=$(basename $REPO_DIR .git)
popd
# Git worktrees
mkdir -p ~/rm4/worktrees/$REPO_NAME
WORKTREE_PATH=$(realpath ~/rm4/worktrees/$REPO_NAME)
cd $WORKTREE_PATH
echo "gitdir: $REPO_PATH" > .git
git config remote.origin.fetch "+refs/heads/*:refs/remotes/origin/*"
git fetch --all
# Assumes main...
git worktree add main main
echo "Bare git repository path: $REPO_PATH"
echo "Git worktree directory to use: $WORKTREE_PATH"
