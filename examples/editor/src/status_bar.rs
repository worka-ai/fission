use crate::model::EditorState;
use fission_core::op::Color;
use fission_core::ui::{Container, Node, Text};
use fission_core::{BuildCtx, View, Widget};
use fission_widgets::{HStack, Spacer};

pub struct StatusBar;

impl Widget<EditorState> for StatusBar {
    fn build(&self, ctx: &mut BuildCtx<EditorState>, view: &View<EditorState>) -> Node {
        let bg = Color { r: 0, g: 122, b: 204, a: 255 }; // VS Code blue
        let text_color = Color { r: 255, g: 255, b: 255, a: 255 };

        let mut items = vec![];

        // Branch indicator
        items.push(
            Text::new("main")
                .size(12.0)
                .color(text_color)
                .into_node(),
        );

        items.push(
            Spacer { width: Some(16.0), ..Default::default() }.into_node()
        );

        // Active file info
        if let Some((tab, buf)) = view.state.active_buffer() {
            items.push(
                Text::new(format!("Ln {}, Col {}", buf.cursor_line + 1, buf.cursor_col + 1))
                    .size(12.0)
                    .color(text_color)
                    .into_node(),
            );

            items.push(
                Spacer { width: Some(16.0), ..Default::default() }.into_node()
            );

            items.push(
                Text::new(buf.language.display_name())
                    .size(12.0)
                    .color(text_color)
                    .into_node(),
            );

            items.push(
                Spacer { width: Some(16.0), ..Default::default() }.into_node()
            );

            items.push(
                Text::new("UTF-8")
                    .size(12.0)
                    .color(text_color)
                    .into_node(),
            );
        }

        items.push(
            Spacer { flex_grow: 1.0, ..Default::default() }.into_node()
        );

        // Status message
        if let Some(msg) = &view.state.status_message {
            items.push(
                Text::new(msg.clone())
                    .size(12.0)
                    .color(text_color)
                    .into_node(),
            );
        }

        // Diagnostics summary
        let error_count: usize = view.state.diagnostics.values()
            .flat_map(|d| d.iter())
            .filter(|d| d.severity == crate::model::DiagSeverity::Error)
            .count();
        let warn_count: usize = view.state.diagnostics.values()
            .flat_map(|d| d.iter())
            .filter(|d| d.severity == crate::model::DiagSeverity::Warning)
            .count();

        if error_count > 0 || warn_count > 0 {
            items.push(
                Text::new(format!("⚠ {} ✕ {}", warn_count, error_count))
                    .size(12.0)
                    .color(text_color)
                    .into_node(),
            );
        }

        Container::new(
            HStack {
                spacing: Some(0.0),
                children: items,
            }
            .into_node(),
        )
        .bg(bg)
        .height(22.0)
        .padding_all(4.0)
        .flex_shrink(0.0)
        .into_node()
    }
}
