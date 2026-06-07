name: rm4-project-git
description: git clone, git worktree, git status, git commit, branch, checkout, merge, rebase, push, pull: use when working in any git repository; prefer RM4 worktrees over ad hoc git commands.
---

# rm4-project-git

Default workflow for repository work in this workspace.

Use this skill before reaching for shell `git` commands. If the task touches repository state, history, branches, remotes, or commits, assume an RM4 worktree unless the user explicitly asks for a different layout.

## Rules

- Never use a bare repository as the editing location.
- Never invent a one-off repo layout.
- Never rely on ad hoc `cd <repo> && git ...` outside the RM4 worktree structure.
- Do all edits and git operations inside `$RM4_BASE_DIR/worktrees/<repository-name>/<branch-name>`.
- Use normal `git` commands only after you are inside the correct worktree.
- Prefer the smallest safe git operation that matches the task.

## Layout

- Bare repositories live in `$RM4_BASE_DIR/bare-repos/`.
- Worktrees live in `$RM4_BASE_DIR/worktrees/`.
- The repository root worktree is `$RM4_BASE_DIR/worktrees/<repository-name>`.
- Branch worktrees are nested under that root.

## Workflow

1. Determine the repository name and whether a worktree already exists.
2. If the repo is missing, clone it with `git-worktree-clone.sh <repository-url>`.
3. If you need a new branch, create it with `git-worktree-branch.sh <repository-name> <branch-name>`.
4. Work only from the worktree path printed by the script.
5. Use regular `git` commands inside the worktree for `status`, `diff`, `log`, `add`, `commit`, `push`, `pull`, `merge`, and `rebase`.
6. For cleanup, remove branch worktrees from the repository root with `git worktree remove <branch-name>`.

## Checks

- Before changing anything, inspect `git status`.
- Before committing, inspect `git diff` and `git log --oneline -10` when history context matters.
- After major operations, re-run `git status` to confirm the result.
- If the task is only informational, do not create or switch worktrees.

## Scripts

- `git-worktree-clone.sh <repository-url>`: clone into the bare-repo plus worktree layout.
- `git-worktree-branch.sh <repository-name> <branch-name>`: create a new branch worktree.

## Examples

- Clone for work: `git-worktree-clone.sh https://github.com/org/repo.git`
- Start a feature branch: `git-worktree-branch.sh repo feature-name`
- Inspect current state: `git status && git diff`
