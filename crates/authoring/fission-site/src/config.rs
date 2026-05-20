use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::app::TemplateId;

#[derive(Clone, Debug, Default)]
pub struct BuildOptions {
    pub project_dir: PathBuf,
    pub release: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ProjectFile {
    pub(crate) site: SiteConfig,
    #[serde(default)]
    pub(crate) targets: BTreeSet<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SiteConfig {
    #[serde(default)]
    pub entry: Option<String>,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_out_dir")]
    pub out_dir: PathBuf,
    #[serde(default = "default_locale")]
    pub default_locale: String,
    #[serde(default)]
    pub locales: Vec<String>,
    #[serde(default = "default_true")]
    pub pretty_urls: bool,
    #[serde(default)]
    pub minify: bool,
    #[serde(default = "default_true")]
    pub generate_sitemap: bool,
    #[serde(default = "default_true")]
    pub generate_robots: bool,
    #[serde(default)]
    pub asset_dirs: Vec<PathBuf>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub sidebar: Option<PathBuf>,
    #[serde(default)]
    pub routes: Vec<SiteRouteConfig>,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            entry: None,
            base_url: default_base_url(),
            out_dir: default_out_dir(),
            default_locale: default_locale(),
            locales: Vec::new(),
            pretty_urls: true,
            minify: false,
            generate_sitemap: true,
            generate_robots: true,
            asset_dirs: Vec::new(),
            title: None,
            description: None,
            sidebar: None,
            routes: vec![SiteRouteConfig::default()],
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct SiteRouteConfig {
    #[serde(default = "default_content_kind")]
    pub kind: String,
    #[serde(default = "default_content_path")]
    pub path: String,
    #[serde(default = "default_content_source")]
    pub source: PathBuf,
    #[serde(default = "default_template")]
    pub template: TemplateId,
    #[serde(default)]
    pub sidebar: Option<PathBuf>,
}

impl Default for SiteRouteConfig {
    fn default() -> Self {
        Self {
            kind: default_content_kind(),
            path: default_content_path(),
            source: default_content_source(),
            template: default_template(),
            sidebar: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FrontMatter {
    pub values: BTreeMap<String, String>,
}

impl FrontMatter {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub fn title(&self) -> Option<&str> {
        self.get("title")
    }

    pub fn description(&self) -> Option<&str> {
        self.get("description")
    }

    pub fn template(&self) -> Option<&str> {
        self.get("template")
    }

    pub fn locale(&self) -> Option<&str> {
        self.get("locale")
    }

    pub fn slug(&self) -> Option<&str> {
        self.get("slug")
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SidebarFile {
    #[serde(default)]
    pub items: Vec<SidebarItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SidebarItem {
    pub title: String,
    pub href: String,
    #[serde(default)]
    pub level: usize,
    #[serde(default)]
    pub group: bool,
}

pub(crate) fn read_project_file(project_dir: &Path) -> Result<ProjectFile> {
    let path = project_dir.join("fission.toml");
    let data =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&data).with_context(|| format!("failed to parse {}", path.display()))
}

pub(crate) fn normalized_site_config(mut site: SiteConfig) -> SiteConfig {
    if site.routes.is_empty() {
        site.routes.push(SiteRouteConfig::default());
    }
    if site.locales.is_empty() {
        site.locales.push(site.default_locale.clone());
    }
    site
}

pub(crate) fn split_front_matter(raw: &str) -> (FrontMatter, String) {
    let normalized = raw.strip_prefix('\u{feff}').unwrap_or(raw);
    if !normalized.starts_with("---\n") {
        return (FrontMatter::default(), normalized.to_string());
    }
    let rest = &normalized[4..];
    let Some(end) = rest.find("\n---") else {
        return (FrontMatter::default(), normalized.to_string());
    };
    let front = &rest[..end];
    let body = rest[end + 4..].trim_start_matches(['\r', '\n']).to_string();
    (parse_front_matter(front), body)
}

fn parse_front_matter(front: &str) -> FrontMatter {
    let mut values = BTreeMap::new();
    for line in front.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_string();
            let mut value = value.trim().to_string();
            value = value.trim_matches('"').trim_matches('\'').to_string();
            if !key.is_empty() {
                values.insert(key, value);
            }
        }
    }
    FrontMatter { values }
}

fn default_base_url() -> String {
    "http://localhost:8123".into()
}

fn default_out_dir() -> PathBuf {
    PathBuf::from("dist/site")
}

fn default_locale() -> String {
    "en-US".into()
}

fn default_true() -> bool {
    true
}

fn default_content_kind() -> String {
    "content".into()
}

fn default_content_path() -> String {
    "/content".into()
}

fn default_content_source() -> PathBuf {
    PathBuf::from("content")
}

pub(crate) fn default_template() -> TemplateId {
    "fission::site::DocumentationTemplate".into()
}
