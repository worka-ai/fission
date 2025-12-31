use fission_core::{BuildCtx, View, Widget, WidgetNodeId, Handler, ActionEnvelope};
use fission_core::ui::{Container, Node, Text, TextContent, Button, ButtonVariant, Scroll, Video};
use fission_core::op::{Color, ImageFit};
use fission_widgets::{VStack, HStack, Avatar, Accordion, AccordionItem, Card, Image, Spinner, Radio, Breadcrumb, BreadcrumbItem, Alert, AlertKind, Divider, Icon, Timeline, TimelineItem, Hero, Wrap, Tag, SimpleGrid, AspectRatio, Code, Kbd};
use crate::model::{InboxState, ToggleDetails, ToggleToast, SelectReplyMode, Navigate};
use fission_icons::material;
use serde_json;

pub struct EmailDetail {
    pub folder: String,
    pub id: usize,
}

impl Widget<InboxState> for EmailDetail {
    fn build(&self, ctx: &mut BuildCtx<InboxState>, view: &View<InboxState>) -> Node {
        let reply_id = ctx.bind(SelectReplyMode(0), (|s: &mut InboxState, a: SelectReplyMode, _| s.reply_mode = a.0) as Handler<InboxState, SelectReplyMode>).id;

        Container::new(
            VStack {
                spacing: Some(16.0),
                children: vec![
                    // Breadcrumb
                    Breadcrumb {
                        items: vec![
                            BreadcrumbItem { label: self.folder.clone(), on_click: Some(ctx.bind(Navigate(format!("/{}", self.folder)), (|s: &mut InboxState, a: Navigate, _| s.current_path = a.0) as Handler<InboxState, Navigate>)) },
                            BreadcrumbItem { label: format!("Email {}", self.id), on_click: None },
                        ]
                    }.build(ctx, view),
                    
                    // Alert
                    Alert {
                        kind: AlertKind::Warning,
                        title: "External Sender".into(),
                        description: Some("This email is from outside your organization.".into()),
                    }.build(ctx, view),

                    // Header
                    HStack {
                        spacing: Some(8.0),
                        children: vec![
                            Hero {
                                tag: format!("email_subject_{}", self.id),
                                child: Box::new(Text {
                                    content: TextContent::Literal(format!("Subject of Email {}", self.id)),
                                    font_size: Some(24.0),
                                    ..Default::default()
                                }.into()),
                            }.build(ctx, view),
                            fission_core::ui::widgets::Spacer { flex_grow: 1.0, ..Default::default() }.into_node(),
                            Button {
                                variant: ButtonVariant::Outline,
                                child: Some(Box::new(Icon::svg(material::action::delete::regular()).size(20.0).into_node())),
                                on_press: Some(ctx.bind(ToggleToast(true), (|s: &mut InboxState, _: ToggleToast, _| s.show_toast = true) as Handler<InboxState, ToggleToast>)),
                                ..Default::default()
                            }.into_node(),
                        ]
                    }.build(ctx, view),
                    
                    // Sender Info
                    HStack {
                        spacing: Some(8.0),
                        children: vec![
                            Avatar {
                                name: Some("John Doe".into()),
                                size: Some(40.0),
                                ..Default::default()
                            }.build(ctx, view),
                            VStack {
                                spacing: Some(2.0),
                                children: vec![
                                    Text { content: TextContent::Literal("John Doe".into()), font_size: Some(14.0), ..Default::default() }.into(),
                                    Text { content: TextContent::Literal("john@example.com".into()), font_size: Some(12.0), color: Some(Color { r: 120, g: 120, b: 120, a: 255 }), ..Default::default() }.into(),
                                ]
                            }.build(ctx, view)
                        ]
                    }.build(ctx, view),

                    Wrap {
                        direction: fission_ir::op::FlexDirection::Row,
                        spacing: Some(6.0),
                        children: vec![
                            Tag { label: "Important".into(), on_close: None }.build(ctx, view),
                            Tag { label: "Receipts".into(), on_close: None }.build(ctx, view),
                            Tag { label: "2025".into(), on_close: None }.build(ctx, view),
                        ],
                    }.build(ctx, view),
                    
                    Accordion {
                        items: vec![
                            AccordionItem {
                                title: "Details".into(),
                                is_expanded: view.state.details_expanded,
                                on_toggle: Some(ctx.bind(ToggleDetails, (|s: &mut InboxState, _: ToggleDetails, _| s.details_expanded = !s.details_expanded) as Handler<InboxState, ToggleDetails>)),
                                content: Text {
                                    content: TextContent::Literal(format!("Date: Dec 28, 2025\nTo: Me\nCc: Boss")),
                                    font_size: Some(12.0),
                                    color: Some(Color { r: 100, g: 100, b: 100, a: 255 }),
                                    ..Default::default()
                                }.into()
                            }
                        ]
                    }.build(ctx, view),
                    
                    Divider { orientation: fission_widgets::divider::Orientation::Horizontal }.build(ctx, view),
                    
                    // Timeline History
                    Text::new("History").size(18.0).into_node(),
                    Timeline {
                        items: vec![
                            TimelineItem { title: "Received".into(), description: Some("Original message received.".into()), timestamp: Some("Dec 20, 10:00 AM".into()) },
                            TimelineItem { title: "Replied".into(), description: Some("You replied: 'Thanks!'".into()), timestamp: Some("Dec 21, 09:30 AM".into()) },
                            TimelineItem { title: "Received".into(), description: Some("Latest response from John.".into()), timestamp: Some("Dec 28, 2:00 PM".into()) },
                        ]
                    }.build(ctx, view),

                    Text::new("Reply mode").size(12.0).into_node(),
                    HStack {
                        spacing: Some(12.0),
                        children: vec![
                            Radio {
                                checked: view.state.reply_mode == 0,
                                label: Some("Reply".into()),
                                on_select: Some(ActionEnvelope {
                                    id: reply_id,
                                    payload: serde_json::to_vec(&SelectReplyMode(0)).unwrap(),
                                }),
                                ..Default::default()
                            }.into_node(),
                            Radio {
                                checked: view.state.reply_mode == 1,
                                label: Some("Reply all".into()),
                                on_select: Some(ActionEnvelope {
                                    id: reply_id,
                                    payload: serde_json::to_vec(&SelectReplyMode(1)).unwrap(),
                                }),
                                ..Default::default()
                            }.into_node(),
                            Radio {
                                checked: view.state.reply_mode == 2,
                                label: Some("Forward".into()),
                                on_select: Some(ActionEnvelope {
                                    id: reply_id,
                                    payload: serde_json::to_vec(&SelectReplyMode(2)).unwrap(),
                                }),
                                ..Default::default()
                            }.into_node(),
                        ]
                    }.build(ctx, view),
                    
                    fission_core::ui::widgets::Spacer { height: Some(16.0), ..Default::default() }.into_node(),
                    
                    // Body
                    Container::new(
                        VStack {
                            children: vec![
                                Scroll {
                                    child: Some(Box::new(
                                        Text {
                                            content: TextContent::Literal(
                                                "Hey there,\n\nThis is the latest message body.\n\nLorem ipsum dolor sit amet...".into()
                                            ),
                                            ..Default::default()
                                        }.into()
                                    )),
                                    show_scrollbar: true,
                                    ..Default::default()
                                }.into_node()
                            ],
                            ..Default::default()
                        }.into_node()
                    ).flex_grow(1.0).into_node(),

                    Text::new("Attachments").size(16.0).into_node(),
                    HStack {
                        spacing: Some(8.0),
                        children: vec![
                            Spinner { id: WidgetNodeId::explicit("attachments_spinner"), color: None }.build(ctx, view),
                            Text::new("Scanning attachments...").size(12.0).color(Color { r: 120, g: 120, b: 120, a: 255 }).into_node(),
                        ],
                    }.build(ctx, view),
                    SimpleGrid {
                        min_child_width: 120.0,
                        gap: Some(8.0),
                        children: vec![
                            AspectRatio {
                                ratio: 4.0 / 3.0,
                                child: Box::new(
                                    Image {
                                        source: "https://picsum.photos/200/150".into(),
                                        fit: Some(ImageFit::Cover),
                                        ..Default::default()
                                    }.into_node()
                                ),
                            }.build(ctx, view),
                            AspectRatio {
                                ratio: 4.0 / 3.0,
                                child: Box::new(
                                    Image {
                                        source: "https://picsum.photos/201/150".into(),
                                        fit: Some(ImageFit::Cover),
                                        ..Default::default()
                                    }.into_node()
                                ),
                            }.build(ctx, view),
                            AspectRatio {
                                ratio: 16.0 / 9.0,
                                child: Box::new(
                                    Video {
                                        source: "sample.mp4".into(),
                                        autoplay: false,
                                        loop_playback: false,
                                        ..Default::default()
                                    }.into_node()
                                ),
                            }.build(ctx, view),
                        ],
                    }.build(ctx, view),

                    Card {
                        child: Box::new(
                            VStack {
                                spacing: Some(8.0),
                                children: vec![
                                    Text::new("Power user tip").size(14.0).into_node(),
                                    Code { text: "label:important after:2025/01/01".into() }.build(ctx, view),
                                    HStack {
                                        spacing: Some(6.0),
                                        children: vec![
                                            Kbd { text: "g".into() }.build(ctx, view),
                                            Kbd { text: "i".into() }.build(ctx, view),
                                            Text::new("to jump to Inbox").size(12.0).into_node(),
                                        ],
                                    }.into_node(),
                                ],
                            }.into_node()
                        ),
                        ..Default::default()
                    }.build(ctx, view),
                ]
            }
            .build(ctx, view)
        )
        .padding_all(32.0)
        .bg(Color::WHITE)
        .flex_grow(1.0)
        .into_node()
    }
}
