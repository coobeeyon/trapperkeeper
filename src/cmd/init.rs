use crate::git;

const TOC_CONTENT: &str = "# Table of Contents\n\n_No pages yet. Run a wiki update to populate._\n";
const INDEX_CONTENT: &str = "# Index\n\n_No entries yet. Run a wiki update to populate._\n";
const LOG_CONTENT: &str = "# Log\n";
const KEEP: &str = "";

pub fn run() -> Result<(), String> {
    // Verify we're in a git repo
    git::git(&["rev-parse", "--git-dir"])?;

    if git::branch_exists() {
        return Err(format!(
            "trapperkeeper already initialized (branch '{}' exists)",
            git::BRANCH
        ));
    }

    // Create blobs for skeleton files
    let toc_blob = git::hash_blob(TOC_CONTENT)?;
    let index_blob = git::hash_blob(INDEX_CONTENT)?;
    let log_blob = git::hash_blob(LOG_CONTENT)?;
    let keep_blob = git::hash_blob(KEEP)?;

    // Create subtrees for pages/ and sources/
    let pages_tree = git::mktree(&[("100644", "blob", &keep_blob, ".gitkeep")])?;
    let sources_tree = git::mktree(&[("100644", "blob", &keep_blob, ".gitkeep")])?;

    // Create root tree
    let root_tree = git::mktree(&[
        ("100644", "blob", &toc_blob, "toc.md"),
        ("100644", "blob", &index_blob, "index.md"),
        ("100644", "blob", &log_blob, "log.md"),
        ("040000", "tree", &pages_tree, "pages"),
        ("040000", "tree", &sources_tree, "sources"),
    ])?;

    // Create orphan commit (no parent)
    let commit = git::commit_tree(&root_tree, "Initialize trapperkeeper")?;

    // Point the branch at the commit
    git::update_ref(git::BRANCH, &commit)?;

    // Set up the gitignored worktree
    setup_worktree()?;

    eprintln!(
        "Initialized trapperkeeper on branch '{}'",
        git::BRANCH
    );
    Ok(())
}

const WORKTREE_DIR: &str = ".trapper_keeper";

fn setup_worktree() -> Result<(), String> {
    git::git(&["worktree", "add", WORKTREE_DIR, git::BRANCH])?;

    // Ensure .trapper_keeper is in .gitignore
    let gitignore_path = ".gitignore";
    let entry = format!("/{WORKTREE_DIR}");
    let contents = std::fs::read_to_string(gitignore_path).unwrap_or_default();
    if !contents.lines().any(|l| l.trim() == entry) {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(gitignore_path)
            .map_err(|e| format!("Failed to open .gitignore: {e}"))?;
        // Add newline before entry if file doesn't end with one
        if !contents.is_empty() && !contents.ends_with('\n') {
            writeln!(f).map_err(|e| format!("Failed to write .gitignore: {e}"))?;
        }
        writeln!(f, "{entry}").map_err(|e| format!("Failed to write .gitignore: {e}"))?;
    }

    eprintln!("Created worktree at {WORKTREE_DIR}/");
    Ok(())
}
