mod charts;

use anyhow::Result;
use fission::op::{AlignItems, Color, Fill, FlexWrap, JustifyContent, TextAlign};
use fission::prelude::*;
use fission_shell_site::{build_from_cli, FissionSite};
use std::sync::Arc;

fn main() -> Result<()> {
    build_from_cli(site_app())
}

fn site_app() -> FissionSite {
    FissionSite::new()
        .theme(Theme::dark())
        .route_widget::<DocsState, _>(
            "/",
            "Fission",
            Some(
                "Build production desktop, web, Android, and iOS apps with one Rust UI framework."
                    .to_string(),
            ),
            RoutedHomePage {
                current_path: "/".to_string(),
            },
        )
        .content_transform(charts::expand_documentation_mdx)
}

#[derive(Debug, Default)]
struct DocsState;
impl AppState for DocsState {}

#[derive(Clone, Debug)]
struct RoutedHomePage {
    current_path: String,
}

impl Widget<DocsState> for RoutedHomePage {
    fn build(&self, ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        Router {
            current_path: self.current_path.clone(),
            routes: vec![Route {
                path: "/".to_string(),
                builder: Arc::new(|ctx, view, _params| HomePage.build(ctx, view)),
            }],
            not_found: None,
        }
        .build(ctx, view)
    }
}

#[derive(Clone, Debug)]
struct HomePage;

impl Widget<DocsState> for HomePage {
    fn build(&self, _ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        Container::new(
            Column {
                children: vec![
                    top_nav(tokens),
                    Row {
                        children: vec![Container::new(
                            Column {
                                children: vec![
                                    hero(tokens),
                                    architecture_section(tokens),
                                    proof_strip(tokens),
                                    targets_section(tokens),
                                    charts_section(tokens),
                                    final_cta(tokens),
                                ],
                                gap: Some(tokens.spacing.xxxl),
                                align_items: AlignItems::Center,
                                ..Default::default()
                            }
                            .into_node(),
                        )
                        .width(content_width(tokens))
                        .padding([0.0, 0.0, tokens.spacing.xxl, tokens.spacing.xxxxl])
                        .into_node()],
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    }
                    .into_node(),
                ],
                gap: Some(0.0),
                flex_grow: 1.0,
                ..Default::default()
            }
            .into_node(),
        )
        .min_height(tokens.spacing.xxxxl * 9.0)
        .bg_fill(page_fill(tokens))
        .into_node()
    }
}

const NAV_ITEMS: &[(&str, &str)] = &[
    ("Learn", "/docs/learn/overview/"),
    ("Guides", "/docs/guides/app-structure/"),
    ("Charts", "/reference/charts/overview/"),
    ("Cookbook", "/docs/cookbook/build-a-counter/"),
    ("Reference", "/reference/overview/overview/"),
    ("Examples", "/docs/learn/examples-and-targets/"),
];

fn top_nav(tokens: &Tokens) -> Node {
    Container::new(
        Row {
            children: vec![
                Row {
                    children: vec![
                        Image {
                            source: "/img/fission-mark.svg".to_string(),
                            width: Some(tokens.spacing.l),
                            height: Some(tokens.spacing.l),
                            ..Default::default()
                        }
                        .into_node(),
                        Text::new("Fission")
                            .size(tokens.typography.font_size_lg)
                            .weight(tokens.typography.font_weight_bold)
                            .color(tokens.colors.heading)
                            .into_node(),
                    ],
                    gap: Some(tokens.spacing.s),
                    align_items: AlignItems::Center,
                    ..Default::default()
                }
                .into_node(),
                Row {
                    children: NAV_ITEMS
                        .iter()
                        .map(|(label, href)| nav_link(label, href, tokens))
                        .collect(),
                    gap: Some(tokens.spacing.l),
                    justify_content: JustifyContent::End,
                    ..Default::default()
                }
                .into_node(),
            ],
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..Default::default()
        }
        .into_node(),
    )
    .padding([
        nav_inset(tokens),
        nav_inset(tokens),
        tokens.spacing.m,
        tokens.spacing.m,
    ])
    .bg_fill(Fill::Solid(tokens.colors.surface.with_alpha(232)))
    .border(tokens.colors.border, 1.0)
    .into_node()
}

fn hero(tokens: &Tokens) -> Node {
    Container::new(
        Column {
            children: vec![
                pill("Production-ready Rust user interface", tokens),
                Text::new("Build desktop, web, Android, and iOS apps in Rust.")
                    .size(tokens.typography.display_md_size)
                    .family(tokens.typography.font_family_serif.clone())
                    .line_height(
                        tokens.typography.display_md_size * tokens.typography.line_height_display,
                    )
                    .weight(tokens.typography.font_weight_bold)
                    .color(tokens.colors.heading)
                    .width(hero_text_width(tokens))
                    .max_width(hero_text_width(tokens))
                    .text_align(TextAlign::Center)
                    .flex_shrink(1.0)
                    .into_node(),
                Text::new("Fission is a cross-platform user interface framework with one shared runtime, explicit state, explicit side effects, and a GPU-backed rendering pipeline.")
                    .size(tokens.typography.font_size_lg)
                    .line_height(tokens.typography.font_size_lg * tokens.typography.line_height_relaxed)
                    .color(tokens.colors.text_secondary)
                    .width(prose_width(tokens))
                    .max_width(prose_width(tokens))
                    .text_align(TextAlign::Center)
                    .flex_shrink(1.0)
                    .into_node(),
                Text::new("You write app state as plain Rust data, update it with reducers, and let Fission keep layout, input, time, rendering, and platform boundaries consistent across every target.")
                    .size(tokens.typography.body_large_size)
                    .line_height(tokens.typography.body_large_size * tokens.typography.line_height_relaxed)
                    .color(tokens.colors.text_muted)
                    .width(prose_width(tokens))
                    .max_width(prose_width(tokens))
                    .text_align(TextAlign::Center)
                    .flex_shrink(1.0)
                    .into_node(),
                Row {
                    children: vec![
                        cta("Start with Quickstart", "/docs/learn/quickstart/", true, tokens),
                        cta("Read Learn overview", "/docs/learn/overview/", false, tokens),
                        nav_link("Browse Reference", "/reference/overview/overview/", tokens),
                    ],
                    gap: Some(tokens.spacing.m),
                    wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                }
                .into_node(),
                command_panel(tokens),
            ],
            gap: Some(tokens.spacing.l),
            align_items: AlignItems::Center,
            ..Default::default()
        }
        .into_node(),
    )
    .padding([0.0, 0.0, tokens.spacing.xxxl, tokens.spacing.xxxxl])
    .into_node()
}

fn command_panel(tokens: &Tokens) -> Node {
    Container::new(
        Row {
            children: vec![
                code_card("Run a real app", "cargo run -p counter", tokens),
                code_card("Create your project", "fission init my-app", tokens),
            ],
            gap: Some(tokens.spacing.m),
            wrap: FlexWrap::Wrap,
            justify_content: JustifyContent::Center,
            ..Default::default()
        }
        .into_node(),
    )
    .into_node()
}

fn proof_strip(tokens: &Tokens) -> Node {
    shell_section(
        Row {
            children: vec![
                signal("One shared runtime", "State, reducers, layout, semantics, rendering, and diagnostics stay in one app model.", "/docs/learn/runtime-model/", tokens),
                signal("Real target families", "Desktop, web, Android, and iOS shells host the same Rust application code.", "/docs/learn/examples-and-targets/", tokens),
                signal("Built for verification", "Live tests, diagnostics, semantics, and layout inspection are part of the product workflow.", "/docs/guides/testing-and-diagnostics/", tokens),
            ],
            gap: Some(tokens.spacing.m),
            wrap: FlexWrap::Wrap,
            ..Default::default()
        }
        .into_node(),
        tokens,
    )
}

fn architecture_section(tokens: &Tokens) -> Node {
    centered_section(
        "What Fission is",
        "A cross-platform Rust framework built for real products.",
        "Fission keeps state flow, layout, semantics, input routing, and rendering in one runtime, while platform shells handle packaging, windows, browser surfaces, lifecycle, and operating-system integration.",
        vec![
            mini_step("01", "State", "Plain Rust data holds product truth instead of hiding it in widgets or host callbacks.", tokens),
            mini_step("02", "Reducers", "Typed actions describe user intent and reducers make durable state changes reviewable.", tokens),
            mini_step("03", "Host work", "Files, timers, services, capabilities, and background jobs use explicit runtime paths.", tokens),
            mini_step("04", "Render", "Layout, semantics, paint order, and diagnostics remain available for tests and debugging.", tokens),
        ],
        tokens,
    )
}

fn targets_section(tokens: &Tokens) -> Node {
    centered_section(
        "Targets",
        "Ship across desktop, web, Android, and iOS.",
        "The shells are platform-specific where they need to be, but the application model stays shared. That is the difference between porting a product and rebuilding it four times.",
        vec![
            target("Desktop", "Windows, macOS, and Linux shells for local development and production desktop apps.", tokens),
            target("Web", "Browser-hosted Fission apps with generated scaffolding and smoke tests.", tokens),
            target("Android", "Android target scaffolding, emulator workflow, device logs, and CLI-driven runs.", tokens),
            target("iOS", "iOS simulator workflow, bundle generation, test control, and platform integration.", tokens),
        ],
        tokens,
    )
}

fn charts_section(tokens: &Tokens) -> Node {
    centered_section(
        "Beautiful charts",
        "Beautiful charts belong in the framework.",
        "Fission Charts is a first-class visualization layer for product dashboards, analytics tools, operations consoles, and interactive reports.",
        vec![
            chart_tile("Cartesian and radial", "Line, bar, area, pie, radar, gauge, and polar families.", tokens),
            chart_tile("Operations and analytics", "Heatmaps, timelines, funnels, sankey, treemaps, graph, and monitoring views.", tokens),
            chart_tile("3D and GL", "Surface, globe, scatter3D, graph3D, terrain, mesh, and point-cloud catalog entries.", tokens),
        ],
        tokens,
    )
}

fn final_cta(tokens: &Tokens) -> Node {
    shell_section(
        Column {
            children: vec![
                Text::new("Start with the hand-held path, then go deeper.")
                    .size(tokens.typography.heading2_size)
                    .line_height(tokens.typography.heading2_size * tokens.typography.line_height_heading)
                    .weight(tokens.typography.font_weight_bold)
                    .color(tokens.colors.heading)
                    .into_node(),
                Text::new("The documentation introduces each concept before relying on it, then backs the explanation with examples and reference pages.")
                    .size(tokens.typography.body_large_size)
                    .line_height(tokens.typography.body_large_size * tokens.typography.line_height_relaxed)
                    .color(tokens.colors.text_secondary)
                    .into_node(),
                Row {
                    children: vec![
                        cta("Read the docs", "/docs/intro/", true, tokens),
                        cta("Open the reference", "/reference/overview/overview/", false, tokens),
                    ],
                    gap: Some(tokens.spacing.m),
                    wrap: FlexWrap::Wrap,
                    ..Default::default()
                }
                .into_node(),
            ],
            gap: Some(tokens.spacing.l),
            ..Default::default()
        }
        .into_node(),
        tokens,
    )
}

fn shell_section(child: Node, tokens: &Tokens) -> Node {
    Container::new(child)
        .padding_all(tokens.spacing.xl)
        .bg_fill(Fill::Solid(tokens.colors.surface))
        .border(tokens.colors.border, 1.0)
        .border_radius(tokens.radii.xxl)
        .into_node()
}

fn centered_section(
    eyebrow: &str,
    title: &str,
    body: &str,
    cards: Vec<Node>,
    tokens: &Tokens,
) -> Node {
    Container::new(
        Column {
            children: vec![
                Text::new(eyebrow.to_string())
                    .size(tokens.typography.font_size_sm)
                    .weight(tokens.typography.font_weight_bold)
                    .color(tokens.colors.secondary)
                    .into_node(),
                Text::new(title.to_string())
                    .size(tokens.typography.heading2_size)
                    .family(tokens.typography.font_family_serif.clone())
                    .line_height(
                        tokens.typography.heading2_size * tokens.typography.line_height_heading,
                    )
                    .weight(tokens.typography.font_weight_bold)
                    .color(tokens.colors.heading)
                    .max_width(headline_width(tokens))
                    .text_align(TextAlign::Center)
                    .flex_shrink(1.0)
                    .into_node(),
                Text::new(body.to_string())
                    .size(tokens.typography.body_large_size)
                    .line_height(
                        tokens.typography.body_large_size * tokens.typography.line_height_relaxed,
                    )
                    .color(tokens.colors.text_secondary)
                    .max_width(prose_width(tokens))
                    .text_align(TextAlign::Center)
                    .flex_shrink(1.0)
                    .into_node(),
                Row {
                    children: cards,
                    gap: Some(tokens.spacing.m),
                    wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                }
                .into_node(),
            ],
            gap: Some(tokens.spacing.l),
            align_items: AlignItems::Center,
            ..Default::default()
        }
        .into_node(),
    )
    .into_node()
}

fn nav_link(label: &str, href: &str, tokens: &Tokens) -> Node {
    Text::new(label.to_string())
        .size(tokens.typography.label_large_size)
        .weight(tokens.typography.font_weight_semibold)
        .color(tokens.colors.text_link)
        .semantics_identifier(format!("site-route:{href}"))
        .into_node()
}

fn cta(label: &str, href: &str, primary: bool, tokens: &Tokens) -> Node {
    let (background, foreground, border) = if primary {
        (
            tokens.colors.primary,
            tokens.colors.on_primary,
            tokens.colors.primary,
        )
    } else {
        (
            tokens.colors.surface_raised,
            tokens.colors.text_primary,
            tokens.colors.border,
        )
    };
    Container::new(
        Text::new(label.to_string())
            .size(tokens.typography.label_large_size)
            .weight(tokens.typography.font_weight_bold)
            .color(foreground)
            .semantics_identifier(format!("site-route:{href}"))
            .into_node(),
    )
    .padding([
        tokens.spacing.l,
        tokens.spacing.l,
        tokens.spacing.m,
        tokens.spacing.m,
    ])
    .bg_fill(Fill::Solid(background))
    .border(border, 1.0)
    .border_radius(tokens.radii.full)
    .into_node()
}

fn pill(label: &str, tokens: &Tokens) -> Node {
    Container::new(
        Text::new(label.to_string())
            .size(tokens.typography.font_size_sm)
            .weight(tokens.typography.font_weight_bold)
            .color(tokens.colors.primary)
            .into_node(),
    )
    .padding([
        tokens.spacing.m,
        tokens.spacing.m,
        tokens.spacing.s,
        tokens.spacing.s,
    ])
    .bg_fill(Fill::Solid(tokens.colors.primary_subtle))
    .border(tokens.colors.focus_ring, 1.0)
    .border_radius(tokens.radii.full)
    .into_node()
}

fn code_card(label: &str, command: &str, tokens: &Tokens) -> Node {
    Container::new(
        Column {
            children: vec![
                Text::new(label.to_string())
                    .size(tokens.typography.font_size_xs)
                    .weight(tokens.typography.font_weight_bold)
                    .color(tokens.colors.secondary)
                    .into_node(),
                Text::new(command.to_string())
                    .size(tokens.typography.font_size_sm)
                    .line_height(
                        tokens.typography.font_size_sm * tokens.typography.line_height_snug,
                    )
                    .family(tokens.typography.font_family_mono.clone())
                    .color(tokens.colors.text_primary)
                    .into_node(),
            ],
            gap: Some(tokens.spacing.s),
            ..Default::default()
        }
        .into_node(),
    )
    .padding_all(tokens.spacing.m)
    .bg_fill(Fill::Solid(tokens.colors.surface_raised))
    .border(tokens.colors.border_strong, 1.0)
    .border_radius(tokens.radii.xl)
    .into_node()
}

fn signal(title: &str, body: &str, href: &str, tokens: &Tokens) -> Node {
    card(
        vec![
            Text::new(title.to_string())
                .size(tokens.typography.font_size_lg)
                .weight(tokens.typography.font_weight_bold)
                .color(tokens.colors.heading)
                .semantics_identifier(format!("site-route:{href}"))
                .into_node(),
            paragraph(body, tokens),
        ],
        tile_width(tokens),
        tokens,
    )
}

fn mini_step(number: &str, title: &str, body: &str, tokens: &Tokens) -> Node {
    card(
        vec![
            Text::new(number.to_string())
                .size(tokens.typography.font_size_xs)
                .weight(tokens.typography.font_weight_bold)
                .color(tokens.colors.primary)
                .into_node(),
            Text::new(title.to_string())
                .size(tokens.typography.font_size_lg)
                .weight(tokens.typography.font_weight_bold)
                .color(tokens.colors.heading)
                .into_node(),
            paragraph(body, tokens),
        ],
        compact_tile_width(tokens),
        tokens,
    )
}

fn target(title: &str, body: &str, tokens: &Tokens) -> Node {
    card(
        vec![
            Text::new(title.to_string())
                .size(tokens.typography.font_size_lg)
                .weight(tokens.typography.font_weight_bold)
                .color(tokens.colors.heading)
                .into_node(),
            paragraph(body, tokens),
            nav_link(
                "See target workflow",
                "/docs/guides/platform-shells-cli-and-testing/",
                tokens,
            ),
        ],
        compact_tile_width(tokens),
        tokens,
    )
}

fn chart_tile(title: &str, body: &str, tokens: &Tokens) -> Node {
    card(
        vec![
            chart_preview(tokens),
            Text::new(title.to_string())
                .size(tokens.typography.font_size_lg)
                .weight(tokens.typography.font_weight_bold)
                .color(tokens.colors.heading)
                .semantics_identifier("site-route:/reference/charts/overview/")
                .into_node(),
            paragraph(body, tokens),
        ],
        tile_width(tokens),
        tokens,
    )
}

fn paragraph(body: &str, tokens: &Tokens) -> Node {
    Text::new(body.to_string())
        .size(tokens.typography.body_medium_size)
        .line_height(tokens.typography.body_medium_size * tokens.typography.line_height_normal)
        .color(tokens.colors.text_secondary)
        .flex_shrink(1.0)
        .into_node()
}

fn card(children: Vec<Node>, width: f32, tokens: &Tokens) -> Node {
    Container::new(
        Column {
            children,
            gap: Some(tokens.spacing.m),
            ..Default::default()
        }
        .into_node(),
    )
    .width(width)
    .flex_shrink(1.0)
    .padding_all(tokens.spacing.l)
    .bg_fill(Fill::Solid(tokens.colors.surface_raised))
    .border(tokens.colors.border, 1.0)
    .border_radius(tokens.radii.xl)
    .into_node()
}

fn chart_preview(tokens: &Tokens) -> Node {
    Row {
        children: vec![
            bar(tokens.spacing.xl, tokens.colors.primary, tokens),
            bar(tokens.spacing.xxxl, tokens.colors.info, tokens),
            bar(tokens.spacing.xxl, tokens.colors.warning, tokens),
            bar(
                tokens.spacing.xxxl + tokens.spacing.m,
                tokens.colors.secondary,
                tokens,
            ),
        ],
        gap: Some(tokens.spacing.s),
        align_items: AlignItems::End,
        ..Default::default()
    }
    .into_node()
}

fn bar(height: f32, color: Color, tokens: &Tokens) -> Node {
    Container::new(Text::new("").into_node())
        .width(tokens.spacing.l)
        .height(height)
        .bg_fill(Fill::Solid(color))
        .border_radius(tokens.radii.medium)
        .into_node()
}

fn page_fill(tokens: &Tokens) -> Fill {
    Fill::LinearGradient {
        start: (0.0, 0.0),
        end: (1.0, 1.0),
        stops: vec![
            (0.0, tokens.colors.background),
            (0.6, tokens.colors.surface_sunken),
            (1.0, tokens.colors.surface),
        ],
    }
}

fn hero_text_width(tokens: &Tokens) -> f32 {
    tokens.spacing.xxxxl * 8.25
}

fn prose_width(tokens: &Tokens) -> f32 {
    tokens.spacing.xxxxl * 9.0
}

fn headline_width(tokens: &Tokens) -> f32 {
    tokens.spacing.xxxxl * 6.5
}

fn tile_width(tokens: &Tokens) -> f32 {
    tokens.spacing.xxxxl * 3.5
}

fn compact_tile_width(tokens: &Tokens) -> f32 {
    tokens.spacing.xxxxl * 2.8
}

fn content_width(tokens: &Tokens) -> f32 {
    tokens.spacing.xxxxl * 17.5
}

fn nav_inset(tokens: &Tokens) -> f32 {
    tokens.spacing.xxxxl
}
