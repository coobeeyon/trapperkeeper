# Decisions

Retrieval hooks: concepts: orphan branch default, in-tree mode,
agent-specific setup, hook-based integration, Rust CLI, thin CLI,
AI-maintained wiki. Key files: `SPEC.md`, `src/main.rs`, `src/cmd/init.rs`,
`src/cmd/setup_claude.rs`, `src/cmd/setup_codex.rs`, `src/cmd/prime.rs`.
Useful when evaluating whether a change matches the project's existing
design rationale.

Key decisions and the reasoning behind them.

## Orphan branch for storage (default)

Default mode. Wiki data lives on a git orphan branch (`trapperkeeper`),
checked out as a gitignored worktree at `.trapper_keeper/`. Avoids
polluting code commits with wiki updates. Same pattern litebrite uses
for its tracker.

## In-tree mode (alternative, opt-in)

`trk init --in-tree <path>` stores the wiki as a committed directory
instead. Trades PR-diff noise for human visibility — collaborators can
read the wiki without knowing about a gitignored worktree. Required a
merge-friendly flat-sorted format for `toc.md` / `index.md` and
`merge=union` for `log.md`. Added 2026-04-23 after collaborator feedback.
See pages/in-tree-mode.md.

## Hook-based integration

Context injection happens via coding-agent hooks running `trk prime`, not
via checked-in instruction files. This means the tool controls its own
context format and can evolve independently.

Claude Code uses SessionStart and PreCompact hooks plus the `Bash(trk:*)`
permission. Codex uses its hooks feature, a SessionStart hook for
`startup|resume|clear`, and a rules file that allows the `trk` command
prefix. Codex currently has no PreCompact hook equivalent.

Generated agent settings are local machine state. `.claude/` and `.codex/`
are gitignored; each clone should run the relevant setup commands instead
of committing generated hook/config files.

## Prime carries wiki sync discipline

`trk prime` now teaches agents to treat the repo's Trapper Keeper wiki as
shared project memory, not local scratch. The guidance is remote-first:
fetch before reading or editing, integrate remote wiki changes before new
edits, update toc/index/log/pages proactively during meaningful work, check
for conflict markers, commit wiki changes separately where practical, push
at the end, and never force-push remote wiki history unless explicitly
instructed. The orphan `trapperkeeper` branch remains the default layout;
in-tree mode is supported as a normal committed directory with analogous
normal-branch sync guidance.

## Agent-specific setup commands

Setup is split by target: `trk setup claude` and `trk setup codex`.
Claude and Codex use different config files and hook semantics, so keeping
target-specific modules avoids a lowest-common-denominator abstraction.
Both commands are idempotent and preserve unrelated existing settings.

## Rust

Consistent with litebrite. Both tools share the same git plumbing
patterns and agent-hook integration approach.

## Thin CLI, AI does the work

The CLI provides setup and context plumbing (`init`, `setup`, `prime`).
The AI is responsible for deciding what pages to create, maintaining the
toc and index, and keeping cross-references current. The tool doesn't
enforce structure; it provides the mechanism.
