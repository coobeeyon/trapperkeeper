pub mod corpus;
use corpus::{Corpus, Page};
use serde::Serialize;
use std::collections::BTreeSet;

pub const RANKER: &str = "lexical-v1";

pub fn normalize(s: &str) -> String {
    s.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
pub fn terms(s: &str) -> BTreeSet<String> {
    let mut result = BTreeSet::new();
    for token in s
        .to_lowercase()
        .split(|c: char| !(c.is_alphanumeric() || "_-./".contains(c)))
    {
        let token = token.trim_matches(['.', '/', '-', '_']);
        for t in std::iter::once(token).chain(token.split(['-', '_', '/', '.'])) {
            if !t.is_empty()
                && ![
                    "a", "an", "the", "is", "are", "was", "were", "how", "what", "where", "when",
                    "why", "who", "do", "does", "did", "i", "we", "you", "to", "of", "for", "in",
                    "on", "and", "or", "it", "this", "that", "with", "can", "my", "which",
                ]
                .contains(&t)
            {
                result.insert(t.to_string());
            }
        }
    }
    result
}

#[derive(Clone, Serialize)]
pub struct Hit {
    pub path: String,
    pub title: String,
    pub heading: String,
    pub line_start: usize,
    pub line_end: usize,
    pub anchor: Option<String>,
    pub digest: String,
    pub navigation: String,
    pub excerpt: String,
    pub context: String,
    pub matched_terms: Vec<String>,
    pub passage_terms: Vec<String>,
    pub partial_match: bool,
    pub exact_match: bool,
    pub score: usize,
    pub excerpt_truncated: bool,
    #[serde(skip)]
    preference: usize,
    #[serde(skip)]
    anchored: bool,
}

fn plain(s: &str) -> String {
    // Strip link destinations and inline markup from aliases, without evaluating content.
    let mut out = String::new();
    let mut skip = false;
    for c in s.chars() {
        if c == '(' {
            skip = true;
        } else if c == ')' {
            skip = false;
        } else if !skip && !"*`[]#".contains(c) {
            out.push(c);
        }
    }
    normalize(&out).trim_matches([' ', '-']).to_string()
}
fn exact(q: &str, s: &str) -> bool {
    let p = plain(s);
    p == q && (q.contains(' ') || q.contains(['-', '_', '/', '.']))
}
fn alias_exact(query: &str, text: &str) -> bool {
    let aliases = text
        .split_once("**")
        .and_then(|(_, rest)| rest.split_once("**"))
        .map_or(text, |(aliases, _)| aliases);
    aliases.split(" / ").any(|alias| exact(query, alias))
}
fn anchor(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
        .map(|c| if c.is_whitespace() { '-' } else { c })
        .collect()
}
fn clip(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

pub fn rank(corpus: &Corpus, query: &str) -> Vec<Hit> {
    let query = normalize(query);
    let qt = terms(&query);
    let page_terms: Vec<_> = corpus
        .pages
        .iter()
        .map(|p| {
            terms(&format!(
                "{} {}",
                p.text,
                p.nav
                    .iter()
                    .map(|n| n.text.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            ))
        })
        .collect();
    let weights: Vec<_> = qt
        .iter()
        .map(|t| {
            (
                t.clone(),
                (1024 * (corpus.pages.len() + 1))
                    .div_ceil(page_terms.iter().filter(|set| set.contains(t)).count() + 1),
            )
        })
        .collect();
    let mut hits = vec![];
    for page in &corpus.pages {
        if let Some(hit) = rank_page(page, &query, &qt, &weights) {
            hits.push(hit);
        }
    }
    hits.sort_by(|a, b| {
        b.exact_match
            .cmp(&a.exact_match)
            .then(b.matched_terms.len().cmp(&a.matched_terms.len()))
            .then(b.passage_terms.len().cmp(&a.passage_terms.len()))
            .then(b.score.cmp(&a.score))
            .then(a.preference.cmp(&b.preference))
            .then(a.path.cmp(&b.path))
            .then(a.line_start.cmp(&b.line_start))
    });
    hits
}

fn rank_page(
    page: &Page,
    query: &str,
    qt: &BTreeSet<String>,
    weights: &[(String, usize)],
) -> Option<Hit> {
    let lines: Vec<_> = page.text.lines().collect();
    let title = lines
        .iter()
        .find_map(|l| l.strip_prefix("# "))
        .unwrap_or(&page.path);
    let mut starts = vec![0];
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.starts_with('#') && line.trim_start_matches('#').starts_with(' ') {
            starts.push(i);
        }
    }
    starts.push(lines.len());
    let title_terms = terms(title);
    let nav = page.nav.iter().max_by_key(|n| {
        let t = terms(&n.text);
        (
            alias_exact(query, &n.text),
            qt.intersection(&t).count(),
            usize::MAX - n.preference,
        )
    })?;
    let nav_terms = terms(&nav.text);
    let alias_exact = alias_exact(query, &nav.text);
    let mut best: Option<Hit> = None;
    for section in starts.windows(2) {
        let (start, end) = (section[0], section[1]);
        if start == end {
            continue;
        }
        let heading = lines[start]
            .strip_prefix('#')
            .map(|_| lines[start].trim_start_matches('#').trim())
            .unwrap_or(title);
        let header_terms = terms(heading);
        let body = lines[start..end].join("\n");
        let body_terms = terms(&body);
        let mut matched = vec![];
        let mut score = 0;
        for (term, weight) in weights {
            let field = if nav_terms.contains(term) && nav.file == "index.md" {
                6
            } else if title_terms.contains(term) || header_terms.contains(term) {
                4
            } else if nav_terms.contains(term) {
                2
            } else if body_terms.contains(term) {
                1
            } else {
                0
            };
            if field > 0 {
                matched.push(term.clone());
                score += weight * field;
            }
            // Matching actual passage breaks ties between sections on a broadly aliased page.
            if body_terms.contains(term) {
                score += weight;
            }
        }
        if matched.is_empty() {
            continue;
        }
        let focus = (start..end)
            .max_by_key(|&i| {
                let t = terms(lines[i]);
                (qt.intersection(&t).count(), usize::MAX - i)
            })
            .unwrap_or(start);
        let preceding = focus.saturating_sub(1).max(start);
        let snippet_start = if preceding != focus
            && lines[preceding].chars().count() + lines[focus].chars().count() + 1 > 600
        {
            focus
        } else {
            preceding
        };
        let mut excerpt = String::new();
        let mut snippet_end = snippet_start;
        let mut line_cut = false;
        for (i, line) in lines.iter().enumerate().take(end).skip(snippet_start) {
            if excerpt.chars().count() >= 599 {
                break;
            }
            if !excerpt.is_empty() {
                excerpt.push('\n');
            }
            let remaining = 600 - excerpt.chars().count();
            line_cut = line.chars().count() > remaining;
            excerpt.push_str(&clip(line, remaining));
            snippet_end = i;
            if excerpt.chars().count() >= 600 {
                break;
            }
        }
        let context = lines
            .iter()
            .take(20)
            .enumerate()
            .filter(|(i, l)| {
                *i == 0
                    || [
                        "status",
                        "verified",
                        "supersed",
                        "historical",
                        "updated",
                        "current",
                        "date",
                    ]
                    .iter()
                    .any(|key| l.to_lowercase().contains(key))
            })
            .map(|(i, l)| format!("L{}: {}", i + 1, l))
            .collect::<Vec<_>>()
            .join("\n");
        let hit = Hit {
            anchored: alias_exact && nav.anchor.as_deref() == Some(anchor(heading).as_str()),
            path: page.path.clone(),
            title: clip(title, 160),
            heading: clip(heading, 160),
            line_start: snippet_start + 1,
            line_end: snippet_end + 1,
            anchor: nav.anchor.clone(),
            digest: page.digest.clone(),
            navigation: format!("{}:{}", nav.file, nav.line),
            excerpt_truncated: snippet_start > start || snippet_end + 1 < end || line_cut,
            excerpt,
            context: clip(&context, 400),
            matched_terms: matched.clone(),
            passage_terms: qt
                .iter()
                .filter(|t| body_terms.contains(*t) || title_terms.contains(*t))
                .cloned()
                .collect(),
            partial_match: matched.len() < qt.len(),
            exact_match: alias_exact
                || exact(query, title)
                || exact(query, heading)
                || exact(query, &page.path),
            score,
            preference: nav.preference,
        };
        if best.as_ref().is_none_or(|b| {
            (
                hit.anchored,
                hit.exact_match,
                hit.matched_terms.len(),
                hit.passage_terms.len(),
                hit.score,
            ) > (
                b.anchored,
                b.exact_match,
                b.matched_terms.len(),
                b.passage_terms.len(),
                b.score,
            )
        }) {
            best = Some(hit);
        }
    }
    best
}
