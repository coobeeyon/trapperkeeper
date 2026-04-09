# LLM Wiki Pattern

Andrej Karpathy published a [gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
on 2026-04-04 describing a pattern where an LLM incrementally builds and
maintains a persistent wiki from raw sources, rather than doing RAG at
query time. The gist went viral (17M+ views, 5000+ stars) and spawned
dozens of implementations within days.

Trapperkeeper is a direct implementation of this pattern.

## Core Thesis

**Stop re-deriving, start compiling.** RAG retrieves source fragments and
forgets between queries. A wiki accumulates and compounds — cross-references
are pre-built, contradictions are pre-flagged, synthesis reflects everything
ingested so far.

The human curates sources, directs analysis, asks questions. The LLM does
the bookkeeping — summarizing, cross-referencing, filing, maintaining
consistency across pages. Wikis die because humans won't do the maintenance.
LLMs don't get bored.

## Three-Layer Architecture

| Layer | Contents | Who owns it |
|-------|----------|-------------|
| **Raw sources** | Immutable documents — articles, papers, transcripts | Human curates, LLM reads |
| **Wiki** | Compiled markdown — summaries, entity pages, concept pages | LLM writes, human reads |
| **Schema** | Configuration telling the LLM how the wiki is structured | Co-evolved by both |

In trapperkeeper terms: `sources/` = raw, `pages/` = wiki, the
SessionStart hook prompt = schema.

## Three Operations

**Ingest** — Add a source, LLM reads it, writes/updates wiki pages,
updates index and log. A single source can touch 10-15 pages. Can be
interactive (source-by-source with human review) or batched.

**Query** — Ask questions against the wiki. LLM reads index first to find
relevant pages, drills into them, synthesizes answer. Good answers get
filed back as new pages — explorations compound too.

**Lint** — Periodic health check. Find contradictions between pages, stale
claims superseded by newer sources, orphan pages with no inbound links,
concepts referenced but lacking their own page, missing cross-references.

## What Works

- The three-layer architecture is sound. Multiple independent teams confirm.
- Scale is fine for personal use (~100 sources, ~400k words). Index-file
  navigation suffices — no vector DB needed.
- The schema file is the secret weapon. It encodes entity types, page
  creation rules, and conventions. It's what makes the LLM a disciplined
  maintainer rather than a generic chatbot.
- Plain markdown is the right substrate. Human-readable, tool-agnostic,
  version-controllable, no black-box embeddings.

## Known Failure Modes

**Knowledge rot.** All wiki content is treated as equally valid forever.
In practice, recent observations matter more than old ones, and frequently
confirmed patterns are more reliable than one-offs. Without confidence
scoring or supersession tracking, the wiki fills with stale claims
indistinguishable from current ones.

**Schema drift.** The schema is a prompt, not a contract. The LLM drifts
from it over long sessions or agent switches. Active linting is required.

**Shallow compilation.** The LLM sometimes produces generic encyclopedia
summaries rather than genuine cross-source synthesis. Fully automated
compilation is fast but shallow; human-in-the-loop is better but slower.
The tension is real — the solution is interactive ingestion with the human
steering emphasis.

**No safety net.** The LLM has write access to the wiki. A bad session
can corrupt good content. Git history is the only protection (which is
why trapperkeeper puts the wiki on a git branch).

**Scaling ceiling.** The index-based approach stops working reliably
around 50k-100k tokens of index. Beyond that you need a retrieval layer
(search, embeddings). For personal/single-researcher use this is rarely
a problem.

## Relation to Trapperkeeper

Trapperkeeper implements the pattern with these specific choices:

| Karpathy concept | Trapperkeeper implementation |
|------------------|------------------------------|
| Raw sources | `sources/` directory on orphan branch |
| Wiki pages | `pages/` directory on orphan branch |
| Schema | SessionStart hook prompt (injected by `trk prime`) |
| Index | `index.md` — alphabetical concept-to-location map |
| Log | `log.md` — chronological operation record |
| Persistence | Git orphan branch `trapperkeeper` |
| Viewer | Direct file reads (no Obsidian dependency) |

Key divergence: Karpathy's gist assumes a human-facing wiki viewed in
Obsidian. Trapperkeeper's wiki is primarily LLM-facing — it exists so
future sessions don't re-derive understanding that a previous session
already worked out. The human *can* read it, but the LLM is the primary
consumer.

## Sources

- [Karpathy gist](sources/karpathy-llm-wiki.md) — original idea file
- [Research notes](sources/llm-wiki-research.md) — implementations, what works, what breaks
