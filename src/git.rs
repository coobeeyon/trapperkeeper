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

/// Read the current tree hash for the branch tip.
pub fn read_tree(branch: &str) -> Result<String, String> {
    git(&["rev-parse", &format!("{branch}^{{tree}}")])
}

/// List entries in a tree as (mode, type, hash, name) tuples.
pub fn ls_tree(tree: &str) -> Result<Vec<(String, String, String, String)>, String> {
    let output = git(&["ls-tree", tree])?;
    let mut entries = Vec::new();
    for line in output.lines() {
        // format: "mode type hash\tname"
        let (meta, name) = line.split_once('\t')
            .ok_or_else(|| format!("bad ls-tree line: {line}"))?;
        let parts: Vec<&str> = meta.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(format!("bad ls-tree meta: {meta}"));
        }
        entries.push((
            parts[0].to_string(),
            parts[1].to_string(),
            parts[2].to_string(),
            name.to_string(),
        ));
    }
    Ok(entries)
}

/// Create a commit with no parent.
pub fn commit_tree(tree: &str, message: &str) -> Result<String, String> {
    git(&["commit-tree", tree, "-m", message])
}

/// Create a commit with a parent.
pub fn commit_tree_with_parent(tree: &str, parent: &str, message: &str) -> Result<String, String> {
    git(&["commit-tree", tree, "-p", parent, "-m", message])
}

/// Get the current tip commit of a branch.
pub fn branch_tip(branch: &str) -> Result<String, String> {
    git(&["rev-parse", &format!("refs/heads/{branch}")])
}

/// Write a file at the given path into the branch tree, returning the new root tree hash.
/// Handles nested paths (e.g. "pages/architecture.md") by rebuilding subtrees.
pub fn write_file_to_tree(root_tree: &str, path: &str, content: &str) -> Result<String, String> {
    let blob = hash_blob(content)?;
    let parts: Vec<&str> = path.split('/').collect();
    insert_into_tree(root_tree, &parts, &blob)
}

fn insert_into_tree(tree_hash: &str, path: &[&str], blob_hash: &str) -> Result<String, String> {
    let mut entries = ls_tree(tree_hash)?;

    if path.len() == 1 {
        // Replace or add the file entry
        let name = path[0];
        let mut found = false;
        for entry in &mut entries {
            if entry.3 == name {
                entry.2 = blob_hash.to_string();
                entry.1 = "blob".to_string();
                entry.0 = "100644".to_string();
                found = true;
                break;
            }
        }
        if !found {
            entries.push(("100644".to_string(), "blob".to_string(), blob_hash.to_string(), name.to_string()));
        }
    } else {
        // Recurse into subtree
        let dir = path[0];
        let mut found = false;
        for entry in &mut entries {
            if entry.3 == dir && entry.1 == "tree" {
                let new_subtree = insert_into_tree(&entry.2, &path[1..], blob_hash)?;
                entry.2 = new_subtree;
                found = true;
                break;
            }
        }
        if !found {
            // Create empty tree and recurse
            let empty_tree = mktree(&[])?;
            let new_subtree = insert_into_tree(&empty_tree, &path[1..], blob_hash)?;
            entries.push(("040000".to_string(), "tree".to_string(), new_subtree, dir.to_string()));
        }
    }

    let refs: Vec<(&str, &str, &str, &str)> = entries
        .iter()
        .map(|(a, b, c, d)| (a.as_str(), b.as_str(), c.as_str(), d.as_str()))
        .collect();
    mktree(&refs)
}

/// Update a branch ref.
pub fn update_ref(branch: &str, commit: &str) -> Result<(), String> {
    git(&["update-ref", &format!("refs/heads/{branch}"), commit])?;
    Ok(())
}
