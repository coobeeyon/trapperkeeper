use assert_cmd::Command;
use serde_json::{json, Value};
use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command as Process;
use tempfile::TempDir;

struct Repo {
    _tmp: TempDir,
    root: PathBuf,
    wiki: PathBuf,
}
impl Repo {
    fn new() -> Self {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path().join("repo");
        let wiki = root.join("wiki");
        assert!(Process::new("git")
            .args(["init", "-q"])
            .arg(&root)
            .status()
            .unwrap()
            .success());
        fs::create_dir_all(wiki.join("pages")).unwrap();
        fs::write(
            root.join(".trapperkeeper.json"),
            json!({"version":1,"mode":"in-tree","path":"wiki"}).to_string(),
        )
        .unwrap();
        fs::write(wiki.join("toc.md"),"# Contents\n- [Release workflow](pages/releases.md) — releases and deployment\n- [Queue](pages/queue.md)\n").unwrap();
        fs::write(wiki.join("index.md"),"- **shipping / release authority / deploy-prod** — [Release](pages/releases.md#permission)\n- **queue / pending messages** — [Queue](pages/queue.md)\n").unwrap();
        fs::write(wiki.join("pages/releases.md"),"# Release workflow\nStatus: current, verified 2026-01-01\n## Permission\nShipping requires an explicit release decision.\nDo not infer deployment from service health.\n## History\nAn older release used a manual copy.\n").unwrap();
        fs::write(
            wiki.join("pages/queue.md"),
            "# Pending queue\n## Recovery\nMessages survive a process restart.\n",
        )
        .unwrap();
        Self {
            _tmp: tmp,
            root,
            wiki,
        }
    }
    fn cmd(&self) -> Command {
        let mut c = Command::cargo_bin("trk").unwrap();
        c.current_dir(&self.root)
            .env_remove("TRK_TELEMETRY_DIR")
            .env_remove("TRK_ACTOR")
            .env_remove("TRK_EPISODE")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null");
        c
    }
    fn find(&self, q: &str) -> Value {
        let out = self
            .cmd()
            .args(["find", q, "--json", "--no-telemetry"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
        serde_json::from_slice(&out.stdout).unwrap()
    }
    fn log(&self) -> PathBuf {
        self._tmp.path().join("private-events")
    }
}
fn logs(path: &Path) -> Vec<Value> {
    fs::read_dir(path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
        .flat_map(|p| {
            fs::read_to_string(p)
                .unwrap()
                .lines()
                .map(|l| serde_json::from_str(l).unwrap())
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn finds_aliases_sections_and_deduplicates_with_provenance() {
    let r = Repo::new();
    let v = r.find("release authority");
    assert_eq!(v["status"], "ok");
    assert_eq!(v["hits"].as_array().unwrap().len(), 1);
    let h = &v["hits"][0];
    assert_eq!(h["path"], "pages/releases.md");
    assert_eq!(h["heading"], "Permission");
    assert_eq!(h["anchor"], "permission");
    assert!(h["digest"].as_str().unwrap().len() == 64);
    assert!(h["context"].as_str().unwrap().contains("verified"));
    assert_eq!(
        r.find("deploy-prod")["hits"][0]["path"],
        "pages/releases.md"
    );
}
#[test]
fn ranking_is_stable_and_repetition_does_not_win() {
    let r = Repo::new();
    fs::write(
        r.wiki.join("pages/queue.md"),
        format!("# Queue\n{}", "shipping release authority\n".repeat(400)),
    )
    .unwrap();
    let a = r.find("shipping");
    let b = r.find("shipping");
    assert_eq!(a["hits"], b["hits"]);
    assert_eq!(a["hits"][0]["path"], "pages/releases.md");
}
#[test]
fn ranked_output_is_capped_in_both_formats() {
    let r = Repo::new();
    for name in ["releases", "queue"] {
        fs::write(
            r.wiki.join(format!("pages/{name}.md")),
            format!(
                "# {name}\n{}\n",
                "release messages evidence with enough surrounding context. ".repeat(12)
            ),
        )
        .unwrap();
    }
    for as_json in [false, true] {
        let mut cmd = r.cmd();
        cmd.args([
            "find",
            "release messages",
            "--max-chars",
            "1800",
            "--limit",
            "10",
        ]);
        if as_json {
            cmd.arg("--json");
        }
        let out = cmd.output().unwrap();
        assert!(out.status.success());
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(text.chars().count() <= 1800);
        if as_json {
            let v: Value = serde_json::from_str(&text).unwrap();
            assert_eq!(v["truncated"], true);
        }
    }
}
#[test]
fn literal_queries_stdin_and_validation_do_not_execute_shell() {
    let r = Repo::new();
    let out = r
        .cmd()
        .args(["find", "--query-stdin", "--json"])
        .write_stdin("shipping $(touch OWNED)")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(!r.root.join("OWNED").exists());
    for q in ["", "the and how", "shipping\u{1b}[31m"] {
        r.cmd().args(["find", q, "--json"]).assert().code(2);
    }
    r.cmd()
        .args(["find", "--query-stdin"])
        .write_stdin("x".repeat(4097))
        .assert()
        .code(2);
    r.cmd()
        .args(["find", "x", "--limit", "11"])
        .assert()
        .code(2);
    r.cmd()
        .args(["find", "x", "--max-chars", "20", "--json"])
        .assert()
        .code(2);
}
#[test]
fn no_match_partial_and_unavailable_are_distinct() {
    let r = Repo::new();
    let out = r
        .cmd()
        .args(["find", "zyxwvut", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["status"], "no_match");
    fs::remove_file(r.wiki.join("pages/queue.md")).unwrap();
    let v = r.find("zyxwvut");
    assert_eq!(v["status"], "partial");
    assert_eq!(v["hits"], json!([]));
    fs::remove_file(r.wiki.join("index.md")).unwrap();
    fs::remove_file(r.wiki.join("toc.md")).unwrap();
    r.cmd()
        .args(["find", "shipping", "--json"])
        .assert()
        .code(1);
    fs::remove_file(r.root.join(".trapperkeeper.json")).unwrap();
    r.cmd()
        .args(["find", "shipping", "--json"])
        .assert()
        .code(1);
}
#[test]
fn traversal_symlinks_hardlinks_and_devices_are_not_read() {
    let r = Repo::new();
    fs::write(r.root.join("outside.md"), "PRIVATE_SENTINEL").unwrap();
    symlink(r.root.join("outside.md"), r.wiki.join("link.md")).unwrap();
    symlink(&r.root, r.wiki.join("escape")).unwrap();
    fs::hard_link(r.root.join("outside.md"), r.wiki.join("hard.md")).unwrap();
    assert!(Process::new("mkfifo")
        .arg(r.wiki.join("pipe.md"))
        .status()
        .unwrap()
        .success());
    fs::write(r.wiki.join("index.md"),format!("[a](../outside.md)\n[b](%2e%2e/outside.md)\n[c](link.md)\n[d](escape/outside.md)\n[e](hard.md)\n[f]({})\n[g](pipe.md)\n",r.root.join("outside.md").display())).unwrap();
    let v = r.find("PRIVATE_SENTINEL");
    assert_eq!(v["status"], "partial");
    assert_eq!(v["hits"], json!([]));
    assert!(!v.to_string().contains("PRIVATE_SENTINEL"));
}

#[test]
fn passage_coverage_beats_broad_navigation_aliases() {
    let r = Repo::new();
    fs::write(r.wiki.join("index.md"), "- **queue recovery release shipping** — [Queue](pages/queue.md)\n- **queue recovery release shipping** — [Release](pages/releases.md)\n").unwrap();
    fs::write(r.wiki.join("pages/queue.md"), "# Queue\n## Unrelated\nShipping an unrelated parcel.\n## Recovery\nRelease shipping needs queue recovery.\n").unwrap();
    let v = r.find("release shipping queue recovery");
    assert_eq!(v["hits"][0]["path"], "pages/queue.md");
    assert_eq!(v["hits"][0]["heading"], "Recovery");
    assert_eq!(v["hits"][0]["passage_terms"].as_array().unwrap().len(), 4);
}

#[test]
fn giant_navigation_entries_are_bounded_and_reported() {
    let r = Repo::new();
    fs::write(
        r.wiki.join("index.md"),
        format!("{}\n", "[shipping](pages/releases.md) ".repeat(2000)),
    )
    .unwrap();
    let v = r.find("shipping");
    assert_eq!(v["status"], "partial");
    assert!(v["coverage"]["excluded"].as_u64().unwrap() > 0);
}

#[test]
fn snapshot_changes_and_unlogged_searches_leave_wiki_untouched() {
    let r = Repo::new();
    let before = fs::read(r.wiki.join("pages/releases.md")).unwrap();
    let a = r.find("shipping");
    assert_eq!(fs::read(r.wiki.join("pages/releases.md")).unwrap(), before);
    assert!(!r.log().exists());
    fs::write(
        r.wiki.join("pages/releases.md"),
        "# Shipping\nStatus: superseded\n",
    )
    .unwrap();
    let b = r.find("shipping");
    assert_ne!(
        a["snapshot"]["corpus_sha256"],
        b["snapshot"]["corpus_sha256"]
    );
    assert_ne!(a["hits"][0]["digest"], b["hits"][0]["digest"]);
    assert!(b["hits"][0]["context"]
        .as_str()
        .unwrap()
        .contains("superseded"));
}

#[test]
fn empty_report_does_not_create_directory_and_prime_teaches_feedback() {
    let r = Repo::new();
    r.cmd()
        .args(["telemetry", "--telemetry-dir"])
        .arg(r.log())
        .assert()
        .success();
    assert!(!r.log().exists());
    let out = r.cmd().arg("prime").output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("trk find"));
    assert!(text.contains("trk feedback"));
}
#[test]
fn oversized_binary_and_unlinked_pages_are_excluded() {
    let r = Repo::new();
    fs::write(
        r.wiki.join("pages/releases.md"),
        "x".repeat(1024 * 1024 + 1),
    )
    .unwrap();
    fs::write(r.wiki.join("pages/queue.md"), [0xff, 0xfe]).unwrap();
    fs::write(r.wiki.join("pages/unlinked.md"), "shipping").unwrap();
    let v = r.find("shipping");
    assert_eq!(v["status"], "partial");
    assert_eq!(v["coverage"]["pages_read"], 0);
    assert_eq!(v["hits"], json!([]));
}
#[test]
fn reference_links_spaces_unicode_and_controls_are_handled() {
    let r = Repo::new();
    fs::write(
        r.wiki.join("café notes.md"),
        "# Café notes\nCrème brûlée query.\n\u{1b}[31m literal terminal control\n",
    )
    .unwrap();
    fs::write(
        r.wiki.join("toc.md"),
        "[Café][ref]\n\n[ref]: <café notes.md>\n",
    )
    .unwrap();
    let v = r.find("crème");
    assert_eq!(v["hits"][0]["path"], "café notes.md");
    let out = r.cmd().args(["find", "crème"]).output().unwrap();
    assert!(!out.stdout.contains(&27));
}
#[test]
fn subdirectory_and_orphan_modes_use_same_discovery() {
    let r = Repo::new();
    let out = r
        .cmd()
        .current_dir(r.wiki.join("pages"))
        .args(["find", "shipping", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    fs::rename(&r.wiki, r.root.join(".trapper_keeper")).unwrap();
    fs::write(
        r.root.join(".trapperkeeper.json"),
        "{\"version\":1,\"mode\":\"orphan\"}",
    )
    .unwrap();
    let v = r.find("shipping");
    assert!(v["wiki_root"]
        .as_str()
        .unwrap()
        .ends_with(".trapper_keeper"));
}
#[test]
fn telemetry_records_success_miss_and_failure_without_raw_text() {
    let r = Repo::new();
    let dir = r.log();
    for q in ["shipping", "unknown-private-query"] {
        let _ = r
            .cmd()
            .args(["find", q, "--json", "--telemetry-dir"])
            .arg(&dir)
            .output()
            .unwrap();
    }
    fs::remove_file(r.root.join(".trapperkeeper.json")).unwrap();
    r.cmd()
        .args(["find", "shipping", "--json", "--telemetry-dir"])
        .arg(&dir)
        .assert()
        .code(1);
    let entries = logs(&dir);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]["status"], "ok");
    assert_eq!(entries[1]["status"], "no_match");
    assert_eq!(entries[2]["status"], "unavailable");
    for e in &entries {
        assert!(e.get("query").is_none());
        assert!(e.get("query_sha256").is_some());
        assert!(!e.to_string().contains("Shipping requires"));
    }
    assert_eq!(
        fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
        0o700
    );
    for f in fs::read_dir(&dir).unwrap() {
        assert_eq!(
            f.unwrap().metadata().unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
#[test]
fn explicit_query_logging_attribution_feedback_and_summary() {
    let r = Repo::new();
    let dir = r.log();
    r.cmd()
        .args(["find", "shipping", "--telemetry-dir"])
        .arg(&dir)
        .args(["--log-query", "--actor", "peer-a", "--episode", "episode-1"])
        .assert()
        .success();
    let entries = logs(&dir);
    assert_eq!(entries[0]["query"], "shipping");
    assert_eq!(entries[0]["attribution"]["actor"], "peer-a");
    let id = entries[0]["search_id"].as_str().unwrap();
    for outcome in ["irrelevant", "useful"] {
        r.cmd()
            .args([
                "feedback",
                id,
                "--outcome",
                outcome,
                "--rank",
                "1",
                "--telemetry-dir",
            ])
            .arg(&dir)
            .assert()
            .success();
    }
    r.cmd()
        .args([
            "feedback",
            id,
            "--outcome",
            "useful",
            "--rank",
            "10",
            "--telemetry-dir",
        ])
        .arg(&dir)
        .assert()
        .failure();
    let out = r
        .cmd()
        .args(["telemetry", "--telemetry-dir"])
        .arg(&dir)
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["searches"], 1);
    assert_eq!(v["searches_with_feedback"], 1);
    assert_eq!(v["latest_feedback"]["useful"], 1);
    assert!(v["latest_feedback"].get("irrelevant").is_none());
}
#[test]
fn telemetry_failure_is_visible_but_does_not_break_retrieval() {
    let r = Repo::new();
    let dir = r.log();
    fs::create_dir(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap();
    let out = r
        .cmd()
        .args(["find", "shipping", "--json", "--telemetry-dir"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["telemetry"], "failed");
    assert!(String::from_utf8_lossy(&out.stderr).contains("not recorded"));
}
#[test]
fn telemetry_opt_out_overrides_config_and_symlink_log_is_refused() {
    let r = Repo::new();
    let dir = r.log();
    r.cmd()
        .env("TRK_TELEMETRY_DIR", &dir)
        .args(["find", "shipping", "--no-telemetry"])
        .assert()
        .success();
    assert!(!dir.exists());
    fs::create_dir(&dir).unwrap();
    fs::set_permissions(&dir, fs::Permissions::from_mode(0o700)).unwrap();
    let day = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 86400;
    let outside = r.root.join("outside-log");
    fs::write(&outside, "preserve").unwrap();
    symlink(&outside, dir.join(format!("events-{day}.jsonl"))).unwrap();
    let out = r
        .cmd()
        .args(["find", "shipping", "--json", "--telemetry-dir"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["telemetry"], "failed");
    assert_eq!(fs::read_to_string(outside).unwrap(), "preserve");
}

#[test]
fn configured_wiki_cannot_escape_repository() {
    let r = Repo::new();
    let outside = r._tmp.path().join("outside-wiki");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("index.md"), "[Secret](secret.md)\n").unwrap();
    fs::write(outside.join("toc.md"), "").unwrap();
    fs::write(outside.join("secret.md"), "# PRIVATE_SENTINEL\n").unwrap();
    symlink(&outside, r.root.join("escaped")).unwrap();
    for path in [
        outside.to_string_lossy().as_ref(),
        "../outside-wiki",
        "escaped",
        "../repo/wiki",
    ] {
        fs::write(
            r.root.join(".trapperkeeper.json"),
            json!({"version":1,"mode":"in-tree","path":path}).to_string(),
        )
        .unwrap();
        let out = r
            .cmd()
            .args(["find", "PRIVATE_SENTINEL", "--json", "--no-telemetry"])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "path: {path}");
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(v["status"], "unavailable");
        assert!(!v.to_string().contains("PRIVATE_SENTINEL"));
    }
    fs::write(
        r.root.join(".trapperkeeper.json"),
        json!({"version":1,"mode":"orphan"}).to_string(),
    )
    .unwrap();
    symlink(&outside, r.root.join(".trapper_keeper")).unwrap();
    r.cmd().args(["find", "secret", "--json"]).assert().code(1);
    symlink(&r.wiki, r.root.join("inside")).unwrap();
    fs::write(
        r.root.join(".trapperkeeper.json"),
        json!({"version":1,"mode":"in-tree","path":"inside"}).to_string(),
    )
    .unwrap();
    assert_eq!(r.find("shipping")["status"], "ok");
}

#[test]
fn telemetry_never_creates_logs_inside_a_repository() {
    let r = Repo::new();
    let other = r._tmp.path().join("other-repo");
    assert!(Process::new("git")
        .args(["init", "-q"])
        .arg(&other)
        .status()
        .unwrap()
        .success());
    let bare = r._tmp.path().join("bare-repo");
    assert!(Process::new("git")
        .args(["init", "--bare", "-q"])
        .arg(&bare)
        .status()
        .unwrap()
        .success());
    let worktree = r._tmp.path().join("linked-worktree");
    fs::create_dir(&worktree).unwrap();
    fs::write(worktree.join(".git"), "gitdir: elsewhere\n").unwrap();
    for parent in [
        &r.root,
        &r.wiki,
        &r.root.join(".git"),
        &other,
        &bare,
        &worktree,
    ] {
        let dir = parent.join("private-events");
        let out = r
            .cmd()
            .args(["find", "shipping", "--json", "--telemetry-dir"])
            .arg(&dir)
            .output()
            .unwrap();
        assert!(out.status.success());
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(v["telemetry"], "failed", "{}", dir.display());
        assert!(!dir.exists());
        r.cmd()
            .args(["telemetry", "--telemetry-dir"])
            .arg(&dir)
            .assert()
            .failure();
    }
}

#[test]
fn telemetry_uninspectable_paths_are_errors_not_empty_reports() {
    let r = Repo::new();
    let dangling = r._tmp.path().join("dangling");
    symlink(r._tmp.path().join("does-not-exist"), &dangling).unwrap();
    let linked = r._tmp.path().join("linked-parent");
    symlink(&r.root, &linked).unwrap();
    for path in [
        dangling,
        linked.join("events"),
        PathBuf::from("relative-missing"),
    ] {
        r.cmd()
            .args(["telemetry", "--telemetry-dir"])
            .arg(&path)
            .assert()
            .failure();
        let out = r
            .cmd()
            .args(["find", "shipping", "--json", "--telemetry-dir"])
            .arg(&path)
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(v["telemetry"], "failed");
    }
    let private = r.log();
    fs::create_dir(&private).unwrap();
    fs::set_permissions(&private, fs::Permissions::from_mode(0o0)).unwrap();
    let out = r
        .cmd()
        .args(["telemetry", "--telemetry-dir"])
        .arg(private.join("events"))
        .output()
        .unwrap();
    fs::set_permissions(&private, fs::Permissions::from_mode(0o700)).unwrap();
    if Process::new("id").arg("-u").output().unwrap().stdout != b"0\n" {
        assert!(!out.status.success());
    }
}

#[test]
fn empty_git_folder_in_parent_is_not_a_repository() {
    let r = Repo::new();
    fs::create_dir(r._tmp.path().join(".git")).unwrap();
    let out = r
        .cmd()
        .args(["find", "shipping", "--json", "--telemetry-dir"])
        .arg(r.log())
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["telemetry"], "enabled");
    r.cmd()
        .args(["telemetry", "--telemetry-dir"])
        .arg(r.log())
        .assert()
        .success();
    assert_eq!(logs(&r.log()).len(), 1);
}
