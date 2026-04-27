use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

const CONFIG_PATH: &str = ".codex/config.toml";
const HOOKS_PATH: &str = ".codex/hooks.json";
const RULES_PATH: &str = ".codex/rules/default.rules";
const COMMAND: &str = "trk prime";
const SESSION_MATCHER: &str = "startup|resume|clear";
const STATUS_MESSAGE: &str = "Loading Trapperkeeper wiki context";
const RULE: &str = r#"prefix_rule(pattern=["trk"], decision="allow")"#;

pub fn run() -> Result<(), String> {
    ensure_codex_hooks_feature()?;
    ensure_hooks_json()?;
    ensure_rules()?;

    eprintln!("Configured Codex hooks and rules in .codex/");
    Ok(())
}

fn ensure_codex_hooks_feature() -> Result<(), String> {
    let path = PathBuf::from(CONFIG_PATH);
    let content = if path.exists() {
        fs::read_to_string(&path).map_err(|e| format!("failed to read {CONFIG_PATH}: {e}"))?
    } else {
        String::new()
    };

    let output = with_codex_hooks_enabled(&content);
    write_file(&path, &output, ".codex/", CONFIG_PATH)
}

fn ensure_hooks_json() -> Result<(), String> {
    let path = PathBuf::from(HOOKS_PATH);

    let mut settings: Value = if path.exists() {
        let content =
            fs::read_to_string(&path).map_err(|e| format!("failed to read {HOOKS_PATH}: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("failed to parse {HOOKS_PATH}: {e}"))?
    } else {
        json!({})
    };

    if settings.get("hooks").is_none() {
        settings["hooks"] = json!({});
    }

    ensure_session_hook(&mut settings, COMMAND)?;

    let output = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("failed to serialize hooks: {e}"))?;

    write_file(&path, &(output + "\n"), ".codex/", HOOKS_PATH)
}

fn ensure_rules() -> Result<(), String> {
    let path = PathBuf::from(RULES_PATH);
    let content = if path.exists() {
        fs::read_to_string(&path).map_err(|e| format!("failed to read {RULES_PATH}: {e}"))?
    } else {
        String::new()
    };

    let output = with_rule(&content, RULE);
    write_file(&path, &output, ".codex/rules/", RULES_PATH)
}

fn write_file(
    path: &PathBuf,
    content: &str,
    dir_name: &str,
    display_path: &str,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create {dir_name}: {e}"))?;
    }

    fs::write(path, content).map_err(|e| format!("failed to write {display_path}: {e}"))
}

fn with_codex_hooks_enabled(content: &str) -> String {
    let mut lines: Vec<String> = content.lines().map(str::to_owned).collect();

    let Some(features_start) = lines.iter().position(|line| is_table(line, "features")) else {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push("[features]".to_string());
        lines.push("codex_hooks = true".to_string());
        return finish_lines(lines);
    };

    let features_end = lines
        .iter()
        .enumerate()
        .skip(features_start + 1)
        .find_map(|(idx, line)| is_any_table(line).then_some(idx))
        .unwrap_or(lines.len());

    if let Some(idx) = (features_start + 1..features_end)
        .find(|idx| is_assignment_to_key(&lines[*idx], "codex_hooks"))
    {
        let indent_len = lines[idx]
            .char_indices()
            .find_map(|(i, ch)| (!ch.is_whitespace()).then_some(i))
            .unwrap_or(0);
        let indent = &lines[idx][..indent_len];
        lines[idx] = format!("{indent}codex_hooks = true");
    } else {
        lines.insert(features_start + 1, "codex_hooks = true".to_string());
    }

    finish_lines(lines)
}

fn with_rule(content: &str, rule: &str) -> String {
    if content.lines().any(|line| line.trim() == rule) {
        return ensure_trailing_newline(content.to_string());
    }

    let mut output = ensure_trailing_newline(content.to_string());
    output.push_str(rule);
    output.push('\n');
    output
}

fn ensure_session_hook(settings: &mut Value, command: &str) -> Result<(), String> {
    let hooks = settings["hooks"]
        .as_object_mut()
        .ok_or("hooks is not an object")?;

    let entries = hooks.entry("SessionStart").or_insert_with(|| json!([]));
    let binding = vec![];
    let arr = entries.as_array().unwrap_or(&binding);

    let already_present = arr.iter().any(|entry| {
        entry["hooks"]
            .as_array()
            .map(|hooks| {
                hooks
                    .iter()
                    .any(|hook| hook["command"].as_str() == Some(command))
            })
            .unwrap_or(false)
    });

    if !already_present {
        let arr = entries
            .as_array_mut()
            .ok_or("hooks.SessionStart is not an array")?;
        arr.push(json!({
            "matcher": SESSION_MATCHER,
            "hooks": [{
                "type": "command",
                "command": command,
                "statusMessage": STATUS_MESSAGE
            }]
        }));
    }

    Ok(())
}

fn is_table(line: &str, name: &str) -> bool {
    line.trim() == format!("[{name}]")
}

fn is_any_table(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('[') && trimmed.ends_with(']') && !trimmed.starts_with('#')
}

fn is_assignment_to_key(line: &str, key: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') || !trimmed.starts_with(key) {
        return false;
    }

    trimmed[key.len()..].trim_start().starts_with('=')
}

fn finish_lines(lines: Vec<String>) -> String {
    ensure_trailing_newline(lines.join("\n"))
}

fn ensure_trailing_newline(mut content: String) -> String {
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_features_table_when_missing() {
        let actual = with_codex_hooks_enabled("model = \"gpt-5.5\"\n");

        assert_eq!(
            actual,
            "model = \"gpt-5.5\"\n\n[features]\ncodex_hooks = true\n"
        );
    }

    #[test]
    fn adds_codex_hooks_to_existing_features_table() {
        let actual = with_codex_hooks_enabled("[features]\nfast_mode = true\n[tools]\n");

        assert_eq!(
            actual,
            "[features]\ncodex_hooks = true\nfast_mode = true\n[tools]\n"
        );
    }

    #[test]
    fn enables_existing_codex_hooks_feature() {
        let actual = with_codex_hooks_enabled("[features]\n  codex_hooks = false\n");

        assert_eq!(actual, "[features]\n  codex_hooks = true\n");
    }

    #[test]
    fn appends_rule_once() {
        let once = with_rule("", RULE);
        let twice = with_rule(&once, RULE);

        assert_eq!(once, RULE.to_string() + "\n");
        assert_eq!(twice, once);
    }

    #[test]
    fn adds_session_hook_without_replacing_existing_hooks() {
        let mut settings = json!({
            "hooks": {
                "SessionStart": [{
                    "matcher": "startup",
                    "hooks": [{
                        "type": "command",
                        "command": "other prime"
                    }]
                }]
            }
        });

        ensure_session_hook(&mut settings, COMMAND).unwrap();
        ensure_session_hook(&mut settings, COMMAND).unwrap();

        let entries = settings["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1]["matcher"], SESSION_MATCHER);
        assert_eq!(entries[1]["hooks"][0]["command"], COMMAND);
    }

    #[test]
    fn errors_when_existing_session_hook_is_not_an_array() {
        let mut settings = json!({
            "hooks": {
                "SessionStart": {}
            }
        });

        let err = ensure_session_hook(&mut settings, COMMAND).unwrap_err();

        assert_eq!(err, "hooks.SessionStart is not an array");
    }
}
