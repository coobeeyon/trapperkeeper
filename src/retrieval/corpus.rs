//! Bounded, descriptor-relative reads. Linked content cannot escape the wiki.
use pulldown_cmark::{Event, Parser, Tag};
use rustix::fs::{open, openat, Mode, OFlags};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

pub const FILE_LIMIT: usize = 1024 * 1024;
const CORPUS_LIMIT: usize = 64 * 1024 * 1024;
const PAGE_LIMIT: usize = 2000;

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub struct Reader {
    dir: File,
}

impl Reader {
    pub fn new(root: &Path) -> Result<Self, String> {
        let fd = open(
            root,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW,
            Mode::empty(),
        )
        .map_err(|e| e.to_string())?;
        Ok(Self { dir: fd.into() })
    }

    pub fn read(&self, path: &Path) -> Result<String, String> {
        let parts: Vec<_> = path.components().collect();
        if parts.is_empty() || parts.iter().any(|c| !matches!(c, Component::Normal(_))) {
            return Err("not a relative contained path".into());
        }
        let mut dir = self.dir.try_clone().map_err(|e| e.to_string())?;
        for (i, part) in parts.iter().enumerate() {
            let mut flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK;
            if i + 1 < parts.len() {
                flags |= OFlags::DIRECTORY;
            }
            let fd =
                openat(&dir, part.as_os_str(), flags, Mode::empty()).map_err(|e| e.to_string())?;
            dir = fd.into();
        }
        let before = dir.metadata().map_err(|e| e.to_string())?;
        if !before.is_file() || before.len() > FILE_LIMIT as u64 {
            return Err("not a regular file within 1 MiB".into());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if before.nlink() != 1 {
                return Err("hard-linked file excluded".into());
            }
        }
        let mut bytes = Vec::new();
        (&mut dir)
            .take((FILE_LIMIT + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        let after = dir.metadata().map_err(|e| e.to_string())?;
        if bytes.len() > FILE_LIMIT
            || before.len() != after.len()
            || before.modified().ok() != after.modified().ok()
        {
            return Err("file changed while reading or exceeded limit".into());
        }
        String::from_utf8(bytes).map_err(|_| "not UTF-8".into())
    }
}

#[derive(Clone)]
pub struct Nav {
    pub file: String,
    pub line: usize,
    pub text: String,
    pub anchor: Option<String>,
    pub preference: usize,
}

pub struct Page {
    pub path: String,
    pub text: String,
    pub digest: String,
    pub nav: Vec<Nav>,
}

pub struct Corpus {
    pub pages: Vec<Page>,
    pub warnings: Vec<String>,
    pub excluded: usize,
    pub bytes: usize,
    pub candidates: usize,
    pub fingerprint: String,
}

// Decode before validating, so encoded traversal/separators receive the same policy.
fn destination(raw: &str) -> Option<(PathBuf, Option<String>)> {
    let mut bytes = Vec::new();
    let source = raw.as_bytes();
    let mut i = 0;
    while i < source.len() {
        if source[i] == b'%' {
            let pair = source.get(i + 1..i + 3)?;
            bytes.push(u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()?);
            i += 3;
        } else {
            bytes.push(source[i]);
            i += 1;
        }
    }
    let decoded = String::from_utf8(bytes).ok()?;
    if decoded.chars().any(|c| c.is_control()) || decoded.contains(['\\', ':', '?']) {
        return None;
    }
    let (name, anchor) = decoded
        .split_once('#')
        .map_or((decoded.as_str(), None), |(p, a)| (p, Some(a.to_string())));
    let path = PathBuf::from(name.strip_prefix("./").unwrap_or(name));
    if path.extension()?.to_str()? != "md"
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return None;
    }
    Some((path, anchor))
}

pub fn load(root: &Path) -> Result<Corpus, String> {
    let reader = Reader::new(root)?;
    let mut corpus = Corpus {
        pages: vec![],
        warnings: vec![],
        excluded: 0,
        bytes: 0,
        candidates: 0,
        fingerprint: String::new(),
    };
    let mut refs: BTreeMap<PathBuf, Vec<Nav>> = BTreeMap::new();
    let mut fingerprints = String::new();
    let mut navigation_read = 0;
    for name in ["index.md", "toc.md"] {
        let text = match reader.read(Path::new(name)) {
            Ok(text) => text,
            Err(_) => {
                corpus.excluded += 1;
                corpus
                    .warnings
                    .push(format!("{name}: unavailable, unsafe or oversized"));
                continue;
            }
        };
        corpus.bytes += text.len();
        navigation_read += 1;
        fingerprints.push_str(&digest(text.as_bytes()));
        let mut previous_line = 0;
        let mut preference = 0;
        let line_starts: Vec<_> = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(i, _)| i + 1))
            .collect();
        let mut links = 0;
        for (event, range) in Parser::new(&text).into_offset_iter() {
            let Event::Start(Tag::Link { dest_url, .. }) = event else {
                continue;
            };
            links += 1;
            if links > 10000 {
                corpus.excluded += 1;
                break;
            }
            // Remote/source links are pointers, not part of the declared Markdown corpus.
            if dest_url.contains("://") || dest_url.starts_with('#') {
                continue;
            }
            let Some((path, anchor)) = destination(&dest_url) else {
                if dest_url.contains(".md") {
                    corpus.excluded += 1;
                }
                continue;
            };
            let line = line_starts.partition_point(|&i| i <= range.start);
            let start = line_starts[line - 1];
            let end = line_starts.get(line).map_or(text.len(), |i| i - 1);
            if line != previous_line {
                preference = 0;
                previous_line = line;
            }
            // No recursive traversal and no navigation/self-page results.
            if path == Path::new("index.md") || path == Path::new("toc.md") {
                continue;
            }
            if refs.len() >= PAGE_LIMIT && !refs.contains_key(&path) {
                corpus.excluded += 1;
                continue;
            }
            let entries = refs.entry(path).or_default();
            if entries.len() >= 64 || end - start > 4096 {
                corpus.excluded += 1;
                continue;
            }
            entries.push(Nav {
                file: name.into(),
                line,
                text: text[start..end].into(),
                anchor,
                preference,
            });
            preference += 1;
        }
    }
    if navigation_read == 0 {
        return Err("neither index.md nor toc.md could be safely read".into());
    }
    corpus.candidates = refs.len();
    let mut budget_exhausted = false;
    for (path, nav) in refs {
        if budget_exhausted || corpus.bytes >= CORPUS_LIMIT || nav.is_empty() {
            corpus.excluded += 1;
            continue;
        }
        let text = match reader.read(&path) {
            Ok(text) => text,
            Err(_) => {
                corpus.excluded += 1;
                continue;
            }
        };
        if corpus.bytes + text.len() > CORPUS_LIMIT {
            budget_exhausted = true;
            corpus.excluded += 1;
            continue;
        }
        corpus.bytes += text.len();
        let hash = digest(text.as_bytes());
        fingerprints.push_str(&path.to_string_lossy());
        fingerprints.push_str(&hash);
        corpus.pages.push(Page {
            path: path.to_string_lossy().into(),
            text,
            digest: hash,
            nav,
        });
    }
    if corpus.excluded > 0 {
        corpus.warnings.push(format!(
            "{} navigation targets or files excluded (containment, read or resource limits)",
            corpus.excluded
        ));
    }
    corpus.fingerprint = digest(fingerprints.as_bytes());
    Ok(corpus)
}
