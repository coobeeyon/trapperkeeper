use crate::config::{self, Config};
use crate::git;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const TOC_CONTENT: &str = "\
# Table of Contents
<!-- Hierarchical navigation. Group pages under `## <Section>` headers; sort sections and entries alphabetically. -->

_No pages yet._
";

const INDEX_CONTENT: &str = "\
# Index
<!-- Concept lookup. Flat, one entry per line, sorted alphabetically. No section headers. -->

_No entries yet._
";

const LOG_CONTENT: &str = "# Log\n";
const KEEP: &str = "";
const WORKTREE_DIR: &str = ".trapper_keeper";

pub fn run(in_tree: Option<PathBuf>, adopt: bool) -> Result<(), String> {
    // Verify we're in a git repo
    git::git(&["rev-parse", "--git-dir"])?;

    if config::exists()? {
        return Err(format!(
            "trapperkeeper already initialized ({} exists)",
            config::FILE
        ));
    }
    if git::branch_exists() {
        return Err(format!(
            "trapperkeeper already initialized (branch '{}' exists)",
            git::BRANCH
        ));
    }

    match in_tree {
        None => init_orphan(),
        Some(path) => init_in_tree(path, adopt),
    }
}

fn init_orphan() -> Result<(), String> {
    let toc_blob = git::hash_blob(TOC_CONTENT)?;
    let index_blob = git::hash_blob(INDEX_CONTENT)?;
    let log_blob = git::hash_blob(LOG_CONTENT)?;
    let keep_blob = git::hash_blob(KEEP)?;

    let pages_tree = git::mktree(&[("100644", "blob", &keep_blob, ".gitkeep")])?;
    let sources_tree = git::mktree(&[("100644", "blob", &keep_blob, ".gitkeep")])?;

    let root_tree = git::mktree(&[
        ("100644", "blob", &toc_blob, "toc.md"),
        ("100644", "blob", &index_blob, "index.md"),
        ("100644", "blob", &log_blob, "log.md"),
        ("040000", "tree", &pages_tree, "pages"),
        ("040000", "tree", &sources_tree, "sources"),
    ])?;

    let commit = git::commit_tree(&root_tree, "Initialize trapperkeeper")?;
    git::update_ref(git::BRANCH, &commit)?;

    setup_worktree()?;

    ensure_gitattributes_union_merge(&format!("{WORKTREE_DIR}/log.md"))?;
    config::save(&Config::orphan())?;

    eprintln!("Initialized trapperkeeper on branch '{}'", git::BRANCH);
    Ok(())
}

fn init_in_tree(path: PathBuf, adopt: bool) -> Result<(), String> {
    if path.is_absolute() {
        return Err(format!(
            "--in-tree path must be repo-relative, got: {}",
            path.display()
        ));
    }

    if adopt {
        let has_wiki_file = ["toc.md", "index.md", "log.md"]
            .iter()
            .any(|f| path.join(f).exists());
        if !has_wiki_file {
            return Err(format!(
                "--adopt requires an existing wiki at '{}' (expected at \
                 least one of toc.md, index.md, or log.md)",
                path.display()
            ));
        }
    } else if path.exists() {
        let is_empty = fs::read_dir(&path)
            .map(|mut it| it.next().is_none())
            .unwrap_or(false);
        if !is_empty {
            return Err(format!(
                "--in-tree path '{}' already exists and is not empty \
                 (pass --adopt to wire up an existing wiki)",
                path.display()
            ));
        }
    }

    let pages = path.join("pages");
    let sources = path.join("sources");
    fs::create_dir_all(&pages).map_err(|e| format!("failed to create {}: {e}", pages.display()))?;
    fs::create_dir_all(&sources)
        .map_err(|e| format!("failed to create {}: {e}", sources.display()))?;

    write_file_if_missing(&path.join("toc.md"), TOC_CONTENT)?;
    write_file_if_missing(&path.join("index.md"), INDEX_CONTENT)?;
    write_file_if_missing(&path.join("log.md"), LOG_CONTENT)?;
    write_file_if_missing(&pages.join(".gitkeep"), "")?;
    write_file_if_missing(&sources.join(".gitkeep"), "")?;

    let sources_ignore = format!("/{}/sources/", path.to_string_lossy());
    append_line_if_missing(Path::new(".gitignore"), &sources_ignore)?;

    let log_rel = format!("{}/log.md", path.to_string_lossy());
    ensure_gitattributes_union_merge(&log_rel)?;

    config::save(&Config::in_tree(path.clone()))?;

    if adopt {
        eprintln!("Adopted existing wiki at '{}'", path.display());
    } else {
        eprintln!("Initialized trapperkeeper in-tree at '{}'", path.display());
    }
    eprintln!("Review and commit the new files as part of your next commit.");
    Ok(())
}

fn write_file_if_missing(path: &Path, content: &str) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }
    fs::write(path, content).map_err(|e| format!("failed to write {}: {e}", path.display()))
}

fn setup_worktree() -> Result<(), String> {
    git::git(&["worktree", "add", WORKTREE_DIR, git::BRANCH])?;

    let entry = format!("/{WORKTREE_DIR}");
    append_line_if_missing(Path::new(".gitignore"), &entry)?;

    eprintln!("Created worktree at {WORKTREE_DIR}/");
    Ok(())
}

fn append_line_if_missing(path: &Path, entry: &str) -> Result<(), String> {
    let contents = fs::read_to_string(path).unwrap_or_default();
    if contents.lines().any(|l| l.trim() == entry) {
        return Ok(());
    }
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("failed to open {}: {e}", path.display()))?;
    if !contents.is_empty() && !contents.ends_with('\n') {
        writeln!(f).map_err(|e| format!("failed to write {}: {e}", path.display()))?;
    }
    writeln!(f, "{entry}").map_err(|e| format!("failed to write {}: {e}", path.display()))?;
    Ok(())
}

fn ensure_gitattributes_union_merge(path: &str) -> Result<(), String> {
    let entry = format!("{path} merge=union");
    append_line_if_missing(Path::new(".gitattributes"), &entry)
}
