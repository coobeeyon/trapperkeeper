# LLM Wiki Pattern — Research Notes

Research compiled on the Karpathy LLM Wiki pattern, its implementations,
and what works vs. what breaks in practice.

## The Core Pattern

Karpathy posted a GitHub gist on April 4, 2026 describing a pattern where instead of doing RAG at query time, the LLM incrementally builds and maintains a persistent wiki — a structured, interlinked collection of markdown files that sits between you and the raw sources. He framed it as an "idea file" — not code, not an app, just a description you paste into your agent and let it build out collaboratively.

The architecture has three layers: a `raw/` directory of immutable source material that neither you nor the LLM modifies, a `wiki/` directory of compiled markdown that the LLM owns entirely, and a schema/index file that keeps the LLM behaving like a disciplined maintainer. The practical setup is typically Claude Code (or Codex, or Cursor) on one side, Obsidian on the other as the viewer. The LLM makes edits based on conversation, and you browse the results in real time via Obsidian's graph view and wiki links.

The key insight versus RAG: stop re-deriving, start compiling. RAG retrieves and forgets. A wiki accumulates and compounds.

## How People Are Actually Implementing It

The gist went massively viral — over 17 million views, 5000+ stars, 2776 forks — and spawned a bunch of implementations within days:

**The simple version** that most people are running: drop markdown-converted sources into `raw/`, point Claude Code at the gist as a system prompt, and ask it to compile articles into `wiki/`. Karpathy uses the Obsidian Web Clipper to convert web content into markdown files, ensuring even images are stored locally so the LLM can reference them via vision capabilities. At query time, the LLM reads `index.md` first, identifies which articles are relevant, and loads only those — no embedding, no vector search.

**Tooling that's emerged:** Astro-Han's `karpathy-llm-wiki` packages it as an agent skill that works across Claude Code, Cursor, and Codex with ingest, query, and lint commands. SwarmVault (now at v0.6.1) has gone beyond code repos to support transcripts, Slack exports, email, calendar files, EPUBs, and spreadsheets as first-class sources. Ss1024sS's implementation generates 30 files including wiki structure, frontmatter templates, manifests, validation scripts, and CI workflows. The Peking University team built ΩmegaWiki extending it into a full research lifecycle platform.

**The maintenance loop** is where the pattern gets interesting. Karpathy describes running "health checks" or linting passes where the LLM scans the wiki for inconsistencies, missing data, or new connections. This includes auditing for contradictions between pages, identifying orphan pages with no inbound links, and flagging concepts that are referenced but don't yet have their own pages.

## What Actually Works

**The core thesis holds up.** The three-layer architecture (raw sources, wiki, schema) works. The operations (ingest, query, lint) cover the basics. Multiple independent teams have confirmed this.

**Scale is fine for personal use.** At roughly 100 articles and 400,000 words, the LLM's ability to navigate via summaries and index files is more than sufficient. For a research project or personal knowledge base, the overhead of a vector DB and embedding pipeline is genuinely unnecessary.

**The schema file is the secret weapon.** The schema document (CLAUDE.md, AGENTS.md) is the most important file in the system — it encodes what entity types exist, when to create vs. update pages, and what's private vs. shared. You and the LLM co-evolve it over time.

**Plain markdown is the right substrate.** By treating markdown files as the source of truth, you avoid the black box problem of vector embeddings — every claim can be traced back to a specific .md file a human can read, edit, or delete. This also makes it tool-agnostic. You're not locked into anything.

## What Breaks or Doesn't Work

**Knowledge rot is real.** The original treats all wiki content as equally valid forever. In practice, knowledge has a lifecycle — a bug you discovered last week matters more than one from six months ago, and a pattern you've seen twelve times is more reliable than one you've seen once. The v2 extension from the agentmemory team proposes confidence scoring, supersession tracking, and forgetting curves (Ebbinghaus-style retention decay) to address this. Without something like that, the wiki gradually fills with stale claims that sit alongside current ones with no distinction.

**The schema file is a wish, not a discipline.** There's no enforcement mechanism — it's a prompt, not a contract. The LLM can and does drift from the schema, especially over long sessions or when you switch agents. You have to actively lint and correct.

**It doesn't scale past personal use.** The 50,000-100,000 token threshold is where the wiki approach stops working reliably — beyond that, the index cannot fit in context, and you're forced into a retrieval layer regardless of storage format. Karpathy explicitly scoped this to individual researchers. Multiple simultaneous users create write conflicts without transactional database support, and there's no role-based access control.

**Compilation quality varies.** The LLM sometimes produces generic encyclopedia-style articles rather than genuinely synthesizing across sources. One commenter in the gist thread extended the pattern so that the wiki only stores knowledge that's passed through your own thinking — dialogue, challenge, practice — not auto-compiled summaries. This is a real tension: fully automated compilation is fast but shallow; human-in-the-loop compilation is better but defeats much of the convenience.

**No real security model.** The LLM has write access to the wiki directory. There's nothing preventing it from corrupting or overwriting good content during a bad session. Some implementations use the Hermes model as an independent supervisor to score and validate articles before promoting them to the live wiki, but that's not in the base pattern.

**The "idea file" concept itself is fragile.** Karpathy's framing is that you just paste the idea into your agent and it builds itself. In practice, a lot of people are finding the gap between "idea" and "working system" is nontrivial — people are fighting with file paths and shell scripts to get basic ingestion working.

## Where It's Heading

The v2 extension proposes consolidation tiers: working memory (recent observations), episodic memory (session summaries), semantic memory (cross-session facts), and procedural memory (workflows and patterns). This maps pretty cleanly to how actual knowledge management systems work.

Karpathy himself points toward eventually fine-tuning a smaller model on the wiki contents, so the knowledge lives in weights rather than context. That's the logical endpoint — the wiki as training data for a personalized model.

The honest assessment: the pattern is sound for personal/single-researcher use, the tooling is still early and rough, and the "RAG is dead" framing is overstated. It's solving a different problem at a different scale. If you're already doing the Claude Code orchestration work you're doing, this slots in pretty naturally as the persistent knowledge layer your agents read from and write to between sessions.

Source: Research compiled 2026-04-09
