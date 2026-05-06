use crate::config::{self, Mode};

pub fn run() -> Result<(), String> {
    let cfg = match config::load()? {
        Some(cfg) => cfg,
        None => return Ok(()),
    };

    let wiki_path = cfg.wiki_path().to_string_lossy().into_owned();

    let (commit_block, sync_block) = match &cfg.mode {
        Mode::Orphan => (
            format!(
                "The wiki lives on the `{branch}` orphan branch, checked out as a\n\
                 gitignored worktree. This is the default Trapper Keeper layout. Read\n\
                 and write files there directly. Commit your changes with:\n\
                 `git -C {path} add -A && git -C {path} commit -m \"<message>\"`.",
                branch = crate::git::BRANCH,
                path = wiki_path,
            ),
            orphan_sync_block(&wiki_path),
        ),
        Mode::InTree { .. } => (
            format!(
                "The wiki lives in `{path}/` as a normal committed directory — the\n\
                 in-tree layout supported by Trapper Keeper as an alternative to the\n\
                 default branch-based storage. Read and write files there directly.\n\
                 Stage and commit wiki changes as part of your normal git workflow —\n\
                 ideally in a separate commit from code changes so wiki updates don't\n\
                 clutter code review.",
                path = wiki_path,
            ),
            in_tree_sync_block(&wiki_path),
        ),
    };

    print!(
        "\
# Trapperkeeper Wiki

This repo has an LLM-maintained wiki at `{path}/` in the repo root.
The wiki is shared project memory, not local scratch — you maintain
it, and you (and any future session on any machine) are its primary
consumers. A good wiki page saves dozens of tool calls.

{commit_block}

## Consult the wiki before exploring

Before reading source files to understand the codebase, check the wiki first.

- `{path}/toc.md` — find the relevant page
- `{path}/index.md` — look up a concept
- `{path}/pages/<page>.md` — read compiled knowledge

If the wiki answers your question, use it. If it's incomplete or wrong,
fix it after you've done the exploration.

{sync_block}

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
  No section headers — letter-headers like `## A` cluster adds and add
  no semantic value. Keep the file in sort order after every edit.
- **log.md — append-only chronological.** Merges via `merge=union`
  (configured in `.gitattributes`), so concurrent appends always merge
  cleanly. Not sorted.

## Pages

Pages live in `{path}/pages/`. A page should capture synthesized understanding,
not raw notes. Good pages: architecture overviews, module summaries,
decision records (the *why* behind choices), concept pages that span
multiple files.
",
        path = wiki_path,
        commit_block = commit_block,
        sync_block = sync_block,
    );

    Ok(())
}

fn orphan_sync_block(wiki_path: &str) -> String {
    let branch = crate::git::BRANCH;
    format!(
        "\
## Wiki sync discipline

Treat the wiki as shared project memory backed by a remote, not local
notes. The `{branch}` branch is the source of truth — fetch before you
edit, push as soon as you're done. Do this proactively, not only when
the user asks. Any work that produces durable context another session
or machine will need belongs in the wiki.

### Before reading or editing the wiki

- Fetch the wiki remote: `git -C {path} fetch`.
- Integrate the remote `{branch}` branch into your local worktree
  before making new edits (e.g. `git -C {path} pull --ff-only`, or
  merge / rebase if you already have local commits).
- Never overwrite remote wiki history with `--force` unless the user
  explicitly asks for it.

### During meaningful work

Update the wiki whenever understanding crystallizes — plan ratified,
implementation finished, review or debugging done. Specifically:

- Add or update pages for decisions, PRs, branches, investigations,
  blockers, and shipped behavior.
- Append a dated entry to `log.md`: `## [YYYY-MM-DD] verb | summary`.
- Update `toc.md` so every navigable page is listed under the right
  section.
- Update `index.md` with searchable concepts, branches, tickets, PRs,
  and important file/module names.
- Preserve raw external material under `{path}/sources/` when used.

This bookkeeping — toc, index, log cross-references — is where the
real value compounds. Do not skip it.

### At the end of wiki-relevant work

- Grep the wiki for stray conflict markers (`<<<<<<<`, `=======`,
  `>>>>>>>`) and trailing-whitespace mistakes before committing.
- Commit wiki changes separately from service-code changes — the wiki
  lives on its own branch and must not be mixed with code commits.
- Fetch again, merge any new remote wiki changes, and resolve conflicts
  by preserving useful content from both sides rather than discarding.
- Push the wiki: `git -C {path} push origin {branch}`.
- Report the commit hash and confirm the wiki worktree is clean.

Do not leave the wiki dirty at the end of a turn unless the user
explicitly asks you to pause before committing or pushing.",
        path = wiki_path,
        branch = branch,
    )
}

fn in_tree_sync_block(wiki_path: &str) -> String {
    format!(
        "\
## Wiki sync discipline

Treat the wiki as shared project memory backed by a remote, not local
notes. The repo's normal remote branch is the source of truth — fetch
before you edit, push as soon as you're done. Do this proactively, not
only when the user asks. Any work that produces durable context another
session or machine will need belongs in the wiki.

### Before reading or editing the wiki

- Fetch the repo's remote so you can see any new wiki changes.
- Integrate the latest remote branch into your working tree before
  making new wiki edits.
- Never overwrite remote history with `--force` unless the user
  explicitly asks for it.

### During meaningful work

Update the wiki whenever understanding crystallizes — plan ratified,
implementation finished, review or debugging done. Specifically:

- Add or update pages for decisions, PRs, branches, investigations,
  blockers, and shipped behavior.
- Append a dated entry to `log.md`: `## [YYYY-MM-DD] verb | summary`.
- Update `toc.md` so every navigable page is listed under the right
  section.
- Update `index.md` with searchable concepts, branches, tickets, PRs,
  and important file/module names.
- Preserve raw external material under `{path}/sources/` when used.

This bookkeeping — toc, index, log cross-references — is where the
real value compounds. Do not skip it.

### At the end of wiki-relevant work

- Grep the wiki for stray conflict markers (`<<<<<<<`, `=======`,
  `>>>>>>>`) and trailing-whitespace mistakes before committing.
- Keep wiki changes logically separate from code changes where
  practical (separate commits, or a clear split inside one commit).
- Fetch again, merge any new remote changes, and resolve conflicts by
  preserving useful content from both sides rather than discarding.
- Push to the repo's normal remote branch.
- Report the commit hash and confirm the wiki directory is clean.

Do not leave the wiki dirty at the end of a turn unless the user
explicitly asks you to pause before committing or pushing.",
        path = wiki_path,
    )
}
