use crate::config::{self, Mode};

pub fn run() -> Result<(), String> {
    let cfg = match config::load()? {
        Some(cfg) => cfg,
        None => return Ok(()),
    };

    let wiki_path = cfg.wiki_path().to_string_lossy().into_owned();
    let commit_block = match &cfg.mode {
        Mode::Orphan => format!(
            "The wiki lives on the `{branch}` orphan branch, checked out as a\n\
             gitignored worktree. Read and write files there directly. Commit your\n\
             changes with `git -C {path} add -A && git -C {path} commit -m \"<message>\"`.",
            branch = crate::git::BRANCH,
            path = wiki_path,
        ),
        Mode::InTree { .. } => format!(
            "The wiki lives in `{path}/` as a normal committed directory. Read and\n\
             write files there directly. Stage and commit wiki changes as part of\n\
             your normal git workflow — ideally in a separate commit from code\n\
             changes so wiki updates don't clutter code review.",
            path = wiki_path,
        ),
    };

    print!(
        "\
# Trapperkeeper Wiki

This repo has an LLM-maintained wiki at `{path}/` in the repo root.
The wiki is yours — you maintain it, and you are its primary consumer.
Treat it as a retrieval system: it exists so future agents can search
before reading source and avoid re-deriving understanding that a previous
session already worked out. A good wiki page saves dozens of tool calls.

{commit_block}

## Consult the wiki before exploring

Before reading source files to understand the codebase, check the wiki first.

- `{path}/toc.md` — find the relevant page
- `{path}/index.md` — search by the words a future agent would actually use
- `{path}/pages/<page>.md` — read compiled knowledge

If the wiki answers your question, use it. If it's incomplete or wrong,
fix it after you've done the exploration.

## Update the wiki when understanding crystallizes

These are the key moments:

- **Plan ratified** — record the *why* and the intended approach
- **Implementation finished** — record *what* was built and how
- **Review or debugging done** — record *what you learned*

Only add durable knowledge: architecture, ownership boundaries, invariants,
cross-file flows, operational commands, surprising traps, decisions, and
file locations future agents will need. Do not log transcript notes,
obvious edits, or facts that are cheaper to rediscover than to maintain.

On every durable update, maintain consistency across all three:
- `toc.md` — navigation with page summaries
- `index.md` — search phrases and aliases mapped to pages/files
- `log.md` — append a dated entry: `## [YYYY-MM-DD] verb | summary`

This bookkeeping is where the real value compounds. Do not skip it.

## Format invariants

Both `toc.md` and `index.md` use one entry per line (no multi-line
entries) so git's line-based merge can handle concurrent edits. Beyond
that the two files serve different purposes and look different:

- **toc.md — hierarchical navigation.** Group pages under `## <Section>`
  headers (e.g. `## Architecture`, `## Concepts`). Sort sections
  alphabetically by heading; sort entries alphabetically within each
  section. The sections are the value — this is a table of contents,
  not a flat list. Small sections may still conflict on concurrent adds
  to the same section; that's an accepted cost for navigability.
- **index.md — flat concept lookup.** One entry per line, sorted
  alphabetically by entry text (ASCII-lowercased, ties by raw bytes).
  Write entries for likely search phrases, synonyms, file names, commands,
  config keys, error text, domain concepts, and where-is-this-handled
  questions. Each entry should point to the best wiki page and, when useful,
  source files. No section headers — letter-headers like `## A` cluster adds
  and add no semantic value. Keep the file in sort order after every edit.
- **log.md — append-only chronological.** Merges via `merge=union`
  (configured in `.gitattributes`), so concurrent appends always merge
  cleanly. Not sorted.

## Pages

Pages live in `{path}/pages/`. A page should capture synthesized understanding,
not raw notes. Good pages: architecture overviews, module summaries,
decision records (the *why* behind choices), concept pages that span
multiple files. Add retrieval hooks near the top: concepts, key files,
commands/config, and useful-when cues that help an agent decide whether
to read the page.
",
        path = wiki_path,
        commit_block = commit_block,
    );

    Ok(())
}
