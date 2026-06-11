# Architecture

Retrieval hooks: concepts: `trk`, `trk init`, `trk prime`, `trk setup
claude`, `trk setup codex`, installation instructions, README,
retrieval-oriented wiki prompt, remote-first wiki sync discipline,
storage mode, wiki update threshold, toc/index/page hooks. Key files:
`README.md`, `src/main.rs`, `src/cmd/init.rs`, `src/cmd/prime.rs`,
`src/cmd/setup_claude.rs`, `src/cmd/setup_codex.rs`, `src/config.rs`,
`SPEC.md`, `tests/merge.rs`. Useful when changing the command surface,
installation docs, session-start instructions, skeleton wiki files,
wiki-sync instructions, or the rules agents see before editing code.

Trapperkeeper is a small Rust CLI (`trk`) that installs and primes an
LLM-maintained wiki for a repository. The wiki is compiled knowledge for
future agents: read `toc.md` and `index.md` first, then open pages that
save source exploration. Storage is selected once at `trk init` time:
orphan-branch worktree by default, or an in-tree committed directory.

## Command Surface

`README.md` is the user-facing entry point. It documents Cargo
installation from GitHub or a local checkout, the initial `trk init`
flows, `trk setup claude`, `trk setup codex`, `trk prime`, and the
manual wiki commit workflow for orphan versus in-tree mode.

- `trk init` — creates the default orphan-branch wiki, checks it out at
  `.trapper_keeper/`, gitignores that worktree, writes
  `.trapperkeeper.json`, and adds `merge=union` for `.trapper_keeper/log.md`.
- `trk init --in-tree <path>` — creates a normal repo directory wiki at
  `<path>/`, gitignores `<path>/sources/`, writes `.trapperkeeper.json`,
  and adds `merge=union` for `<path>/log.md`.
- `trk init --in-tree <path> --adopt` — wires up an existing partial wiki
  without overwriting existing `toc.md`, `index.md`, `log.md`, or pages.
- `trk setup claude` — configures Claude Code hooks and permission for
  `trk prime`.
- `trk setup codex` — configures Codex hooks, hook feature flag, and rule
  permission for `trk prime`.
- `trk prime` — emits mode-aware wiki operating instructions; exits
  silently when the repo is not initialized.

There is no current `trk write` command. Agents edit wiki files directly
and commit the wiki using the storage mode's normal git workflow.

## Prime Prompt Discipline

`src/cmd/prime.rs` is the authoritative agent prompt. It frames the wiki
as a retrieval system: use `toc.md` to choose pages, write `index.md`
entries for phrases future agents will search, put retrieval hooks at
the top of pages, and update the wiki only for durable architecture,
invariants, ownership, decisions, traps, and file-location knowledge.

The prompt also includes remote-first wiki sync discipline: fetch before
wiki reads or edits, integrate remote wiki changes, update pages, toc,
index, and log proactively during meaningful work, commit wiki changes
separately where appropriate, push at the end, and avoid force-pushing
remote wiki history.

## Module Structure

- `src/main.rs` — Clap CLI definitions and subcommand dispatch.
- `src/config.rs` — reads/writes `.trapperkeeper.json`, maps config to
  the wiki path, and falls back to legacy orphan-branch detection if the
  config file is absent.
- `src/git.rs` — minimal git plumbing used by orphan initialization:
  `git()`, `branch_exists()`, `hash_blob()`, `mktree()`, `commit_tree()`,
  and `update_ref()`.
- `src/cmd/init.rs` — all initialization flows, skeleton file contents,
  `.gitignore` appends, and `.gitattributes` `merge=union` appends.
- `src/cmd/prime.rs` — mode-aware wiki operating instructions for
  coding-agent hook injection.
- `src/cmd/setup_claude.rs` — JSON merge/update logic for
  `.claude/settings.local.json`.
- `src/cmd/setup_codex.rs` — TOML-ish line editing for
  `.codex/config.toml`, JSON merge/update logic for `.codex/hooks.json`,
  and rule append logic for `.codex/rules/default.rules`.

## Data Storage

### Orphan mode

Default. Wiki state lives on the `trapperkeeper` orphan branch and is
checked out as `.trapper_keeper/`, which is added to `.gitignore`. The
init path builds the initial branch using git object/tree plumbing, then
adds a worktree for direct file edits. See `pages/git-plumbing.md`.

### In-tree mode

Opt-in. Wiki state lives as a normal committed directory in the repo at a
user-provided relative path such as `docs/wiki/`. The `sources/` subdir is
gitignored because it holds raw materials that should not create PR noise.
See `pages/in-tree-mode.md`.

## Integration

Hook-based. `trk setup claude` adds hooks to `.claude/settings.local.json`:

- SessionStart -> `trk prime`
- PreCompact -> `trk prime`
- Bash permission -> `Bash(trk:*)`

`trk setup codex` configures Codex in `.codex/`:

- `.codex/config.toml` enables hooks
- `.codex/hooks.json` installs a SessionStart hook for `trk prime`
- `.codex/rules/default.rules` allows the `trk` prefix

Codex currently has no PreCompact hook equivalent, so only SessionStart
is configured there. `trk prime` is mode-aware, so the same hook works
for orphan and in-tree wiki storage.

## Tests

`tests/merge.rs` is the integration test suite. It covers orphan and
in-tree initialization, `--adopt`, init guards, `prime` output,
merge-friendly wiki file behavior, and orphan/in-tree `log.md` union
merge setup. `src/cmd/setup_codex.rs` also has unit tests for Codex
config line editing, idempotent rule appends, and hook merge behavior.

## Dependencies

Runtime: `clap` for CLI parsing and `serde_json` for config/hook JSON.
Dev: `assert_cmd` and `tempfile` for integration tests.
