use fission_core::op::Color;
use fission_core::ui::{Column, Container, Node, Row, Text};
use fission_core::{AppState, BuildCtx, Handler, View, Widget, WidgetNodeId};
use fission_macros::Action;
use fission_shell_desktop::DesktopApp;
use fission_widgets::{HStack, SplitDirection, SplitView, VStack, Spacer};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod model;
mod file_tree;
mod editor_surface;
mod tab_bar;
mod status_bar;
mod terminal_panel;
mod command_palette;

use model::*;
use file_tree::FileTree;
use editor_surface::EditorSurface;
use tab_bar::TabBar;
use status_bar::StatusBar;
use terminal_panel::TerminalPanel;
use command_palette::CommandPalette;

// --- Activity bar (left icon strip, like VS Code) ---

struct ActivityBar;

impl Widget<EditorState> for ActivityBar {
    fn build(&self, ctx: &mut BuildCtx<EditorState>, view: &View<EditorState>) -> Node {
        let bg = Color { r: 51, g: 51, b: 51, a: 255 };
        let active_color = Color { r: 255, g: 255, b: 255, a: 255 };
        let inactive_color = Color { r: 120, g: 120, b: 120, a: 255 };

        let toggle_sidebar = ctx.bind(
            ToggleSidebar,
            (|s: &mut EditorState, _, _| s.sidebar_visible = !s.sidebar_visible)
                as Handler<EditorState, ToggleSidebar>,
        );

        let sections = vec![
            ("☰", SidebarSection::Explorer, "Explorer"),
            ("🔍", SidebarSection::Search, "Search"),
            ("⎇", SidebarSection::Git, "Source Control"),
            ("⧉", SidebarSection::Extensions, "Extensions"),
        ];

        let set_section_id = ctx.bind(
            SetSidebarSection(SidebarSection::Explorer),
            (|s: &mut EditorState, a: SetSidebarSection, _| {
                if s.sidebar_section == a.0 {
                    s.sidebar_visible = !s.sidebar_visible;
                } else {
                    s.sidebar_section = a.0;
                    s.sidebar_visible = true;
                }
            }) as Handler<EditorState, SetSidebarSection>,
        ).id;

        let mut icons = Vec::new();
        for (icon, section, _label) in &sections {
            let is_active = view.state.sidebar_visible && view.state.sidebar_section == *section;
            let color = if is_active { active_color } else { inactive_color };

            icons.push(
                fission_core::ui::Button {
                    variant: fission_core::ui::ButtonVariant::Ghost,
                    child: Some(Box::new(
                        Text::new(*icon)
                            .size(18.0)
                            .color(color)
                            .into_node(),
                    )),
                    on_press: Some(fission_core::ActionEnvelope {
                        id: set_section_id,
                        payload: serde_json::to_vec(&SetSidebarSection(*section)).unwrap(),
                    }),
                    width: Some(48.0),
                    height: Some(48.0),
                    padding: Some([0.0; 4]),
                    ..Default::default()
                }
                .into_node(),
            );
        }

        Container::new(
            Column {
                children: icons,
                ..Default::default()
            }
            .into_node(),
        )
        .width(48.0)
        .bg(bg)
        .flex_shrink(0.0)
        .into_node()
    }
}

// --- Main app ---

struct EditorApp;

impl Widget<EditorState> for EditorApp {
    fn build(&self, ctx: &mut BuildCtx<EditorState>, view: &View<EditorState>) -> Node {
        let dark_bg = Color { r: 30, g: 30, b: 30, a: 255 };
        let surface_bg = Color { r: 37, g: 37, b: 38, a: 255 };

        // Activity bar (leftmost strip)
        let activity_bar = ActivityBar.build(ctx, view);

        // Sidebar (file tree)
        let sidebar = if view.state.sidebar_visible {
            let header = Container::new(
                Text::new("EXPLORER")
                    .size(11.0)
                    .color(Color { r: 187, g: 187, b: 187, a: 255 })
                    .into_node(),
            )
            .bg(surface_bg)
            .height(24.0)
            .padding_all(6.0)
            .flex_shrink(0.0)
            .into_node();

            Container::new(
                Column {
                    children: vec![header, FileTree.build(ctx, view)],
                    flex_grow: 1.0,
                    ..Default::default()
                }
                .into_node(),
            )
            .width(view.state.sidebar_width)
            .bg(surface_bg)
            .flex_shrink(0.0)
            .into_node()
        } else {
            Spacer { width: Some(0.0), ..Default::default() }.into_node()
        };

        // Editor area (tabs + editor surface)
        let editor_area = Column {
            children: vec![
                TabBar.build(ctx, view),
                EditorSurface.build(ctx, view),
            ],
            flex_grow: 1.0,
            ..Default::default()
        }
        .into_node();

        // Center: editor area + terminal
        let center = Column {
            children: if view.state.terminal_visible {
                vec![
                    Container::new(editor_area).flex_grow(1.0).into_node(),
                    TerminalPanel.build(ctx, view),
                ]
            } else {
                vec![Container::new(editor_area).flex_grow(1.0).into_node()]
            },
            flex_grow: 1.0,
            ..Default::default()
        }
        .into_node();

        // Main layout: activity bar | sidebar | center
        let main_layout = Row {
            children: vec![activity_bar, sidebar, center],
            align_items: fission_ir::op::AlignItems::Stretch,
            flex_grow: 1.0,
            ..Default::default()
        }
        .into_node();

        // Root: main + status bar
        let root = Column {
            children: vec![
                Container::new(main_layout).flex_grow(1.0).into_node(),
                StatusBar.build(ctx, view),
            ],
            flex_grow: 1.0,
            ..Default::default()
        }
        .into_node();

        // Command palette overlay
        CommandPalette.build(ctx, view);

        root
    }
}

fn main() -> anyhow::Result<()> {
    // Determine the root path - default to current directory or the fission repo
    let root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let app = DesktopApp::new(EditorApp)
        .with_title("Fission Editor")
        .with_sync_env(move |state: &EditorState, env: &mut fission_core::Env| {
            // Use dark theme
            env.theme = fission_theme::Theme::dark();
        });

    // Set root path on state - we need to set it before the first render
    // Since DesktopApp uses Default for state, we'll set it in a reducer
    // that fires on the first frame. For now, we use an env var.
    std::env::set_var("FISSION_EDITOR_ROOT", root.to_string_lossy().as_ref());

    app.run()
}
