use crate::git;

const MAX_LOG_LINES: usize = 20;

/// Strip a leading "# Title\n" line from content to avoid duplication
/// when we wrap it with our own header.
fn strip_title(content: &str) -> String {
    let trimmed = content.trim_start();
    if let Some(rest) = trimmed.strip_prefix("# ") {
        // Skip the first line (the title)
        rest.split_once('\n')
            .map(|(_, body)| body.trim_start_matches('\n').to_string())
            .unwrap_or_default()
    } else {
        content.to_string()
    }
}

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

    // Strip leading "# Title\n" from toc/log since we add our own headers
    let toc_body = strip_title(&toc);
    let log_body = strip_title(&log_recent);

    // Output context for Claude Code injection
    println!("# Trapperkeeper Wiki");
    println!();
    println!("## Table of Contents");
    println!();
    println!("{toc_body}");
    println!();
    println!("## Recent Activity");
    println!();
    println!("{log_body}");
    println!();
    println!("## CLI Reference");
    println!();
    println!("- `trk init` — create orphan branch with skeleton wiki files");
    println!("- `trk setup claude` — wire hooks into .claude/settings.local.json");
    println!("- `trk prime` — output wiki context (this output)");
    println!("- `trk write <path> [-m <message>]` — write a file to the wiki (content from stdin)");

    Ok(())
}
