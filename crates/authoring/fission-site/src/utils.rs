use std::path::{Path, PathBuf};

use crate::config::SiteConfig;

pub(crate) fn content_route_path(base: &str, relative: &Path, slug: Option<&str>) -> String {
    let mut pieces = Vec::new();
    if let Some(slug) = slug {
        pieces.push(slug.trim_matches('/').to_string());
    } else {
        for component in relative.components() {
            let value = component.as_os_str().to_string_lossy();
            let mut part = value.to_string();
            if let Some((stem, _)) = part.rsplit_once('.') {
                part = stem.to_string();
            }
            if part == "index" {
                continue;
            }
            pieces.push(slugify(&part));
        }
    }
    let base = normalize_route_path(base);
    if pieces.is_empty() {
        base
    } else {
        normalize_route_path(&format!(
            "{}/{}",
            base.trim_end_matches('/'),
            pieces.join("/")
        ))
    }
}

pub(crate) fn output_path_for_route(path: &str, pretty: bool) -> PathBuf {
    let normalized = normalize_route_path(path);
    if normalized == "/" {
        return PathBuf::from("index.html");
    }
    let relative = normalized.trim_matches('/');
    if pretty {
        PathBuf::from(relative).join("index.html")
    } else {
        PathBuf::from(format!("{relative}.html"))
    }
}

pub(crate) fn normalize_route_path(path: &str) -> String {
    let mut out = String::new();
    out.push('/');
    out.push_str(path.trim().trim_matches('/'));
    if out != "/" && !out.ends_with('/') {
        out.push('/');
    }
    out
}

pub(crate) fn normalize_markdown_href(href: &str, route_path: Option<&str>) -> String {
    if is_passthrough_href(href) {
        return href.to_string();
    }
    if href.starts_with('/') {
        return normalize_route_path(href);
    }
    let Some(route_path) = route_path else {
        return href.to_string();
    };
    normalize_relative_href(href, route_path)
}

fn is_passthrough_href(href: &str) -> bool {
    href.starts_with("http://")
        || href.starts_with("https://")
        || href.starts_with('#')
        || href.starts_with("mailto:")
        || href.starts_with("tel:")
        || href.starts_with("data:")
}

fn normalize_relative_href(href: &str, route_path: &str) -> String {
    let (path_and_query, anchor) = href.split_once('#').unwrap_or((href, ""));
    let (path_part, query) = path_and_query
        .split_once('?')
        .unwrap_or((path_and_query, ""));
    let mut segments = normalize_route_path(route_path)
        .trim_matches('/')
        .split('/')
        .filter(|part| !part.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if !segments.is_empty() {
        segments.pop();
    }
    for part in path_part.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            value => segments.push(strip_markdown_extension(value).to_string()),
        }
    }
    let mut normalized = if segments.is_empty() {
        "/".to_string()
    } else {
        normalize_route_path(&format!("/{}", segments.join("/")))
    };
    if !query.is_empty() {
        normalized.push('?');
        normalized.push_str(query);
    }
    if !anchor.is_empty() {
        normalized.push('#');
        normalized.push_str(anchor);
    }
    normalized
}

fn strip_markdown_extension(value: &str) -> &str {
    value
        .strip_suffix(".mdx")
        .or_else(|| value.strip_suffix(".md"))
        .unwrap_or(value)
}

pub(crate) fn title_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .map(|value| {
            value
                .split(['-', '_'])
                .filter(|part| !part.is_empty())
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_else(|| "Page".to_string())
}

pub(crate) fn slugify(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

pub(crate) fn canonical_url(site: &SiteConfig, path: &str) -> String {
    format!(
        "{}{}",
        site.base_url.trim_end_matches('/'),
        normalize_route_path(path)
    )
}

pub(crate) fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(crate) fn escape_attr(value: &str) -> String {
    escape_html(value).replace('"', "&quot;")
}
