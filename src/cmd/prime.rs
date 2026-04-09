use crate::git;

pub fn run() -> Result<(), String> {
    // Silent exit if trapperkeeper is not initialized
    if !git::branch_exists() {
        return Ok(());
    }

    let branch = git::BRANCH;

    print!(
        "\
# Trapperkeeper Wiki

This repo has an LLM-maintained wiki at `.trapper_keeper/` in the repo root.
The wiki is yours — you maintain it, and you are its primary consumer.
It exists so you don't re-derive understanding that a previous session
already worked out. A good wiki page saves dozens of tool calls.

The wiki lives on the `{branch}` orphan branch, checked out as a
gitignored worktree. Read and write files there directly. Commit your
changes with `git -C .trapper_keeper add -A && git -C .trapper_keeper commit -m \"<message>\"`.

## Consult the wiki before exploring

Before reading source files to understand the codebase, check the wiki first.

- `.trapper_keeper/toc.md` — find the relevant page
- `.trapper_keeper/index.md` — look up a concept
- `.trapper_keeper/pages/<page>.md` — read compiled knowledge

If the wiki answers your question, use it. If it's incomplete or wrong,
fix it after you've done the exploration.

## Update the wiki when understanding crystallizes

These are the key moments:

- **Plan ratified** — record the *why* and the intended approach
- **Implementation finished** — record *what* was built and how
- **Review or debugging done** — record *what you learned*

On every update, maintain consistency across all three:
- `toc.md` — hierarchical navigation with summaries
- `index.md` — concept-to-location cross-reference
- `log.md` — append a dated entry: `## [YYYY-MM-DD] verb | summary`

This bookkeeping is where the real value compounds. Do not skip it.

## Pages

Pages live in `pages/`. A page should capture synthesized understanding,
not raw notes. Good pages: architecture overviews, module summaries,
decision records (the *why* behind choices), concept pages that span
multiple files.
"
    );

    Ok(())
}
