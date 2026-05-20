mod app;
mod builder;
mod config;
mod markdown;
mod render;
mod utils;

pub use app::{
    marketing_page, CustomRoute, DocumentationTemplate, HtmlPage, MarkdownRender,
    PageRenderContext, PageShell, Router, StaticMarkdownPage, StaticMarkdownTemplate, StaticPage,
    StaticSiteApp, TemplateId, TrustedStaticHtml,
};
pub use builder::{build_from_cli, build_site, check_site, list_routes};
pub use config::{
    BuildOptions, FrontMatter, SidebarFile, SidebarItem, SiteConfig, SiteRouteConfig,
};
pub use markdown::{
    markdown_to_html, markdown_to_html_with_route, HeadingLink, MarkdownHtml, ParsedMarkdown,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::content_route_path;
    use std::fs;
    use std::path::{Path, PathBuf};
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
                    level: 1,
                },
                HeadingLink {
                    id: "intro-2".into(),
                    title: "Intro".into(),
                    level: 1,
                },
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
