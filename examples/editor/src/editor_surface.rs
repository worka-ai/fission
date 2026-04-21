use crate::model::{EditorState, FileBuffer, Language, UpdateFileContent};
use fission_core::op::Color;
use fission_core::ui::{Container, Node, Row, Scroll, Text, TextContent, TextInput};
use fission_core::{ActionEnvelope, BuildCtx, FlexDirection, Handler, View, Widget};
use fission_widgets::{HStack, VStack, Spacer};
use serde_json;

pub struct EditorSurface;

impl Widget<EditorState> for EditorSurface {
    fn build(&self, ctx: &mut BuildCtx<EditorState>, view: &View<EditorState>) -> Node {
        let tokens = &view.env.theme.tokens;

        let Some((tab, buffer)) = view.state.active_buffer() else {
            return Container::new(
                fission_widgets::center::Center {
                    child: Box::new(
                        VStack {
                            spacing: Some(8.0),
                            children: vec![
                                Text::new("Fission Editor")
                                    .size(24.0)
                                    .color(tokens.colors.text_secondary)
                                    .into_node(),
                                Text::new("Open a file from the explorer to start editing")
                                    .size(14.0)
                                    .color(tokens.colors.text_secondary)
                                    .into_node(),
                            ],
                        }
                        .into_node(),
                    ),
                }
                .build(ctx, view),
            )
            .bg(Color { r: 30, g: 30, b: 30, a: 255 })
            .flex_grow(1.0)
            .into_node();
        };

        let content = &buffer.content;
        let language = buffer.language;
        let lines: Vec<&str> = content.lines().collect();
        let line_count = lines.len().max(1);

        let update_id = ctx.bind(
            UpdateFileContent(String::new()),
            (|s: &mut EditorState, a: UpdateFileContent, _| {
                if let Some(tab) = s.open_tabs.get(s.active_tab) {
                    let path = tab.path.clone();
                    if let Some(buf) = s.file_contents.get_mut(&path) {
                        buf.content = a.0;
                    }
                    if let Some(tab) = s.open_tabs.get_mut(s.active_tab) {
                        tab.is_dirty = true;
                    }
                }
            }) as Handler<EditorState, UpdateFileContent>,
        ).id;

        // Line numbers column
        let mut line_num_children = Vec::new();
        for i in 1..=line_count {
            line_num_children.push(
                Container::new(
                    Text::new(format!("{}", i))
                        .size(13.0)
                        .color(Color { r: 120, g: 120, b: 120, a: 255 })
                        .into_node(),
                )
                .height(20.0)
                .into_node(),
            );
        }

        let line_numbers = Container::new(
            VStack {
                spacing: Some(0.0),
                children: line_num_children,
            }
            .into_node(),
        )
        .width(50.0)
        .padding_all(4.0)
        .bg(Color { r: 37, g: 37, b: 38, a: 255 })
        .flex_shrink(0.0)
        .into_node();

        // Text content with syntax coloring
        let colored_lines = colorize(content, language);
        let mut text_children = Vec::new();
        for line_text in &colored_lines {
            text_children.push(
                Container::new(
                    Text::new(line_text.clone())
                        .size(13.0)
                        .color(Color { r: 212, g: 212, b: 212, a: 255 })
                        .into_node(),
                )
                .height(20.0)
                .into_node(),
            );
        }

        let text_area = Container::new(
            VStack {
                spacing: Some(0.0),
                children: text_children,
            }
            .into_node(),
        )
        .padding_all(4.0)
        .flex_grow(1.0)
        .into_node();

        // Editor content: line numbers + text, inside a scroll
        let editor_content = Row {
            children: vec![line_numbers, text_area],
            align_items: fission_ir::op::AlignItems::Start,
            ..Default::default()
        }
        .into_node();

        Container::new(
            Scroll {
                direction: FlexDirection::Column,
                child: Some(Box::new(editor_content)),
                show_scrollbar: true,
                flex_grow: 1.0,
                flex_shrink: 1.0,
                ..Default::default()
            }
            .into_node(),
        )
        .bg(Color { r: 30, g: 30, b: 30, a: 255 })
        .flex_grow(1.0)
        .into_node()
    }
}

/// Simple syntax colorization - returns lines with their text content
/// In a real implementation this would produce TextRuns with per-token colors
fn colorize(content: &str, language: Language) -> Vec<String> {
    content.lines().map(|l| l.to_string()).collect()
}
