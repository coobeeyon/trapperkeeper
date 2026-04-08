# Trapperkeeper Specification

An LLM-maintained wiki system for codebases. The wiki is primarily for
the AI's own use — compiled knowledge that saves tokens, broadens
context, and eliminates redundant exploration across sessions.

Inspired by [Karpathy's LLM Wiki pattern](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f).

## Core Concept

Instead of re-deriving understanding of a codebase every conversation,
the LLM incrementally builds and maintains a persistent wiki. Knowledge
is compiled once and kept current, not re-derived on every query.

The tedious part of maintaining a knowledge base is not the reading or
the thinking — it's the bookkeeping. LLMs handle cross-reference updates
and consistency maintenance at near-zero cost.

## Architecture

### Raw Layers (Virtual)

Raw sources are not copied or managed by trapperkeeper. They already
exist and trapperkeeper knows how to reach them.

- **Code** — always present in the repo
- **Claude logs** — conversation transcripts persisted at `~/claude`
- **Assembled materials** — design docs, notes, etc. stored in `sources/`
  on the orphan branch when they have no other home

### Wiki Layer (Orphan Branch)

All wiki state lives on a git orphan branch (like litebrite's tracker).
Clean separation from code commits, git-native persistence, travels with
the repo.

```
toc.md            # hierarchical, top-down navigation with summaries
index.md          # cross-reference lookup, bottom-up (topic -> locations)
log.md            # append-only chronological record of wiki activity
pages/            # synthesized wiki pages
sources/          # raw materials that need a home
```

#### toc.md

Top-down navigation. Describes the semantic hierarchy of the codebase —
not the filesystem layout, but how things relate and what they do.
Includes summaries. Reading the toc should orient the AI to the entire
project without touching a single source file.

#### index.md

Bottom-up cross-reference. Look up a concept and find everywhere it
appears — wiki pages, source files, log entries. This is the AI's
primary lookup tool. A good index eliminates exploration.

#### log.md

Append-only chronological record with parseable prefixes:

```
## [2026-04-08] plan | Redesign caching layer
## [2026-04-08] implement | Auth module
## [2026-04-09] review | PR #42 security audit
```

#### pages/

Synthesized wiki pages. Architecture overviews, module summaries,
decision records (the *why* behind choices), concept pages (cross-cutting
concerns spanning multiple files).

These pages exist to replace expensive exploration. A good page saves
dozens of tool calls.

#### sources/

Raw materials that are generated or assembled and have no other home.
Design docs, meeting notes, diagrams, research — things that inform the
wiki but aren't code and aren't in the repo.

## Reference Implementation

Litebrite is a git-native issue tracker that uses the same orphan branch
pattern and Claude Code hook integration. Use it as the primary reference
for git plumbing, CLI structure, and `setup claude` / `prime` commands.

- **Source**: `../litebrite` (relative to this repo)
- **Repo**: https://github.com/coobeeyon/litebrite

## CLI: `trk`

Language: Rust. Binary: `trk`.

### Initial Commands

- `trk init` — create the orphan branch with skeleton files
- `trk setup claude` — wire hooks into `.claude/settings.local.json`
- `trk prime` — output wiki context for Claude Code injection

### Integration

Hook-based, following litebrite's pattern:

- **SessionStart** hook runs `trk prime` — AI sees wiki state at
  conversation start
- **PreCompact** hook runs `trk prime` — wiki state refreshed before
  context compaction
- **Bash permission** `Bash(trk:*)` — AI can run trk commands

`trk setup claude` is idempotent and merges with existing configuration.
`trk prime` exits silently in repos without trapperkeeper initialized.

## Wiki Update Triggers

The wiki should be updated at key moments when understanding
crystallizes:

- **Plan ratified** — captures the *why* and the intended approach
- **Implementation finished** — captures the *what* — what was built
  and how
- **Review finished** — captures *what we learned* — corrections,
  insights, edge cases discovered

On-demand updates are always available. Automatic triggers may evolve
as we learn what's useful.

## Design Principles

1. **For the AI, by the AI** — the wiki's primary consumer is the LLM.
   The key metric: does reading a wiki page save expensive exploration?
2. **Bookkeeping is the job** — every update maintains toc, index, and
   cross-references. This is where the biggest token savings come from.
3. **Start simple** — orphan branch, markdown files, thin CLI. Grow
   only when needs demand it.
4. **Raw layers are virtual** — don't copy what already exists. Know
   how to reach it.
5. **Git-native** — all state in git. Any machine that clones the repo
   gets the wiki.

