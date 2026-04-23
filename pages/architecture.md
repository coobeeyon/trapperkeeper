# Architecture

Trapperkeeper is a Rust CLI (`trk`) that maintains a wiki for LLM
consumption — compiled knowledge that saves tokens and eliminates
redundant codebase exploration. Two storage modes, chosen at init time:
orphan branch (default) or in-tree directory.

## Module Structure

- **src/main.rs** — CLI entry point. Clap-based subcommand dispatch.
  Commands: `init [--in-tree PATH]`, `setup claude`, `prime`.
- **src/config.rs** — Reads/writes `.trapperkeeper.json` at repo root.
  Holds mode selection and (for in-tree) wiki path. Falls back to
  orphan-branch detection for repos predating the config file.
- **src/git.rs** — Git plumbing layer: `git()`, `branch_exists()`,
  `hash_blob()`, `mktree()`, `commit_tree()`, `update_ref()`.
- **src/cmd/** — One file per command:
  - **init.rs** — Two paths: orphan (creates branch + worktree + .gitignore
    entry) or in-tree (creates on-disk directory, gitignores sources/,
    leaves files uncommitted for the user to include in their next commit).
    Both write `.trapperkeeper.json` and a `merge=union` gitattributes
    entry for `log.md`.
  - **setup_claude.rs** — Wires SessionStart/PreCompact hooks and
    Bash(trk:*) permission into .claude/settings.local.json. Idempotent.
    Mode-agnostic.
  - **prime.rs** — Loads config, outputs mode-aware instructions for
    Claude Code injection. Silent exit if not initialized.

## Data Storage

### Orphan mode (default)

Wiki lives on the `trapperkeeper` orphan branch, checked out as a
gitignored worktree at `.trapper_keeper/`. See pages/git-plumbing.md for
how the initial tree is built via plumbing commands.

### In-tree mode

Wiki lives as a normal directory in the repo at a user-chosen path
(e.g. `docs/wiki/`). Committed by the user as part of normal workflow.
`<path>/sources/` is gitignored. See pages/in-tree-mode.md.

## Integration

Hook-based. `trk setup claude` adds hooks to .claude/settings.local.json:
- SessionStart → `trk prime` (context on session start)
- PreCompact → `trk prime` (context refresh before compaction)

## Dependencies

Runtime: clap (CLI parsing), serde_json (config + settings.local.json).
Dev: assert_cmd, tempfile (integration tests in tests/merge.rs).
