# In-Tree Mode

Alternative storage mode where the wiki lives as a committed directory
in the repo instead of on the `trapperkeeper` orphan branch. Chosen once
at `trk init` time; orphan remains the default.

## Motivation

Orphan-branch mode hides the wiki behind a gitignored worktree. Any
collaborator who doesn't know about trapperkeeper can't discover or read
it. For team or OSS projects that's a real cost. In-tree mode trades PR
noise (wiki edits show up in diffs) for human visibility.

## How to use it

```
trk init --in-tree docs/wiki           # fresh wiki
trk init --in-tree docs/wiki --adopt   # wrap an existing wiki
```

The path is required — there's no safe cross-ecosystem default. After
init:

- `docs/wiki/toc.md`, `index.md`, `log.md`, `pages/`, `sources/` are
  created on disk as untracked files (if they don't already exist).
- `.trapperkeeper.json` records the mode and path.
- `docs/wiki/sources/` is added to `.gitignore` (sources are intentionally
  not committed — see below).
- `.gitattributes` gets a `docs/wiki/log.md merge=union` entry.
- Nothing is committed. The user commits the new files as part of their
  next commit.

## `--adopt` for existing wikis

If you already have a wiki directory — e.g. hand-maintained docs, or
wiki content from a prior tool — use `--adopt` to wire it up without
clobbering:

```
trk init --in-tree docs/wiki --adopt
```

`--adopt` behavior:

- Requires `--in-tree` (no orphan equivalent — orphan mode is always fresh).
- Requires at least one of `toc.md` / `index.md` / `log.md` to exist in
  the path (safety check so it can't silently turn into "clobber off").
- Never overwrites existing files. Any missing skeleton piece
  (`pages/`, `sources/`, `.gitkeep`s, missing `toc.md`, etc.) is filled in.
- Still writes `.trapperkeeper.json`, the `.gitignore` entry for
  `sources/`, and the `.gitattributes` union-merge entry for `log.md`.

If the existing toc/index aren't in the flat-sorted format, run `trk
prime` to see the invariants and migrate them by hand (not automated —
the format is enforced by convention, not tooling).

## Why `sources/` is gitignored

Per SPEC, `sources/` holds "raw materials that aren't in the repo" —
pasted design docs, transcripts, research notes. In orphan mode those
live on the orphan branch, out of the main repo. In in-tree mode the
wiki directory IS in the main repo, so committing sources would re-create
the exact PR-noise problem in-tree mode exists to solve. Solution:
gitignore `<path>/sources/` so raw materials stay local.

## Merge-friendly format for toc.md and index.md

In-tree mode means concurrent branches can edit wiki files. The wiki's
two index files (`toc.md`, `index.md`) must be written in a format that
git's line-based 3-way merge handles well.

Invariants (enforced by convention, documented in `trk prime` output):

- **One entry per line.** No multi-line entries.
- **Sorted alphabetically**, ASCII-lowercased, tiebreak by raw bytes.
- **No section headers.** Grouping entries under `## Architecture` or
  `## A` clusters adds at the same lines and forces conflicts.

### What merges cleanly

- Adds that land in different regions of the sorted file (the common case
  once the wiki has more than a handful of entries).
- Appends to `log.md` — handled by `merge=union`, always clean.

### What conflicts (by design)

- Two branches inserting different entries into the *same gap* between
  adjacent existing entries. Line-based merge can't pick an order.
  Acceptable — rare in practice, trivial to resolve.
- Two branches editing the *same entry* with different content. This is
  a real disagreement and should surface.

See `tests/merge.rs` for end-to-end exercises of these cases.

## Config file

`.trapperkeeper.json` at repo root, committed. Structure:

```json
{
  "version": 1,
  "mode": "in-tree",
  "path": "docs/wiki"
}
```

Orphan mode writes `{"version": 1, "mode": "orphan"}`. For backward
compat, if the config file is absent but the `trapperkeeper` branch
exists, the tool treats that as orphan mode.

## Limitations and out-of-scope

- **No mode switch** today. To change modes, manually delete the wiki
  directory (or orphan branch) and the config file, then re-init.
- **Prime assumes cwd = repo root.** Existing latent issue; in-tree
  paths amplify it slightly. Fix separately.
- **No `trk check`** command to validate format invariants. The AI
  maintains them by convention.
