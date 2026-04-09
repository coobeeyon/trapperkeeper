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

This repo has an LLM-maintained wiki on the `{branch}` orphan branch.
It contains compiled knowledge about the codebase — architecture, decisions,
cross-references — so you don't have to re-derive understanding each session.

## How to use it

- **Read a file:** `git show {branch}:<path>`
- **List all files:** `git ls-tree -r --name-only {branch}`
- **Start with:** `toc.md` (navigation), `index.md` (concept lookup)

## Structure

- `toc.md` — hierarchical navigation with summaries
- `index.md` — concept-to-location cross-reference
- `log.md` — chronological record of wiki activity
- `pages/` — synthesized wiki pages
- `sources/` — raw materials (design docs, notes)

## When to update the wiki

Update when understanding crystallizes: after a plan is ratified,
implementation finishes, or a review surfaces new insights. Keep
toc, index, and cross-references consistent when you do.
"
    );

    Ok(())
}
