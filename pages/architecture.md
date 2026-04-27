# Architecture

Trapperkeeper is a Rust CLI (`trk`) that maintains a wiki for LLM
consumption: compiled knowledge that saves tokens and eliminates
redundant codebase exploration. Two storage modes are chosen at init
time: orphan branch (default) or in-tree directory.

## Module Structure

- **src/main.rs** — CLI entry point. Clap-based subcommand dispatch.
  Commands: `init [--in-tree PATH] [--adopt]`, `setup claude`,
  `setup codex`, `prime`.
- **src/config.rs** — Reads/writes `.trapperkeeper.json` at repo root.
  Holds mode selection and (for in-tree mode) wiki path. Falls back to
  orphan-branch detection for repos predating the config file.
- **src/git.rs** — Git plumbing layer: `git()`, `branch_exists()`,
  `hash_blob()`, `mktree()`, `commit_tree()`, `update_ref()`.
- **src/cmd/** — One file per command:
  - **init.rs** — Two paths: orphan (creates branch + worktree +
    .gitignore entry) or in-tree (creates on-disk directory, gitignores
    sources/, leaves files uncommitted for the user to include in their
    next commit). Both write `.trapperkeeper.json` and a `merge=union`
    gitattributes entry for `log.md`. `--adopt` wires up an existing
    wiki directory without clobbering content.
  - **setup_claude.rs** — Wires SessionStart/PreCompact hooks and
    Bash(trk:*) permission into .claude/settings.local.json. Idempotent.
    Mode-agnostic.
  - **setup_codex.rs** — Enables Codex hooks, wires SessionStart to
    `trk prime`, and allows `trk` via Codex rules. Touches local
    .codex/config.toml, .codex/hooks.json, and
    .codex/rules/default.rules. Idempotent and mode-agnostic.
  - **prime.rs** — Loads config, outputs mode-aware instructions for
    coding-agent hook injection. Silent exit if not initialized.

## Data Storage

### Orphan Mode

Default. Wiki lives on the `trapperkeeper` orphan branch, checked out as
a gitignored worktree at `.trapper_keeper/`. See pages/git-plumbing.md
for how the initial tree is built via plumbing commands.

### In-Tree Mode

Wiki lives as a normal directory in the repo at a user-chosen path
(for example `docs/wiki/`). The user commits wiki changes as part of the
normal workflow, ideally separately from code changes. `<path>/sources/`
is gitignored. See pages/in-tree-mode.md.

## Integration

Hook-based. `trk setup claude` adds hooks to .claude/settings.local.json:
- SessionStart -> `trk prime` (context on session start)
- PreCompact -> `trk prime` (context refresh before compaction)

`trk setup codex` configures Codex in .codex/:
- .codex/config.toml: `[features] codex_hooks = true`
- .codex/hooks.json: SessionStart matcher `startup|resume|clear` runs
  `trk prime` with status message "Loading Trapperkeeper wiki context"
- .codex/rules/default.rules: allows the `trk` prefix

Codex currently has no PreCompact hook equivalent, so only SessionStart
is configured there. `trk prime` is mode-aware, so the same hook works
for orphan and in-tree wiki storage.

## Dependencies

Runtime: clap (CLI parsing), serde_json (config + agent setup files).
Dev: assert_cmd, tempfile (integration tests in tests/merge.rs).
