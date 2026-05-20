use anyhow::{bail, Context, Result};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::app::{
    DocumentationTemplate, HtmlPage, MarkdownRender, PageRenderContext, StaticMarkdownPage,
    StaticMarkdownTemplate, StaticSiteApp,
};
use crate::config::{
    default_template, normalized_site_config, read_project_file, split_front_matter, BuildOptions,
    FrontMatter, SidebarFile, SidebarItem, SiteConfig, SiteRouteConfig,
};
use crate::markdown::markdown_to_html;
use crate::render::{
    copy_site_assets, render_robots, render_shell, render_sitemap, write_assets, ArtifactManifest,
    ManifestRoute, RenderedRoute,
};
use crate::utils::{
    content_route_path, normalize_route_path, output_path_for_route, title_from_path,
};

pub fn build_from_cli(app: StaticSiteApp) -> Result<()> {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "build".to_string());
    let mut project_dir = PathBuf::from(".");
    let mut release = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--project-dir" => {
                let value = args.next().context("--project-dir requires a value")?;
                project_dir = PathBuf::from(value);
            }
            "--release" => release = true,
            _ => bail!("unsupported site builder argument `{arg}`"),
        }
    }

    match command.as_str() {
        "build" | "--fission-site-build" => build_site(
            app,
            BuildOptions {
                project_dir,
                release,
            },
        ),
        "routes" => list_routes(app, &project_dir),
        "check" => check_site(
            app,
            BuildOptions {
                project_dir,
                release,
            },
        ),
        other => bail!("unsupported site command `{other}`"),
    }
}

pub fn build_site(app: StaticSiteApp, options: BuildOptions) -> Result<()> {
    let project = read_project_file(&options.project_dir)?;
    if !project.targets.is_empty() && !project.targets.contains("site") {
        bail!("fission.toml does not enable the `site` target");
    }
    let site = normalized_site_config(project.site);
    let out_dir = options.project_dir.join(&site.out_dir);
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir)
            .with_context(|| format!("failed to clear {}", out_dir.display()))?;
    }
    fs::create_dir_all(&out_dir)?;

    let sidebar = load_sidebar(&options.project_dir, site.sidebar.as_ref())?;
    let mut routes = Vec::new();
    routes.extend(render_custom_routes(
        &app,
        &options.project_dir,
        &site,
        &sidebar,
    )?);
    routes.extend(render_content_routes(
        &app,
        &options.project_dir,
        &site,
        &sidebar,
    )?);
    detect_route_conflicts(&routes)?;

    write_assets(&out_dir)?;
    copy_site_assets(&options.project_dir, &site, &out_dir)?;
    let mut manifest_routes = Vec::new();
    for route in &routes {
        let full = out_dir.join(&route.output);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent)?;
        }
        let document = render_shell(&site, route);
        fs::write(&full, document)
            .with_context(|| format!("failed to write {}", full.display()))?;
        manifest_routes.push(ManifestRoute {
            path: route.path.clone(),
            output: route.output.display().to_string(),
            title: route.title.clone(),
            locale: route.locale.clone(),
            source: route.source.clone(),
        });
    }

    if site.generate_sitemap {
        fs::write(out_dir.join("sitemap.xml"), render_sitemap(&site, &routes))?;
    }
    if site.generate_robots {
        fs::write(out_dir.join("robots.txt"), render_robots(&site))?;
    }
    let manifest = ArtifactManifest {
        base_url: site.base_url.clone(),
        routes: manifest_routes,
        assets: vec!["assets/site.css".into(), "assets/site.js".into()],
    };
    fs::write(
        out_dir.join("artifact-manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    println!(
        "Generated {} static routes into {}",
        routes.len(),
        out_dir.display()
    );
    Ok(())
}

pub fn check_site(app: StaticSiteApp, options: BuildOptions) -> Result<()> {
    build_site(app, options)
}

pub fn list_routes(app: StaticSiteApp, project_dir: &Path) -> Result<()> {
    let project = read_project_file(project_dir)?;
    let site = normalized_site_config(project.site);
    for route in app.routes() {
        println!("custom\t{}", route.path);
    }
    for route in &site.routes {
        for page in discover_content_pages(project_dir, route, &site.default_locale)? {
            println!("content\t{}\t{}", page.path, page.source_path.display());
        }
    }
    Ok(())
}

fn render_custom_routes(
    app: &StaticSiteApp,
    project_dir: &Path,
    site: &SiteConfig,
    sidebar: &[SidebarItem],
) -> Result<Vec<RenderedRoute>> {
    let mut routes = Vec::new();
    for route in app.routes() {
        let ctx = PageRenderContext {
            route_path: &route.path,
            project_dir,
            site,
        };
        let page = route.page.render(&ctx)?;
        routes.push(RenderedRoute {
            path: route.path.clone(),
            output: output_path_for_route(&route.path, site.pretty_urls),
            title: page.title,
            description: page.description,
            locale: site.default_locale.clone(),
            body: page.body,
            shell: page.template,
            sidebar: sidebar.to_vec(),
            h1_links: Vec::new(),
            source: "custom".into(),
        });
    }
    Ok(routes)
}

#[derive(Clone, Debug)]
struct ContentPage {
    path: String,
    source_path: PathBuf,
    front_matter: FrontMatter,
    body: String,
    locale: String,
}

fn render_content_routes(
    app: &StaticSiteApp,
    project_dir: &Path,
    site: &SiteConfig,
    default_sidebar: &[SidebarItem],
) -> Result<Vec<RenderedRoute>> {
    let mut routes = Vec::new();
    for route in &site.routes {
        if route.kind != "content" {
            bail!("unsupported site route kind `{}`", route.kind);
        }
        let sidebar = load_sidebar(project_dir, route.sidebar.as_ref())?
            .into_iter()
            .chain(default_sidebar.iter().cloned())
            .collect::<Vec<_>>();
        let sidebar = if sidebar.is_empty() {
            auto_sidebar(project_dir, route, &site.default_locale)?
        } else {
            sidebar
        };
        for page in discover_content_pages(project_dir, route, &site.default_locale)? {
            let parsed = markdown_to_html(&page.body);
            let route_path = page.path.clone();
            let markdown_page = StaticMarkdownPage {
                route_path: &route_path,
                source_path: &page.source_path,
                front_matter: &page.front_matter,
                body: &page.body,
                markdown: MarkdownRender::new(&page.body),
            };
            let ctx = PageRenderContext {
                route_path: &route_path,
                project_dir,
                site,
            };
            let template_id = page
                .front_matter
                .template()
                .unwrap_or(&route.template)
                .to_string();
            let rendered_page = render_markdown_template(app, &template_id, markdown_page, &ctx)?;
            let title = if rendered_page.title == "Documentation" {
                parsed
                    .h1_links
                    .first()
                    .map(|heading| heading.title.clone())
                    .unwrap_or_else(|| title_from_path(&page.source_path))
            } else {
                rendered_page.title
            };
            let description = if rendered_page.description.is_empty() {
                site.description.clone().unwrap_or_default()
            } else {
                rendered_page.description
            };
            routes.push(RenderedRoute {
                path: page.path.clone(),
                output: output_path_for_route(&page.path, site.pretty_urls),
                title,
                description,
                locale: page.locale.clone(),
                body: rendered_page.body,
                shell: rendered_page.template,
                sidebar: sidebar.clone(),
                h1_links: parsed
                    .h1_links
                    .into_iter()
                    .filter(|heading| heading.level == 1)
                    .collect(),
                source: page.source_path.display().to_string(),
            });
        }
    }
    Ok(routes)
}

fn render_markdown_template(
    app: &StaticSiteApp,
    template_id: &str,
    page: StaticMarkdownPage<'_>,
    ctx: &PageRenderContext<'_>,
) -> Result<HtmlPage> {
    if let Some(template) = app.template_for(template_id) {
        return template.render(page, ctx);
    }
    if template_id == default_template() {
        return DocumentationTemplate.render(page, ctx);
    }
    bail!("content page selected unknown static site template `{template_id}`")
}

fn discover_content_pages(
    project_dir: &Path,
    route: &SiteRouteConfig,
    default_locale_value: &str,
) -> Result<Vec<ContentPage>> {
    let root = project_dir.join(&route.source);
    let mut files = Vec::new();
    collect_markdown_files(&root, &mut files)?;
    files.sort();

    let mut pages = Vec::new();
    for source_path in files {
        let raw = fs::read_to_string(&source_path)
            .with_context(|| format!("failed to read {}", source_path.display()))?;
        let (front_matter, body) = split_front_matter(&raw);
        let rel = source_path
            .strip_prefix(&root)
            .unwrap_or(&source_path)
            .to_path_buf();
        let locale = front_matter
            .locale()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| default_locale_value.to_string());
        let mut path = content_route_path(&route.path, &rel, front_matter.slug());
        if locale != default_locale_value {
            path = format!("/{locale}{}", normalize_route_path(&path));
        }
        pages.push(ContentPage {
            path,
            source_path,
            front_matter,
            body,
            locale,
        });
    }
    Ok(pages)
}

fn collect_markdown_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if file_name.starts_with('.') || file_name == "node_modules" || file_name == "target" {
            continue;
        }
        if path.is_dir() {
            collect_markdown_files(&path, out)?;
        } else if matches!(
            path.extension().and_then(|value| value.to_str()),
            Some("md" | "mdx")
        ) {
            out.push(path);
        }
    }
    Ok(())
}

fn auto_sidebar(
    project_dir: &Path,
    route: &SiteRouteConfig,
    default_locale_value: &str,
) -> Result<Vec<SidebarItem>> {
    let mut items = Vec::new();
    for page in discover_content_pages(project_dir, route, default_locale_value)? {
        let title = page
            .front_matter
            .title()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| title_from_path(&page.source_path));
        let level = page
            .source_path
            .strip_prefix(project_dir.join(&route.source))
            .ok()
            .map(|path| path.components().count().saturating_sub(1))
            .unwrap_or(0);
        items.push(SidebarItem {
            title,
            href: page.path,
            level,
            group: false,
        });
    }
    Ok(items)
}

fn load_sidebar(project_dir: &Path, sidebar_path: Option<&PathBuf>) -> Result<Vec<SidebarItem>> {
    let Some(sidebar_path) = sidebar_path else {
        return Ok(Vec::new());
    };
    let path = project_dir.join(sidebar_path);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let parsed: SidebarFile =
        toml::from_str(&data).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(parsed.items)
}

fn detect_route_conflicts(routes: &[RenderedRoute]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for route in routes {
        if !seen.insert(route.path.clone()) {
            bail!("duplicate static route `{}`", route.path);
        }
    }
    Ok(())
}
