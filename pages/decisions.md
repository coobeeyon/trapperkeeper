# Decisions

Key decisions and the reasoning behind them.

## Orphan branch for storage

Wiki data lives on a git orphan branch, not in the working tree. This
avoids polluting code commits with wiki updates while keeping everything
in git. The same pattern litebrite uses for its tracker. Future direction:
an "admin repo" could replace this for multi-repo wikis.

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
