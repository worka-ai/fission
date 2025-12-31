use fission_core::{BuildCtx, View, Widget, Handler, ActionEnvelope};
use fission_core::ui::{Container, Node, Text, Button, ButtonVariant};
use fission_core::op::Color;
use fission_widgets::{VStack, HStack, TreeView, TreeItem, Divider, Icon, Tag, Wrap, ProgressBar, Link};
use crate::model::{
    InboxState, Folder, SelectFolder, SetSettingsOpen, SetContactsOpen, ToggleBrowserDemo
};

pub struct Sidebar;

impl Widget<InboxState> for Sidebar {
    fn build(&self, ctx: &mut BuildCtx<InboxState>, view: &View<InboxState>) -> Node {
        // ... (routes logic if any) ...
        
        Container::new(
            VStack {
                spacing: Some(8.0),
                children: vec![
                    Text::new("Fission Inbox").size(20.0).into_node(),
                    fission_core::ui::widgets::Spacer { height: Some(16.0), ..Default::default() }.into_node(),
                    
                    TreeView {
                        selected_id: None, // Simplified for now
                        expanded_ids: Default::default(),
                        items: vec![
                            TreeItem { id: "inbox".into(), label: "Inbox".into(), icon: None, children: vec![], on_toggle: None, on_select: Some(ctx.bind(SelectFolder(Folder::Inbox), (|s: &mut InboxState, a: SelectFolder, _| s.selected_folder = a.0) as Handler<InboxState, SelectFolder>)) },
                            TreeItem { id: "starred".into(), label: "Starred".into(), icon: None, children: vec![], on_toggle: None, on_select: Some(ctx.bind(SelectFolder(Folder::Starred), (|s: &mut InboxState, a: SelectFolder, _| s.selected_folder = a.0) as Handler<InboxState, SelectFolder>)) },
                            TreeItem { id: "sent".into(), label: "Sent".into(), icon: None, children: vec![], on_toggle: None, on_select: Some(ctx.bind(SelectFolder(Folder::Sent), (|s: &mut InboxState, a: SelectFolder, _| s.selected_folder = a.0) as Handler<InboxState, SelectFolder>)) },
                        ],
                    }.build(ctx, view),

                    Text::new("Labels").size(12.0).into_node(),
                    Wrap {
                        direction: fission_ir::op::FlexDirection::Row,
                        spacing: Some(6.0),
                        children: vec![
                            Tag { label: "Work".into(), on_close: None }.build(ctx, view),
                            Tag { label: "Personal".into(), on_close: None }.build(ctx, view),
                            Tag { label: "Travel".into(), on_close: None }.build(ctx, view),
                            Tag { label: "Receipts".into(), on_close: None }.build(ctx, view),
                        ],
                    }.build(ctx, view),
                    
                    Button {
                        variant: ButtonVariant::Ghost,
                        child: Some(Box::new(
                            HStack {
                                spacing: Some(12.0),
                                children: vec![
                                    Icon::svg(fission_icons::material::action::language::regular()).size(20.0).into_node(),
                                    Text::new("Browser Demo").into_node(),
                                ]
                            }.into_node()
                        )),
                        on_press: Some(ctx.bind(ToggleBrowserDemo(true), (|s: &mut InboxState, a, _| s.show_browser_demo = a.0) as Handler<InboxState, ToggleBrowserDemo>)),
                        ..Default::default()
                    }.into_node(),

                    fission_core::ui::widgets::Spacer { flex_grow: 1.0, ..Default::default() }.into_node(),
                    
                    Divider { orientation: fission_widgets::divider::Orientation::Horizontal }.build(ctx, view),

                    Text::new("Storage").size(12.0).into_node(),
                    ProgressBar {
                        value: view.state.storage_usage,
                        ..Default::default()
                    }.build(ctx, view),
                    Link {
                        text: "Manage storage".into(),
                        on_click: None,
                    }.build(ctx, view),
                    
                    Button {
                        variant: ButtonVariant::Ghost,
                        child: Some(Box::new(Text::new("Contacts").into_node())),
                        on_press: Some(ctx.bind(SetContactsOpen(true), (|s: &mut InboxState, a: SetContactsOpen, _| s.show_contacts = a.0) as Handler<InboxState, SetContactsOpen>)),
                        ..Default::default()
                    }.into_node(),
                    
                    Button {
                        variant: ButtonVariant::Ghost,
                        child: Some(Box::new(Text::new("Settings").into_node())),
                        on_press: Some(ctx.bind(SetSettingsOpen(true), (|s: &mut InboxState, a: SetSettingsOpen, _| s.show_settings = a.0) as Handler<InboxState, SetSettingsOpen>)),
                        ..Default::default()
                    }.into_node(),
                ],
            }.build(ctx, view)
        )
        .bg(Color { r: 245, g: 245, b: 247, a: 255 })
        .padding_all(16.0)
        .into_node()
    }
}
