use crate::model::EditorState;
use fission_core::op::Color;
use fission_core::ui::{Container, Node, Scroll, Text};
use fission_core::{BuildCtx, FlexDirection, View, Widget};
use fission_widgets::{VStack, HStack, Spacer};

pub struct TerminalPanel;

impl Widget<EditorState> for TerminalPanel {
    fn build(&self, ctx: &mut BuildCtx<EditorState>, view: &View<EditorState>) -> Node {
        let text_color = Color { r: 204, g: 204, b: 204, a: 255 };
        let bg = Color { r: 24, g: 24, b: 24, a: 255 };
        let header_bg = Color { r: 37, g: 37, b: 38, a: 255 };

        // Header
        let header = Container::new(
            HStack {
                spacing: Some(8.0),
                children: vec![
                    Text::new("TERMINAL")
                        .size(11.0)
                        .color(Color { r: 200, g: 200, b: 200, a: 255 })
                        .into_node(),
                    Spacer { flex_grow: 1.0, ..Default::default() }.into_node(),
                    Text::new("×")
                        .size(14.0)
                        .color(Color { r: 160, g: 160, b: 160, a: 255 })
                        .into_node(),
                ],
            }
            .into_node(),
        )
        .bg(header_bg)
        .height(24.0)
        .padding_all(4.0)
        .flex_shrink(0.0)
        .into_node();

        // Terminal output
        let mut lines = Vec::new();
        for line in &view.state.terminal_lines {
            lines.push(
                Text::new(line.clone())
                    .size(13.0)
                    .color(text_color)
                    .into_node(),
            );
        }

        let output = Container::new(
            Scroll {
                direction: FlexDirection::Column,
                child: Some(Box::new(
                    VStack {
                        spacing: Some(2.0),
                        children: lines,
                    }
                    .into_node(),
                )),
                show_scrollbar: true,
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..Default::default()
            }
            .into_node(),
        )
        .bg(bg)
        .padding_all(8.0)
        .flex_grow(1.0)
        .into_node();

        Container::new(
            fission_core::ui::Column {
                children: vec![header, output],
                flex_grow: 1.0,
                ..Default::default()
            }
            .into_node(),
        )
        .height(view.state.terminal_height)
        .flex_shrink(0.0)
        .into_node()
    }
}
