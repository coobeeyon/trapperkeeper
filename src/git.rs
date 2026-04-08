use std::process::Command;

pub const BRANCH: &str = "trapperkeeper";

/// Run a git command and return stdout, or an error with stderr.
pub fn git(args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run git: {e}"))?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Check whether the trapperkeeper branch exists.
pub fn branch_exists() -> bool {
    git(&["rev-parse", "--verify", &format!("refs/heads/{BRANCH}")])
        .is_ok()
}

/// Write a blob and return its hash.
pub fn hash_blob(content: &str) -> Result<String, String> {
    use std::io::Write;
    let mut child = Command::new("git")
        .args(["hash-object", "-w", "--stdin"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to run git hash-object: {e}"))?;

    child
        .stdin
        .take()
        .unwrap()
        .write_all(content.as_bytes())
        .map_err(|e| format!("failed to write blob: {e}"))?;

    let out = child
        .wait_with_output()
        .map_err(|e| format!("git hash-object failed: {e}"))?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Create a tree from a list of (mode, type, hash, name) entries.
pub fn mktree(entries: &[(& str, &str, &str, &str)]) -> Result<String, String> {
    use std::io::Write;
    let mut child = Command::new("git")
        .args(["mktree"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to run git mktree: {e}"))?;

    {
        let stdin = child.stdin.take().unwrap();
        let mut writer = std::io::BufWriter::new(stdin);
        for (mode, kind, hash, name) in entries {
            writeln!(writer, "{mode} {kind} {hash}\t{name}")
                .map_err(|e| format!("failed to write tree entry: {e}"))?;
        }
    }

    let out = child
        .wait_with_output()
        .map_err(|e| format!("git mktree failed: {e}"))?;

    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// Create a commit with no parent.
pub fn commit_tree(tree: &str, message: &str) -> Result<String, String> {
    git(&["commit-tree", tree, "-m", message])
}

/// Update a branch ref.
pub fn update_ref(branch: &str, commit: &str) -> Result<(), String> {
    git(&["update-ref", &format!("refs/heads/{branch}"), commit])?;
    Ok(())
}
