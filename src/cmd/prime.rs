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
It exists so you don't re-derive understanding that a previous session
already worked out. A good wiki page saves dozens of tool calls.

{commit_block}

## Consult the wiki before exploring

Before reading source files to understand the codebase, check the wiki first.

- `{path}/toc.md` — find the relevant page
- `{path}/index.md` — look up a concept
- `{path}/pages/<page>.md` — read compiled knowledge

If the wiki answers your question, use it. If it's incomplete or wrong,
fix it after you've done the exploration.

## Update the wiki when understanding crystallizes

These are the key moments:

- **Plan ratified** — record the *why* and the intended approach
- **Implementation finished** — record *what* was built and how
- **Review or debugging done** — record *what you learned*

On every update, maintain consistency across all three:
- `toc.md` — navigation with page summaries
- `index.md` — concept-to-location cross-reference
- `log.md` — append a dated entry: `## [YYYY-MM-DD] verb | summary`

This bookkeeping is where the real value compounds. Do not skip it.

## Format invariants for toc.md and index.md

Both files must stay merge-friendly so concurrent branches don't collide:

- **One entry per line.** No multi-line entries.
- **Sorted alphabetically** by entry text (ASCII-lowercased, ties broken
  by raw bytes). Keep the file in sort order after every edit.
- **No section headers.** Do not group entries under `## Architecture`,
  `## A`, etc. — adds cluster at headers and collide.

`log.md` is append-only and merges via `merge=union` (configured in
`.gitattributes`), so concurrent appends always merge cleanly.

## Pages

Pages live in `{path}/pages/`. A page should capture synthesized understanding,
not raw notes. Good pages: architecture overviews, module summaries,
decision records (the *why* behind choices), concept pages that span
multiple files.
",
        path = wiki_path,
        commit_block = commit_block,
    );

    Ok(())
}
