use crate::model::{EditorState, FileBuffer, Language, ShowContextMenu, UpdateFileContent};
use crate::syntax;
use fission_core::op::Color;
use fission_core::ui::{Container, GestureDetector, Node, Row, Scroll, Text, TextContent, TextInput};
use fission_core::{ActionEnvelope, BuildCtx, FlexDirection, Handler, View, Widget, WidgetNodeId};
use fission_widgets::{HStack, VStack, Spacer};
use serde_json;

/// Maximum lines to render in the gutter to avoid GPU buffer overflow.
/// The TextInput handles scrolling internally for the content.
const MAX_GUTTER_LINES: usize = 200;

/// Line count threshold above which syntax highlighting is skipped to avoid
/// generating too many IR nodes (TextRuns) that would stall the paint cycle.
const SYNTAX_HIGHLIGHT_LINE_LIMIT: usize = 1000;

pub struct EditorSurface;

impl Widget<EditorState> for EditorSurface {
    fn build(&self, ctx: &mut BuildCtx<EditorState>, view: &View<EditorState>) -> Node {
        let tokens = &view.env.theme.tokens;

        let Some((tab, buffer)) = view.state.active_buffer() else {
            return self.build_welcome_screen(ctx, view);
        };

        let content = &buffer.content;
        let path = tab.path.clone();

        let update_id = ctx.bind(
            UpdateFileContent(String::new()),
            (|s: &mut EditorState, a: UpdateFileContent, _| {
                if let Some(tab) = s.open_tabs.get(s.active_tab) {
                    let path = tab.path.clone();
                    if let Some(buf) = s.file_contents.get_mut(&path) {
                        buf.push_undo();
                        buf.content = a.0;
                        buf.version += 1;
                    }
                    if let Some(tab) = s.open_tabs.get_mut(s.active_tab) {
                        tab.is_dirty = true;
                    }
                    // Notify LSP of the content change
                    if let Some(ref handle) = s.lsp_handle {
                        if let Some(buf) = s.file_contents.get(&path) {
                            handle.notify_change(&path, &buf.content);
                        }
                    }
                }
            }) as Handler<EditorState, UpdateFileContent>,
        );

        // Bind context menu action for long-press
        let context_menu_action = ctx.bind(
            ShowContextMenu { x: 0.0, y: 0.0, target: None },
            (|s: &mut EditorState, a: ShowContextMenu, _| {
                s.context_menu_visible = true;
                s.context_menu_position = (a.x, a.y);
                s.context_menu_target = a.target;
            }) as Handler<EditorState, ShowContextMenu>,
        );

        let line_count = content.lines().count().max(1);
        let visible_lines = line_count.min(MAX_GUTTER_LINES);
        let gutter_width = format!("{}", line_count).len() as f32 * 9.0 + 16.0;
        let is_large_file = line_count > MAX_GUTTER_LINES;

        // Line numbers gutter (capped to MAX_GUTTER_LINES)
        let mut line_num_children = Vec::new();
        for i in 1..=visible_lines {
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
        if is_large_file {
            line_num_children.push(
                Text::new(format!("... +{} lines", line_count - MAX_GUTTER_LINES))
                    .size(11.0)
                    .color(Color { r: 80, g: 80, b: 80, a: 255 })
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

        // Editable text area - for very large files, only put first N lines
        // in the TextInput to avoid GPU overflow
        let edit_content = if is_large_file {
            content.lines().take(MAX_GUTTER_LINES).collect::<Vec<_>>().join("\n")
        } else {
            content.clone()
        };

        // Generate syntax-highlighted runs.
        // For files larger than SYNTAX_HIGHLIGHT_LINE_LIMIT we skip per-line
        // highlighting entirely and emit a single unstyled run.  This avoids
        // creating hundreds of TextRun IR nodes that would overflow the GPU
        // buffer or stall the build/layout/paint cycle.
        let lang = buffer.language;
        let visible_line_count = edit_content.lines().count().max(1);
        let styled_runs: Vec<fission_ir::op::TextRun> = if line_count > SYNTAX_HIGHLIGHT_LINE_LIMIT {
            // Large file — single run, no syntax colours
            vec![fission_ir::op::TextRun {
                text: edit_content.clone(),
                style: fission_ir::op::TextStyle {
                    font_size: 13.0,
                    color: fission_ir::op::Color { r: 212, g: 212, b: 212, a: 255 },
                    underline: false,
                },
            }]
        } else {
            edit_content
                .lines()
                .enumerate()
                .flat_map(|(i, line)| {
                    let spans = syntax::highlight_line(line, lang);
                    let mut runs: Vec<fission_ir::op::TextRun> = spans
                        .into_iter()
                        .map(|span| fission_ir::op::TextRun {
                            text: span.text,
                            style: fission_ir::op::TextStyle {
                                font_size: 13.0,
                                color: fission_ir::op::Color {
                                    r: span.color.r,
                                    g: span.color.g,
                                    b: span.color.b,
                                    a: span.color.a,
                                },
                                underline: false,
                            },
                        })
                        .collect();
                    // Add newline between lines (except last)
                    if i < visible_line_count - 1 {
                        runs.push(fission_ir::op::TextRun {
                            text: "\n".to_string(),
                            style: fission_ir::op::TextStyle {
                                font_size: 13.0,
                                color: fission_ir::op::Color { r: 212, g: 212, b: 212, a: 255 },
                                underline: false,
                            },
                        });
                    }
                    runs
                })
                .collect()
        };

        let editor_input = TextInput {
            id: Some(fission_ir::NodeId::explicit(&format!("editor_{}", path))),
            value: edit_content,
            placeholder: None,
            on_change: Some(update_id),
            width: None,
            height: None,
            multiline: true,
            min_lines: None,
            max_lines: None,
            obscure_text: false,
            obscuring_character: '•',
            mask: None,
            styled_runs: Some(styled_runs),
            borderless: true,
        }
        .into_node();

        let editor_area = Container::new(editor_input)
            .flex_grow(1.0)
            .bg(Color { r: 30, g: 30, b: 30, a: 255 })
            .into_node();

        // Wrap editor area in a GestureDetector for long-press context menu
        let editor_with_gesture = GestureDetector {
            child: Box::new(editor_area),
            on_long_press: Some(context_menu_action),
            ..Default::default()
        }
        .into_node();

        // 1px gutter separator
        let gutter_separator = Container::new(Spacer::default().into_node())
            .width(1.0)
            .bg(Color { r: 48, g: 48, b: 49, a: 255 })
            .flex_shrink(0.0)
            .into_node();

        // Build large file indicator if needed
        let mut editor_column_children = Vec::new();

        if is_large_file {
            let indicator = Container::new(
                HStack {
                    spacing: Some(8.0),
                    children: vec![
                        Text::new(format!(
                            "Large file mode — showing first {} of {} lines",
                            MAX_GUTTER_LINES, line_count
                        ))
                        .size(11.0)
                        .color(Color { r: 180, g: 160, b: 80, a: 255 })
                        .into_node(),
                    ],
                }
                .into_node(),
            )
            .padding_all(4.0)
            .bg(Color { r: 50, g: 45, b: 25, a: 255 })
            .into_node();

            editor_column_children.push(indicator);
        }

        let editor_row = Row {
            children: vec![gutter, gutter_separator, editor_with_gesture],
            align_items: fission_ir::op::AlignItems::Stretch,
            flex_grow: 1.0,
            ..Default::default()
        }
        .into_node();

        // Wrap the editor row in a Scroll widget with visible scrollbar
        let scrollable_editor = Scroll {
            child: Some(Box::new(editor_row)),
            direction: FlexDirection::Column,
            show_scrollbar: true,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        }
        .into_node();

        editor_column_children.push(scrollable_editor);

        let editor_column = VStack {
            spacing: Some(0.0),
            children: editor_column_children,
        }
        .into_node();

        Container::new(editor_column)
            .bg(Color { r: 30, g: 30, b: 30, a: 255 })
            .flex_grow(1.0)
            .into_node()
    }
}

impl EditorSurface {
    fn build_welcome_screen(
        &self,
        ctx: &mut BuildCtx<EditorState>,
        view: &View<EditorState>,
    ) -> Node {
        let dim = Color { r: 100, g: 100, b: 100, a: 255 };
        let shortcut_color = Color { r: 130, g: 130, b: 130, a: 255 };
        let key_color = Color { r: 160, g: 160, b: 160, a: 255 };
        let heading_color = Color { r: 150, g: 150, b: 150, a: 255 };

        let shortcut_row = |keys: &str, desc: &str| -> Node {
            HStack {
                spacing: Some(16.0),
                children: vec![
                    Container::new(
                        Text::new(keys).size(12.0).color(key_color).into_node(),
                    ).width(140.0).into_node(),
                    Text::new(desc).size(12.0).color(shortcut_color).into_node(),
                ],
            }.into_node()
        };

        Container::new(
            fission_widgets::center::Center {
                child: Box::new(
                    VStack {
                        spacing: Some(8.0),
                        children: vec![
                            Text::new("Fission Editor")
                                .size(36.0)
                                .color(Color { r: 80, g: 80, b: 80, a: 255 })
                                .into_node(),
                            Spacer { height: Some(4.0), ..Default::default() }.into_node(),
                            Text::new("Open a file from the explorer to begin")
                                .size(14.0)
                                .color(dim)
                                .into_node(),
                            Spacer { height: Some(16.0), ..Default::default() }.into_node(),
                            // Keyboard shortcuts section
                            Text::new("Keyboard Shortcuts")
                                .size(14.0)
                                .color(heading_color)
                                .into_node(),
                            Spacer { height: Some(4.0), ..Default::default() }.into_node(),
                            shortcut_row("Ctrl+Shift+P", "Command Palette"),
                            shortcut_row("Ctrl+B", "Toggle Sidebar"),
                            shortcut_row("Ctrl+`", "Toggle Terminal"),
                            shortcut_row("Ctrl+S", "Save File"),
                            Spacer { height: Some(20.0), ..Default::default() }.into_node(),
                            // Recent files section
                            Text::new("Recent Files")
                                .size(14.0)
                                .color(heading_color)
                                .into_node(),
                            Spacer { height: Some(4.0), ..Default::default() }.into_node(),
                            Text::new("No recent files")
                                .size(12.0)
                                .color(dim)
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
        .into_node()
    }
}
