use crate::git;

const MAX_LOG_LINES: usize = 20;

pub fn run() -> Result<(), String> {
    // Silent exit if trapperkeeper is not initialized
    if !git::branch_exists() {
        return Ok(());
    }

    let branch = git::BRANCH;

    // Read toc.md
    let toc = git::git(&["show", &format!("{branch}:toc.md")])
        .unwrap_or_default();

    // Read recent log entries (last N lines of log.md)
    let log_full = git::git(&["show", &format!("{branch}:log.md")])
        .unwrap_or_default();
    let log_lines: Vec<&str> = log_full.lines().collect();
    let log_recent = if log_lines.len() > MAX_LOG_LINES {
        log_lines[log_lines.len() - MAX_LOG_LINES..].join("\n")
    } else {
        log_full.clone()
    };

    // Output context for Claude Code injection
    println!("# Trapperkeeper Wiki");
    println!();
    println!("## Table of Contents");
    println!();
    println!("{toc}");
    println!();
    println!("## Recent Activity");
    println!();
    println!("{log_recent}");
    println!();
    println!("## CLI Reference");
    println!();
    println!("- `trk init` — create orphan branch with skeleton wiki files");
    println!("- `trk setup claude` — wire hooks into .claude/settings.local.json");
    println!("- `trk prime` — output wiki context (this output)");

    Ok(())
}
