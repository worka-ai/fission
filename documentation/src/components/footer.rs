use super::state::DocsState;
use fission::op::{AlignItems, Fill, FlexWrap, JustifyContent};
use fission::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct DocsFooter;

impl Widget<DocsState> for DocsFooter {
    fn build(&self, ctx: &mut BuildCtx<DocsState>, view: &View<DocsState>) -> Node {
        let tokens = &view.env.theme.tokens;
        Container::new(
            Row {
                children: vec![
                    Column {
                        children: vec![
                            Text::new("Fission")
                                .size(tokens.typography.font_size_lg)
                                .weight(tokens.typography.font_weight_bold)
                                .color(tokens.colors.heading)
                                .into_node(),
                            Text::new("Production Rust UI for desktop, web, Android, and iOS.")
                                .size(tokens.typography.body_medium_size)
                                .line_height(
                                    tokens.typography.body_medium_size
                                        * tokens.typography.line_height_normal,
                                )
                                .color(tokens.colors.text_secondary)
                                .into_node(),
                        ],
                        gap: Some(tokens.spacing.s),
                        ..Default::default()
                    }
                    .into_node(),
                    Row {
                        children: vec![
                            FooterLink::new("Learn", "/docs/learn/overview/").build(ctx, view),
                            FooterLink::new("Guides", "/docs/guides/app-structure/")
                                .build(ctx, view),
                            FooterLink::new("Charts", "/reference/charts/overview/")
                                .build(ctx, view),
                            FooterLink::new("Reference", "/reference/overview/overview/")
                                .build(ctx, view),
                        ],
                        gap: Some(tokens.spacing.l),
                        wrap: FlexWrap::Wrap,
                        justify_content: JustifyContent::End,
                        ..Default::default()
                    }
                    .into_node(),
                ],
                gap: Some(tokens.spacing.xl),
                wrap: FlexWrap::Wrap,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            }
            .into_node(),
        )
        .padding([
            tokens.spacing.xxxxl,
            tokens.spacing.xxxxl,
            tokens.spacing.xl,
            tokens.spacing.xl,
        ])
        .bg_fill(Fill::Solid(tokens.colors.surface))
        .border(tokens.colors.border, 1.0)
        .into_node()
    }
}

#[derive(Clone, Debug)]
struct FooterLink {
    label: &'static str,
    href: &'static str,
}

impl FooterLink {
    fn new(label: &'static str, href: &'static str) -> Self {
        Self { label, href }
    }
}

impl Widget<DocsState> for FooterLink {
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
