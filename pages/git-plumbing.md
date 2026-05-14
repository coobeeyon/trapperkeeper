# Git Plumbing

Retrieval hooks: concepts: orphan branch initialization, `git hash-object`,
`git mktree`, `git commit-tree`, `git update-ref`, `.trapper_keeper`
worktree. Key files: `src/git.rs`, `src/cmd/init.rs`, `tests/merge.rs`.
Useful when changing orphan-mode initialization or diagnosing why a fresh
wiki branch/worktree was not created.

`src/git.rs` is intentionally small. It wraps only the git commands needed
to create the initial orphan-branch wiki and update the branch ref. After
initialization, agents edit files in the checked-out `.trapper_keeper/`
worktree directly and commit with normal git commands inside that worktree.

## Current Functions

| Function | Purpose |
|----------|---------|
| `git(args)` | Run a git command, returning trimmed stdout or stderr as an error |
| `branch_exists()` | Check for `refs/heads/trapperkeeper` |
| `hash_blob(content)` | Write content to the object store with `git hash-object -w --stdin` |
| `mktree(entries)` | Build a tree from `(mode, type, hash, name)` entries using `git mktree` |
| `commit_tree(tree, message)` | Create an initial commit with no parent using `git commit-tree` |
| `update_ref(branch, commit)` | Point `refs/heads/<branch>` at a commit |

## Orphan Init Flow

`trk init` with no `--in-tree` follows this flow:

1. Verify the current directory is inside a git repo.
2. Refuse to continue if `.trapperkeeper.json` already exists.
3. Refuse to continue if the `trapperkeeper` branch already exists.
4. Write blobs for skeleton `toc.md`, `index.md`, `log.md`, and empty
   `.gitkeep` files.
5. Build `pages/` and `sources/` trees with `mktree()`.
6. Build the root wiki tree with `toc.md`, `index.md`, `log.md`, `pages/`,
   and `sources/`.
7. Create the first orphan commit with `commit_tree()`.
8. Advance `refs/heads/trapperkeeper` with `update_ref()`.
9. Run `git worktree add .trapper_keeper trapperkeeper`.
10. Append `/.trapper_keeper` to `.gitignore`.
11. Append `.trapper_keeper/log.md merge=union` to `.gitattributes`.
12. Write `.trapperkeeper.json` with `{"version": 1, "mode": "orphan"}`.

## Important Boundary

There is no generic tree-editing API in the current implementation. If a
future command needs to mutate orphan-branch files without using the
worktree, it will need new helpers for reading the existing tree, replacing
entries, committing with a parent, and handling nested paths.
