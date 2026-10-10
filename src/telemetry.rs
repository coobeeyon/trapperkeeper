//! Optional local receipts. No network, raw excerpts, or implicit query logging.
use clap::{Args, ValueEnum};
use rustix::fs::{flock, mkdirat, open, openat, statat, AtFlags, FlockOperation, Mode, OFlags};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Args, Clone, Default)]
pub struct Options {
    /// Private local event directory (or TRK_TELEMETRY_DIR / git trk.telemetryDir)
    #[arg(long)]
    pub telemetry_dir: Option<PathBuf>,
    /// Disable telemetry even when configured
    #[arg(long)]
    pub no_telemetry: bool,
    /// Include literal queries in the private log (never excerpts)
    #[arg(long)]
    pub log_query: bool,
    /// Stable peer/agent identity for attribution
    #[arg(long)]
    pub actor: Option<String>,
    /// Caller-supplied episode/session identifier
    #[arg(long)]
    pub episode: Option<String>,
}

pub fn millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
pub fn id() -> String {
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    crate::retrieval::corpus::digest(format!("{}:{ns}", std::process::id()).as_bytes())[..24]
        .to_string()
}
fn config(key: &str) -> Option<String> {
    crate::git::git(&["config", "--get", key])
        .ok()
        .filter(|v| !v.is_empty())
}
impl Options {
    pub fn directory(&self) -> Option<PathBuf> {
        if self.no_telemetry {
            None
        } else {
            self.telemetry_dir
                .clone()
                .or_else(|| std::env::var_os("TRK_TELEMETRY_DIR").map(PathBuf::from))
                .or_else(|| config("trk.telemetryDir").map(PathBuf::from))
        }
    }
    pub fn query_enabled(&self) -> bool {
        self.log_query || config("trk.logQueries").as_deref() == Some("true")
    }
    pub fn attribution(&self) -> Value {
        let actor = self
            .actor
            .clone()
            .or_else(|| std::env::var("TRK_ACTOR").ok())
            .or_else(|| std::env::var("MYBUDDY_PEER_ID").ok());
        let episode = self
            .episode
            .clone()
            .or_else(|| std::env::var("TRK_EPISODE").ok())
            .or_else(|| std::env::var("CODEX_SESSION_ID").ok())
            .or_else(|| std::env::var("CLAUDE_SESSION_ID").ok());
        // An opaque origin hint, not a verified peer identity. Never read the bootstrap.
        let origin = std::env::var_os("MYBUDDY_BOOTSTRAP_FILE").and_then(|s| {
            PathBuf::from(s)
                .file_name()
                .map(|s| s.to_string_lossy().chars().take(200).collect::<String>())
        });
        json!({"actor":actor.map(|s|s.chars().take(200).collect::<String>()),"episode":episode.map(|s|s.chars().take(200).collect::<String>()),"origin_hint":origin})
    }
}

fn entry_exists(dir: &File, name: &str) -> Result<bool, String> {
    match statat(dir, name, AtFlags::SYMLINK_NOFOLLOW) {
        Ok(_) => Ok(true),
        Err(rustix::io::Errno::NOENT) => Ok(false),
        Err(e) => Err(format!("cannot inspect telemetry location: {e}")),
    }
}

fn outside_git(dir: &File) -> Result<(), String> {
    if entry_exists(dir, ".git")?
        || (entry_exists(dir, "HEAD")?
            && entry_exists(dir, "objects")?
            && entry_exists(dir, "refs")?)
    {
        return Err("telemetry directory must be outside Git worktrees and repositories".into());
    }
    Ok(())
}

// Walk every component using descriptors: a symlink (including a dangling one)
// is an error, never an empty report or an indirect write into a repository.
fn directory(path: &Path, create: bool) -> Result<Option<File>, String> {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, Component::RootDir | Component::Normal(_)))
    {
        return Err("telemetry directory must be absolute without traversal".into());
    }
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW;
    let mut file: File = open("/", flags, Mode::empty())
        .map_err(|e| e.to_string())?
        .into();
    outside_git(&file)?;
    for part in path.components() {
        let Component::Normal(name) = part else {
            continue;
        };
        let fd = match openat(&file, name, flags, Mode::empty()) {
            Ok(fd) => fd,
            Err(rustix::io::Errno::NOENT) if create => {
                match mkdirat(&file, name, Mode::RUSR | Mode::WUSR | Mode::XUSR) {
                    Ok(()) | Err(rustix::io::Errno::EXIST) => (),
                    Err(e) => return Err(e.to_string()),
                }
                openat(&file, name, flags, Mode::empty()).map_err(|e| e.to_string())?
            }
            Err(rustix::io::Errno::NOENT) => return Ok(None),
            Err(e) => return Err(format!("cannot safely open telemetry directory: {e}")),
        };
        file = fd.into();
        outside_git(&file)?;
    }
    if file
        .metadata()
        .map_err(|e| e.to_string())?
        .permissions()
        .mode()
        & 0o077
        != 0
    {
        return Err("telemetry directory must be owner-only (0700)".into());
    }
    Ok(Some(file))
}

pub fn append(path: &Path, event: &Value) -> Result<(), String> {
    let dir = directory(path, true)?.ok_or("telemetry directory unavailable")?;
    let name = format!("events-{}.jsonl", millis() / 86_400_000);
    let fd = openat(
        &dir,
        name.as_str(),
        OFlags::WRONLY | OFlags::CREATE | OFlags::APPEND | OFlags::NOFOLLOW | OFlags::NONBLOCK,
        Mode::RUSR | Mode::WUSR,
    )
    .map_err(|e| e.to_string())?;
    let mut file: File = fd.into();
    let meta = file.metadata().map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.nlink() != 1 || meta.permissions().mode() & 0o077 != 0 {
        return Err("unsafe telemetry file type, links or permissions".into());
    }
    flock(&file, FlockOperation::NonBlockingLockExclusive)
        .map_err(|_| "telemetry log busy; receipt not saved".to_string())?;
    let line = serde_json::to_string(event).map_err(|e| e.to_string())? + "\n";
    if file.metadata().map_err(|e| e.to_string())?.len() + line.len() as u64 > 16 * 1024 * 1024 {
        return Err("daily telemetry file reached 16 MiB; receipt not saved".into());
    }
    file.write_all(line.as_bytes())
        .and_then(|_| file.sync_data())
        .map_err(|e| e.to_string())
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Outcome {
    Useful,
    Irrelevant,
    Missing,
    Stale,
    Incorrect,
}

#[derive(Args)]
pub struct Feedback {
    pub search_id: String,
    #[arg(long, value_enum)]
    pub outcome: Outcome,
    /// Returned rank being assessed, if applicable
    #[arg(long,value_parser=clap::value_parser!(u8).range(1..=10))]
    pub rank: Option<u8>,
    #[command(flatten)]
    pub telemetry: Options,
}

fn read_events(path: &Path, days: u16) -> Result<(Vec<Value>, usize), String> {
    let Some(dir) = directory(path, false)? else {
        return Ok((vec![], 0));
    };
    let today = (millis() / 86_400_000) as u64;
    let mut events = vec![];
    let mut malformed = 0;
    for day in today.saturating_sub(u64::from(days) - 1)..=today {
        let name = format!("events-{day}.jsonl");
        let fd = match openat(
            &dir,
            name.as_str(),
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK,
            Mode::empty(),
        ) {
            Ok(fd) => fd,
            Err(rustix::io::Errno::NOENT) => continue,
            Err(_) => return Err("cannot safely open telemetry file".into()),
        };
        let file: File = fd.into();
        let meta = file.metadata().map_err(|e| e.to_string())?;
        if !meta.is_file()
            || meta.nlink() != 1
            || meta.permissions().mode() & 0o077 != 0
            || meta.len() > 16 * 1024 * 1024
        {
            return Err("unsafe or oversized telemetry file".into());
        }
        flock(&file, FlockOperation::NonBlockingLockShared)
            .map_err(|_| "telemetry log busy".to_string())?;
        for line in BufReader::new(file).lines() {
            let line = line.map_err(|e| e.to_string())?;
            match serde_json::from_str::<Value>(&line) {
                Ok(v) if v["schema_version"] == 1 => events.push(v),
                _ => malformed += 1,
            }
        }
    }
    Ok((events, malformed))
}

pub fn feedback(args: Feedback) -> Result<(), String> {
    let path = args
        .telemetry
        .directory()
        .ok_or("configure a telemetry directory first")?;
    if args.search_id.len() != 24 || !args.search_id.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid search ID".into());
    }
    let (events, _) = read_events(&path, 30)?;
    let event = events
        .iter()
        .find(|v| v["kind"] == "search" && v["search_id"] == args.search_id)
        .ok_or("search ID not found in the last 30 days")?;
    if let Some(rank) = args.rank {
        if rank as usize > event["hits"].as_array().map_or(0, Vec::len) {
            return Err("rank was not returned by this search".into());
        }
    }
    append(
        &path,
        &json!({"schema_version":1,"kind":"feedback","timestamp_ms":millis(),"search_id":args.search_id,"outcome":format!("{:?}",args.outcome).to_lowercase(),"rank":args.rank,"attribution":args.telemetry.attribution()}),
    )?;
    println!(
        "Feedback recorded for {}. This is a reported judgment, not independent correctness proof.",
        args.search_id
    );
    Ok(())
}

#[derive(Args)]
pub struct Report {
    #[arg(long,default_value_t=7,value_parser=clap::value_parser!(u16).range(1..=30))]
    pub days: u16,
    #[command(flatten)]
    pub telemetry: Options,
}
pub fn report(args: Report) -> Result<(), String> {
    let path = args
        .telemetry
        .directory()
        .ok_or("configure a telemetry directory first")?;
    let (events, malformed) = read_events(&path, args.days)?;
    let searches: Vec<_> = events.iter().filter(|v| v["kind"] == "search").collect();
    let known: BTreeSet<_> = searches
        .iter()
        .filter_map(|v| v["search_id"].as_str())
        .collect();
    let mut latest = BTreeMap::new();
    let mut statuses = BTreeMap::new();
    let mut actors = BTreeMap::new();
    let mut pages = BTreeMap::new();
    let mut latencies = vec![];
    let mut chars = 0u64;
    let mut zero_matches = 0usize;
    let mut truncated = 0usize;
    let mut query_hashes = BTreeSet::new();
    for s in &searches {
        if s["matched_pages"] == 0 {
            zero_matches += 1;
        }
        if s["truncated"] == true {
            truncated += 1;
        }
        if let Some(hash) = s["query_sha256"].as_str() {
            query_hashes.insert(hash);
        }
        *statuses
            .entry(s["status"].as_str().unwrap_or("unknown"))
            .or_insert(0usize) += 1;
        *actors
            .entry(s["attribution"]["actor"].as_str().unwrap_or("unattributed"))
            .or_insert(0usize) += 1;
        for h in s["hits"].as_array().into_iter().flatten() {
            *pages
                .entry(h["path"].as_str().unwrap_or("unknown"))
                .or_insert(0usize) += 1;
        }
        latencies.push(s["latency_ms"].as_u64().unwrap_or(0));
        chars += s["output_chars"].as_u64().unwrap_or(0);
    }
    for e in &events {
        if e["kind"] == "feedback" {
            if let Some(id) = e["search_id"].as_str() {
                if known.contains(id) {
                    latest.insert(
                        (id, e["rank"].as_u64()),
                        e["outcome"].as_str().unwrap_or("unknown"),
                    );
                }
            }
        }
    }
    let mut outcomes = BTreeMap::new();
    let mut assessed = BTreeSet::new();
    for ((id, _), outcome) in latest {
        assessed.insert(id);
        *outcomes.entry(outcome).or_insert(0usize) += 1;
    }
    latencies.sort();
    let percentile = |pct: usize| {
        latencies
            .get((latencies.len() * pct).div_ceil(100).saturating_sub(1))
            .copied()
    };
    println!("{}",serde_json::to_string_pretty(&json!({"schema_version":1,"days":args.days,"searches":searches.len(),"zero_match_searches_including_partial":zero_matches,"unique_query_hashes":query_hashes.len(),"truncated_searches":truncated,"statuses":statuses,"actors":actors,"returned_pages":pages,"latency_ms":{"p50":percentile(50),"p95":percentile(95)},"output_chars":chars,"searches_with_feedback":assessed.len(),"latest_feedback":outcomes,"malformed_records":malformed,"limitations":"Counts cover logged trk find calls only, not direct reads or SQL. Feedback is self-reported; missing feedback is unknown, not success. Old log files are retained; this report reads only the requested window."})).unwrap());
    Ok(())
}
