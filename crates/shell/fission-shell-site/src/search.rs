use crate::document::ContentRoute;
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SiteSearchOptions {
    pub enabled: bool,
    pub output_path: String,
    pub min_token_len: usize,
}

impl Default for SiteSearchOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            output_path: "search".to_string(),
            min_token_len: 2,
        }
    }
}

#[derive(Debug, Serialize)]
struct SearchManifest {
    version: u32,
    locale: String,
    documents: String,
    shards: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct SearchDocument {
    id: usize,
    title: String,
    section: Option<String>,
    href: String,
    excerpt: String,
}

#[derive(Debug)]
struct SearchRecord {
    title: String,
    section: Option<String>,
    href: String,
    excerpt: String,
    weighted_text: Vec<(u16, String)>,
}

pub fn write_search_index(
    search_dir: &Path,
    routes: &[ContentRoute],
    locale: &str,
    options: &SiteSearchOptions,
) -> Result<()> {
    let records = search_records(routes);
    let documents = records
        .iter()
        .enumerate()
        .map(|(id, record)| SearchDocument {
            id,
            title: record.title.clone(),
            section: record.section.clone(),
            href: record.href.clone(),
            excerpt: record.excerpt.clone(),
        })
        .collect::<Vec<_>>();
    let shards = build_shards(&records, options.min_token_len);
    let shard_manifest = write_shards(search_dir, shards)?;

    let manifest = SearchManifest {
        version: 1,
        locale: locale.to_string(),
        documents: "docs.json".to_string(),
        shards: shard_manifest,
    };

    write_json(search_dir.join("manifest.json").as_path(), &manifest)?;
    write_json(search_dir.join("docs.json").as_path(), &documents)
}

fn search_records(routes: &[ContentRoute]) -> Vec<SearchRecord> {
    let mut records = Vec::new();
    for route in routes {
        if route.rendered.is_some() {
            records.push(SearchRecord {
                title: route.title.clone(),
                section: None,
                href: search_href(&route.path, None),
                excerpt: route.description.clone().unwrap_or_default(),
                weighted_text: vec![
                    (8, route.title.clone()),
                    (4, route.description.clone().unwrap_or_default()),
                ],
            });
            continue;
        }

        let sections = markdown_sections(&route.body);
        if sections.is_empty() {
            records.push(route_record(route, None, &route.body));
        } else {
            records.push(route_record(route, None, &intro_text(&route.body)));
            for section in sections {
                let href = search_href(&route.path, Some(&section.anchor));
                let excerpt = excerpt(&section.body);
                records.push(SearchRecord {
                    title: route.title.clone(),
                    section: Some(section.title.clone()),
                    href,
                    excerpt,
                    weighted_text: vec![
                        (8, route.title.clone()),
                        (5, section.title),
                        (4, route.description.clone().unwrap_or_default()),
                        (1, strip_markdown(&section.body)),
                    ],
                });
            }
        }
    }
    records
}

fn route_record(route: &ContentRoute, anchor: Option<&str>, body: &str) -> SearchRecord {
    let stripped = strip_markdown(body);
    SearchRecord {
        title: route.title.clone(),
        section: None,
        href: search_href(&route.path, anchor),
        excerpt: route
            .description
            .clone()
            .unwrap_or_else(|| excerpt(&stripped)),
        weighted_text: vec![
            (8, route.title.clone()),
            (4, route.description.clone().unwrap_or_default()),
            (
                3,
                route
                    .headings
                    .iter()
                    .map(|heading| heading.title.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
            (1, stripped),
        ],
    }
}

#[derive(Debug)]
struct MarkdownSection {
    title: String,
    anchor: String,
    body: String,
}

fn markdown_sections(markdown: &str) -> Vec<MarkdownSection> {
    let mut sections = Vec::new();
    let mut current: Option<MarkdownSection> = None;
    for line in markdown.lines() {
        if let Some((level, title)) = markdown_heading(line) {
            if level <= 3 {
                if let Some(section) = current.take() {
                    sections.push(section);
                }
                current = Some(MarkdownSection {
                    title: title.to_string(),
                    anchor: slug(title),
                    body: String::new(),
                });
                continue;
            }
        }
        if let Some(section) = &mut current {
            section.body.push_str(line);
            section.body.push('\n');
        }
    }
    if let Some(section) = current {
        sections.push(section);
    }
    sections
        .into_iter()
        .filter(|section| !strip_markdown(&section.body).trim().is_empty())
        .collect()
}

fn intro_text(markdown: &str) -> String {
    let mut out = String::new();
    for line in markdown.lines() {
        if markdown_heading(line).is_some() && !out.trim().is_empty() {
            break;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn build_shards(
    records: &[SearchRecord],
    min_token_len: usize,
) -> BTreeMap<String, BTreeMap<String, Vec<[u32; 2]>>> {
    let mut postings: BTreeMap<String, HashMap<usize, u32>> = BTreeMap::new();
    for (id, record) in records.iter().enumerate() {
        for (weight, text) in &record.weighted_text {
            for token in tokenise(text, min_token_len) {
                let score = postings.entry(token).or_default().entry(id).or_insert(0);
                *score = (*score).saturating_add((*weight).into());
            }
        }
    }

    let mut shards: BTreeMap<String, BTreeMap<String, Vec<[u32; 2]>>> = BTreeMap::new();
    for (token, docs) in postings {
        let shard = shard_id(&token);
        let mut values = docs
            .into_iter()
            .map(|(id, score)| [id as u32, score])
            .collect::<Vec<_>>();
        values.sort_by(|a, b| b[1].cmp(&a[1]).then_with(|| a[0].cmp(&b[0])));
        shards.entry(shard).or_default().insert(token, values);
    }
    shards
}

fn write_shards(
    search_dir: &Path,
    shards: BTreeMap<String, BTreeMap<String, Vec<[u32; 2]>>>,
) -> Result<BTreeMap<String, String>> {
    let mut manifest = BTreeMap::new();
    for (shard, data) in shards {
        let file = format!("index-{shard}.json");
        write_json(search_dir.join(&file).as_path(), &data)?;
        manifest.insert(shard, file);
    }
    Ok(manifest)
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let json = serde_json::to_vec(value)?;
    fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn search_href(route_path: &str, anchor: Option<&str>) -> String {
    let mut href = route_path.trim_start_matches('/').to_string();
    if href.is_empty() {
        href.push_str("./");
    }
    if !href.ends_with('/') && !href.contains('#') {
        href.push('/');
    }
    if let Some(anchor) = anchor.filter(|anchor| !anchor.is_empty()) {
        href.push('#');
        href.push_str(anchor);
    }
    href
}

fn tokenise(text: &str, min_token_len: usize) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '+' || ch == '-' {
            current.push(ch);
        } else {
            push_token(&mut tokens, &mut current, min_token_len);
        }
    }
    push_token(&mut tokens, &mut current, min_token_len);
    tokens
}

fn push_token(tokens: &mut Vec<String>, current: &mut String, min_token_len: usize) {
    if current.len() >= min_token_len && current.chars().any(|ch| ch.is_ascii_alphanumeric()) {
        tokens.push(std::mem::take(current));
    } else {
        current.clear();
    }
}

fn shard_id(token: &str) -> String {
    let first = token.chars().next().unwrap_or('o');
    if first.is_ascii_alphanumeric() {
        first.to_string()
    } else {
        "other".to_string()
    }
}

fn strip_markdown(markdown: &str) -> String {
    let mut out = String::new();
    let mut in_fence = false;
    for raw in markdown.lines() {
        let line = raw.trim();
        if line.starts_with("```") || line.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            out.push_str(raw);
            out.push('\n');
            continue;
        }
        let line = line
            .trim_start_matches('#')
            .trim_start_matches('>')
            .trim_start_matches('-')
            .trim_start_matches('*')
            .trim();
        let line = strip_markdown_links(line);
        out.push_str(&line);
        out.push('\n');
    }
    out
}

fn strip_markdown_links(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(open) = rest.find('[') {
        out.push_str(&rest[..open]);
        let after_open = &rest[open + 1..];
        let Some(close) = after_open.find("](") else {
            out.push_str(&rest[open..]);
            return out;
        };
        out.push_str(&after_open[..close]);
        let after_target = &after_open[close + 2..];
        let Some(end) = after_target.find(')') else {
            out.push_str(after_target);
            return out;
        };
        rest = &after_target[end + 1..];
    }
    out.push_str(rest);
    out
}

fn excerpt(text: &str) -> String {
    let stripped = strip_markdown(text);
    let mut out = String::new();
    for word in stripped.split_whitespace() {
        if out.len() + word.len() + 1 > 180 {
            break;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    out
}

fn markdown_heading(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    let hashes = trimmed.chars().take_while(|ch| *ch == '#').count();
    if !(1..=6).contains(&hashes) {
        return None;
    }
    let title = trimmed.get(hashes..)?.trim();
    if title.is_empty() {
        return None;
    }
    Some((hashes, title))
}

fn slug(value: &str) -> String {
    let mut out = String::new();
    let mut previous_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            previous_dash = false;
        } else if !previous_dash && !out.is_empty() {
            out.push('-');
            previous_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenise_keeps_programming_terms() {
        assert_eq!(
            tokenise("Fission Rust UI cargo-run", 2),
            ["fission", "rust", "ui", "cargo-run"]
        );
    }

    #[test]
    fn strips_markdown_links_without_losing_text() {
        assert_eq!(
            strip_markdown_links("Read [Quickstart](./quickstart) now"),
            "Read Quickstart now"
        );
    }
}
