use super::state::DocsState;
use fission::op::{AlignItems, Color, Fill, FlexWrap, JustifyContent, TextAlign};
use fission::prelude::*;

#[derive(Clone, Debug)]
pub(super) struct CenteredSection {
    eyebrow: &'static str,
    title: &'static str,
    body: &'static str,
    cards: Vec<Node>,
}

impl CenteredSection {
    pub(super) fn new(
        eyebrow: &'static str,
        title: &'static str,
        body: &'static str,
        cards: Vec<Node>,
    ) -> Self {
        Self {
            eyebrow,
            title,
            body,
            cards,
        }
    }
}

impl Widget<DocsState> for CenteredSection {
    fn build(&self, _ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        Container::new(
            Column {
                children: vec![
                    Text::new(self.eyebrow)
                        .size(tokens.typography.font_size_sm)
                        .weight(tokens.typography.font_weight_bold)
                        .color(tokens.colors.secondary)
                        .into_node(),
                    Text::new(self.title)
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
                    Text::new(self.body)
                        .size(tokens.typography.body_large_size)
                        .line_height(
                            tokens.typography.body_large_size
                                * tokens.typography.line_height_relaxed,
                        )
                        .color(tokens.colors.text_secondary)
                        .max_width(prose_width(tokens))
                        .text_align(TextAlign::Center)
                        .flex_shrink(1.0)
                        .into_node(),
                    Row {
                        children: self.cards.clone(),
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
}

#[derive(Clone, Debug)]
pub(super) struct ShellSection {
    child: Node,
}

impl ShellSection {
    pub(super) fn new(child: Node) -> Self {
        Self { child }
    }
}

impl Widget<DocsState> for ShellSection {
    fn build(&self, _ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        Container::new(self.child.clone())
            .padding_all(tokens.spacing.xl)
            .bg_fill(Fill::Solid(tokens.colors.surface))
            .border(tokens.colors.border, 1.0)
            .border_radius(tokens.radii.xxl)
            .into_node()
    }
}

#[derive(Clone, Debug)]
pub(super) struct NavLink {
    label: &'static str,
    href: &'static str,
}

impl NavLink {
    pub(super) fn new(label: &'static str, href: &'static str) -> Self {
        Self { label, href }
    }
}

impl Widget<DocsState> for NavLink {
    fn build(&self, _ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        Text::new(self.label)
            .size(tokens.typography.label_large_size)
            .weight(tokens.typography.font_weight_semibold)
            .color(tokens.colors.text_link)
            .semantics_identifier(format!("site-route:{}", self.href))
            .into_node()
    }
}

#[derive(Clone, Debug)]
pub(super) struct ThemeToggle;

impl Widget<DocsState> for ThemeToggle {
    fn build(&self, _ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        Text::new("Theme")
            .size(tokens.typography.label_large_size)
            .weight(tokens.typography.font_weight_semibold)
            .color(tokens.colors.text_link)
            .semantics_identifier("site-theme-toggle")
            .into_node()
    }
}

#[derive(Clone, Debug)]
pub(super) struct Cta {
    label: &'static str,
    href: &'static str,
    primary: bool,
}

impl Cta {
    pub(super) fn new(label: &'static str, href: &'static str, primary: bool) -> Self {
        Self {
            label,
            href,
            primary,
        }
    }
}

impl Widget<DocsState> for Cta {
    fn build(&self, _ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        let (background, foreground, border) = if self.primary {
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
            Text::new(self.label)
                .size(tokens.typography.label_large_size)
                .weight(tokens.typography.font_weight_bold)
                .color(foreground)
                .semantics_identifier(format!("site-route:{}", self.href))
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
}

#[derive(Clone, Debug)]
pub(super) struct Pill {
    label: &'static str,
}

impl Pill {
    pub(super) fn new(label: &'static str) -> Self {
        Self { label }
    }
}

impl Widget<DocsState> for Pill {
    fn build(&self, _ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        Container::new(
            Text::new(self.label)
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
}

#[derive(Clone, Debug)]
pub(super) struct CodeCard {
    label: &'static str,
    command: &'static str,
}

impl CodeCard {
    pub(super) fn new(label: &'static str, command: &'static str) -> Self {
        Self { label, command }
    }
}

impl Widget<DocsState> for CodeCard {
    fn build(&self, _ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        Container::new(
            Column {
                children: vec![
                    Text::new(self.label)
                        .size(tokens.typography.font_size_xs)
                        .weight(tokens.typography.font_weight_bold)
                        .color(tokens.colors.secondary)
                        .into_node(),
                    Text::new(self.command)
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
}

#[derive(Clone, Debug)]
pub(super) struct Signal {
    title: &'static str,
    body: &'static str,
    href: &'static str,
}

impl Signal {
    pub(super) fn new(title: &'static str, body: &'static str, href: &'static str) -> Self {
        Self { title, body, href }
    }
}

impl Widget<DocsState> for Signal {
    fn build(&self, ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        Card::new(
            vec![
                Text::new(self.title)
                    .size(view.env.theme.tokens.typography.font_size_lg)
                    .weight(view.env.theme.tokens.typography.font_weight_bold)
                    .color(view.env.theme.tokens.colors.heading)
                    .semantics_identifier(format!("site-route:{}", self.href))
                    .into_node(),
                Paragraph::new(self.body).build(ctx, view),
            ],
            tile_width(&view.env.theme.tokens),
        )
        .build(ctx, view)
    }
}

#[derive(Clone, Debug)]
pub(super) struct MiniStep {
    number: &'static str,
    title: &'static str,
    body: &'static str,
}

impl MiniStep {
    pub(super) fn new(number: &'static str, title: &'static str, body: &'static str) -> Self {
        Self {
            number,
            title,
            body,
        }
    }
}

impl Widget<DocsState> for MiniStep {
    fn build(&self, ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        Card::new(
            vec![
                Text::new(self.number)
                    .size(tokens.typography.font_size_xs)
                    .weight(tokens.typography.font_weight_bold)
                    .color(tokens.colors.primary)
                    .into_node(),
                Text::new(self.title)
                    .size(tokens.typography.font_size_lg)
                    .weight(tokens.typography.font_weight_bold)
                    .color(tokens.colors.heading)
                    .into_node(),
                Paragraph::new(self.body).build(ctx, view),
            ],
            compact_tile_width(tokens),
        )
        .build(ctx, view)
    }
}

#[derive(Clone, Debug)]
pub(super) struct TargetCard {
    title: &'static str,
    body: &'static str,
}

impl TargetCard {
    pub(super) fn new(title: &'static str, body: &'static str) -> Self {
        Self { title, body }
    }
}

impl Widget<DocsState> for TargetCard {
    fn build(&self, ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        Card::new(
            vec![
                Text::new(self.title)
                    .size(tokens.typography.font_size_lg)
                    .weight(tokens.typography.font_weight_bold)
                    .color(tokens.colors.heading)
                    .into_node(),
                Paragraph::new(self.body).build(ctx, view),
                NavLink::new(
                    "See target workflow",
                    "/docs/guides/platform-shells-cli-and-testing/",
                )
                .build(ctx, view),
            ],
            compact_tile_width(tokens),
        )
        .build(ctx, view)
    }
}

#[derive(Clone, Debug)]
pub(super) struct ChartTile {
    title: &'static str,
    body: &'static str,
}

impl ChartTile {
    pub(super) fn new(title: &'static str, body: &'static str) -> Self {
        Self { title, body }
    }
}

impl Widget<DocsState> for ChartTile {
    fn build(&self, ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        Card::new(
            vec![
                ChartPreview.build(ctx, view),
                Text::new(self.title)
                    .size(tokens.typography.font_size_lg)
                    .weight(tokens.typography.font_weight_bold)
                    .color(tokens.colors.heading)
                    .semantics_identifier("site-route:/reference/charts/overview/")
                    .into_node(),
                Paragraph::new(self.body).build(ctx, view),
            ],
            tile_width(tokens),
        )
        .build(ctx, view)
    }
}

#[derive(Clone, Debug)]
pub(super) struct Paragraph {
    body: &'static str,
}

impl Paragraph {
    pub(super) fn new(body: &'static str) -> Self {
        Self { body }
    }
}

impl Widget<DocsState> for Paragraph {
    fn build(&self, _ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        Text::new(self.body)
            .size(tokens.typography.body_medium_size)
            .line_height(tokens.typography.body_medium_size * tokens.typography.line_height_normal)
            .color(tokens.colors.text_secondary)
            .flex_shrink(1.0)
            .into_node()
    }
}

#[derive(Clone, Debug)]
pub(super) struct Card {
    children: Vec<Node>,
    width: f32,
}

impl Card {
    pub(super) fn new(children: Vec<Node>, width: f32) -> Self {
        Self { children, width }
    }
}

impl Widget<DocsState> for Card {
    fn build(&self, _ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        Container::new(
            Column {
                children: self.children.clone(),
                gap: Some(tokens.spacing.m),
                ..Default::default()
            }
            .into_node(),
        )
        .width(self.width)
        .flex_shrink(1.0)
        .padding_all(tokens.spacing.l)
        .bg_fill(Fill::Solid(tokens.colors.surface_raised))
        .border(tokens.colors.border, 1.0)
        .border_radius(tokens.radii.xl)
        .into_node()
    }
}

#[derive(Clone, Debug)]
pub(super) struct ChartPreview;

impl Widget<DocsState> for ChartPreview {
    fn build(&self, _ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
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
}

fn bar(height: f32, color: Color, tokens: &Tokens) -> Node {
    Container::new(Text::new("").into_node())
        .width(tokens.spacing.l)
        .height(height)
        .bg_fill(Fill::Solid(color))
        .border_radius(tokens.radii.medium)
        .into_node()
}

pub(super) fn page_fill(tokens: &Tokens) -> Fill {
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

pub(super) fn hero_text_width(tokens: &Tokens) -> f32 {
    tokens.spacing.xxxxl * 8.25
}

pub(super) fn prose_width(tokens: &Tokens) -> f32 {
    tokens.spacing.xxxxl * 9.0
}

pub(super) fn headline_width(tokens: &Tokens) -> f32 {
    tokens.spacing.xxxxl * 6.5
}

pub(super) fn tile_width(tokens: &Tokens) -> f32 {
    tokens.spacing.xxxxl * 3.5
}

pub(super) fn compact_tile_width(tokens: &Tokens) -> f32 {
    tokens.spacing.xxxxl * 2.8
}

pub(super) fn content_width(tokens: &Tokens) -> f32 {
    tokens.spacing.xxxxl * 17.5
}

pub(super) fn nav_inset(tokens: &Tokens) -> f32 {
    tokens.spacing.xxxxl
}
