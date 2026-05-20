use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

use crate::config::{FrontMatter, SiteConfig};
use crate::markdown::{
    markdown_to_html, markdown_to_html_with_route, parse_markdown, ParsedMarkdown,
};
use crate::utils::normalize_route_path;

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
    pub(crate) page: Arc<dyn StaticPage>,
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

    pub(crate) fn template_for(&self, id: &str) -> Option<&dyn StaticMarkdownTemplate> {
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
