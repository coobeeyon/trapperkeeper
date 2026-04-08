use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

const SETTINGS_PATH: &str = ".claude/settings.local.json";
const PERMISSION: &str = "Bash(trk:*)";

fn hook_entry(command: &str) -> Value {
    json!([{
        "matcher": "*",
        "hooks": [{
            "type": "command",
            "command": command
        }]
    }])
}

pub fn run() -> Result<(), String> {
    let path = PathBuf::from(SETTINGS_PATH);

    // Read existing settings or start fresh
    let mut settings: Value = if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {SETTINGS_PATH}: {e}"))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("failed to parse {SETTINGS_PATH}: {e}"))?
    } else {
        json!({})
    };

    // Ensure hooks object exists
    if settings.get("hooks").is_none() {
        settings["hooks"] = json!({});
    }

    // Add SessionStart hook if not already present
    ensure_hook(&mut settings, "SessionStart", "trk prime");

    // Add PreCompact hook if not already present
    ensure_hook(&mut settings, "PreCompact", "trk prime");

    // Ensure permissions.allow exists and contains our permission
    if settings.get("permissions").is_none() {
        settings["permissions"] = json!({});
    }
    if settings["permissions"].get("allow").is_none() {
        settings["permissions"]["allow"] = json!([]);
    }

    let allow = settings["permissions"]["allow"]
        .as_array_mut()
        .ok_or("permissions.allow is not an array")?;

    if !allow.iter().any(|v| v.as_str() == Some(PERMISSION)) {
        allow.push(json!(PERMISSION));
    }

    // Write back
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create .claude/: {e}"))?;
    }

    let output = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("failed to serialize settings: {e}"))?;

    fs::write(&path, output + "\n")
        .map_err(|e| format!("failed to write {SETTINGS_PATH}: {e}"))?;

    eprintln!("Configured Claude Code hooks and permissions in {SETTINGS_PATH}");
    Ok(())
}

/// Ensure a hook entry exists for the given event with the given command.
fn ensure_hook(settings: &mut Value, event: &str, command: &str) {
    let hooks = settings["hooks"]
        .as_object_mut()
        .expect("hooks is an object");

    let entries = hooks
        .entry(event)
        .or_insert_with(|| json!([]));

    let binding = vec![];
    let arr = entries.as_array().unwrap_or(&binding);

    // Check if any existing entry already runs this command
    let already_present = arr.iter().any(|entry| {
        entry["hooks"]
            .as_array()
            .map(|h| {
                h.iter()
                    .any(|hook| hook["command"].as_str() == Some(command))
            })
            .unwrap_or(false)
    });

    if !already_present {
        let arr = entries.as_array_mut().expect("hook event is an array");
        arr.push(json!({
            "matcher": "*",
            "hooks": [{
                "type": "command",
                "command": command
            }]
        }));
    }
}
