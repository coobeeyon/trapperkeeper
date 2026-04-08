use std::io::Read;
use crate::git;

pub fn run(path: &str, message: Option<&str>) -> Result<(), String> {
    if !git::branch_exists() {
        return Err("trapperkeeper not initialized (run `trk init` first)".to_string());
    }

    // Read content from stdin
    let mut content = String::new();
    std::io::stdin()
        .read_to_string(&mut content)
        .map_err(|e| format!("failed to read stdin: {e}"))?;

    let branch = git::BRANCH;
    let parent = git::branch_tip(branch)?;
    let root_tree = git::read_tree(branch)?;

    // Write the file into the tree
    let new_tree = git::write_file_to_tree(&root_tree, path, &content)?;

    // Commit with parent
    let default_msg = format!("Update {path}");
    let msg = message.unwrap_or(&default_msg);
    let commit = git::commit_tree_with_parent(&new_tree, &parent, msg)?;
    git::update_ref(branch, &commit)?;

    eprintln!("wrote {path}");
    Ok(())
}
