use fission_core::op::{AlignItems, Color, Fill, JustifyContent};
use fission_core::ui::{Column, Container, Row, Text};
use fission_core::{AppState, BuildCtx, Node, View, Widget};
use fission_widgets::MarkdownViewer;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(crate) struct ContentRoute {
    pub path: String,
    pub title: String,
    pub description: Option<String>,
    pub body: String,
    pub headings: Vec<HeadingLink>,
    pub sidebar: Vec<SidebarLink>,
    pub source_path: PathBuf,
    pub rendered: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HeadingLink {
    pub title: String,
    pub anchor: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SidebarLink {
    pub title: String,
    pub href: String,
    pub level: usize,
    pub group: bool,
}

#[derive(Debug, Default)]
pub(crate) struct SitePageState;
impl AppState for SitePageState {}

pub(crate) struct DocumentationPage<'a> {
    pub site_title: &'a str,
    pub route: &'a ContentRoute,
    pub all_routes: &'a [ContentRoute],
}

impl<'a> Widget<SitePageState> for DocumentationPage<'a> {
    fn build(&self, ctx: &mut BuildCtx<SitePageState>, view: &View<SitePageState>) -> Node {
        Column {
            children: vec![self.header(), self.document_grid(ctx, view)],
            flex_grow: 1.0,
            ..Default::default()
        }
        .into_node()
    }
}

impl DocumentationPage<'_> {
    fn header(&self) -> Node {
        Container::new(
            Row {
                children: vec![
                    Text::new(self.site_title)
                        .size(18.0)
                        .weight(760)
                        .color(text_color())
                        .into_node(),
                    Text::new("Static site")
                        .size(13.0)
                        .color(muted_color())
                        .into_node(),
                ],
                gap: Some(16.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            }
            .into_node(),
        )
        .padding([24.0, 24.0, 18.0, 18.0])
        .into_node()
    }

    fn document_grid(&self, ctx: &mut BuildCtx<SitePageState>, view: &View<SitePageState>) -> Node {
        Row {
            children: vec![self.sidebar(), self.article(ctx, view), self.toc()],
            gap: Some(28.0),
            align_items: AlignItems::Start,
            ..Default::default()
        }
        .into_node()
    }

    fn sidebar(&self) -> Node {
        let mut children = vec![Text::new("Content")
            .size(12.0)
            .weight(760)
            .color(muted_color())
            .into_node()];
        if self.route.sidebar.is_empty() {
            let section_prefix = section_prefix(&self.route.path);
            for route in self
                .all_routes
                .iter()
                .filter(|route| route.path.starts_with(section_prefix))
            {
                children.push(self.sidebar_item(&route.title, &route.path, 0, false));
            }
        } else {
            for item in &self.route.sidebar {
                children.push(self.sidebar_item(&item.title, &item.href, item.level, item.group));
            }
        }
        Container::new(
            Column {
                children,
                gap: Some(10.0),
                ..Default::default()
            }
            .into_node(),
        )
        .padding_all(18.0)
        .bg_fill(Fill::Solid(surface_soft_color()))
        .border(border_color(), 1.0)
        .border_radius(20.0)
        .width(260.0)
        .into_node()
    }

    fn sidebar_item(&self, title: &str, href: &str, level: usize, group: bool) -> Node {
        let active = normalize_link_path(href) == self.route.path;
        let color = if active { accent_color() } else { text_color() };
        Container::new(
            Text::new(title.to_string())
                .size(if group { 13.0 } else { 14.0 })
                .weight(if active || group { 760 } else { 500 })
                .color(color)
                .semantics_identifier(format!("site-route:{href}"))
                .into_node(),
        )
        .padding([level as f32 * 12.0, 0.0, 0.0, 0.0])
        .into_node()
    }

    fn article(&self, ctx: &mut BuildCtx<SitePageState>, view: &View<SitePageState>) -> Node {
        let mut children = vec![Text::new(self.route.title.clone())
            .size(42.0)
            .weight(800)
            .line_height(48.0)
            .color(text_color())
            .into_node()];
        if let Some(description) = &self.route.description {
            children.push(
                Text::new(description.clone())
                    .size(18.0)
                    .line_height(28.0)
                    .color(muted_color())
                    .into_node(),
            );
        }
        let markdown = MarkdownViewer {
            markdown: self.route.body.clone(),
            show_scrollbar: false,
        };
        children.push(markdown.build(ctx, view));

        Container::new(
            Column {
                children,
                gap: Some(18.0),
                flex_grow: 1.0,
                ..Default::default()
            }
            .into_node(),
        )
        .padding_all(42.0)
        .bg_fill(Fill::Solid(surface_color()))
        .border(border_color(), 1.0)
        .border_radius(28.0)
        .flex_grow(1.0)
        .into_node()
    }

    fn toc(&self) -> Node {
        let mut children = vec![Text::new("On this page")
            .size(12.0)
            .weight(760)
            .color(muted_color())
            .into_node()];
        for heading in &self.route.headings {
            children.push(
                Text::new(heading.title.clone())
                    .size(13.0)
                    .color(text_color())
                    .semantics_identifier(format!("site-heading:{}", heading.anchor))
                    .into_node(),
            );
        }
        Container::new(
            Column {
                children,
                gap: Some(10.0),
                ..Default::default()
            }
            .into_node(),
        )
        .padding_all(18.0)
        .width(220.0)
        .into_node()
    }
}

pub(crate) fn extract_h1_links(markdown: &str) -> Vec<HeadingLink> {
    markdown
        .lines()
        .filter_map(|line| {
            let title = line.strip_prefix("# ")?.trim();
            if title.is_empty() {
                return None;
            }
            Some(HeadingLink {
                title: title.to_string(),
                anchor: slug(title),
            })
        })
        .collect()
}

fn section_prefix(path: &str) -> &str {
    let trimmed = path.trim_start_matches('/');
    let Some((first, _)) = trimmed.split_once('/') else {
        return path;
    };
    match first {
        "docs" => "/docs/",
        "reference" => "/reference/",
        "blog" => "/blog/",
        _ => "/",
    }
}

fn normalize_link_path(path: &str) -> String {
    let mut out = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    if out.len() > 1 && !out.ends_with('/') {
        out.push('/');
    }
    out
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

fn text_color() -> Color {
    Color {
        r: 22,
        g: 19,
        b: 15,
        a: 255,
    }
}

fn muted_color() -> Color {
    Color {
        r: 104,
        g: 96,
        b: 86,
        a: 255,
    }
}

fn accent_color() -> Color {
    Color {
        r: 192,
        g: 91,
        b: 42,
        a: 255,
    }
}

fn surface_color() -> Color {
    Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    }
}

fn surface_soft_color() -> Color {
    Color {
        r: 244,
        g: 240,
        b: 232,
        a: 255,
    }
}

fn border_color() -> Color {
    Color {
        r: 222,
        g: 215,
        b: 203,
        a: 255,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn h1_links_are_slugged() {
        let links = extract_h1_links("# Getting Started\n\n## Not listed\n# API & Widgets");
        assert_eq!(links[0].anchor, "getting-started");
        assert_eq!(links[1].anchor, "api-widgets");
    }
}
