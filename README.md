# Trapperkeeper

Trapperkeeper is a small Rust CLI for maintaining an LLM-oriented wiki inside a
codebase. The wiki stores compiled project knowledge so future coding-agent
sessions can read the table of contents, index, and synthesized pages before
re-deriving the same context from source.

The installed binary is `trk`.

## Installation

### Prerequisites

- Git
- Rust and Cargo, installed with [rustup](https://rustup.rs/) or your package
  manager

### Install from GitHub

```sh
cargo install --git https://github.com/coobeeyon/trapperkeeper.git --bin trk
```

Confirm the binary is available:

```sh
trk --help
```

### Install from a local checkout

```sh
git clone https://github.com/coobeeyon/trapperkeeper.git
cd trapperkeeper
cargo install --path . --bin trk
```

For development, build and test with:

```sh
cargo build
cargo test
```

## Quick Start

Run these commands from the root of the repository that should get a wiki.

Initialize the default orphan-branch wiki:

```sh
trk init
```

This creates the `trapperkeeper` branch, checks it out as the gitignored
`.trapper_keeper/` worktree, writes `.trapperkeeper.json`, and configures
`log.md` with `merge=union`.

Alternatively, store the wiki as a normal committed directory:

```sh
trk init --in-tree docs/wiki
```

If you already have a partial in-tree wiki, adopt it without clobbering existing
files:

```sh
trk init --in-tree docs/wiki --adopt
```

## Agent Setup

Trapperkeeper can wire `trk prime` into supported coding-agent startup hooks.

For Claude Code:

```sh
trk setup claude
```

For Codex:

```sh
trk setup codex
```

Both setup commands are idempotent and merge with existing configuration.

## Usage

Print the wiki instructions and current retrieval context:

```sh
trk prime
```

## Finding compiled knowledge

`trk find "release approval"` searches the local wiki discovered from the
current repository (including subdirectories and both storage modes). It follows
only direct Markdown links in `index.md` and `toc.md`, not recursive links,
unlinked pages, raw sources, conversation history, external URLs or Git history.
It works offline and does not use a model, embeddings, daemon or persistent index.
Navigation therefore remains the maintained front door to the corpus.

The deterministic `lexical-v2` ranker uses index aliases, page titles, headings
and visible passage terms, with logarithmic corpus rarity weighting. Exact
multiword aliases and identifiers receive priority. Other results favor actual
passage evidence over navigation-only matches, then weighted relevance rather
than raw matched-word counts. Leading bold record labels receive a topic boost.
Bounded overlapping windows reach matches deep inside long paragraphs; reported
`passage_terms` describe only the returned excerpt, not distant words in its
section. A conservative trailing-s fold matches simple English plurals; it is
not a general stemmer or synonym model. Scores are not comparable across ranker
versions; telemetry records the version for before/after analysis.
It returns one section per page, deduplicated, with source path/lines, navigation
origin, SHA-256 digest, dates/status context where present, and coverage warnings.
Scores are ordering signals, not confidence or proof. This is lexical matching:
paraphrases can miss, older sections can rank highly, and a few shared words can
produce irrelevant lower-ranked results. Read the citation and verify current
facts before acting. Known-page reads and scoped `rg` remain appropriate.

Use `--json` for schema-versioned output. `--query-stdin` reads one UTF-8 literal
query instead of a positional argument; neither form executes the query.
Limits: 1,024 query characters / 64 words; 1–10 results (default 3); rendered
output 1,024–12,000 Unicode characters (default 6,000), **not model tokens**.
Each excerpt is at most 600 characters. Lower results may be omitted to meet the
budget, even all results at a small budget; `matched_pages` and `truncated`
distinguish this from a miss. Unusable metadata budgets return an error.

Exit codes: 0 = results or partial coverage (inspect `hits`); 3 = no match in
complete declared coverage; 2 = invalid input/budget; 1 = unavailable wiki.
An empty `partial` result does not prove that knowledge is absent.

Reads are bounded to 1 MiB per file, 2,000 targets, 64 MiB accepted corpus,
10,000 navigation links per navigation file, 64 entries per target across both
navigation files, and 4 KiB per navigation entry. Exceeding a
limit marks coverage partial. Non-UTF-8, nonregular, hard-linked, symlinked and
traversing targets are excluded. Descriptor-relative reads prevent linked pages
from escaping the configured canonical wiki root. The configured path must be
repository-relative without `..`, and its canonical root must remain within
the repository (including in orphan mode). External wiki-root symlinks are
not supported by find. The implementation currently
requires Unix. Results identify the bytes read, plus Git revision and dirty state;
they are **not an atomic snapshot** of concurrently edited files. Changed files
during a read and revision changes during a scan are reported; edits between
individual file reads can still produce a mixed snapshot.

## Private retrieval telemetry

Telemetry is off by default. To opt in for a repository, choose a private,
absolute path **outside tracked repositories**:

```sh
git config --local trk.telemetryDir /absolute/private/path/trk-events
trk find "release approval" --actor example-peer --episode example-session
trk feedback <SEARCH_ID> --outcome useful --rank 1
trk telemetry --days 7
```

`--telemetry-dir` overrides `TRK_TELEMETRY_DIR`, then Git `trk.telemetryDir`.
`--no-telemetry` overrides all three. No telemetry is sent over the network.
Directories are created owner-only (0700); logs require 0600 with no symlink or
hard-link targets. Every directory component is opened without following
symlinks. Locations within Git worktrees, Git metadata or bare repositories
are rejected before creating logs. An inaccessible or unsafe location is an
error, not an empty report. Git global config can intentionally opt in all repositories.

Receipts record successful, partial, no-match and failed searches, ranker version,
query hash/length, corpus revision/digest/coverage, returned paths/digests/ranks,
output size, truncation, exit code and search/render latency. Latency excludes
log synchronization and stdout delivery; receipts do not prove the agent read
the result. Explicit `--actor` / `--episode` override `TRK_ACTOR` / `TRK_EPISODE`;
actor also accepts `MYBUDDY_PEER_ID`, episode falls back to `CODEX_SESSION_ID` or
`CLAUDE_SESSION_ID`. A MyBuddy bootstrap **basename only** is recorded as an
unverified origin hint when available; its contents are never read.

Literal queries are omitted unless `--log-query` or Git `trk.logQueries=true`
explicitly opts in. Excerpts are never logged. Hashes are not anonymization:
queries may be guessable, and paths/identities are sensitive. Keep logs private.
Use raw-query opt-in only for an appropriate private evaluation.

Feedback outcomes are `useful`, `irrelevant`, `missing`, `stale`, `incorrect`.
Feedback must reference a logged search from the last 30 days and, if specified,
a rank actually returned. The report summarizes searches, status, actor, returned
pages, latency, output size and latest feedback per search/rank. Feedback is a
reported judgment, not independently verified correctness; no feedback means
unknown. Reported page counts are appearances, not confirmed reads. This covers
**logged `trk find` only**, not direct file/SQL access or all eligible episodes.
Pair aggregate counts with sampled evidence review to improve navigation and
detect stale or incomplete knowledge; do not optimize for more searches.

Daily JSONL files are capped at 16 MiB. Reports read at most the last 30 days;
older files are retained, not automatically deleted. Apply the instance's private
retention policy separately. Appends use nonblocking locks; a busy/full/unsafe
log produces a visible warning and `telemetry: failed`, without breaking useful
retrieval. These failed receipts are not available to the report, so the report
is not a complete attempt denominator. Disable logging with `--no-telemetry` or
remove the Git setting; no wiki content or history is changed by a lookup.

In initialized repositories, agents should read the wiki before exploring source
files:

- `toc.md` for top-down navigation
- `index.md` for lookup by concept, command, filename, or error text
- `pages/` for synthesized architecture, decisions, and module knowledge
- `log.md` for append-only wiki activity

There is no `trk write` command. Update wiki files directly when useful
understanding crystallizes.

For the default orphan-branch mode:

```sh
git -C .trapper_keeper status
git -C .trapper_keeper add -A
git -C .trapper_keeper commit -m "Update trapperkeeper wiki"
```

For in-tree mode, commit the wiki files with the rest of the repository.

## Commands

```text
trk init [--in-tree <PATH>] [--adopt]
trk setup claude
trk setup codex
trk prime
trk find "release approval" [--json] [--limit 3] [--max-chars 6000]
trk find --query-stdin
trk feedback <SEARCH_ID> --outcome useful [--rank 1]
trk telemetry [--days 7]
```

## Storage Modes

Default orphan mode keeps wiki history separate from code history while still
traveling with the repository through Git. In-tree mode keeps the wiki visible
in normal code review and is useful when human collaborators should read or edit
the wiki without using a separate worktree.
