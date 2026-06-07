---
name: rm4-project-git
description: git project operations for worktree feature development. Default development methodology!
---

Opinionated `git` project operations for worktree feature development, contained within `~/rm4/`.
Decouples the git repository from working directories, enabling worktree-based development.
Bare git repositories are stored in `~/bare-repos/` and worktrees are stored in `~/rm4/worktrees/`.
The repositories in `~/rm4/worktrees/`, i.e. `~/rm4/worktrees/<repository-name>`, are the actual working directories where development happens.

Scripts:
- `git-worktree-clone.sh <repository-url>`: Clones a repository to the expected bare repository and worktree structure.
- `git-worktree-branch.sh <repository-name> <branch-name>`: Creates a new worktree for the specified branch.

All work should happen out of worktree directories, i.e. `~/rm4/worktrees/<repository-name>/<branch-name>`, to ensure the benefits of worktree-based development.

Regular git commands are to be used within the worktree directories, e.g. `cd ~/rm4/worktrees/<repository-name>/<branch-name> && git status`, and for worktree cleanup, etc. i.e. `cd ~/rm4/worktrees/<repository-name> && git worktree remove <branch-name>`.
