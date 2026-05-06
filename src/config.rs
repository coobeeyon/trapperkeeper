use crate::git;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub const FILE: &str = ".trapperkeeper.json";
pub const VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Orphan,
    InTree { path: PathBuf },
}

#[derive(Debug, Clone)]
pub struct Config {
    pub version: u32,
    pub mode: Mode,
}

impl Config {
    pub fn orphan() -> Self {
        Self {
            version: VERSION,
            mode: Mode::Orphan,
        }
    }

    pub fn in_tree(path: PathBuf) -> Self {
        Self {
            version: VERSION,
            mode: Mode::InTree { path },
        }
    }

    /// Directory (relative to repo root) where the wiki files live.
    pub fn wiki_path(&self) -> &Path {
        match &self.mode {
            Mode::Orphan => Path::new(".trapper_keeper"),
            Mode::InTree { path } => path.as_path(),
        }
    }
}

fn repo_root() -> Result<PathBuf, String> {
    let out = git::git(&["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(out))
}

fn config_path() -> Result<PathBuf, String> {
    Ok(repo_root()?.join(FILE))
}

/// Load config. Returns None if trapperkeeper is not initialized in this
/// repo. Falls back to legacy-orphan detection (branch exists without a
/// config file) for backward compatibility.
pub fn load() -> Result<Option<Config>, String> {
    let path = config_path()?;
    if path.exists() {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let value: Value = serde_json::from_str(&content)
            .map_err(|e| format!("failed to parse {}: {e}", path.display()))?;
        return Ok(Some(parse(&value)?));
    }
    if git::branch_exists() {
        return Ok(Some(Config::orphan()));
    }
    Ok(None)
}

fn parse(value: &Value) -> Result<Config, String> {
    let version = value
        .get("version")
        .and_then(Value::as_u64)
        .ok_or("config: missing or non-numeric 'version'")? as u32;
    let mode_str = value
        .get("mode")
        .and_then(Value::as_str)
        .ok_or("config: missing 'mode'")?;
    let mode = match mode_str {
        "orphan" => Mode::Orphan,
        "in-tree" => {
            let path = value
                .get("path")
                .and_then(Value::as_str)
                .ok_or("config: in-tree mode requires 'path'")?;
            Mode::InTree {
                path: PathBuf::from(path),
            }
        }
        other => return Err(format!("config: unknown mode '{other}'")),
    };
    Ok(Config { version, mode })
}

pub fn save(cfg: &Config) -> Result<(), String> {
    let path = config_path()?;
    let value = match &cfg.mode {
        Mode::Orphan => json!({ "version": cfg.version, "mode": "orphan" }),
        Mode::InTree { path } => json!({
            "version": cfg.version,
            "mode": "in-tree",
            "path": path.to_string_lossy(),
        }),
    };
    let text = serde_json::to_string_pretty(&value)
        .map_err(|e| format!("failed to serialize config: {e}"))?;
    fs::write(&path, text + "\n")
        .map_err(|e| format!("failed to write {}: {e}", path.display()))?;
    Ok(())
}

pub fn exists() -> Result<bool, String> {
    Ok(config_path()?.exists())
}
