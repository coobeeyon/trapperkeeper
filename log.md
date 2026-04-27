# Log

## [2026-04-27] update | Record local agent config policy
After implementing `trk setup codex`, decided generated `.claude/` and
`.codex/` settings should remain local and gitignored. New clones are
expected to run `trk init` and the relevant setup commands.

## [2026-04-26] update | Document Codex setup command
Updated the wiki to reflect `trk setup codex`. The command enables Codex
hooks in `.codex/config.toml`, installs a SessionStart hook in
`.codex/hooks.json`, and adds a Codex rule allowing the `trk` prefix.

## [2026-04-08] ingest | Initial wiki population
First ingest of the trapperkeeper codebase. Created architecture overview,
git plumbing reference, and decisions page. Populated toc and index.
