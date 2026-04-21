use crate::model::{EditorState, OpenFile, ToggleCommandPalette, ToggleSidebar, ToggleTerminal, UpdateCommandQuery};
use fission_core::op::Color;
use fission_core::ui::{Button, ButtonContentAlign, ButtonVariant, Container, Node, Text, TextInput};
use fission_core::{ActionEnvelope, BuildCtx, Handler, WidgetNodeId, View, Widget};
use fission_widgets::{Modal, ModalAction, VStack, HStack, Spacer, Scroll};
use serde_json;

pub struct CommandPalette;

#[derive(Debug, Clone)]
struct Command {
    label: String,
    description: String,
    action: CommandAction,
}

#[derive(Debug, Clone)]
enum CommandAction {
    ToggleSidebar,
    ToggleTerminal,
    OpenFile(String),
}

impl Widget<EditorState> for CommandPalette {
    fn build(&self, ctx: &mut BuildCtx<EditorState>, view: &View<EditorState>) -> Node {
        if !view.state.show_command_palette {
            return Spacer { height: Some(0.0), ..Default::default() }.into_node();
        }

        let tokens = &view.env.theme.tokens;

        let dismiss = ctx.bind(
            ToggleCommandPalette,
            (|s: &mut EditorState, _, _| s.show_command_palette = false)
                as Handler<EditorState, ToggleCommandPalette>,
        );

        let update_query = ctx.bind(
            UpdateCommandQuery(String::new()),
            (|s: &mut EditorState, a: UpdateCommandQuery, _| s.command_query = a.0)
                as Handler<EditorState, UpdateCommandQuery>,
        );

        // Available commands
        let commands = vec![
            Command {
                label: "Toggle Sidebar".into(),
                description: "Show or hide the side bar".into(),
                action: CommandAction::ToggleSidebar,
            },
            Command {
                label: "Toggle Terminal".into(),
                description: "Show or hide the terminal panel".into(),
                action: CommandAction::ToggleTerminal,
            },
        ];

        // Filter commands by query
        let query = view.state.command_query.to_lowercase();
        let filtered: Vec<&Command> = if query.is_empty() {
            commands.iter().collect()
        } else {
            commands.iter().filter(|c| c.label.to_lowercase().contains(&query)).collect()
        };

        // Build command list
        let toggle_sidebar_id = ctx.bind(
            ToggleSidebar,
            (|s: &mut EditorState, _, _| {
                s.sidebar_visible = !s.sidebar_visible;
                s.show_command_palette = false;
            }) as Handler<EditorState, ToggleSidebar>,
        );

        let toggle_terminal_id = ctx.bind(
            ToggleTerminal,
            (|s: &mut EditorState, _, _| {
                s.terminal_visible = !s.terminal_visible;
                s.show_command_palette = false;
            }) as Handler<EditorState, ToggleTerminal>,
        );

        let mut result_items = Vec::new();
        for cmd in &filtered {
            let action = match &cmd.action {
                CommandAction::ToggleSidebar => toggle_sidebar_id.clone(),
                CommandAction::ToggleTerminal => toggle_terminal_id.clone(),
                CommandAction::OpenFile(_path) => dismiss.clone(),
            };

            result_items.push(
                Button {
                    variant: ButtonVariant::Ghost,
                    content_align: ButtonContentAlign::Start,
                    child: Some(Box::new(
                        HStack {
                            spacing: Some(8.0),
                            children: vec![
                                Text::new(cmd.label.clone())
                                    .size(14.0)
                                    .color(tokens.colors.text_primary)
                                    .into_node(),
                                Spacer { flex_grow: 1.0, ..Default::default() }.into_node(),
                                Text::new(cmd.description.clone())
                                    .size(12.0)
                                    .color(tokens.colors.text_secondary)
                                    .into_node(),
                            ],
                        }
                        .into_node(),
                    )),
                    on_press: Some(action),
                    height: Some(32.0),
                    padding: Some([8.0, 8.0, 0.0, 0.0]),
                    ..Default::default()
                }
                .into_node(),
            );
        }

        // Build modal
        let content = VStack {
            spacing: Some(4.0),
            children: vec![
                TextInput {
                    value: view.state.command_query.clone(),
                    placeholder: Some("Type a command...".into()),
                    on_change: Some(update_query),
                    ..Default::default()
                }
                .into_node(),
                Container::new(
                    Scroll {
                        direction: fission_ir::op::FlexDirection::Column,
                        child: Some(Box::new(
                            VStack {
                                spacing: Some(0.0),
                                children: result_items,
                            }
                            .into_node(),
                        )),
                        show_scrollbar: true,
                        height: Some(200.0),
                        ..Default::default()
                    }
                    .into_node(),
                )
                .into_node(),
            ],
        }
        .into_node();

        Modal {
            id: WidgetNodeId::explicit("command_palette"),
            title: String::new(), // No title for command palette
            content: Box::new(content),
            is_open: true,
            on_dismiss: Some(dismiss),
            actions: vec![],
            width: Some(500.0),
        }
        .build(ctx, view)
    }
}
