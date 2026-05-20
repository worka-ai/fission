mod charts;

use anyhow::Result;
use fission::op::{AlignItems, Color, Fill, FlexWrap, JustifyContent};
use fission::prelude::*;
use fission_shell_site::{build_from_cli, FissionSite};
use std::sync::Arc;

fn main() -> Result<()> {
    build_from_cli(site_app())
}

fn site_app() -> FissionSite {
    FissionSite::new()
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
    fn build(&self, _ctx: &mut BuildCtx<DocsState>, _view: &View<DocsState>) -> Node {
        Container::new(
            Column {
                children: vec![
                    top_nav(),
                    hero(),
                    proof_strip(),
                    architecture_section(),
                    targets_section(),
                    charts_section(),
                    final_cta(),
                ],
                gap: Some(28.0),
                flex_grow: 1.0,
                ..Default::default()
            }
            .into_node(),
        )
        .min_height(900.0)
        .padding([28.0, 28.0, 24.0, 42.0])
        .bg_fill(Fill::LinearGradient {
            start: (0.0, 0.0),
            end: (1.0, 1.0),
            stops: vec![(0.0, paper()), (0.55, mist()), (1.0, warm())],
        })
        .into_node()
    }
}

fn top_nav() -> Node {
    Container::new(
        Row {
            children: vec![
                Text::new("Fission")
                    .size(20.0)
                    .weight(820)
                    .color(ink())
                    .into_node(),
                Row {
                    children: vec![
                        nav_link("Docs", "/docs/intro/"),
                        nav_link("Reference", "/reference/overview/overview/"),
                        nav_link("Charts", "/reference/charts/overview/"),
                    ],
                    gap: Some(18.0),
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
    .padding([24.0, 24.0, 18.0, 18.0])
    .bg_fill(Fill::Solid(glass()))
    .border(border(), 1.0)
    .border_radius(24.0)
    .into_node()
}

fn hero() -> Node {
    shell_section(
        Row {
            children: vec![
                Container::new(
                    Column {
                        children: vec![
                            pill("Production Rust UI for every target family"),
                            Text::new("Build serious apps with one Rust interface model.")
                                .size(58.0)
                                .line_height(62.0)
                                .weight(860)
                                .color(ink())
                                .into_node(),
                            Text::new("Fission is a production-ready UI framework for desktop, web, Android, and iOS. State, reducers, layout, semantics, rendering, diagnostics, and tests stay in one coherent Rust architecture.")
                                .size(20.0)
                                .line_height(32.0)
                                .color(muted())
                                .into_node(),
                            Row {
                                children: vec![cta("Start building", "/docs/learn/quickstart/", true), cta("Understand the model", "/docs/learn/runtime-model/", false)],
                                gap: Some(14.0),
                                wrap: FlexWrap::Wrap,
                                ..Default::default()
                            }
                            .into_node(),
                        ],
                        gap: Some(22.0),
                        flex_grow: 1.0,
                        ..Default::default()
                    }
                    .into_node(),
                )
                .flex_grow(1.0)
                .into_node(),
                command_panel(),
            ],
            gap: Some(34.0),
            wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            ..Default::default()
        }
        .into_node(),
    )
}

fn command_panel() -> Node {
    Container::new(
        Column {
            children: vec![
                Text::new("Create and run")
                    .size(14.0)
                    .weight(760)
                    .color(muted())
                    .into_node(),
                code_card("Create", "cargo fission init my-app"),
                code_card("Run", "cargo fission run --target web"),
                code_card("Inspect", "cargo fission doctor --project-dir ."),
            ],
            gap: Some(12.0),
            ..Default::default()
        }
        .into_node(),
    )
    .width(390.0)
    .padding_all(20.0)
    .bg_fill(Fill::Solid(charcoal()))
    .border_radius(26.0)
    .into_node()
}

fn proof_strip() -> Node {
    shell_section(
        Row {
            children: vec![
                signal("One shared runtime", "State, reducers, layout, semantics, rendering, and diagnostics stay in one app model.", "/docs/learn/runtime-model/"),
                signal("Real target families", "Desktop, web, Android, and iOS shells host the same Rust application code.", "/docs/learn/examples-and-targets/"),
                signal("Built for verification", "Live tests, diagnostics, semantics, and layout inspection are part of the product workflow.", "/docs/guides/testing-and-diagnostics/"),
            ],
            gap: Some(18.0),
            wrap: FlexWrap::Wrap,
            ..Default::default()
        }
        .into_node(),
    )
}

fn architecture_section() -> Node {
    split_section(
        "The app flow stays explicit from input to pixels.",
        "Fission keeps the important parts of a product easy to reason about: data is plain Rust, changes have named causes, outside work has an explicit path, and rendering stays inspectable.",
        vec![
            mini_step("01", "State", "Plain Rust data holds product truth instead of hiding it in widgets or host callbacks."),
            mini_step("02", "Reducers", "Typed actions describe user intent and reducers make durable state changes reviewable."),
            mini_step("03", "Host work", "Files, timers, services, capabilities, and background jobs use explicit runtime paths."),
            mini_step("04", "Render", "Layout, semantics, paint order, and diagnostics remain available for tests and debugging."),
        ],
    )
}

fn targets_section() -> Node {
    split_section(
        "Ship across desktop, web, Android, and iOS.",
        "The shells are platform-specific where they need to be, but the application model stays shared. That is the difference between porting a product and rebuilding it four times.",
        vec![
            target("Desktop", "Windows, macOS, and Linux shells for local development and production desktop apps."),
            target("Web", "Browser-hosted Fission apps with generated scaffolding and smoke tests."),
            target("Android", "Android target scaffolding, emulator workflow, device logs, and CLI-driven runs."),
            target("iOS", "iOS simulator workflow, bundle generation, test control, and platform integration."),
        ],
    )
}

fn charts_section() -> Node {
    split_section(
        "Beautiful charts belong in the framework.",
        "Fission Charts is a first-class visualization layer for product dashboards, analytics tools, operations consoles, and interactive reports.",
        vec![
            chart_tile("Cartesian and radial", "Line, bar, area, pie, radar, gauge, and polar families."),
            chart_tile("Operations and analytics", "Heatmaps, timelines, funnels, sankey, treemaps, graph, and monitoring views."),
            chart_tile("3D and GL", "Surface, globe, scatter3D, graph3D, terrain, mesh, and point-cloud catalog entries."),
        ],
    )
}

fn final_cta() -> Node {
    shell_section(
        Column {
            children: vec![
                Text::new("Start with the hand-held path, then go deeper.")
                    .size(34.0)
                    .line_height(40.0)
                    .weight(820)
                    .color(ink())
                    .into_node(),
                Text::new("The documentation introduces each concept before relying on it, then backs the explanation with examples and reference pages.")
                    .size(18.0)
                    .line_height(28.0)
                    .color(muted())
                    .into_node(),
                Row {
                    children: vec![cta("Read the docs", "/docs/intro/", true), cta("Open the reference", "/reference/overview/overview/", false)],
                    gap: Some(14.0),
                    wrap: FlexWrap::Wrap,
                    ..Default::default()
                }
                .into_node(),
            ],
            gap: Some(18.0),
            ..Default::default()
        }
        .into_node(),
    )
}

fn shell_section(child: Node) -> Node {
    Container::new(child)
        .padding_all(30.0)
        .bg_fill(Fill::Solid(surface()))
        .border(border(), 1.0)
        .border_radius(30.0)
        .into_node()
}

fn split_section(title: &str, body: &str, cards: Vec<Node>) -> Node {
    shell_section(
        Column {
            children: vec![
                Row {
                    children: vec![
                        Text::new(title.to_string())
                            .size(36.0)
                            .line_height(42.0)
                            .weight(820)
                            .color(ink())
                            .into_node(),
                        Text::new(body.to_string())
                            .size(17.0)
                            .line_height(28.0)
                            .color(muted())
                            .into_node(),
                    ],
                    gap: Some(36.0),
                    wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Start,
                    ..Default::default()
                }
                .into_node(),
                Row {
                    children: cards,
                    gap: Some(16.0),
                    wrap: FlexWrap::Wrap,
                    ..Default::default()
                }
                .into_node(),
            ],
            gap: Some(24.0),
            ..Default::default()
        }
        .into_node(),
    )
}

fn nav_link(label: &str, href: &str) -> Node {
    Text::new(label.to_string())
        .size(14.0)
        .weight(650)
        .color(ink())
        .semantics_identifier(format!("site-route:{href}"))
        .into_node()
}

fn cta(label: &str, href: &str, primary: bool) -> Node {
    Container::new(
        Text::new(label.to_string())
            .size(15.0)
            .weight(760)
            .color(if primary { paper() } else { ink() })
            .semantics_identifier(format!("site-route:{href}"))
            .into_node(),
    )
    .padding([18.0, 18.0, 12.0, 12.0])
    .bg_fill(Fill::Solid(if primary { accent() } else { surface_alt() }))
    .border(if primary { accent() } else { border() }, 1.0)
    .border_radius(999.0)
    .into_node()
}

fn pill(label: &str) -> Node {
    Container::new(
        Text::new(label.to_string())
            .size(13.0)
            .weight(760)
            .color(accent())
            .into_node(),
    )
    .padding([14.0, 14.0, 8.0, 8.0])
    .bg_fill(Fill::Solid(accent_wash()))
    .border(accent_border(), 1.0)
    .border_radius(999.0)
    .into_node()
}

fn code_card(label: &str, command: &str) -> Node {
    Container::new(
        Column {
            children: vec![
                Text::new(label.to_string())
                    .size(12.0)
                    .weight(760)
                    .color(warm_text())
                    .into_node(),
                Text::new(command.to_string())
                    .size(14.0)
                    .line_height(20.0)
                    .family("SFMono-Regular, Consolas, monospace")
                    .color(paper())
                    .into_node(),
            ],
            gap: Some(6.0),
            ..Default::default()
        }
        .into_node(),
    )
    .padding_all(14.0)
    .bg_fill(Fill::Solid(code_bg()))
    .border(code_border(), 1.0)
    .border_radius(16.0)
    .into_node()
}

fn signal(title: &str, body: &str, href: &str) -> Node {
    card(
        vec![
            Text::new(title.to_string())
                .size(20.0)
                .weight(780)
                .color(ink())
                .semantics_identifier(format!("site-route:{href}"))
                .into_node(),
            Text::new(body.to_string())
                .size(15.0)
                .line_height(23.0)
                .color(muted())
                .into_node(),
        ],
        355.0,
    )
}

fn mini_step(number: &str, title: &str, body: &str) -> Node {
    card(
        vec![
            Text::new(number.to_string())
                .size(12.0)
                .weight(820)
                .color(accent())
                .into_node(),
            Text::new(title.to_string())
                .size(21.0)
                .weight(800)
                .color(ink())
                .into_node(),
            Text::new(body.to_string())
                .size(15.0)
                .line_height(23.0)
                .color(muted())
                .into_node(),
        ],
        270.0,
    )
}

fn target(title: &str, body: &str) -> Node {
    card(
        vec![
            Text::new(title.to_string())
                .size(21.0)
                .weight(800)
                .color(ink())
                .into_node(),
            Text::new(body.to_string())
                .size(15.0)
                .line_height(23.0)
                .color(muted())
                .into_node(),
            Text::new("See target workflow")
                .size(14.0)
                .weight(700)
                .color(accent())
                .semantics_identifier("site-route:/docs/guides/platform-shells-cli-and-testing/")
                .into_node(),
        ],
        270.0,
    )
}

fn chart_tile(title: &str, body: &str) -> Node {
    card(
        vec![
            chart_preview(),
            Text::new(title.to_string())
                .size(21.0)
                .weight(800)
                .color(ink())
                .semantics_identifier("site-route:/reference/charts/overview/")
                .into_node(),
            Text::new(body.to_string())
                .size(15.0)
                .line_height(23.0)
                .color(muted())
                .into_node(),
        ],
        355.0,
    )
}

fn card(children: Vec<Node>, width: f32) -> Node {
    Container::new(
        Column {
            children,
            gap: Some(12.0),
            ..Default::default()
        }
        .into_node(),
    )
    .width(width)
    .padding_all(20.0)
    .bg_fill(Fill::Solid(surface_alt()))
    .border(border(), 1.0)
    .border_radius(22.0)
    .into_node()
}

fn chart_preview() -> Node {
    Row {
        children: vec![
            bar(28.0, accent()),
            bar(46.0, teal()),
            bar(34.0, gold()),
            bar(56.0, ink()),
        ],
        gap: Some(10.0),
        align_items: AlignItems::End,
        ..Default::default()
    }
    .into_node()
}

fn bar(height: f32, color: Color) -> Node {
    Container::new(Text::new("").into_node())
        .width(22.0)
        .height(height)
        .bg_fill(Fill::Solid(color))
        .border_radius(8.0)
        .into_node()
}

fn color(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b, a: 255 }
}

fn ink() -> Color {
    color(29, 25, 20)
}
fn muted() -> Color {
    color(101, 92, 82)
}
fn paper() -> Color {
    color(255, 252, 246)
}
fn mist() -> Color {
    color(246, 241, 232)
}
fn warm() -> Color {
    color(247, 229, 211)
}
fn surface() -> Color {
    color(255, 250, 242)
}
fn surface_alt() -> Color {
    color(252, 244, 233)
}
fn glass() -> Color {
    Color {
        r: 255,
        g: 250,
        b: 242,
        a: 228,
    }
}
fn border() -> Color {
    color(224, 211, 195)
}
fn accent() -> Color {
    color(189, 84, 42)
}
fn accent_wash() -> Color {
    color(255, 235, 220)
}
fn accent_border() -> Color {
    color(231, 176, 141)
}
fn charcoal() -> Color {
    color(34, 31, 27)
}
fn code_bg() -> Color {
    color(48, 44, 38)
}
fn code_border() -> Color {
    color(83, 74, 63)
}
fn warm_text() -> Color {
    color(236, 186, 132)
}
fn teal() -> Color {
    color(54, 143, 139)
}
fn gold() -> Color {
    color(220, 161, 65)
}
