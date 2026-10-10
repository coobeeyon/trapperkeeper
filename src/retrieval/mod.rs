pub mod corpus;
use corpus::{Corpus, Page};
use serde::Serialize;
use std::collections::BTreeSet;

pub const RANKER: &str = "lexical-v2";

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
                // A deliberately small inflection fold, not synonym expansion.
                // Preserve identifiers and endings such as status/business/analysis.
                let t = if t.len() > 4
                    && t.bytes().all(|c| c.is_ascii_alphabetic())
                    && !["ss", "us", "is"].iter().any(|suffix| t.ends_with(suffix))
                {
                    t.strip_suffix('s').unwrap_or(t)
                } else {
                    t
                };
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

struct Passage {
    text: String,
    terms: BTreeSet<String>,
    score: usize,
    first_match: usize,
    start: usize,
    end: usize,
    truncated: bool,
}

// Score the bounded text that will actually be returned, not all the words
// scattered through an arbitrarily large section. Overlapping windows also
// reach matches beyond the first 600 characters of a single Markdown line.
fn passage(lines: &[&str], weights: &[(String, usize)]) -> Passage {
    let mut chars = Vec::new();
    let mut line_at = Vec::new();
    let mut starts = BTreeSet::new();
    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            chars.push('\n');
            line_at.push(i - 1);
        }
        starts.insert(chars.len());
        for c in line.chars() {
            chars.push(c);
            line_at.push(i);
        }
    }
    starts.extend((0..chars.len()).step_by(300));
    let mut best: Option<Passage> = None;
    for mut start in starts {
        let mut end = (start + 600).min(chars.len());
        // Never create a match by slicing through a larger token. Whitespace
        // boundaries also preserve UTF-8 and keep citations on original lines.
        if start > 0 && !chars[start - 1].is_whitespace() {
            while start < end && !chars[start].is_whitespace() {
                start += 1;
            }
        }
        if end < chars.len() && !chars[end].is_whitespace() {
            while end > start && !chars[end - 1].is_whitespace() {
                end -= 1;
            }
        }
        while start < end && chars[start].is_whitespace() {
            start += 1;
        }
        while end > start && chars[end - 1].is_whitespace() {
            end -= 1;
        }
        if start == end {
            continue;
        }
        let text: String = chars[start..end].iter().collect();
        let visible = terms(&text);
        // Wiki entity records commonly use a leading bold label in a bullet
        // instead of a Markdown heading. Only labels visible in this window
        // receive the same local-topic boost; repetition never adds points.
        let labels: BTreeSet<_> = text
            .lines()
            .filter_map(|line| {
                let line = line.trim_start().trim_start_matches("- ");
                line.strip_prefix("**")?.split_once("**").map(|p| p.0)
            })
            .flat_map(terms)
            .collect();
        let score = weights
            .iter()
            .map(|(t, w)| {
                w * (8 * usize::from(visible.contains(t)) + 8 * usize::from(labels.contains(t)))
            })
            .sum();
        if best.as_ref().is_some_and(|b| score < b.score) {
            continue;
        }
        let mut first_match = 0;
        for word in text.split_inclusive(char::is_whitespace) {
            let word_terms = terms(word);
            if weights.iter().any(|(term, _)| word_terms.contains(term)) {
                break;
            }
            first_match += word.chars().count();
        }
        if best.as_ref().is_none_or(|b| {
            (score, std::cmp::Reverse(first_match)) > (b.score, std::cmp::Reverse(b.first_match))
        }) {
            best = Some(Passage {
                text,
                terms: visible,
                score,
                first_match,
                start: line_at[start],
                end: line_at[end - 1],
                truncated: start > 0 || end < chars.len(),
            });
        }
    }
    best.unwrap_or(Passage {
        text: String::new(),
        terms: BTreeSet::new(),
        score: 0,
        first_match: 0,
        start: 0,
        end: 0,
        truncated: !chars.is_empty(),
    })
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
            let frequency = page_terms.iter().filter(|set| set.contains(t)).count();
            // Logarithmic rarity: one unusual incidental word must not drown
            // out several common words that jointly describe the question.
            let rarity = ((corpus.pages.len() + 1) as f64 / (frequency + 1) as f64).ln();
            (t.clone(), 1024 + (1024.0 * rarity).round() as usize)
        })
        .collect();
    let mut hits = vec![];
    for (page, indexed_terms) in corpus.pages.iter().zip(&page_terms) {
        if qt.is_disjoint(indexed_terms) {
            continue;
        }
        if let Some(hit) = rank_page(page, &query, &qt, &weights) {
            hits.push(hit);
        }
    }
    hits.sort_by(|a, b| {
        b.exact_match
            .cmp(&a.exact_match)
            .then((!b.passage_terms.is_empty()).cmp(&!a.passage_terms.is_empty()))
            .then(b.score.cmp(&a.score))
            .then(b.matched_terms.len().cmp(&a.matched_terms.len()))
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
        let snippet = passage(&lines[start..end], weights);
        let body_terms = &snippet.terms;
        let mut matched = vec![];
        let mut score = snippet.score;
        for (term, weight) in weights {
            let field = if header_terms.contains(term) {
                8
            } else if title_terms.contains(term) {
                4
            } else if nav_terms.contains(term) && nav.file == "index.md" {
                2
            } else if nav_terms.contains(term) {
                1
            } else {
                0
            };
            if field > 0 || body_terms.contains(term) {
                matched.push(term.clone());
                score += weight * field;
            }
        }
        if matched.is_empty() {
            continue;
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
            line_start: start + snippet.start + 1,
            line_end: start + snippet.end + 1,
            anchor: nav.anchor.clone(),
            digest: page.digest.clone(),
            navigation: format!("{}:{}", nav.file, nav.line),
            excerpt_truncated: snippet.truncated,
            excerpt: snippet.text,
            context: clip(&context, 400),
            matched_terms: matched.clone(),
            passage_terms: qt
                .iter()
                .filter(|t| body_terms.contains(*t))
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
                !hit.passage_terms.is_empty(),
                hit.score,
                hit.matched_terms.len(),
            ) > (
                b.anchored,
                b.exact_match,
                !b.passage_terms.is_empty(),
                b.score,
                b.matched_terms.len(),
            )
        }) {
            best = Some(hit);
        }
    }
    best
}
