---
name: rm4-project-git
description: git project operations for worktree feature development. Default development methodology!
---

Opinionated `git` project operations for worktree feature development, contained within `$RM4_BASE_DIR` which is often either `/work/rm4/` or `~/rm4/`.
Decouples the git repository from working directories, enabling worktree-based development.
Bare git repositories are stored in `$RM4_BASE_DIR/bare-repos/` and worktrees are stored in `$RM4_BASE_DIR/worktrees/`.
The repositories in `$RM4_BASE_DIR/worktrees/`, i.e. `$RM4_BASE_DIR/worktrees/<repository-name>`, are the actual working directories where development happens.

Scripts:
- `git-worktree-clone.sh <repository-url>`: Clones a repository to the expected bare repository and worktree structure.
- `git-worktree-branch.sh <repository-name> <branch-name>`: Creates a new worktree for the specified branch.

All work should happen out of worktree directories, i.e. `$RM4_BASE_DIR/worktrees/<repository-name>/<branch-name>`, to ensure the benefits of worktree-based development.

Regular git commands are to be used within the worktree directories, e.g. `cd $RM4_BASE_DIR/worktrees/<repository-name>/<branch-name> && git status`, and for worktree cleanup, etc. i.e. `cd $RM4_BASE_DIR/worktrees/<repository-name> && git worktree remove <branch-name>`.
