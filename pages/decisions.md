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

## Hook-based integration (not CLAUDE.md)

Context injection happens via SessionStart/PreCompact hooks running
`trk prime`, not via CLAUDE.md instructions. This means the tool
controls its own context format and can evolve independently.

## Rust

Consistent with litebrite. Both tools share the same git plumbing
patterns and Claude Code integration approach.

## stdin for write

`trk write` reads from stdin rather than taking a file argument or
inline content. This makes it composable with pipes and heredocs,
and avoids shell escaping issues with markdown content.

## Thin CLI, AI does the work

The CLI provides plumbing (init, write, prime). The AI is responsible
for deciding what pages to create, maintaining the toc and index, and
keeping cross-references current. The tool doesn't enforce structure —
it provides the mechanism.
