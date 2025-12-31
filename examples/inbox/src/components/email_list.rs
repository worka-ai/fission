use fission_core::{BuildCtx, View, Widget, WidgetNodeId, NodeId, Handler};
use fission_core::ui::{Container, Node, Text, TextContent, Button, ButtonVariant, Scroll, Checkbox, Row};
use fission_core::op::Color;
use fission_widgets::{VStack, HStack, LazyColumn, Tabs, TabItem, TextInput, MenuButton, MenuItem, Badge, Divider, Icon, Skeleton, SegmentedControl, Pagination, EmptyState, Hero, DropDown, Tooltip, Popover, DateRangePicker, RangeSlider, Wrap, Tag, Spinner};
use crate::model::{InboxState, SelectTab, UpdateSearch, ToggleFilterDropdown, DismissDropdown, SelectEmail, ToggleEmailSelection, SetComposeOpen, Navigate, SetMobileMenuOpen, SetFilterMode, SetPage, SetAdvancedFiltersOpen, SetSortOption};
use fission_icons::material;
use std::sync::Arc;
use serde_json;
use fission_core::{ActionEnvelope, ActionId};

pub struct EmailList {
    pub folder: String,
}

impl Widget<InboxState> for EmailList {
    fn build(&self, ctx: &mut BuildCtx<InboxState>, view: &View<InboxState>) -> Node {
        let mut list_items = vec![];
        
        // Register handlers and get IDs
        let filter_id = ctx.bind(SetFilterMode(0), (|s: &mut InboxState, a: SetFilterMode, _| s.filter_mode = a.0) as Handler<InboxState, SetFilterMode>).id;
        let page_id = ctx.bind(SetPage(0), (|s: &mut InboxState, a: SetPage, _| s.page = a.0) as Handler<InboxState, SetPage>).id;
        let filters_open_id = ctx.bind(SetAdvancedFiltersOpen(false), (|s: &mut InboxState, a: SetAdvancedFiltersOpen, _| s.show_advanced_filters = a.0) as Handler<InboxState, SetAdvancedFiltersOpen>).id;
        let sort_id = ctx.bind(SetSortOption("Newest".into()), (|s: &mut InboxState, a: SetSortOption, _| s.sort_option = a.0) as Handler<InboxState, SetSortOption>).id;
        let tab_id = ctx.bind(SelectTab(0), (|s: &mut InboxState, a: SelectTab, _| s.active_tab = a.0) as Handler<InboxState, SelectTab>).id;
        let menu_toggle = ctx.bind(ToggleFilterDropdown, (|s: &mut InboxState, _: ToggleFilterDropdown, _| s.show_filter_dropdown = !s.show_filter_dropdown) as Handler<InboxState, ToggleFilterDropdown>);
        
        // Header
        list_items.push(
            Row {
                gap: Some(8.0),
                children: vec![
                    Button {
                        variant: ButtonVariant::Ghost,
                        child: Some(Box::new(Icon::svg(material::navigation::menu::regular()).size(24.0).into_node())),
                        on_press: Some(ctx.bind(SetMobileMenuOpen(true), (|s: &mut InboxState, a: SetMobileMenuOpen, _| s.show_mobile_menu = a.0) as Handler<InboxState, SetMobileMenuOpen>)),
                        ..Default::default()
                    }.into_node(),
                    Text::new(self.folder.clone()).size(24.0).into_node(),
                    Badge { text: "32 new".into(), ..Default::default() }.build(ctx, view),
                    Spinner { id: WidgetNodeId::explicit("sync_spinner"), color: None }.build(ctx, view),
                    fission_core::ui::widgets::Spacer { flex_grow: 1.0, ..Default::default() }.into_node(),
                    Tooltip {
                        id: WidgetNodeId::explicit("compose_tooltip"),
                        text: "Compose new email".into(),
                        is_visible: false,
                        child: Box::new(
                            Button {
                                variant: ButtonVariant::Filled,
                                child: Some(Box::new(Text::new(TextContent::Key("button.compose".into())).color(Color::WHITE).into_node())),
                                on_press: Some(ctx.bind(SetComposeOpen(true), (|s: &mut InboxState, a: SetComposeOpen, _| s.show_compose = a.0) as Handler<InboxState, SetComposeOpen>)),
                                ..Default::default()
                            }.into_node()
                        ),
                    }.build(ctx, view),
                    MenuButton {
                        id: WidgetNodeId::explicit("list_more_menu"),
                        label: "More".into(),
                        is_open: view.state.show_filter_dropdown,
                        on_toggle: Some(menu_toggle),
                        items: vec![
                            MenuItem { label: "Mark all as read".into(), icon: None, on_select: None },
                            MenuItem { label: "Add label".into(), icon: None, on_select: None },
                            MenuItem { label: "Archive all".into(), icon: None, on_select: None },
                        ],
                    }.build(ctx, view),
                ],
                ..Default::default()
            }.into_node()
        );
        
        list_items.push(Divider { orientation: fission_widgets::divider::Orientation::Horizontal }.build(ctx, view));

        // Filter + Search row
        let sort_toggle = if view.state.sort_option == "Newest" { "Oldest" } else { "Newest" };
        let sort_toggle = ActionEnvelope {
            id: sort_id,
            payload: serde_json::to_vec(&SetSortOption(sort_toggle.into())).unwrap(),
        };
        list_items.push(
            HStack {
                spacing: Some(12.0),
                children: vec![
                    SegmentedControl {
                        options: vec!["All".into(), "Unread".into(), "Starred".into()],
                        selected_index: view.state.filter_mode,
                        on_change: Some(Arc::new(move |idx| {
                            ActionEnvelope {
                                id: filter_id,
                                payload: serde_json::to_vec(&SetFilterMode(idx)).unwrap(),
                            }
                        })),
                    }.build(ctx, view),
                    fission_core::ui::widgets::Spacer { flex_grow: 1.0, ..Default::default() }.into_node(),
                    TextInput {
                        value: view.state.search_query.clone(),
                        placeholder: Some("Search mail".into()),
                        width: Some(220.0),
                        ..Default::default()
                    }.into_node(),
                    DropDown {
                        selected: Some(view.state.sort_option.clone()),
                        options: vec!["Newest".into(), "Oldest".into(), "Unread".into()],
                        on_toggle: Some(sort_toggle),
                        ..Default::default()
                    }.build(ctx, view),
                    Popover {
                        id: WidgetNodeId::explicit("advanced_filters"),
                        is_open: view.state.show_advanced_filters,
                        on_toggle: Some(ActionEnvelope {
                            id: filters_open_id,
                            payload: serde_json::to_vec(&SetAdvancedFiltersOpen(!view.state.show_advanced_filters)).unwrap(),
                        }),
                        on_close: Some(ActionEnvelope {
                            id: filters_open_id,
                            payload: serde_json::to_vec(&SetAdvancedFiltersOpen(false)).unwrap(),
                        }),
                        trigger: Box::new(
                            Button {
                                variant: ButtonVariant::Outline,
                                child: Some(Box::new(HStack {
                                    spacing: Some(6.0),
                                    children: vec![
                                        Icon::svg(material::content::filter_list::regular()).size(18.0).into_node(),
                                        Text::new("Filters").into_node(),
                                    ],
                                }.into_node())),
                                ..Default::default()
                            }.into_node()
                        ),
                        content: Box::new(
                            Container::new(
                                VStack {
                                    spacing: Some(12.0),
                                    children: vec![
                                        Text::new("Date range").size(12.0).into_node(),
                                        DateRangePicker {
                                            id_start: WidgetNodeId::explicit("filter_date_start"),
                                            id_end: WidgetNodeId::explicit("filter_date_end"),
                                            start: view.state.schedule_date,
                                            end: view.state.schedule_date,
                                            is_start_open: false,
                                            is_end_open: false,
                                            on_change: None,
                                            on_toggle_start: None,
                                            on_toggle_end: None,
                                            on_close_start: None,
                                            on_close_end: None,
                                        }.build(ctx, view),
                                        Text::new("Size (MB)").size(12.0).into_node(),
                                        RangeSlider {
                                            id: None,
                                            start: 5.0,
                                            end: 50.0,
                                            min: 0.0,
                                            max: 100.0,
                                            on_change: None,
                                        }.build(ctx, view),
                                    ],
                                }.into_node()
                            )
                            .padding_all(12.0)
                            .width(260.0)
                            .into_node()
                        ),
                    }.build(ctx, view),
                ]
            }.build(ctx, view)
        );

        list_items.push(
            Tabs {
                active_index: view.state.active_tab,
                items: vec![
                    TabItem {
                        title: "Primary".into(),
                        content: fission_core::ui::widgets::Spacer::default().into_node(),
                        on_press: Some(ActionEnvelope {
                            id: tab_id,
                            payload: serde_json::to_vec(&SelectTab(0)).unwrap(),
                        }),
                    },
                    TabItem {
                        title: "Social".into(),
                        content: fission_core::ui::widgets::Spacer::default().into_node(),
                        on_press: Some(ActionEnvelope {
                            id: tab_id,
                            payload: serde_json::to_vec(&SelectTab(1)).unwrap(),
                        }),
                    },
                    TabItem {
                        title: "Promotions".into(),
                        content: fission_core::ui::widgets::Spacer::default().into_node(),
                        on_press: Some(ActionEnvelope {
                            id: tab_id,
                            payload: serde_json::to_vec(&SelectTab(2)).unwrap(),
                        }),
                    },
                ],
            }.build(ctx, view)
        );

        if view.state.page == 5 {
            // Empty State Demo
            list_items.push(
                EmptyState {
                    icon: Some(Box::new(Icon::svg(material::content::inbox::regular()).size(48.0).color(Color::BLACK).into_node())),
                    title: "No emails here".into(),
                    description: Some("You have cleared your inbox!".into()),
                    action: Some(Box::new(
                        Button {
                            child: Some(Box::new(Text::new("Reload").into_node())),
                            on_press: None,
                            ..Default::default()
                        }.into_node()
                    )),
                }.build(ctx, view)
            );
        } else {
            let mut email_nodes = Vec::new();
            for i in 0..10 {
                let id = i + (view.state.page * 10);
                let path = format!("/{}/{}", self.folder, id);

                // Demo filter behavior (until we have a real email store in state):
                // - Unread: even ids
                // - Starred: ids divisible by 3
                let is_unread = id % 2 == 0;
                let is_starred = id % 3 == 0;
                match view.state.filter_mode {
                    0 => {} // All
                    1 if !is_unread => continue,
                    2 if !is_starred => continue,
                    _ => {}
                }
                
                let item_content = HStack {
                    spacing: Some(12.0),
                    children: vec![
                        Checkbox { checked: is_unread, ..Default::default() }.into_node(),
                        VStack {
                            spacing: Some(4.0),
                            children: vec![
                                                                HStack {
                                                                    spacing: Some(8.0),
                                                                    children: vec![
                                                                            Hero {
                                                                                tag: format!("email_subject_{}", id),
                                                                                child: Box::new(Text {
                                                                                    content: TextContent::Literal(format!(
                                                                                        "Subject {} — This is a longer subject line to exercise wrapping in layout",
                                                                                        id
                                                                                    )),
                                                                                    font_size: Some(16.0),
                                                                                    ..Default::default()
                                                                                }.into()),
                                                                            }.build(ctx, view),
                                                                        ]
                                                                    }.build(ctx, view),                                Text {
                                    content: TextContent::Literal("Short preview...".into()),
                                    font_size: Some(12.0),
                                    color: Some(Color { r: 100, g: 100, b: 100, a: 255 }),
                                    ..Default::default()
                                }.into(),
                                Wrap {
                                    direction: fission_ir::op::FlexDirection::Row,
                                    spacing: Some(6.0),
                                    children: vec![
                                        Tag { label: if is_starred { "Important" } else { "Updates" }.into(), on_close: None }.build(ctx, view),
                                        Tag { label: "Newsletter".into(), on_close: None }.build(ctx, view),
                                    ],
                                }.build(ctx, view),
                            ]
                        }.build(ctx, view)
                    ]
                }.build(ctx, view);

                let item = Container::new(item_content)
                    .padding_all(12.0)
                    .bg(Color::WHITE)
                    .border(Color { r: 230, g: 230, b: 230, a: 255 }, 1.0)
                    .into_node();

                email_nodes.push(
                    Button {
                        variant: ButtonVariant::Ghost,
                        child: Some(Box::new(item)),
                        on_press: Some(ctx.bind(Navigate(path), (|s: &mut InboxState, a: Navigate, _| s.current_path = a.0) as Handler<InboxState, Navigate>)),
                        ..Default::default()
                    }
                    .into()
                );
            }

            let lazy_id = WidgetNodeId::explicit("email_list");
            let node_id = NodeId::derived(lazy_id.as_u128(), &[view.state.page as u32]); // Re-key on page change

            list_items.push(
                LazyColumn {
                    id: Some(node_id),
                    children: email_nodes,
                    item_height: 100.0, 
                }.into()
            );
        }
        
        // Pagination
        if !view.state.show_compose {
            list_items.push(
                fission_core::ui::widgets::Spacer { height: Some(16.0), ..Default::default() }.into_node()
            );
            list_items.push(
                fission_widgets::center::Center {
                    child: Box::new(Pagination {
                        current_page: view.state.page,
                        total_pages: view.state.total_pages,
                        on_change: Some(Arc::new(move |page| {
                            ActionEnvelope {
                                id: page_id,
                                payload: serde_json::to_vec(&SetPage(page)).unwrap(),
                            }
                        })),
                    }.build(ctx, view))
                }.build(ctx, view)
            );
        }

        Container::new(
            VStack {
                spacing: Some(16.0),
                children: list_items,
            }
            .build(ctx, view)
        )
        .padding_all(16.0)
        .flex_grow(1.0)
        .into_node()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use fission_test::TestHarness;

    fn count_subject_text_nodes(h: &TestHarness<InboxState>) -> usize {
        let ir = h.last_ir.as_ref().unwrap();
        ir.nodes
            .values()
            .filter(|n| {
                matches!(
                    &n.op,
                    fission_ir::Op::Paint(fission_ir::PaintOp::DrawText { text, .. })
                        if text.starts_with("Subject ")
                )
            })
            .count()
    }

    #[test]
    fn filter_mode_changes_list_contents() -> Result<()> {
        struct Root;
        impl Widget<InboxState> for Root {
            fn build(&self, ctx: &mut BuildCtx<InboxState>, view: &View<InboxState>) -> Node {
                EmailList { folder: "inbox".into() }.build(ctx, view)
            }
        }

        let mut h = TestHarness::new(InboxState::default()).with_root_widget(Root);
        h.pump()?;

        let all_count = count_subject_text_nodes(&h);
        assert!(all_count > 0, "expected some subjects in All mode");

        h.dispatch(SetFilterMode(1))?; // Unread
        h.pump()?;
        let unread_count = count_subject_text_nodes(&h);
        assert!(unread_count < all_count, "Unread should show fewer items than All");

        h.dispatch(SetFilterMode(2))?; // Starred
        h.pump()?;
        let starred_count = count_subject_text_nodes(&h);
        assert!(starred_count < all_count, "Starred should show fewer items than All");

        Ok(())
    }
}
