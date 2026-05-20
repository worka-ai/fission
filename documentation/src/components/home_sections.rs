use super::home_widgets::{
    hero_text_width, prose_width, CenteredSection, ChartTile, CodeCard, Cta, MiniStep, NavLink,
    Pill, ShellSection, Signal, TargetCard,
};
use super::state::DocsState;
use fission::op::{AlignItems, FlexWrap, JustifyContent, TextAlign};
use fission::prelude::*;

#[derive(Clone, Debug)]
pub(super) struct HomePageHero;

impl Widget<DocsState> for HomePageHero {
    fn build(&self, ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        Container::new(
            Column {
                children: vec![
                    Pill::new("Production-ready Rust user interface").build(ctx, view),
                    Text::new("Build desktop, web, Android, and iOS apps in Rust.")
                        .size(tokens.typography.display_md_size)
                        .family(tokens.typography.font_family_serif.clone())
                        .line_height(
                            tokens.typography.display_md_size
                                * tokens.typography.line_height_display,
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
                            Cta::new("Start with Quickstart", "/docs/learn/quickstart/", true).build(ctx, view),
                            Cta::new("Read Learn overview", "/docs/learn/overview/", false).build(ctx, view),
                            NavLink::new("Browse Reference", "/reference/overview/overview/").build(ctx, view),
                        ],
                        gap: Some(tokens.spacing.m),
                        wrap: FlexWrap::Wrap,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    }
                    .into_node(),
                    Row {
                        children: vec![
                            CodeCard::new("Run a real app", "cargo run -p counter").build(ctx, view),
                            CodeCard::new("Create your project", "fission init my-app").build(ctx, view),
                        ],
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
        .padding([0.0, 0.0, tokens.spacing.xxxl, tokens.spacing.xxxxl])
        .into_node()
    }
}

#[derive(Clone, Debug)]
pub(super) struct ProofStrip;

impl Widget<DocsState> for ProofStrip {
    fn build(&self, ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        ShellSection::new(
            Row {
                children: vec![
                    Signal::new("One shared runtime", "State, reducers, layout, semantics, rendering, and diagnostics stay in one app model.", "/docs/learn/runtime-model/").build(ctx, view),
                    Signal::new("Real target families", "Desktop, web, Android, and iOS shells host the same Rust application code.", "/docs/learn/examples-and-targets/").build(ctx, view),
                    Signal::new("Built for verification", "Live tests, diagnostics, semantics, and layout inspection are part of the product workflow.", "/docs/guides/testing-and-diagnostics/").build(ctx, view),
                ],
                gap: Some(tokens.spacing.m),
                wrap: FlexWrap::Wrap,
                ..Default::default()
            }
            .into_node(),
        )
        .build(ctx, view)
    }
}

#[derive(Clone, Debug)]
pub(super) struct ArchitectureSection;

impl Widget<DocsState> for ArchitectureSection {
    fn build(&self, ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        CenteredSection::new(
            "What Fission is",
            "A cross-platform Rust framework built for real products.",
            "Fission keeps state flow, layout, semantics, input routing, and rendering in one runtime, while platform shells handle packaging, windows, browser surfaces, lifecycle, and operating-system integration.",
            vec![
                MiniStep::new("01", "State", "Plain Rust data holds product truth instead of hiding it in widgets or host callbacks.").build(ctx, view),
                MiniStep::new("02", "Reducers", "Typed actions describe user intent and reducers make durable state changes reviewable.").build(ctx, view),
                MiniStep::new("03", "Host work", "Files, timers, services, capabilities, and background jobs use explicit runtime paths.").build(ctx, view),
                MiniStep::new("04", "Render", "Layout, semantics, paint order, and diagnostics remain available for tests and debugging.").build(ctx, view),
            ],
        )
        .build(ctx, view)
    }
}

#[derive(Clone, Debug)]
pub(super) struct TargetsSection;

impl Widget<DocsState> for TargetsSection {
    fn build(&self, ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        CenteredSection::new(
            "Targets",
            "Ship across desktop, web, Android, and iOS.",
            "The shells are platform-specific where they need to be, but the application model stays shared. That is the difference between porting a product and rebuilding it four times.",
            vec![
                TargetCard::new("Desktop", "Windows, macOS, and Linux shells for local development and production desktop apps.").build(ctx, view),
                TargetCard::new("Web", "Browser-hosted Fission apps with generated scaffolding and smoke tests.").build(ctx, view),
                TargetCard::new("Android", "Android target scaffolding, emulator workflow, device logs, and CLI-driven runs.").build(ctx, view),
                TargetCard::new("iOS", "iOS simulator workflow, bundle generation, test control, and platform integration.").build(ctx, view),
            ],
        )
        .build(ctx, view)
    }
}

#[derive(Clone, Debug)]
pub(super) struct ChartsSection;

impl Widget<DocsState> for ChartsSection {
    fn build(&self, ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        CenteredSection::new(
            "Beautiful charts",
            "Beautiful charts belong in the framework.",
            "Fission Charts is a first-class visualization layer for product dashboards, analytics tools, operations consoles, and interactive reports.",
            vec![
                ChartTile::new("Cartesian and radial", "Line, bar, area, pie, radar, gauge, and polar families.").build(ctx, view),
                ChartTile::new("Operations and analytics", "Heatmaps, timelines, funnels, sankey, treemaps, graph, and monitoring views.").build(ctx, view),
                ChartTile::new("3D and GL", "Surface, globe, scatter3D, graph3D, terrain, mesh, and point-cloud catalog entries.").build(ctx, view),
            ],
        )
        .build(ctx, view)
    }
}

#[derive(Clone, Debug)]
pub(super) struct FinalCta;

impl Widget<DocsState> for FinalCta {
    fn build(&self, ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        ShellSection::new(
            Column {
                children: vec![
                    Text::new("Start with the hand-held path, then go deeper.")
                        .size(tokens.typography.heading2_size)
                        .line_height(
                            tokens.typography.heading2_size * tokens.typography.line_height_heading,
                        )
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
                            Cta::new("Read the docs", "/docs/intro/", true).build(ctx, view),
                            Cta::new("Open the reference", "/reference/overview/overview/", false).build(ctx, view),
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
        )
        .build(ctx, view)
    }
}
