use super::{ActionButton, ButtonTone, OutputPanel};
use crate::ui::actions::{navigate, toggle_theme, Navigate, ToggleTheme};
use crate::ui::routes::UiRoute;
use crate::ui::state::UiState;
use crate::ui::theme::UiPalette;
use fission::ir::op::{AlignItems, JustifyContent};
use fission::prelude::*;

#[derive(Clone)]
pub(crate) struct AppShell {
    pub(crate) content: Node,
}

impl Widget<UiState> for AppShell {
    fn build(&self, ctx: &mut BuildCtx<UiState>, view: &View<UiState>) -> Node {
        let palette = UiPalette::for_mode(view.state.theme_mode);
        let viewport = view.env.viewport_size;
        let width = viewport.width.max(96.0);
        let height = viewport.height.max(28.0);
        let header_h = 4.0;
        let footer_h = 6.0;
        let body_h = (height - header_h - footer_h - 4.0).max(16.0);
        let sidebar_w = 24.0;
        let content_w = (width - sidebar_w - 7.0).max(48.0);

        Container::new(
            Column {
                gap: Some(1.0),
                children: vec![
                    AppHeader.build(ctx, view),
                    Row {
                        gap: Some(2.0),
                        align_items: AlignItems::Stretch,
                        children: vec![
                            Sidebar {
                                width: sidebar_w,
                                height: body_h,
                            }
                            .build(ctx, view),
                            Container::new(self.content.clone())
                                .width(content_w)
                                .height(body_h)
                                .padding([2.0, 2.0, 1.0, 1.0])
                                .bg(palette.surface)
                                .border(palette.border, 1.0)
                                .into_node(),
                        ],
                        ..Default::default()
                    }
                    .into_node(),
                    OutputPanel {
                        width: width - 4.0,
                        height: footer_h,
                    }
                    .build(ctx, view),
                ],
                ..Default::default()
            }
            .into_node(),
        )
        .width(width)
        .height(height)
        .padding([2.0, 2.0, 1.0, 1.0])
        .bg(palette.background)
        .into_node()
    }
}

#[derive(Clone)]
pub(crate) struct AppHeader;

impl Widget<UiState> for AppHeader {
    fn build(&self, ctx: &mut BuildCtx<UiState>, view: &View<UiState>) -> Node {
        let palette = UiPalette::for_mode(view.state.theme_mode);
        let toggle = with_reducer!(ctx, ToggleTheme, toggle_theme);
        Container::new(
            Row {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                children: vec![
                    Column {
                        gap: Some(0.0),
                        children: vec![
                            Text::new("Fission CLI")
                                .color(palette.accent_text)
                                .into_node(),
                            Text::new(format!(
                                "{}  -  {}",
                                view.state.project_name, view.state.project_status
                            ))
                            .color(palette.accent_text)
                            .into_node(),
                        ],
                        ..Default::default()
                    }
                    .into_node(),
                    Row {
                        gap: Some(1.0),
                        children: vec![
                            Text::new(format!("Theme: {}", view.state.theme_mode.label()))
                                .color(palette.accent_text)
                                .into_node(),
                            ActionButton::new("Switch theme", toggle)
                                .tone(ButtonTone::Neutral)
                                .width(16.0)
                                .build(ctx, view),
                            Text::new("q / Esc exits")
                                .color(palette.accent_text)
                                .into_node(),
                        ],
                        ..Default::default()
                    }
                    .into_node(),
                ],
                ..Default::default()
            }
            .into_node(),
        )
        .height(4.0)
        .padding([2.0, 2.0, 1.0, 1.0])
        .bg(palette.accent)
        .border(palette.accent, 1.0)
        .into_node()
    }
}

#[derive(Clone)]
pub(crate) struct Sidebar {
    pub(crate) width: f32,
    pub(crate) height: f32,
}

impl Widget<UiState> for Sidebar {
    fn build(&self, ctx: &mut BuildCtx<UiState>, view: &View<UiState>) -> Node {
        let palette = UiPalette::for_mode(view.state.theme_mode);
        let mut children = vec![Text::new("Navigation").color(palette.accent).into_node()];
        for route in UiRoute::ALL {
            let action = with_reducer!(ctx, Navigate(route), navigate);
            let tone = if view.state.route == route {
                ButtonTone::Primary
            } else {
                ButtonTone::Neutral
            };
            children.push(
                ActionButton::new(route.title(), action)
                    .tone(tone)
                    .width(self.width - 4.0)
                    .build(ctx, view),
            );
        }
        Container::new(
            Column {
                gap: Some(1.0),
                children,
                ..Default::default()
            }
            .into_node(),
        )
        .width(self.width)
        .height(self.height)
        .padding([1.0, 1.0, 1.0, 1.0])
        .bg(palette.raised)
        .border(palette.border, 1.0)
        .into_node()
    }
}
