# Log

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
