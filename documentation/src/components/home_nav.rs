use super::home_widgets::{nav_inset, semantic_row, ExternalNavLink, SearchPill, ThemeToggle};
use super::state::DocsState;
use fission::op::{AlignItems, Fill, FlexWrap, JustifyContent};
use fission::prelude::*;

#[derive(Clone, Copy, Debug)]
struct NavItem {
    label: &'static str,
    href: &'static str,
    children: &'static [NavItem],
}

const BLOG_POSTS: &[NavItem] = &[
    NavItem {
        label: "Introducing Fission",
        href: "/blog/2026-04-25-introducing-fission/",
        children: &[],
    },
    NavItem {
        label: "Fission 0.1.0",
        href: "/blog/2026-04-25-fission-0-1-0/",
        children: &[],
    },
    NavItem {
        label: "Site built with Fission",
        href: "/blog/welcome/",
        children: &[],
    },
    NavItem {
        label: "Fission 0.2.0",
        href: "/blog/2026-05-26-fission-0-2-0/",
        children: &[],
    },
    NavItem {
        label: "Fission 0.3.0",
        href: "/blog/2026-05-29-fission-0-3-0/",
        children: &[],
    },
    NavItem {
        label: "Fission 0.4.0",
        href: "/blog/2026-06-02-fission-0-4-0-authoring-api/",
        children: &[],
    },
];

const PRODUCT_CHILDREN: &[NavItem] = &[
    NavItem {
        label: "Platform overview",
        href: "/product/overview/",
        children: &[],
    },
    NavItem {
        label: "Cross-platform apps",
        href: "/product/cross-platform-apps/",
        children: &[],
    },
    NavItem {
        label: "Static sites",
        href: "/product/static-sites/",
        children: &[],
    },
    NavItem {
        label: "Terminal apps",
        href: "/product/terminal-apps/",
        children: &[],
    },
    NavItem {
        label: "Charts",
        href: "/product/charts/",
        children: &[],
    },
    NavItem {
        label: "Production lifecycle",
        href: "/product/production-lifecycle/",
        children: &[],
    },
    NavItem {
        label: "Developer tools",
        href: "/product/developer-tools/",
        children: &[],
    },
    NavItem {
        label: "Design systems",
        href: "/product/design-systems/",
        children: &[],
    },
    NavItem {
        label: "Blog",
        href: "/blog/welcome/",
        children: BLOG_POSTS,
    },
];

const NAV_ITEMS: &[NavItem] = &[
    NavItem {
        label: "Product",
        href: "/product/overview/",
        children: PRODUCT_CHILDREN,
    },
    NavItem {
        label: "Setup",
        href: "/docs/learn/quickstart/",
        children: &[],
    },
    NavItem {
        label: "Learn",
        href: "/docs/learn/overview/",
        children: &[],
    },
    NavItem {
        label: "Build",
        href: "/docs/build-and-package/overview/",
        children: &[],
    },
    NavItem {
        label: "Test",
        href: "/docs/test-and-debug/overview/",
        children: &[],
    },
    NavItem {
        label: "Publish",
        href: "/docs/release-and-distribute/overview/",
        children: &[],
    },
];

#[derive(Clone, Debug)]
pub(super) struct HomePageNav;

impl From<HomePageNav> for Widget {
    fn from(_component: HomePageNav) -> Self {
        let (_ctx, view) = fission::build::current::<DocsState>();
        let tokens = &view.env().theme.tokens;
        let nav_items = NAV_ITEMS
            .iter()
            .enumerate()
            .map(|(index, item)| nav_item(item, 0, index, tokens))
            .collect::<Vec<_>>();
        Container::new(Row {
            children: vec![
                semantic_row(
                    "site-route:/",
                    vec![
                        Image::asset("/img/fission-mark.svg")
                            .size(tokens.spacing.l, tokens.spacing.l)
                            .into(),
                        Text::new("Fission")
                            .size(tokens.typography.font_size_lg)
                            .weight(tokens.typography.font_weight_bold)
                            .color(tokens.colors.heading)
                            .into(),
                    ],
                    Some(tokens.spacing.s),
                    FlexWrap::NoWrap,
                    AlignItems::Center,
                    JustifyContent::Start,
                ),
                Row {
                    children: nav_items,
                    gap: Some(tokens.spacing.l),
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::Center,
                    semantics: Some(super::home_widgets::site_semantics("site-main-nav")),
                    ..Default::default()
                }
                .into(),
                Row {
                    children: vec![
                        ExternalNavLink::new("GitHub", "https://github.com/worka-ai/fission")
                            .into(),
                        ThemeToggle.into(),
                        SearchPill.into(),
                    ],
                    gap: Some(tokens.spacing.m),
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::Center,
                    ..Default::default()
                }
                .into(),
            ],
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            wrap: FlexWrap::Wrap,
            ..Default::default()
        })
        .padding([
            nav_inset(tokens),
            nav_inset(tokens),
            tokens.spacing.m,
            tokens.spacing.m,
        ])
        .bg_fill(Fill::Solid(tokens.colors.surface.with_alpha(232)))
        .border(tokens.colors.border, 1.0)
        .into()
    }
}

fn nav_item(item: &NavItem, depth: usize, index: usize, tokens: &fission::theme::Tokens) -> Widget {
    let has_children = !item.children.is_empty();
    let mut label_children = vec![Text::new(item.label)
        .size(tokens.typography.label_large_size)
        .weight(tokens.typography.font_weight_semibold)
        .color(tokens.colors.text_link)
        .semantics_identifier(format!("site-route:{}", item.href))
        .into()];
    if has_children {
        label_children.push(
            Text::new(if depth == 0 { "v" } else { ">" })
                .size(tokens.typography.font_size_xs)
                .weight(tokens.typography.font_weight_bold)
                .color(tokens.colors.text_muted)
                .into(),
        );
    }

    let mut children = vec![Row {
        children: label_children,
        gap: Some(tokens.spacing.xs),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Start,
        semantics: Some(super::home_widgets::site_semantics(format!(
            "site-nav-label:{depth}:{has_children}:{index}"
        ))),
        ..Default::default()
    }
    .into()];

    if has_children {
        children.push(
            Column {
                children: item
                    .children
                    .iter()
                    .enumerate()
                    .map(|(child_index, child)| nav_item(child, depth + 1, child_index, tokens))
                    .collect(),
                gap: Some(tokens.spacing.xs),
                semantics: Some(super::home_widgets::site_semantics(format!(
                    "site-nav-menu:{}:{}",
                    depth + 1,
                    item.children.len()
                ))),
                ..Default::default()
            }
            .into(),
        );
    }

    Column {
        children,
        semantics: Some(super::home_widgets::site_semantics(format!(
            "site-nav-item:{depth}:{has_children}:{index}"
        ))),
        ..Default::default()
    }
    .into()
}
