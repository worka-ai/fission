use crate::model::{EditorState, FileEntry, OpenFile, ToggleTreeNode, SelectTreeNode};
use fission_core::op::Color;
use fission_core::ui::{Button, ButtonContentAlign, ButtonVariant, Container, Node, Text};
use fission_core::{ActionEnvelope, BuildCtx, Handler, View, Widget};
use fission_widgets::{HStack, Icon, Spacer, VStack};
use serde_json;

pub struct FileTree;

impl Widget<EditorState> for FileTree {
    fn build(&self, ctx: &mut BuildCtx<EditorState>, view: &View<EditorState>) -> Node {
        let tokens = &view.env.theme.tokens;
        let entries = crate::model::scan_directory(&view.state.root_path, 0);

        let toggle_id = ctx.bind(
            ToggleTreeNode(String::new()),
            (|s: &mut EditorState, a: ToggleTreeNode, _| {
                if !s.tree_expanded.remove(&a.0) {
                    s.tree_expanded.insert(a.0);
                }
            }) as Handler<EditorState, ToggleTreeNode>,
        ).id;

        let open_id = ctx.bind(
            OpenFile(String::new()),
            (|s: &mut EditorState, a: OpenFile, _| {
                s.open_file(a.0);
            }) as Handler<EditorState, OpenFile>,
        ).id;

        let mut rows = Vec::new();
        for entry in &entries {
            build_tree_rows(
                entry,
                0,
                &mut rows,
                view,
                toggle_id,
                open_id,
            );
        }

        Container::new(
            fission_core::ui::Scroll {
                direction: fission_ir::op::FlexDirection::Column,
                show_scrollbar: true,
                flex_grow: 1.0,
                flex_shrink: 1.0,
                child: Some(Box::new(
                    VStack {
                        spacing: Some(0.0),
                        children: rows,
                    }
                    .into_node(),
                )),
                ..Default::default()
            }
            .into_node(),
        )
        .bg(tokens.colors.surface)
        .flex_grow(1.0)
        .into_node()
    }
}

fn build_tree_rows(
    entry: &FileEntry,
    depth: usize,
    rows: &mut Vec<Node>,
    view: &View<EditorState>,
    toggle_id: fission_core::ActionId,
    open_id: fission_core::ActionId,
) {
    let tokens = &view.env.theme.tokens;
    let is_expanded = view.state.tree_expanded.contains(&entry.path);
    let is_selected = view.state.tree_selected.as_deref() == Some(&entry.path);

    // IntelliJ-style: colored icons, bold modified, compact rows
    let icon_color = if entry.is_dir {
        Color { r: 204, g: 166, b: 75, a: 255 } // Warm yellow for folders
    } else {
        file_icon_color(&entry.name)
    };

    let indent = depth as f32 * 16.0;
    let chevron = if entry.is_dir {
        if is_expanded { "▾" } else { "▸" }
    } else {
        " "
    };

    let file_icon = if entry.is_dir {
        if is_expanded {
            fission_icons::material::file::folder_open::regular()
        } else {
            fission_icons::material::file::folder::regular()
        }
    } else {
        fission_icons::material::action::description::regular()
    };

    let bg = if is_selected {
        tokens.colors.primary.with_alpha(30)
    } else {
        Color { r: 0, g: 0, b: 0, a: 0 }
    };

    let action = if entry.is_dir {
        ActionEnvelope {
            id: toggle_id,
            payload: serde_json::to_vec(&ToggleTreeNode(entry.path.clone())).unwrap(),
        }
    } else {
        ActionEnvelope {
            id: open_id,
            payload: serde_json::to_vec(&OpenFile(entry.path.clone())).unwrap(),
        }
    };

    let row = Button {
        variant: ButtonVariant::Ghost,
        content_align: ButtonContentAlign::Start,
        on_press: Some(action),
        child: Some(Box::new(
            Container::new(
                HStack {
                    spacing: Some(4.0),
                    children: vec![
                        Spacer {
                            width: Some(indent),
                            ..Default::default()
                        }
                        .into_node(),
                        Text::new(chevron)
                            .size(10.0)
                            .color(tokens.colors.text_secondary)
                            .into_node(),
                        Icon::svg(file_icon)
                            .size(16.0)
                            .color(icon_color)
                            .into_node(),
                        Text::new(entry.name.clone())
                            .size(13.0)
                            .color(tokens.colors.text_primary)
                            .flex_grow(1.0)
                            .into_node(),
                    ],
                }
                .into_node(),
            )
            .bg(bg)
            .padding_all(2.0)
            .into_node(),
        )),
        height: Some(24.0),
        padding: Some([0.0; 4]),
        ..Default::default()
    }
    .into_node();

    rows.push(row);

    if entry.is_dir && is_expanded {
        for child in &entry.children {
            build_tree_rows(child, depth + 1, rows, view, toggle_id, open_id);
        }
    }
}

fn file_icon_color(name: &str) -> Color {
    let ext = name.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => Color { r: 222, g: 120, b: 50, a: 255 },   // Rust orange
        "toml" => Color { r: 140, g: 180, b: 100, a: 255 }, // Green
        "md" => Color { r: 66, g: 133, b: 244, a: 255 },    // Blue
        "json" => Color { r: 255, g: 193, b: 7, a: 255 },   // Amber
        "lock" => Color { r: 130, g: 130, b: 130, a: 255 },  // Gray
        _ => Color { r: 160, g: 160, b: 160, a: 255 },       // Default gray
    }
}
