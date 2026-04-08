# Git Plumbing (src/git.rs)

All git operations are in src/git.rs. The key insight: we never check out
the orphan branch. All reads and writes happen through plumbing commands.

## Core Functions

| Function | Purpose |
|----------|---------|
| `git()` | Run any git command, return stdout or error with stderr |
| `branch_exists()` | Check if `trapperkeeper` branch exists |
| `hash_blob()` | Write content to object store, return SHA |
| `mktree()` | Build a tree from (mode, type, hash, name) entries |
| `ls_tree()` | List entries in a tree as tuples |
| `read_tree()` | Get tree hash for branch tip |
| `commit_tree()` | Create commit with no parent (used by init) |
| `commit_tree_with_parent()` | Create commit with parent (used by write) |
| `branch_tip()` | Get current tip commit SHA |
| `update_ref()` | Point branch at a new commit |
| `write_file_to_tree()` | High-level: write a file at a path into a tree |
| `insert_into_tree()` | Recursive: handles nested paths by rebuilding subtrees |

## Write Flow

To write `pages/foo.md`:

1. `branch_tip("trapperkeeper")` → get current commit
2. `read_tree("trapperkeeper")` → get root tree hash
3. `hash_blob(content)` → create blob
4. `ls_tree(root)` → get current entries
5. `ls_tree(pages_tree)` → get pages/ entries
6. Add/replace `foo.md` entry in pages subtree
7. `mktree()` → new pages subtree
8. Replace pages entry in root, `mktree()` → new root tree
9. `commit_tree_with_parent(new_root, old_tip)` → new commit
10. `update_ref("trapperkeeper", new_commit)` → advance branch
