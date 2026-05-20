use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::app::PageShell;
use crate::config::{SidebarItem, SiteConfig};
use crate::markdown::HeadingLink;
use crate::utils::{canonical_url, escape_attr, escape_html, normalize_route_path};

const SITE_JS: &str = include_str!("../assets/site.js");
const SITE_CSS: &str = include_str!("../assets/site.css");

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ArtifactManifest {
    pub(crate) base_url: String,
    pub(crate) routes: Vec<ManifestRoute>,
    pub(crate) assets: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ManifestRoute {
    pub(crate) path: String,
    pub(crate) output: String,
    pub(crate) title: String,
    pub(crate) locale: String,
    pub(crate) source: String,
}

#[derive(Clone, Debug)]
pub(crate) struct RenderedRoute {
    pub(crate) path: String,
    pub(crate) output: PathBuf,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) locale: String,
    pub(crate) body: String,
    pub(crate) shell: PageShell,
    pub(crate) sidebar: Vec<SidebarItem>,
    pub(crate) h1_links: Vec<HeadingLink>,
    pub(crate) source: String,
}

pub(crate) fn render_shell(site: &SiteConfig, route: &RenderedRoute) -> String {
    let site_title = site.title.as_deref().unwrap_or("Fission");
    let full_title = if route.title == site_title {
        route.title.clone()
    } else {
        format!("{} | {}", route.title, site_title)
    };
    let canonical = canonical_url(site, &route.path);
    let sidebar = render_sidebar(&route.sidebar, &route.path);
    let toc = render_toc(&route.h1_links);
    let main_class = match route.shell {
        PageShell::Marketing => "site-main site-main-marketing",
        PageShell::Documentation => "site-main site-main-docs",
    };
    let body = match route.shell {
        PageShell::Marketing => route.body.clone(),
        PageShell::Documentation => format!(
            "<div class=\"docs-layout\"><aside class=\"docs-sidebar\">{sidebar}</aside><article class=\"markdown docs-content\">{}</article><aside class=\"docs-toc\">{toc}</aside></div>",
            route.body
        ),
    };

    format!(
        r#"<!doctype html>
<html lang="{lang}">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>{title}</title>
  <meta name="description" content="{description}" />
  <link rel="canonical" href="{canonical}" />
  <meta property="og:title" content="{title}" />
  <meta property="og:description" content="{description}" />
  <meta property="og:type" content="website" />
  <meta property="og:url" content="{canonical}" />
  <link rel="stylesheet" href="/assets/site.css" />
  <script defer src="/assets/site.js"></script>
</head>
<body>
  {header}
  <main class="{main_class}">{body}</main>
  {footer}
</body>
</html>
"#,
        lang = escape_attr(&route.locale),
        title = escape_html(&full_title),
        description = escape_attr(&route.description),
        canonical = escape_attr(&canonical),
        header = render_header(site_title),
        footer = render_footer(),
        main_class = main_class,
        body = body,
    )
}

fn render_header(site_title: &str) -> String {
    format!(
        r#"<header class="site-header">
  <a class="brand" href="/"><span class="brand-mark">F</span><span>{}</span></a>
  <nav class="top-nav" aria-label="Primary">
    <a href="/docs/intro/">Docs</a>
    <a href="/docs/charts/overview/">Charts</a>
    <a href="/reference/overview/overview/">Reference</a>
    <a href="/docs/cookbook/build-a-counter/">Cookbook</a>
  </nav>
  <button class="theme-toggle" type="button" data-theme-toggle aria-label="Toggle color theme">Theme</button>
</header>"#,
        escape_html(site_title)
    )
}

fn render_footer() -> &'static str {
    r#"<footer class="site-footer">
  <p>Copyright © 2026 Fission. The Fission framework is ready to use today but some areas are actively under development. Widget APIs are expected to remain stable but some runtime or shell APIs may get breaking changes before we get to a 1.0.0 release</p>
</footer>"#
}

fn render_sidebar(items: &[SidebarItem], current_path: &str) -> String {
    let mut html = String::from("<nav aria-label=\"Documentation\"><ul>");
    for item in items {
        let active = normalize_route_path(&item.href) == normalize_route_path(current_path);
        let group = if item.group { " sidebar-group" } else { "" };
        let active_class = if active { " is-active" } else { "" };
        html.push_str(&format!(
            "<li class=\"sidebar-level-{}{}{}\"><a href=\"{}\">{}</a></li>",
            item.level.min(4),
            group,
            active_class,
            escape_attr(&normalize_route_path(&item.href)),
            escape_html(&item.title)
        ));
    }
    html.push_str("</ul></nav>");
    html
}

fn render_toc(headings: &[HeadingLink]) -> String {
    if headings.is_empty() {
        return "<p class=\"toc-empty\">On this page</p>".into();
    }
    let mut html = String::from("<nav aria-label=\"On this page\"><p>On this page</p><ul>");
    for heading in headings {
        html.push_str(&format!(
            "<li><a href=\"#{}\">{}</a></li>",
            escape_attr(&heading.id),
            escape_html(&heading.title)
        ));
    }
    html.push_str("</ul></nav>");
    html
}

pub(crate) fn render_sitemap(site: &SiteConfig, routes: &[RenderedRoute]) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
    for route in routes {
        xml.push_str(&format!(
            "  <url><loc>{}</loc></url>\n",
            escape_html(&canonical_url(site, &route.path))
        ));
    }
    xml.push_str("</urlset>\n");
    xml
}

pub(crate) fn render_robots(site: &SiteConfig) -> String {
    format!(
        "User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml\n",
        site.base_url.trim_end_matches('/')
    )
}

pub(crate) fn write_assets(out_dir: &Path) -> Result<()> {
    let assets = out_dir.join("assets");
    fs::create_dir_all(&assets)?;
    fs::write(assets.join("site.css"), SITE_CSS)?;
    fs::write(assets.join("site.js"), SITE_JS)?;
    Ok(())
}

pub(crate) fn copy_site_assets(
    project_dir: &Path,
    site: &SiteConfig,
    out_dir: &Path,
) -> Result<()> {
    for asset_dir in &site.asset_dirs {
        let source = project_dir.join(asset_dir);
        if !source.exists() {
            bail!(
                "configured site asset directory does not exist: {}",
                source.display()
            );
        }
        copy_dir_contents(&source, out_dir)
            .with_context(|| format!("failed to copy site assets from {}", source.display()))?;
    }
    Ok(())
}

fn copy_dir_contents(source: &Path, target: &Path) -> Result<()> {
    for entry in
        fs::read_dir(source).with_context(|| format!("failed to read {}", source.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let target_path = target.join(entry.file_name());
        if path.is_dir() {
            fs::create_dir_all(&target_path)?;
            copy_dir_contents(&path, &target_path)?;
        } else if path.is_file() {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&path, &target_path).with_context(|| {
                format!(
                    "failed to copy {} to {}",
                    path.display(),
                    target_path.display()
                )
            })?;
        }
    }
    Ok(())
}
