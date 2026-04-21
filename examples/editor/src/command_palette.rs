use crate::model::{EditorState, ToggleCommandPalette, ToggleSidebar, ToggleTerminal, UpdateCommandQuery, SaveFile, SaveAllFiles, RefreshGitStatus, SidebarSection, SetSidebarSection};
use fission_core::op::Color;
use fission_core::ui::{Button, ButtonContentAlign, ButtonVariant, Container, Node, Text, TextInput};
use fission_core::{BuildCtx, Handler, WidgetNodeId, View, Widget};
use fission_widgets::{Modal, ModalAction, VStack, HStack, Spacer, Scroll};

pub struct CommandPalette;

struct Command {
    label: &'static str,
    description: &'static str,
    category: &'static str,
}

impl Widget<EditorState> for CommandPalette {
    fn build(&self, ctx: &mut BuildCtx<EditorState>, view: &View<EditorState>) -> Node {
        if !view.state.show_command_palette {
            return Spacer { height: Some(0.0), ..Default::default() }.into_node();
        }

        let tokens = &view.env.theme.tokens;

        let dismiss = ctx.bind(
            ToggleCommandPalette,
            (|s: &mut EditorState, _, _| {
                s.show_command_palette = false;
                s.command_query.clear();
            }) as Handler<EditorState, ToggleCommandPalette>,
        );

        let update_query = ctx.bind(
            UpdateCommandQuery(String::new()),
            (|s: &mut EditorState, a: UpdateCommandQuery, _| s.command_query = a.0)
                as Handler<EditorState, UpdateCommandQuery>,
        );

        let commands = vec![
            Command { label: "Save", description: "Save the active file", category: "File" },
            Command { label: "Save All", description: "Save all open files", category: "File" },
            Command { label: "Toggle Sidebar", description: "Show or hide the side bar", category: "View" },
            Command { label: "Toggle Terminal", description: "Show or hide the terminal panel", category: "View" },
            Command { label: "Show Explorer", description: "Open the file explorer", category: "View" },
            Command { label: "Show Search", description: "Open the search panel", category: "View" },
            Command { label: "Show Source Control", description: "Open the git panel", category: "View" },
            Command { label: "Refresh Git Status", description: "Fetch latest git status", category: "Git" },
        ];

        let query = view.state.command_query.to_lowercase();
        let filtered: Vec<&Command> = if query.is_empty() {
            commands.iter().collect()
        } else {
            commands.iter().filter(|c| c.label.to_lowercase().contains(&query) || c.description.to_lowercase().contains(&query)).collect()
        };

        // Bind handlers for each command type
        let save = ctx.bind(SaveFile, (|s: &mut EditorState, _, _| { s.save_active_file(); s.show_command_palette = false; }) as Handler<EditorState, SaveFile>);
        let save_all = ctx.bind(SaveAllFiles, (|s: &mut EditorState, _, _| { s.save_all_files(); s.show_command_palette = false; }) as Handler<EditorState, SaveAllFiles>);
        let toggle_sidebar = ctx.bind(ToggleSidebar, (|s: &mut EditorState, _, _| { s.sidebar_visible = !s.sidebar_visible; s.show_command_palette = false; }) as Handler<EditorState, ToggleSidebar>);
        let toggle_terminal = ctx.bind(ToggleTerminal, (|s: &mut EditorState, _, _| { s.terminal_visible = !s.terminal_visible; s.show_command_palette = false; }) as Handler<EditorState, ToggleTerminal>);
        let show_explorer = ctx.bind(SetSidebarSection(SidebarSection::Explorer), (|s: &mut EditorState, _, _| { s.sidebar_section = SidebarSection::Explorer; s.sidebar_visible = true; s.show_command_palette = false; }) as Handler<EditorState, SetSidebarSection>);
        let show_search = ctx.bind(SetSidebarSection(SidebarSection::Search), (|s: &mut EditorState, _, _| { s.sidebar_section = SidebarSection::Search; s.sidebar_visible = true; s.show_command_palette = false; }) as Handler<EditorState, SetSidebarSection>);
        let show_git = ctx.bind(SetSidebarSection(SidebarSection::Git), (|s: &mut EditorState, _, _| { s.sidebar_section = SidebarSection::Git; s.sidebar_visible = true; s.show_command_palette = false; }) as Handler<EditorState, SetSidebarSection>);
        let refresh_git = ctx.bind(RefreshGitStatus, (|s: &mut EditorState, _, _| { s.refresh_git_status(); s.show_command_palette = false; }) as Handler<EditorState, RefreshGitStatus>);

        let action_for = |label: &str| -> fission_core::ActionEnvelope {
            match label {
                "Save" => save.clone(),
                "Save All" => save_all.clone(),
                "Toggle Sidebar" => toggle_sidebar.clone(),
                "Toggle Terminal" => toggle_terminal.clone(),
                "Show Explorer" => show_explorer.clone(),
                "Show Search" => show_search.clone(),
                "Show Source Control" => show_git.clone(),
                "Refresh Git Status" => refresh_git.clone(),
                _ => dismiss.clone(),
            }
        };

        let mut result_items = Vec::new();
        for cmd in &filtered {
            result_items.push(
                Button {
                    variant: ButtonVariant::Ghost,
                    content_align: ButtonContentAlign::Start,
                    child: Some(Box::new(
                        HStack {
                            spacing: Some(8.0),
                            children: vec![
                                Text::new(cmd.label).size(14.0).color(tokens.colors.text_primary).into_node(),
                                Spacer { flex_grow: 1.0, ..Default::default() }.into_node(),
                                Text::new(cmd.description).size(12.0).color(tokens.colors.text_secondary).into_node(),
                            ],
                        }.into_node(),
                    )),
                    on_press: Some(action_for(cmd.label)),
                    height: Some(32.0),
                    padding: Some([8.0, 8.0, 0.0, 0.0]),
                    ..Default::default()
                }.into_node(),
            );
        }

        let content = VStack {
            spacing: Some(4.0),
            children: vec![
                TextInput {
                    value: view.state.command_query.clone(),
                    placeholder: Some("Type a command...".into()),
                    on_change: Some(update_query),
                    ..Default::default()
                }.into_node(),
                Container::new(
                    Scroll {
                        direction: fission_ir::op::FlexDirection::Column,
                        child: Some(Box::new(
                            VStack { spacing: Some(0.0), children: result_items }.into_node(),
                        )),
                        show_scrollbar: true,
                        height: Some(250.0),
                        ..Default::default()
                    }.into_node(),
                ).into_node(),
            ],
        }.into_node();

        Modal {
            id: WidgetNodeId::explicit("command_palette"),
            title: String::new(),
            content: Box::new(content),
            is_open: true,
            on_dismiss: Some(dismiss),
            actions: vec![],
            width: Some(500.0),
        }.build(ctx, view)
    }
}
