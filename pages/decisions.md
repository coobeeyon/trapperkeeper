# Decisions

Key decisions and the reasoning behind them.

## Orphan branch for storage

Wiki data lives on a git orphan branch, not in the working tree. This
avoids polluting code commits with wiki updates while keeping everything
in git. The same pattern litebrite uses for its tracker. Future direction:
an "admin repo" could replace this for multi-repo wikis.

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

## stdin for write

`trk write` reads from stdin rather than taking a file argument or
inline content. This makes it composable with pipes and heredocs,
and avoids shell escaping issues with markdown content.

## Thin CLI, AI does the work

The CLI provides plumbing (init, write, prime). The AI is responsible
for deciding what pages to create, maintaining the toc and index, and
keeping cross-references current. The tool doesn't enforce structure —
it provides the mechanism.
