use anyhow::{bail, Context, Result};
use rushdown::ast::{
    Arena, CodeBlock, Heading, HtmlBlock, Image, KindData, Link, NodeRef, RawHtml, TableCell, Text,
    TextQualifier,
};
use rushdown::parser::{self, Parser};
use rushdown::text::BasicReader;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub type TemplateId = String;

#[derive(Clone, Debug, Default)]
pub struct Router {
    routes: Vec<CustomRoute>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn route<P>(mut self, path: impl Into<String>, page: P) -> Self
    where
        P: StaticPage + 'static,
    {
        self.routes.push(CustomRoute {
            path: normalize_route_path(&path.into()),
            page: Arc::new(page),
        });
        self
    }

    pub fn routes(&self) -> &[CustomRoute] {
        &self.routes
    }
}

#[derive(Clone)]
pub struct CustomRoute {
    pub path: String,
    page: Arc<dyn StaticPage>,
}

impl std::fmt::Debug for CustomRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CustomRoute")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

#[derive(Clone)]
pub struct StaticSiteApp {
    router: Router,
    templates: BTreeMap<TemplateId, Arc<dyn StaticMarkdownTemplate>>,
}

impl std::fmt::Debug for StaticSiteApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticSiteApp")
            .field("router", &self.router)
            .field("templates", &self.templates.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl StaticSiteApp {
    pub fn new(router: Router) -> Self {
        Self {
            router,
            templates: BTreeMap::new(),
        }
    }

    pub fn routes(&self) -> &[CustomRoute] {
        self.router.routes()
    }

    pub fn template<T>(mut self, id: impl Into<TemplateId>, template: T) -> Self
    where
        T: StaticMarkdownTemplate + 'static,
    {
        self.templates.insert(id.into(), Arc::new(template));
        self
    }

    fn template_for(&self, id: &str) -> Option<&dyn StaticMarkdownTemplate> {
        self.templates.get(id).map(|template| template.as_ref())
    }
}

pub trait StaticPage: Send + Sync {
    fn render(&self, ctx: &PageRenderContext<'_>) -> Result<HtmlPage>;
}

impl<F> StaticPage for F
where
    F: Fn(&PageRenderContext<'_>) -> Result<HtmlPage> + Send + Sync,
{
    fn render(&self, ctx: &PageRenderContext<'_>) -> Result<HtmlPage> {
        self(ctx)
    }
}

#[derive(Clone, Debug)]
pub struct PageRenderContext<'a> {
    pub route_path: &'a str,
    pub project_dir: &'a Path,
    pub site: &'a SiteConfig,
}

#[derive(Clone, Debug, Default)]
pub struct HtmlPage {
    pub title: String,
    pub description: String,
    pub body: String,
    pub template: PageShell,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PageShell {
    #[default]
    Marketing,
    Documentation,
}

#[derive(Clone, Debug, Default)]
pub struct DocumentationTemplate;

pub trait StaticMarkdownTemplate: Send + Sync {
    fn render(&self, page: StaticMarkdownPage<'_>, ctx: &PageRenderContext<'_>)
        -> Result<HtmlPage>;
}

impl StaticMarkdownTemplate for DocumentationTemplate {
    fn render(
        &self,
        page: StaticMarkdownPage<'_>,
        _ctx: &PageRenderContext<'_>,
    ) -> Result<HtmlPage> {
        let rendered = page.markdown.html_for_route(page.route_path);
        Ok(HtmlPage {
            title: page
                .front_matter
                .title()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| "Documentation".to_string()),
            description: page
                .front_matter
                .description()
                .map(ToOwned::to_owned)
                .unwrap_or_default(),
            body: rendered.0,
            template: PageShell::Documentation,
        })
    }
}

#[derive(Clone, Debug)]
pub struct StaticMarkdownPage<'a> {
    pub route_path: &'a str,
    pub source_path: &'a Path,
    pub front_matter: &'a FrontMatter,
    pub body: &'a str,
    pub markdown: MarkdownRender<'a>,
}

#[derive(Clone, Debug)]
pub struct MarkdownRender<'a> {
    body: &'a str,
}

impl<'a> MarkdownRender<'a> {
    pub fn new(body: &'a str) -> Self {
        Self { body }
    }

    pub fn body(&self) -> &'a str {
        self.body
    }

    pub fn parse(&self) -> ParsedMarkdown {
        parse_markdown(self.body)
    }

    pub fn html(&self) -> TrustedStaticHtml {
        TrustedStaticHtml(markdown_to_html(self.body).html)
    }

    pub fn html_for_route(&self, route_path: &str) -> TrustedStaticHtml {
        TrustedStaticHtml(markdown_to_html_with_route(self.body, route_path).html)
    }
}

#[derive(Clone, Debug)]
pub struct TrustedStaticHtml(pub String);

#[derive(Debug)]
pub struct ParsedMarkdown {
    pub source: String,
    pub arena: Arena,
    pub document: NodeRef,
}

#[derive(Clone, Debug, Default)]
pub struct MarkdownHtml {
    pub html: String,
    pub h1_links: Vec<HeadingLink>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HeadingLink {
    pub id: String,
    pub title: String,
    pub level: u8,
}

#[derive(Clone, Debug, Default)]
pub struct BuildOptions {
    pub project_dir: PathBuf,
    pub release: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct ProjectFile {
    site: SiteConfig,
    #[serde(default)]
    targets: BTreeSet<String>,
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

#[derive(Clone, Debug, Serialize)]
struct ArtifactManifest {
    base_url: String,
    routes: Vec<ManifestRoute>,
    assets: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct ManifestRoute {
    path: String,
    output: String,
    title: String,
    locale: String,
    source: String,
}

#[derive(Clone, Debug)]
struct RenderedRoute {
    path: String,
    output: PathBuf,
    title: String,
    description: String,
    locale: String,
    body: String,
    shell: PageShell,
    sidebar: Vec<SidebarItem>,
    h1_links: Vec<HeadingLink>,
    source: String,
}

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

fn read_project_file(project_dir: &Path) -> Result<ProjectFile> {
    let path = project_dir.join("fission.toml");
    let data =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&data).with_context(|| format!("failed to parse {}", path.display()))
}

fn normalized_site_config(mut site: SiteConfig) -> SiteConfig {
    if site.routes.is_empty() {
        site.routes.push(SiteRouteConfig::default());
    }
    if site.locales.is_empty() {
        site.locales.push(site.default_locale.clone());
    }
    site
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

fn parse_markdown(source: &str) -> ParsedMarkdown {
    let parser = Parser::with_extensions(
        parser::Options::default(),
        parser::gfm(parser::GfmOptions::default()),
    );
    let mut reader = BasicReader::new(source);
    let (arena, document) = parser.parse(&mut reader);
    ParsedMarkdown {
        source: source.to_string(),
        arena,
        document,
    }
}

pub fn markdown_to_html(source: &str) -> MarkdownHtml {
    markdown_to_html_with_base(source, None)
}

pub fn markdown_to_html_with_route(source: &str, route_path: &str) -> MarkdownHtml {
    markdown_to_html_with_base(source, Some(route_path))
}

fn markdown_to_html_with_base(source: &str, route_path: Option<&str>) -> MarkdownHtml {
    let parsed = parse_markdown(source);
    let mut renderer = MarkdownHtmlRenderer::new(&parsed.source, &parsed.arena, route_path);
    renderer.render_document(parsed.document);
    MarkdownHtml {
        html: renderer.html,
        h1_links: renderer.headings,
    }
}

struct MarkdownHtmlRenderer<'a> {
    source: &'a str,
    arena: &'a Arena,
    html: String,
    headings: Vec<HeadingLink>,
    used_ids: BTreeSet<String>,
    route_path: Option<&'a str>,
}

impl<'a> MarkdownHtmlRenderer<'a> {
    fn new(source: &'a str, arena: &'a Arena, route_path: Option<&'a str>) -> Self {
        Self {
            source,
            arena,
            html: String::new(),
            headings: Vec::new(),
            used_ids: BTreeSet::new(),
            route_path,
        }
    }

    fn render_document(&mut self, node_ref: NodeRef) {
        for child in self.arena[node_ref].children(self.arena) {
            self.block(child);
        }
    }

    fn block(&mut self, node_ref: NodeRef) {
        match self.arena[node_ref].kind_data() {
            KindData::Document(_) => self.render_document(node_ref),
            KindData::Paragraph(_) => {
                self.html.push_str("<p>");
                self.inline_children(node_ref);
                self.html.push_str("</p>\n");
            }
            KindData::Heading(heading) => self.heading(node_ref, heading),
            KindData::ThematicBreak(_) => self.html.push_str("<hr />\n"),
            KindData::CodeBlock(code) => self.code_block(code),
            KindData::Blockquote(_) => {
                self.html.push_str("<blockquote>");
                self.render_document(node_ref);
                self.html.push_str("</blockquote>\n");
            }
            KindData::List(list) => {
                let tag = if list.is_ordered() { "ol" } else { "ul" };
                if list.is_ordered() && list.start() > 1 {
                    self.html
                        .push_str(&format!("<{tag} start=\"{}\">", list.start()));
                } else {
                    self.html.push_str(&format!("<{tag}>"));
                }
                for child in self.arena[node_ref].children(self.arena) {
                    self.block(child);
                }
                self.html.push_str(&format!("</{tag}>\n"));
            }
            KindData::ListItem(_) => {
                self.html.push_str("<li>");
                self.render_document(node_ref);
                self.html.push_str("</li>");
            }
            KindData::HtmlBlock(html) => self.html_block(html),
            KindData::Table(_) => {
                self.html.push_str("<table>");
                for child in self.arena[node_ref].children(self.arena) {
                    self.block(child);
                }
                self.html.push_str("</table>\n");
            }
            KindData::TableHeader(_) => {
                self.html.push_str("<thead>");
                for child in self.arena[node_ref].children(self.arena) {
                    self.block(child);
                }
                self.html.push_str("</thead>");
            }
            KindData::TableBody(_) => {
                self.html.push_str("<tbody>");
                for child in self.arena[node_ref].children(self.arena) {
                    self.block(child);
                }
                self.html.push_str("</tbody>");
            }
            KindData::TableRow(_) => {
                self.html.push_str("<tr>");
                for child in self.arena[node_ref].children(self.arena) {
                    self.block(child);
                }
                self.html.push_str("</tr>");
            }
            KindData::TableCell(cell) => self.table_cell(node_ref, cell),
            KindData::LinkReferenceDefinition(_) => {}
            _ => {
                self.html.push_str("<p>");
                self.inline(node_ref);
                self.html.push_str("</p>\n");
            }
        }
    }

    fn heading(&mut self, node_ref: NodeRef, heading: &Heading) {
        let level = heading.level().clamp(1, 6);
        let text = plain_text(self.source, self.arena, node_ref);
        let id = self.unique_id(&slugify(&text));
        if level == 1 {
            self.headings.push(HeadingLink {
                id: id.clone(),
                title: text.clone(),
                level,
            });
        }
        self.html
            .push_str(&format!("<h{level} id=\"{}\">", escape_attr(&id)));
        self.inline_children(node_ref);
        self.html.push_str(&format!("</h{level}>\n"));
    }

    fn unique_id(&mut self, base: &str) -> String {
        let base = if base.is_empty() { "section" } else { base };
        let mut candidate = base.to_string();
        let mut suffix = 2;
        while self.used_ids.contains(&candidate) {
            candidate = format!("{base}-{suffix}");
            suffix += 1;
        }
        self.used_ids.insert(candidate.clone());
        candidate
    }

    fn code_block(&mut self, code: &CodeBlock) {
        let text = code
            .value()
            .iter(self.source)
            .fold(String::new(), |mut out, line| {
                out.push_str(line.as_ref());
                out
            });
        let class = code
            .language_str(self.source)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|language| format!(" class=\"language-{}\"", escape_attr(language)))
            .unwrap_or_default();
        self.html.push_str(&format!(
            "<pre><code{class}>{}</code></pre>\n",
            escape_html(&text)
        ));
    }

    fn html_block(&mut self, html: &HtmlBlock) {
        let text = html
            .value()
            .iter(self.source)
            .fold(String::new(), |mut out, line| {
                out.push_str(line.as_ref());
                out
            });
        self.html.push_str(&text);
        self.html.push('\n');
    }

    fn table_cell(&mut self, node_ref: NodeRef, cell: &TableCell) {
        let align = match cell.alignment().as_str() {
            "left" | "center" | "right" => {
                format!(" style=\"text-align:{}\"", cell.alignment().as_str())
            }
            _ => String::new(),
        };
        self.html.push_str(&format!("<td{align}>"));
        self.inline_children(node_ref);
        self.html.push_str("</td>");
    }

    fn inline_children(&mut self, node_ref: NodeRef) {
        for child in self.arena[node_ref].children(self.arena) {
            self.inline(child);
        }
    }

    fn inline(&mut self, node_ref: NodeRef) {
        match self.arena[node_ref].kind_data() {
            KindData::Text(text) => self.text(text),
            KindData::CodeSpan(code) => self.html.push_str(&format!(
                "<code>{}</code>",
                escape_html(&code.str(self.source))
            )),
            KindData::Emphasis(_) => {
                self.html.push_str("<em>");
                self.inline_children(node_ref);
                self.html.push_str("</em>");
            }
            KindData::Strong(_) => {
                self.html.push_str("<strong>");
                self.inline_children(node_ref);
                self.html.push_str("</strong>");
            }
            KindData::Strikethrough(_) => {
                self.html.push_str("<del>");
                self.inline_children(node_ref);
                self.html.push_str("</del>");
            }
            KindData::Link(link) => self.link(node_ref, link),
            KindData::Image(image) => self.image(node_ref, image),
            KindData::RawHtml(raw) => self.raw_html(raw),
            _ => self.inline_children(node_ref),
        }
    }

    fn text(&mut self, text: &Text) {
        self.html.push_str(&escape_html(text.str(self.source)));
        if text.has_qualifiers(TextQualifier::HARD_LINE_BREAK) {
            self.html.push_str("<br />");
        } else if text.has_qualifiers(TextQualifier::SOFT_LINE_BREAK) {
            self.html.push(' ');
        }
    }

    fn link(&mut self, node_ref: NodeRef, link: &Link) {
        let href = normalize_markdown_href(link.destination_str(self.source), self.route_path);
        self.html
            .push_str(&format!("<a href=\"{}\">", escape_attr(&href)));
        self.inline_children(node_ref);
        self.html.push_str("</a>");
    }

    fn image(&mut self, node_ref: NodeRef, image: &Image) {
        let alt = plain_text(self.source, self.arena, node_ref);
        self.html.push_str(&format!(
            "<img src=\"{}\" alt=\"{}\" />",
            escape_attr(image.destination_str(self.source)),
            escape_attr(&alt)
        ));
    }

    fn raw_html(&mut self, raw: &RawHtml) {
        self.html.push_str(&raw.str(self.source));
    }
}

fn render_shell(site: &SiteConfig, route: &RenderedRoute) -> String {
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

fn render_sitemap(site: &SiteConfig, routes: &[RenderedRoute]) -> String {
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

fn render_robots(site: &SiteConfig) -> String {
    format!(
        "User-agent: *\nAllow: /\nSitemap: {}/sitemap.xml\n",
        site.base_url.trim_end_matches('/')
    )
}

fn write_assets(out_dir: &Path) -> Result<()> {
    let assets = out_dir.join("assets");
    fs::create_dir_all(&assets)?;
    fs::write(assets.join("site.css"), SITE_CSS)?;
    fs::write(assets.join("site.js"), SITE_JS)?;
    Ok(())
}

fn copy_site_assets(project_dir: &Path, site: &SiteConfig, out_dir: &Path) -> Result<()> {
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

fn split_front_matter(raw: &str) -> (FrontMatter, String) {
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

fn plain_text(source: &str, arena: &Arena, node_ref: NodeRef) -> String {
    let mut out = String::new();
    match arena[node_ref].kind_data() {
        KindData::Text(text) => out.push_str(text.str(source)),
        KindData::CodeSpan(code) => out.push_str(&code.str(source)),
        _ => {
            for child in arena[node_ref].children(arena) {
                out.push_str(&plain_text(source, arena, child));
            }
        }
    }
    out
}

fn content_route_path(base: &str, relative: &Path, slug: Option<&str>) -> String {
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

fn output_path_for_route(path: &str, pretty: bool) -> PathBuf {
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

fn normalize_route_path(path: &str) -> String {
    let mut out = String::new();
    out.push('/');
    out.push_str(path.trim().trim_matches('/'));
    if out != "/" && !out.ends_with('/') {
        out.push('/');
    }
    out
}

fn normalize_markdown_href(href: &str, route_path: Option<&str>) -> String {
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

fn title_from_path(path: &Path) -> String {
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

fn slugify(value: &str) -> String {
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

fn canonical_url(site: &SiteConfig, path: &str) -> String {
    format!(
        "{}{}",
        site.base_url.trim_end_matches('/'),
        normalize_route_path(path)
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(value: &str) -> String {
    escape_html(value).replace('"', "&quot;")
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

fn default_template() -> TemplateId {
    "fission::site::DocumentationTemplate".into()
}

pub fn marketing_page(
    title: impl Into<String>,
    description: impl Into<String>,
    body: impl Into<String>,
) -> HtmlPage {
    HtmlPage {
        title: title.into(),
        description: description.into(),
        body: body.into(),
        template: PageShell::Marketing,
    }
}

const SITE_JS: &str = r#"
(() => {
  const key = 'fission-site-theme';
  const root = document.documentElement;
  const apply = (theme) => {
    root.dataset.theme = theme;
    try { localStorage.setItem(key, theme); } catch (_) {}
  };
  let current = 'light';
  try { current = localStorage.getItem(key) || (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'); } catch (_) {}
  apply(current);
  document.addEventListener('click', (event) => {
    const button = event.target.closest('[data-theme-toggle]');
    if (!button) return;
    apply(root.dataset.theme === 'dark' ? 'light' : 'dark');
  });
})();
"#;

const SITE_CSS: &str = r#"
:root {
  --fs-surface: #ffffff;
  --fs-surface-soft: #f8fafc;
  --fs-surface-raised: #ffffff;
  --fs-fg: #182230;
  --fs-muted: #667085;
  --fs-border: #e4e7ec;
  --fs-heading: #101828;
  --fs-teal: #0f766e;
  --fs-blue: #2563eb;
  --fs-orange: #ea580c;
  --fs-shadow: 0 18px 50px rgba(16, 24, 40, 0.08);
  --fs-font-sans: ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  --fs-font-serif: ui-serif, Georgia, Cambria, "Times New Roman", serif;
  --fs-font-mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

:root[data-theme='dark'] {
  --fs-surface: #0b1220;
  --fs-surface-soft: #111827;
  --fs-surface-raised: #172033;
  --fs-fg: #e5e7eb;
  --fs-muted: #a3aab8;
  --fs-border: #2b3648;
  --fs-heading: #f9fafb;
  --fs-teal: #2dd4bf;
  --fs-blue: #60a5fa;
  --fs-orange: #fb923c;
  --fs-shadow: 0 18px 50px rgba(0, 0, 0, 0.32);
}

* { box-sizing: border-box; }
html { scroll-behavior: smooth; }
body { margin: 0; background: var(--fs-surface); color: var(--fs-fg); font-family: var(--fs-font-sans); line-height: 1.65; }
a { color: var(--fs-teal); text-decoration: none; }
a:hover { text-decoration: underline; }

.site-header {
  position: sticky; top: 0; z-index: 40; display: flex; align-items: center; justify-content: space-between;
  min-height: 64px; padding: 0 32px; border-bottom: 1px solid var(--fs-border);
  background: color-mix(in srgb, var(--fs-surface) 92%, transparent); backdrop-filter: blur(16px);
}
.brand { display: inline-flex; gap: 10px; align-items: center; color: var(--fs-heading); font-weight: 750; }
.brand-mark { display: inline-grid; place-items: center; width: 32px; height: 32px; border-radius: 10px; background: var(--fs-teal); color: white; font-family: var(--fs-font-serif); }
.top-nav { display: flex; align-items: center; gap: 22px; font-size: 14px; font-weight: 650; }
.top-nav a { color: var(--fs-muted); }
.theme-toggle { border: 1px solid var(--fs-border); border-radius: 999px; background: var(--fs-surface-raised); color: var(--fs-fg); padding: 8px 12px; cursor: pointer; }

.site-main-marketing { overflow: hidden; }
.hero { padding: 96px 32px 82px; text-align: center; background: radial-gradient(900px 520px at 15% -15%, color-mix(in srgb, var(--fs-teal) 14%, transparent), transparent 60%), radial-gradient(860px 520px at 90% -10%, color-mix(in srgb, var(--fs-blue) 14%, transparent), transparent 60%); }
.hero-pill { display: inline-flex; border: 1px solid var(--fs-border); background: var(--fs-surface-raised); color: var(--fs-muted); border-radius: 999px; padding: 7px 14px; font-size: 13px; font-weight: 650; box-shadow: var(--fs-shadow); }
.hero h1 { max-width: 840px; margin: 26px auto 0; color: var(--fs-heading); font-family: var(--fs-font-serif); font-size: clamp(2.8rem, 7vw, 5rem); line-height: 0.98; letter-spacing: -0.035em; }
.hero .lead { max-width: 820px; margin: 24px auto 0; color: var(--fs-muted); font-size: clamp(1.05rem, 2vw, 1.28rem); }
.hero-actions { margin-top: 34px; display: flex; gap: 12px; justify-content: center; flex-wrap: wrap; }
.btn { display: inline-flex; align-items: center; justify-content: center; border-radius: 12px; padding: 12px 18px; font-weight: 750; border: 1px solid var(--fs-border); }
.btn.primary { background: var(--fs-teal); color: white; border-color: var(--fs-teal); }
.btn.secondary { color: var(--fs-heading); background: var(--fs-surface-raised); }
.command-grid, .signal-grid, .target-grid, .chart-strip { max-width: 1180px; margin: 48px auto 0; padding: 0 32px; display: grid; gap: 18px; }
.command-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); max-width: 820px; }
.command-card, .signal-card, .target-card, .chart-card { border: 1px solid var(--fs-border); background: var(--fs-surface-raised); border-radius: 18px; padding: 20px; box-shadow: var(--fs-shadow); }
.command-card code { font-family: var(--fs-font-mono); color: var(--fs-heading); }
.section { padding: 88px 32px; }
.section-inner { max-width: 1180px; margin: 0 auto; }
.section h2 { max-width: 760px; margin: 0; color: var(--fs-heading); font-family: var(--fs-font-serif); font-size: clamp(2rem, 4vw, 3rem); line-height: 1.08; letter-spacing: -0.02em; }
.section .section-lead { max-width: 760px; margin: 16px 0 0; color: var(--fs-muted); font-size: 18px; }
.signal-grid { grid-template-columns: repeat(4, minmax(0, 1fr)); padding: 0; }
.signal-card h3, .target-card h3, .chart-card h3 { margin: 12px 0 8px; color: var(--fs-heading); }
.signal-icon { width: 42px; height: 42px; border-radius: 14px; display: grid; place-items: center; background: color-mix(in srgb, var(--fs-teal) 14%, transparent); color: var(--fs-teal); font-weight: 800; }
.architecture { background: var(--fs-surface-soft); }
.arch-steps { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 14px; margin-top: 36px; }
.arch-step { padding: 20px; border-left: 3px solid var(--fs-teal); background: var(--fs-surface-raised); border-radius: 14px; }
.arch-step span { font-family: var(--fs-font-mono); color: var(--fs-teal); font-size: 12px; font-weight: 800; }
.target-grid { grid-template-columns: repeat(4, minmax(0, 1fr)); padding: 0; }
.chart-strip { grid-template-columns: repeat(3, minmax(0, 1fr)); padding: 0; }
.chart-preview { height: 160px; border-radius: 14px; background: linear-gradient(135deg, color-mix(in srgb, var(--fs-teal) 25%, transparent), color-mix(in srgb, var(--fs-blue) 20%, transparent)); display: flex; align-items: end; gap: 8px; padding: 18px; }
.chart-preview span { flex: 1; border-radius: 8px 8px 0 0; background: color-mix(in srgb, var(--fs-teal) 80%, white); min-height: 30px; }
.chart-preview span:nth-child(2) { min-height: 75px; background: color-mix(in srgb, var(--fs-blue) 75%, white); }
.chart-preview span:nth-child(3) { min-height: 115px; background: color-mix(in srgb, var(--fs-orange) 75%, white); }

.docs-layout { display: grid; grid-template-columns: minmax(220px, 280px) minmax(0, 820px) minmax(180px, 240px); gap: 32px; max-width: 1440px; margin: 0 auto; padding: 32px; }
.docs-sidebar, .docs-toc { position: sticky; top: 84px; align-self: start; max-height: calc(100vh - 108px); overflow: auto; }
.docs-sidebar nav, .docs-toc nav { font-size: 14px; }
.docs-sidebar ul, .docs-toc ul { list-style: none; padding: 0; margin: 0; }
.docs-sidebar li { margin: 2px 0; }
.docs-sidebar a, .docs-toc a { display: block; color: var(--fs-muted); border-radius: 8px; padding: 6px 8px; }
.docs-sidebar .is-active a { color: var(--fs-teal); background: color-mix(in srgb, var(--fs-teal) 10%, transparent); font-weight: 750; }
.sidebar-level-1 { padding-left: 12px; }
.sidebar-level-2 { padding-left: 24px; }
.sidebar-level-3 { padding-left: 36px; }
.sidebar-group a { margin-top: 12px; color: var(--fs-heading); font-weight: 800; }
.docs-toc p { margin: 0 0 8px; color: var(--fs-heading); font-weight: 800; }
.docs-content { min-width: 0; }
.markdown h1 { color: var(--fs-heading); font-family: var(--fs-font-serif); font-size: clamp(2.2rem, 5vw, 3.5rem); line-height: 1.05; letter-spacing: -0.025em; margin: 12px 0 24px; }
.markdown h2 { color: var(--fs-heading); margin-top: 44px; font-size: 1.75rem; line-height: 1.2; }
.markdown h3 { color: var(--fs-heading); margin-top: 30px; font-size: 1.25rem; }
.markdown p, .markdown li { color: var(--fs-fg); }
.markdown code { background: color-mix(in srgb, var(--fs-teal) 10%, transparent); border-radius: 5px; padding: 2px 5px; font-family: var(--fs-font-mono); }
.markdown pre { overflow: auto; padding: 16px; border: 1px solid var(--fs-border); border-radius: 14px; background: var(--fs-surface-soft); }
.markdown pre code { background: transparent; padding: 0; }
.markdown table { width: 100%; border-collapse: collapse; margin: 24px 0; font-size: 14px; }
.markdown td, .markdown th { border: 1px solid var(--fs-border); padding: 10px 12px; vertical-align: top; }
.markdown blockquote { margin: 24px 0; border-left: 4px solid var(--fs-teal); padding: 10px 18px; background: color-mix(in srgb, var(--fs-teal) 8%, transparent); border-radius: 0 12px 12px 0; }
.chart-catalog-groups { display: grid; gap: 48px; margin: 28px 0; }
.chart-family-header p, .chart-compact-card p, .chart-summary-card p { margin: 0 0 6px; color: var(--fs-muted); font-size: 12px; font-weight: 800; letter-spacing: 0.08em; text-transform: uppercase; }
.chart-family-header h2, .chart-compact-card h3, .chart-summary-card h3 { margin: 0; color: var(--fs-heading); }
.chart-catalog-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 18px; }
.chart-card, .chart-compact-card, .chart-summary-card { display: block; color: inherit; text-decoration: none; border: 1px solid var(--fs-border); border-radius: 18px; background: var(--fs-surface-raised); box-shadow: var(--fs-shadow); overflow: hidden; }
.chart-card:hover, .chart-compact-card:hover, .chart-summary-card:hover { text-decoration: none; border-color: color-mix(in srgb, var(--fs-teal) 45%, var(--fs-border)); }
.chart-card img { display: block; width: 100%; aspect-ratio: 640 / 420; object-fit: cover; background: var(--fs-surface-soft); }
.chart-card-body { padding: 18px; }
.chart-card-body h3 { margin: 0 0 8px; color: var(--fs-heading); font-size: 18px; }
.chart-card-body p, .chart-card-body dd { margin: 0; color: var(--fs-fg); }
.chart-card-body dl { display: grid; gap: 10px; margin: 14px 0 0; }
.chart-card-body dt { color: var(--fs-muted); font-size: 12px; font-weight: 800; letter-spacing: 0.08em; text-transform: uppercase; }
.chart-tags { display: flex; flex-wrap: wrap; gap: 6px; margin-top: 14px; }
.chart-tags span { border-radius: 999px; padding: 3px 8px; background: color-mix(in srgb, var(--fs-teal) 10%, transparent); color: var(--fs-heading); font-size: 12px; font-weight: 650; }
.chart-compact-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 14px; margin: 24px 0; }
.chart-compact-card { display: grid; grid-template-columns: 44% 1fr; gap: 14px; align-items: center; padding: 10px; }
.chart-compact-card img { display: block; width: 100%; border-radius: 12px; aspect-ratio: 640 / 420; object-fit: cover; background: var(--fs-surface-soft); }
.chart-summary-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 14px; margin: 24px 0; }
.chart-summary-card { padding: 18px; }
.chart-summary-card span { display: inline-block; margin-top: 8px; color: var(--fs-teal); font-weight: 750; }
.site-footer { border-top: 1px solid var(--fs-border); padding: 32px; color: var(--fs-muted); font-size: 13px; text-align: center; }

@media (max-width: 1050px) {
  .docs-layout { grid-template-columns: minmax(0, 1fr); }
  .docs-sidebar, .docs-toc { position: static; max-height: none; }
  .signal-grid, .target-grid, .arch-steps, .chart-strip, .chart-summary-grid, .chart-compact-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
}

@media (max-width: 720px) {
  .site-header { padding: 0 16px; }
  .top-nav { display: none; }
  .command-grid, .signal-grid, .target-grid, .arch-steps, .chart-strip, .chart-catalog-grid, .chart-summary-grid, .chart-compact-grid { grid-template-columns: 1fr; }
  .chart-compact-card { grid-template-columns: 1fr; }
  .docs-layout { padding: 20px; }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn content_route_path_respects_front_matter_slug() {
        let route = content_route_path("/docs", Path::new("ignored/name.md"), Some("/guide/start"));
        assert_eq!(route, "/docs/guide/start/");
    }

    #[test]
    fn markdown_html_collects_unique_h1_links() {
        let rendered = markdown_to_html("# Intro\n\n# Intro\n\n## Detail\n");
        assert!(rendered.html.contains("id=\"intro\""));
        assert!(rendered.html.contains("id=\"intro-2\""));
        assert_eq!(
            rendered.h1_links,
            vec![
                HeadingLink {
                    id: "intro".into(),
                    title: "Intro".into(),
                    level: 1
                },
                HeadingLink {
                    id: "intro-2".into(),
                    title: "Intro".into(),
                    level: 1
                }
            ]
        );
    }

    #[test]
    fn markdown_html_resolves_relative_links_against_route() {
        let rendered = markdown_to_html_with_route(
            "[Overview](./overview)\n\n[Parent](../core/widget-trait.mdx#api)",
            "/reference/charts/cartesian/line-basic/",
        );
        assert!(rendered
            .html
            .contains("href=\"/reference/charts/cartesian/overview/\""));
        assert!(rendered
            .html
            .contains("href=\"/reference/charts/core/widget-trait/#api\""));
    }

    #[test]
    fn build_site_renders_content_and_copies_static_assets() {
        let root = temp_project_dir("fission_site_build");
        fs::create_dir_all(root.join("content/docs")).unwrap();
        fs::create_dir_all(root.join("static/img")).unwrap();
        fs::write(root.join("static/img/logo.txt"), "asset").unwrap();
        fs::write(
            root.join("content/docs/intro.md"),
            "---\ntitle: Intro\ndescription: Intro page\n---\n\n# Intro\n\nWelcome.\n",
        )
        .unwrap();
        fs::write(
            root.join("fission.toml"),
            r#"
targets = ["site"]

[site]
out_dir = "dist/site"
asset_dirs = ["static"]

[[site.routes]]
kind = "content"
path = "/docs"
source = "content/docs"
"#,
        )
        .unwrap();

        let app = StaticSiteApp::new(Router::new().route("/", |_ctx: &PageRenderContext<'_>| {
            Ok(marketing_page(
                "Home",
                "Home page",
                "<section>Home</section>",
            ))
        }));
        build_site(
            app,
            BuildOptions {
                project_dir: root.clone(),
                release: false,
            },
        )
        .unwrap();

        assert!(root.join("dist/site/index.html").exists());
        assert!(root.join("dist/site/docs/intro/index.html").exists());
        assert_eq!(
            fs::read_to_string(root.join("dist/site/img/logo.txt")).unwrap(),
            "asset"
        );
        fs::remove_dir_all(root).unwrap();
    }

    fn temp_project_dir(prefix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), stamp));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
