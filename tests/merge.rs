//! Integration tests for trapperkeeper. Exercises both storage modes,
//! init guards, `prime` output, and the merge-friendliness of the
//! flat-sorted wiki file format.

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use tempfile::TempDir;

const WIKI: &str = "docs/wiki";

// --------------------------- harness ---------------------------------

struct Repo {
    _tmp: TempDir,
    path: PathBuf,
}

impl Repo {
    fn new() -> Self {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().to_path_buf();
        git(&path, &["init", "-q", "-b", "main"]);
        fs::write(path.join("README.md"), "# test\n").unwrap();
        git(&path, &["add", "README.md"]);
        git(&path, &["commit", "-q", "-m", "initial"]);
        Self { _tmp: tmp, path }
    }

    fn trk(&self, args: &[&str]) {
        Command::cargo_bin("trk")
            .unwrap()
            .current_dir(&self.path)
            .envs(hermetic_env())
            .args(args)
            .assert()
            .success();
    }

    fn trk_fails(&self, args: &[&str]) {
        Command::cargo_bin("trk")
            .unwrap()
            .current_dir(&self.path)
            .envs(hermetic_env())
            .args(args)
            .assert()
            .failure();
    }

    fn trk_stdout(&self, args: &[&str]) -> String {
        let out = Command::cargo_bin("trk")
            .unwrap()
            .current_dir(&self.path)
            .envs(hermetic_env())
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "trk {args:?} failed");
        String::from_utf8(out.stdout).unwrap()
    }

    fn commit_all(&self, msg: &str) {
        git(&self.path, &["add", "-A"]);
        git(&self.path, &["commit", "-q", "-m", msg]);
    }

    fn checkout_new(&self, branch: &str) {
        git(&self.path, &["checkout", "-q", "-b", branch]);
    }

    fn checkout(&self, branch: &str) {
        git(&self.path, &["checkout", "-q", branch]);
    }

    fn try_merge(&self, branch: &str) -> bool {
        let out = StdCommand::new("git")
            .args(["merge", "--no-edit", "-q", branch])
            .current_dir(&self.path)
            .envs(hermetic_env())
            .output()
            .unwrap();
        out.status.success()
    }

    fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.path.join(rel)).unwrap()
    }

    fn write(&self, rel: &str, content: &str) {
        fs::write(self.path.join(rel), content).unwrap();
    }
}

fn git(cwd: &Path, args: &[&str]) {
    let out = StdCommand::new("git")
        .args(args)
        .current_dir(cwd)
        .envs(hermetic_env())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn hermetic_env() -> Vec<(&'static str, &'static str)> {
    vec![
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_CONFIG_SYSTEM", "/dev/null"),
        ("GIT_AUTHOR_NAME", "test"),
        ("GIT_AUTHOR_EMAIL", "test@example.com"),
        ("GIT_COMMITTER_NAME", "test"),
        ("GIT_COMMITTER_EMAIL", "test@example.com"),
        ("GIT_TERMINAL_PROMPT", "0"),
        ("HOME", "/dev/null"),
    ]
}

fn wiki_body(entries: &[&str]) -> String {
    let mut sorted: Vec<&&str> = entries.iter().collect();
    sorted.sort_by_key(|e| e.to_ascii_lowercase());
    let joined: String = sorted.iter().map(|e| format!("{e}\n")).collect();
    format!(
        "# Table of Contents\n<!-- sorted alphabetically, one entry per line, no section headers -->\n\n{joined}"
    )
}

// --------------------------- init: orphan ----------------------------

#[test]
fn orphan_init_creates_branch_worktree_and_config() {
    let repo = Repo::new();
    repo.trk(&["init"]);

    assert!(repo.path.join(".trapper_keeper").exists(), "worktree dir");
    assert!(
        repo.path.join(".trapper_keeper/toc.md").exists(),
        "worktree populated"
    );
    assert!(
        repo.path.join(".trapperkeeper.json").exists(),
        "config file"
    );

    let cfg = fs::read_to_string(repo.path.join(".trapperkeeper.json")).unwrap();
    assert!(cfg.contains("\"mode\": \"orphan\""));
    assert!(cfg.contains("\"version\": 1"));

    let gitignore = fs::read_to_string(repo.path.join(".gitignore")).unwrap();
    assert!(
        gitignore.contains("/.trapper_keeper"),
        "worktree dir gitignored: {gitignore}"
    );

    let attrs = fs::read_to_string(repo.path.join(".gitattributes")).unwrap();
    assert!(
        attrs.contains(".trapper_keeper/log.md merge=union"),
        "orphan mode log.md should have merge=union: {attrs}"
    );
}

// --------------------------- init: in-tree ---------------------------

#[test]
fn in_tree_init_creates_skeleton_and_config() {
    let repo = Repo::new();
    repo.trk(&["init", "--in-tree", WIKI]);

    for f in ["toc.md", "index.md", "log.md"] {
        assert!(
            repo.path.join(WIKI).join(f).exists(),
            "missing skeleton file: {f}"
        );
    }
    assert!(repo.path.join(WIKI).join("pages").is_dir());
    assert!(repo.path.join(WIKI).join("sources").is_dir());

    let cfg = fs::read_to_string(repo.path.join(".trapperkeeper.json")).unwrap();
    assert!(cfg.contains("\"mode\": \"in-tree\""));
    assert!(cfg.contains(&format!("\"path\": \"{WIKI}\"")));
    assert!(cfg.contains("\"version\": 1"));

    let gitignore = fs::read_to_string(repo.path.join(".gitignore")).unwrap();
    assert!(
        gitignore.contains(&format!("/{WIKI}/sources/")),
        "sources gitignored: {gitignore}"
    );

    let attrs = fs::read_to_string(repo.path.join(".gitattributes")).unwrap();
    assert!(
        attrs.contains(&format!("{WIKI}/log.md merge=union")),
        "log.md merge=union: {attrs}"
    );
}

#[test]
fn in_tree_init_accepts_existing_empty_dir() {
    let repo = Repo::new();
    fs::create_dir_all(repo.path.join(WIKI)).unwrap();
    repo.trk(&["init", "--in-tree", WIKI]);
    assert!(repo.path.join(WIKI).join("toc.md").exists());
}

#[test]
fn in_tree_init_refuses_non_empty_dir() {
    let repo = Repo::new();
    fs::create_dir_all(repo.path.join(WIKI)).unwrap();
    fs::write(repo.path.join(WIKI).join("something.txt"), "pre-existing").unwrap();
    repo.trk_fails(&["init", "--in-tree", WIKI]);
}

// --------------------------- init: --adopt ---------------------------

#[test]
fn adopt_wires_up_existing_wiki_without_clobbering() {
    let repo = Repo::new();
    let wiki = repo.path.join(WIKI);
    fs::create_dir_all(wiki.join("pages")).unwrap();
    fs::write(
        wiki.join("toc.md"),
        "# Table of Contents\n\n- [Existing](pages/existing.md) — kept\n",
    )
    .unwrap();
    fs::write(wiki.join("index.md"), "# Index\n\n- **existing** — kept\n").unwrap();
    fs::write(
        wiki.join("log.md"),
        "# Log\n\n## [2026-04-01] pre | already there\n",
    )
    .unwrap();
    fs::write(wiki.join("pages/existing.md"), "# Existing Page\n").unwrap();

    repo.trk(&["init", "--in-tree", WIKI, "--adopt"]);

    // Existing files preserved.
    assert!(
        repo.read(&format!("{WIKI}/toc.md")).contains("Existing"),
        "adopt must not clobber existing toc.md"
    );
    assert!(
        repo.read(&format!("{WIKI}/log.md"))
            .contains("already there"),
        "adopt must not clobber existing log.md"
    );
    assert!(repo.path.join(WIKI).join("pages/existing.md").exists());

    // Bookkeeping written.
    assert!(repo.path.join(".trapperkeeper.json").exists());
    let cfg = repo.read(".trapperkeeper.json");
    assert!(cfg.contains("\"mode\": \"in-tree\""));
    assert!(cfg.contains(&format!("\"path\": \"{WIKI}\"")));

    let gitignore = repo.read(".gitignore");
    assert!(gitignore.contains(&format!("/{WIKI}/sources/")));

    let attrs = repo.read(".gitattributes");
    assert!(attrs.contains(&format!("{WIKI}/log.md merge=union")));

    // Missing scaffolding (sources/, .gitkeep) filled in.
    assert!(repo.path.join(WIKI).join("sources").is_dir());
}

#[test]
fn adopt_creates_missing_skeleton_files() {
    // User has a partial wiki — just pages/ and log.md, no toc/index yet.
    let repo = Repo::new();
    let wiki = repo.path.join(WIKI);
    fs::create_dir_all(wiki.join("pages")).unwrap();
    fs::write(wiki.join("log.md"), "# Log\n").unwrap();

    repo.trk(&["init", "--in-tree", WIKI, "--adopt"]);

    // log.md preserved, toc/index created from skeleton.
    assert!(repo.path.join(WIKI).join("toc.md").exists());
    assert!(repo.path.join(WIKI).join("index.md").exists());
    assert!(repo
        .read(&format!("{WIKI}/toc.md"))
        .contains("Hierarchical"));
    assert!(repo.read(&format!("{WIKI}/index.md")).contains("Flat"));
}

#[test]
fn adopt_refuses_when_path_is_empty() {
    let repo = Repo::new();
    fs::create_dir_all(repo.path.join(WIKI)).unwrap();
    // Empty dir — there's nothing to adopt.
    repo.trk_fails(&["init", "--in-tree", WIKI, "--adopt"]);
}

#[test]
fn adopt_refuses_when_path_has_unrelated_content() {
    let repo = Repo::new();
    let wiki = repo.path.join(WIKI);
    fs::create_dir_all(&wiki).unwrap();
    fs::write(wiki.join("README.md"), "not a wiki").unwrap();
    // Dir has content but none of toc/index/log — reject.
    repo.trk_fails(&["init", "--in-tree", WIKI, "--adopt"]);
}

#[test]
fn adopt_requires_in_tree_flag() {
    let repo = Repo::new();
    repo.trk_fails(&["init", "--adopt"]);
}

#[test]
fn in_tree_init_rejects_absolute_path() {
    let repo = Repo::new();
    repo.trk_fails(&["init", "--in-tree", "/tmp/not-allowed"]);
}

// --------------------------- init: guards ----------------------------

#[test]
fn init_refuses_when_config_exists() {
    let repo = Repo::new();
    repo.trk(&["init", "--in-tree", WIKI]);
    repo.trk_fails(&["init", "--in-tree", "other/wiki"]);
    repo.trk_fails(&["init"]);
}

#[test]
fn init_refuses_when_orphan_branch_exists_without_config() {
    let repo = Repo::new();
    // Create the orphan branch but no config file (simulates pre-config repo).
    repo.trk(&["init"]);
    fs::remove_file(repo.path.join(".trapperkeeper.json")).unwrap();
    repo.trk_fails(&["init", "--in-tree", WIKI]);
    repo.trk_fails(&["init"]);
}

// --------------------------- prime: output ---------------------------

#[test]
fn prime_outputs_nothing_when_uninitialized() {
    let repo = Repo::new();
    let out = repo.trk_stdout(&["prime"]);
    assert!(
        out.is_empty(),
        "prime should be silent when uninitialized, got: {out:?}"
    );
}

#[test]
fn prime_outputs_orphan_mode_text() {
    let repo = Repo::new();
    repo.trk(&["init"]);
    let out = repo.trk_stdout(&["prime"]);

    assert!(out.contains("`.trapper_keeper/`"), "wiki path in output");
    assert!(
        out.contains("`trapperkeeper` orphan branch"),
        "orphan mode language: {out}"
    );
    assert!(
        out.contains("git -C .trapper_keeper add -A"),
        "orphan commit instructions: {out}"
    );
    assert_format_invariants_documented(&out);
}

#[test]
fn prime_outputs_in_tree_mode_text() {
    let repo = Repo::new();
    repo.trk(&["init", "--in-tree", WIKI]);
    let out = repo.trk_stdout(&["prime"]);

    assert!(out.contains(&format!("`{WIKI}/`")), "wiki path: {out}");
    assert!(
        out.contains("normal committed directory") || out.contains("normal git workflow"),
        "in-tree language: {out}"
    );
    assert!(
        !out.contains("orphan branch"),
        "in-tree output should not mention orphan branch: {out}"
    );
    assert_format_invariants_documented(&out);
}

#[test]
fn prime_output_respects_custom_path() {
    let repo = Repo::new();
    let custom = "wiki";
    repo.trk(&["init", "--in-tree", custom]);
    let out = repo.trk_stdout(&["prime"]);

    assert!(out.contains(&format!("`{custom}/`")));
    assert!(out.contains(&format!("`{custom}/toc.md`")));
    assert!(out.contains(&format!("`{custom}/pages/")));
}

#[test]
fn prime_backward_compat_uses_orphan_when_only_branch_exists() {
    let repo = Repo::new();
    repo.trk(&["init"]);
    // Pretend this is a pre-config-file repo.
    fs::remove_file(repo.path.join(".trapperkeeper.json")).unwrap();

    let out = repo.trk_stdout(&["prime"]);
    assert!(
        out.contains("`.trapper_keeper/`"),
        "should fall back to orphan output: {out}"
    );
    assert!(out.contains("`trapperkeeper` orphan branch"));
}

fn assert_format_invariants_documented(prime_out: &str) {
    assert!(
        prime_out.contains("one entry per line") || prime_out.contains("One entry per line"),
        "one-entry-per-line invariant missing: {prime_out}"
    );
    // toc.md: hierarchical with section headers
    assert!(
        prime_out.contains("toc.md") && prime_out.contains("hierarchical"),
        "toc.md hierarchy documented: {prime_out}"
    );
    // index.md: flat
    assert!(
        prime_out.contains("index.md") && prime_out.contains("flat"),
        "index.md flat documented: {prime_out}"
    );
    // log.md: union merge
    assert!(
        prime_out.contains("merge=union"),
        "log.md merge=union documented: {prime_out}"
    );
    assert!(
        prime_out.contains("retrieval system"),
        "wiki retrieval purpose documented: {prime_out}"
    );
    assert!(
        prime_out.contains("future agents") && prime_out.contains("likely search phrases"),
        "search-oriented index guidance missing: {prime_out}"
    );
    assert!(
        prime_out.contains("retrieval hooks") && prime_out.contains("useful-when"),
        "page retrieval hook guidance missing: {prime_out}"
    );
    assert!(
        prime_out.contains("durable knowledge") && prime_out.contains("surprising traps"),
        "durable update threshold missing: {prime_out}"
    );
}

// --------------------------- merge behavior --------------------------

/// Common case: concurrent adds land in different regions of the sorted
/// file and merge cleanly.
#[test]
fn non_adjacent_toc_adds_merge_cleanly() {
    let repo = Repo::new();
    repo.trk(&["init", "--in-tree", WIKI]);
    let toc = format!("{WIKI}/toc.md");

    let base = [
        "- apples",
        "- bananas",
        "- grapes",
        "- oranges",
        "- pears",
        "- zebras",
    ];
    repo.write(&toc, &wiki_body(&base));
    repo.commit_all("base toc");

    repo.checkout_new("branch-a");
    let mut a: Vec<&str> = base.to_vec();
    a.push("- avocados");
    repo.write(&toc, &wiki_body(&a));
    repo.commit_all("add avocados");

    repo.checkout("main");
    repo.checkout_new("branch-b");
    let mut b: Vec<&str> = base.to_vec();
    b.push("- tomatoes");
    repo.write(&toc, &wiki_body(&b));
    repo.commit_all("add tomatoes");

    assert!(repo.try_merge("branch-a"), "merge should succeed");

    let merged = repo.read(&toc);
    for needle in ["apples", "avocados", "tomatoes", "zebras"] {
        assert!(
            merged.contains(needle),
            "merged toc missing {needle}:\n{merged}"
        );
    }
}

/// Hierarchical toc: two branches add entries to DIFFERENT sections
/// merge cleanly — the section headers separate them into distinct hunks.
#[test]
fn toc_section_adds_in_different_sections_merge_cleanly() {
    let repo = Repo::new();
    repo.trk(&["init", "--in-tree", WIKI]);
    let toc = format!("{WIKI}/toc.md");

    let base = "\
# Table of Contents

## Architecture
- [Architecture Overview](pages/architecture.md) — module structure
- [Git Plumbing](pages/git-plumbing.md) — orphan branch IO

## Concepts
- [LLM Wiki Pattern](pages/llm-wiki-pattern.md) — compile-don't-retrieve

## Project
- [Decisions](pages/decisions.md) — key choices
";
    repo.write(&toc, base);
    repo.commit_all("base toc");

    // Branch A adds under Architecture.
    repo.checkout_new("branch-a");
    let a = base.replace(
        "- [Git Plumbing](pages/git-plumbing.md) — orphan branch IO\n",
        "- [Git Plumbing](pages/git-plumbing.md) — orphan branch IO\n- [In-Tree Mode](pages/in-tree-mode.md) — alternative storage\n",
    );
    repo.write(&toc, &a);
    repo.commit_all("add in-tree page");

    // Branch B adds under Project.
    repo.checkout("main");
    repo.checkout_new("branch-b");
    let b = base.replace(
        "- [Decisions](pages/decisions.md) — key choices\n",
        "- [Decisions](pages/decisions.md) — key choices\n- [Roadmap](pages/roadmap.md) — what's next\n",
    );
    repo.write(&toc, &b);
    repo.commit_all("add roadmap page");

    assert!(
        repo.try_merge("branch-a"),
        "adds to different sections should merge cleanly"
    );

    let merged = repo.read(&toc);
    assert!(merged.contains("In-Tree Mode"));
    assert!(merged.contains("Roadmap"));
}

/// Pathological case: two branches insert into the same gap. Git can't
/// pick an order and flags a conflict. Documented limitation.
#[test]
fn same_gap_adds_conflict() {
    let repo = Repo::new();
    repo.trk(&["init", "--in-tree", WIKI]);
    let toc = format!("{WIKI}/toc.md");

    repo.write(&toc, &wiki_body(&["- apples", "- zebras"]));
    repo.commit_all("base toc");

    repo.checkout_new("branch-a");
    repo.write(&toc, &wiki_body(&["- apples", "- coconuts", "- zebras"]));
    repo.commit_all("add coconuts");

    repo.checkout("main");
    repo.checkout_new("branch-b");
    repo.write(&toc, &wiki_body(&["- apples", "- walnuts", "- zebras"]));
    repo.commit_all("add walnuts");

    assert!(
        !repo.try_merge("branch-a"),
        "same-gap adds should conflict (documented)"
    );
}

#[test]
fn same_entry_edits_conflict() {
    let repo = Repo::new();
    repo.trk(&["init", "--in-tree", WIKI]);
    let toc = format!("{WIKI}/toc.md");

    repo.write(&toc, &wiki_body(&["- apples — red fruit"]));
    repo.commit_all("base toc");

    repo.checkout_new("branch-a");
    repo.write(&toc, &wiki_body(&["- apples — a red fruit with seeds"]));
    repo.commit_all("edit a");

    repo.checkout("main");
    repo.checkout_new("branch-b");
    repo.write(
        &toc,
        &wiki_body(&["- apples — a fruit that grows on trees"]),
    );
    repo.commit_all("edit b");

    assert!(
        !repo.try_merge("branch-a"),
        "same-entry edits should conflict"
    );
}

#[test]
fn log_appends_merge_via_union() {
    let repo = Repo::new();
    repo.trk(&["init", "--in-tree", WIKI]);
    let log = format!("{WIKI}/log.md");

    repo.commit_all("base log");

    repo.checkout_new("branch-a");
    let base = repo.read(&log);
    repo.write(&log, &format!("{base}## [2026-04-23] work | branch a\n"));
    repo.commit_all("log a");

    repo.checkout("main");
    repo.checkout_new("branch-b");
    let base = repo.read(&log);
    repo.write(&log, &format!("{base}## [2026-04-23] work | branch b\n"));
    repo.commit_all("log b");

    assert!(repo.try_merge("branch-a"), "log merge should succeed");

    let merged = repo.read(&log);
    assert!(merged.contains("branch a"), "union keeps branch a");
    assert!(merged.contains("branch b"), "union keeps branch b");
}

/// Three branches each appending to log merge in sequence without conflict.
#[test]
fn log_union_handles_three_way_appends() {
    let repo = Repo::new();
    repo.trk(&["init", "--in-tree", WIKI]);
    let log = format!("{WIKI}/log.md");

    repo.commit_all("base log");

    for branch in ["branch-a", "branch-b", "branch-c"] {
        repo.checkout("main");
        repo.checkout_new(branch);
        let base = repo.read(&log);
        repo.write(&log, &format!("{base}## [2026-04-23] work | {branch}\n"));
        repo.commit_all(&format!("log {branch}"));
    }

    repo.checkout("main");
    assert!(repo.try_merge("branch-a"));
    assert!(repo.try_merge("branch-b"));
    assert!(repo.try_merge("branch-c"));

    let merged = repo.read(&log);
    for b in ["branch-a", "branch-b", "branch-c"] {
        assert!(merged.contains(b), "merged log missing {b}:\n{merged}");
    }
}

/// Orphan-mode log.md gets the same merge=union treatment (even though
/// orphan mode makes concurrent appends rare, they're still possible if
/// two worktrees or two clones both have the branch).
#[test]
fn orphan_log_also_has_union_merge() {
    let repo = Repo::new();
    repo.trk(&["init"]);

    let attrs = fs::read_to_string(repo.path.join(".gitattributes")).unwrap();
    assert!(
        attrs.contains(".trapper_keeper/log.md merge=union"),
        "orphan log needs union: {attrs}"
    );
}
