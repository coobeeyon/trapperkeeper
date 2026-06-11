# Log

## [2026-05-14] groom | Comprehensive wiki refresh
Audited the wiki against the current source and SPEC. Removed stale
`trk write`/tree-mutation notes, added Codex setup coverage, added
retrieval hooks to concept pages, deleted the orphan `pages/test.md`, and
rebuilt toc/index entries around current commands, config files, and hook
targets.

## [2026-05-14] implement | Retrieval-oriented wiki prompt
Updated `trk prime` guidance so future agents treat the wiki as a retrieval
system. The prompt now calls for likely-search-phrase index entries,
page-top retrieval hooks, and a durable-knowledge threshold for wiki updates.
Updated SPEC, skeleton toc/index comments, tests, and architecture wiki notes.

## [2026-05-06] release | Install and publish prime sync update
Installed the current `trk` binary from commit `6a6e839` with
`cargo install --path . --force`, replacing `/home/mdaum/.cargo/bin/trk`.
Confirmed the feature branch
`lb-vm3n-add-trapper-keeper-wiki-sync-discipline-to-trk-pri` and wiki
branch `trapperkeeper` were pushed and up to date. Litebrite item
`lb-vm3n` is closed and synced.

## [2026-05-06] implement | Prime wiki sync discipline
Added remote-first Trapper Keeper wiki discipline to `trk prime` for both
orphan-branch and in-tree modes. The output now tells agents to fetch
before reading/editing, integrate remote wiki changes, update pages, toc,
index, and log proactively during meaningful work, check conflict markers,
commit wiki work separately where appropriate, push at the end, and avoid
force-pushing shared wiki history. Tests cover the new prime expectations.

## [2026-04-27] update | Record local agent config policy
After implementing `trk setup codex`, decided generated `.claude/` and
`.codex/` settings should remain local and gitignored. New clones are
expected to run `trk init` and the relevant setup commands.

## [2026-04-26] update | Document Codex setup command
Updated the wiki to reflect `trk setup codex`. The command enables Codex
hooks in `.codex/config.toml`, installs a SessionStart hook in
`.codex/hooks.json`, and adds a Codex rule allowing the `trk` prefix.

## [2026-04-24] fix | Restore hierarchical toc.md
Earlier flat-sorting of toc.md lost the navigational structure — a table
of contents without sections isn't really a toc. Reverted toc.md to
grouped-under-sections form; sections sorted alphabetically by heading,
entries sorted alphabetically within each section. index.md stays flat
(letter-headers added no semantic value there). Updated skeleton in
init.rs, invariants in prime.rs, assertion helper, and added
`toc_section_adds_in_different_sections_merge_cleanly` test proving
concurrent adds to different sections merge. 24 tests.

## [2026-04-23] implement | --adopt flag for existing wikis
Added `trk init --in-tree <path> --adopt` which wires up an existing
wiki directory without clobbering content. Requires at least one of
toc/index/log.md to exist. Fills in missing skeleton pieces. Tests for
adopt (preserve existing, create missing, refuse empty, refuse
unrelated content, refuse --adopt without --in-tree). 23 tests total.

## [2026-04-23] implement | In-tree mode
Added `trk init --in-tree <path>` alternative storage where the wiki
is a committed directory instead of the orphan branch. New src/config.rs
persists mode in .trapperkeeper.json at repo root. Prime templated to
be mode-aware. toc.md and index.md moved to flat-sorted merge-friendly
format; log.md gets merge=union via .gitattributes. Integration tests
in tests/merge.rs exercise the clean-merge, same-gap-conflict, and
log-union cases. Driven by collaborator feedback preferring a visible
docs dir over a gitignored worktree.

## [2026-04-23] plan | In-tree mode for trapperkeeper
Ratified plan at ~/.claude/plans/crystalline-wobbling-finch.md. Chose
`.trapperkeeper.json` for config (reuses serde_json), required --in-tree
path arg (no cross-ecosystem default), gitignored sources/ in in-tree
mode (preserves "raw materials not in repo" semantics), and flat-sorted
format for toc/index to minimize merge conflicts. Out of scope: mode
switching command, prime-from-subdirectory, lint command.

## [2026-04-09] ingest | Karpathy LLM Wiki pattern
Digested sources/karpathy-llm-wiki.md (original gist) and
sources/llm-wiki-research.md (implementations + failure modes).
Created pages/llm-wiki-pattern.md synthesizing the pattern, what works,
what breaks, and how trapperkeeper maps to it. Updated toc and index.

## [2026-04-08] ingest | Initial wiki population
First ingest of the trapperkeeper codebase. Created architecture overview,
git plumbing reference, and decisions page. Populated toc and index.

## [2026-06-11] implement | README installation instructions
Added `README.md` as the user-facing entry point for installing `trk`,
running initial `trk init` flows, wiring Claude Code or Codex hooks,
using `trk prime`, and committing wiki changes in orphan versus in-tree
mode. Updated architecture/index hooks so future agents can find the docs
contract quickly.
