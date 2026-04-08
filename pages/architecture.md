# Architecture

Trapperkeeper is a Rust CLI (`trk`) that maintains a wiki on a git orphan
branch. The wiki is for LLM consumption — compiled knowledge that saves
tokens and eliminates redundant codebase exploration.

## Module Structure

- **src/main.rs** — CLI entry point. Clap-based subcommand dispatch.
  Commands: `init`, `setup claude`, `prime`, `write`.
- **src/git.rs** — Git plumbing layer. All git operations go through here.
  Wraps raw git commands (hash-object, mktree, commit-tree, ls-tree,
  update-ref). Handles blob creation, tree manipulation, and commits on
  the orphan branch without touching the working tree.
- **src/cmd/** — One file per command:
  - **init.rs** — Creates the orphan branch with skeleton files (toc.md,
    index.md, log.md, pages/, sources/).
  - **setup_claude.rs** — Wires SessionStart/PreCompact hooks and
    Bash(trk:*) permission into .claude/settings.local.json. Idempotent.
  - **prime.rs** — Reads toc.md and recent log.md from the orphan branch,
    outputs formatted context for Claude Code injection. Silent exit if
    not initialized.
  - **write.rs** — Reads content from stdin, writes it to a path on the
    orphan branch. Handles nested paths (e.g. pages/foo.md) by rebuilding
    subtrees. Each write is an atomic commit.

## Data Storage

All wiki data lives on the `trapperkeeper` orphan branch. No files appear
in the working tree. Git plumbing commands manipulate the branch directly:

1. `hash-object -w` to create blobs
2. `ls-tree` to read existing tree entries
3. `mktree` to build new trees
4. `commit-tree -p` to create commits with parent
5. `update-ref` to advance the branch

This mirrors litebrite's approach (see ../litebrite).

## Integration

Hook-based. `trk setup claude` adds hooks to .claude/settings.local.json:
- SessionStart → `trk prime` (context on session start)
- PreCompact → `trk prime` (context refresh before compaction)

## Dependencies

Minimal: clap (CLI parsing), serde_json (settings.local.json manipulation).
