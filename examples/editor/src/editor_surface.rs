use crate::model::{EditorState, FileBuffer, Language, UpdateFileContent};
use fission_core::op::Color;
use fission_core::ui::{Container, Node, Row, Scroll, Text, TextContent, TextInput};
use fission_core::{ActionEnvelope, BuildCtx, FlexDirection, Handler, View, Widget, WidgetNodeId};
use fission_widgets::{HStack, VStack, Spacer};
use serde_json;

pub struct EditorSurface;

impl Widget<EditorState> for EditorSurface {
    fn build(&self, ctx: &mut BuildCtx<EditorState>, view: &View<EditorState>) -> Node {
        let tokens = &view.env.theme.tokens;

        let Some((tab, buffer)) = view.state.active_buffer() else {
            // Welcome screen when no file is open
            return Container::new(
                fission_widgets::center::Center {
                    child: Box::new(
                        VStack {
                            spacing: Some(12.0),
                            children: vec![
                                Text::new("Fission Editor")
                                    .size(28.0)
                                    .color(Color { r: 100, g: 100, b: 100, a: 255 })
                                    .into_node(),
                                Text::new("Open a file from the explorer to start editing")
                                    .size(14.0)
                                    .color(Color { r: 120, g: 120, b: 120, a: 255 })
                                    .into_node(),
                                Text::new("Ctrl+Shift+P  Command Palette")
                                    .size(12.0)
                                    .color(Color { r: 90, g: 90, b: 90, a: 255 })
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
        let path = tab.path.clone();

        // Action for content updates
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
        );

        // Line numbers gutter
        let line_count = content.lines().count().max(1);
        let gutter_width = format!("{}", line_count).len() as f32 * 9.0 + 16.0;
        let mut line_num_children = Vec::new();
        for i in 1..=line_count {
            line_num_children.push(
                Container::new(
                    Text::new(format!("{:>width$}", i, width = format!("{}", line_count).len()))
                        .size(13.0)
                        .color(Color { r: 120, g: 120, b: 120, a: 255 })
                        .into_node(),
                )
                .height(20.0)
                .into_node(),
            );
        }

        let gutter = Container::new(
            VStack { spacing: Some(0.0), children: line_num_children }.into_node(),
        )
        .width(gutter_width)
        .padding_all(4.0)
        .bg(Color { r: 37, g: 37, b: 38, a: 255 })
        .flex_shrink(0.0)
        .into_node();

        // Actual editable text area using multiline TextInput
        let editor_input = TextInput {
            id: Some(fission_ir::NodeId::explicit(&format!("editor_{}", path))),
            value: content.clone(),
            placeholder: None,
            on_change: Some(update_id),
            width: None,
            height: None,
            multiline: true,
            min_lines: Some(line_count),
            max_lines: None,
            obscure_text: false,
            obscuring_character: '•',
            mask: None,
        }
        .into_node();

        let editor_area = Container::new(editor_input)
            .flex_grow(1.0)
            .bg(Color { r: 30, g: 30, b: 30, a: 255 })
            .into_node();

        // Wrap gutter + editor in a row, both inside a vertical scroll
        let editor_row = Row {
            children: vec![gutter, editor_area],
            align_items: fission_ir::op::AlignItems::Start,
            flex_grow: 1.0,
            ..Default::default()
        }
        .into_node();

        Container::new(editor_row)
            .bg(Color { r: 30, g: 30, b: 30, a: 255 })
            .flex_grow(1.0)
            .into_node()
    }
}
