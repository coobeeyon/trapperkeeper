# Decisions

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

## Rust

Consistent with litebrite. Both tools share the same git plumbing
patterns and agent-hook integration approach.

## Thin CLI, AI does the work

The CLI provides setup and context plumbing (`init`, `setup`, `prime`).
The AI is responsible for deciding what pages to create, maintaining the
toc and index, and keeping cross-references current. The tool doesn't
enforce structure; it provides the mechanism.
