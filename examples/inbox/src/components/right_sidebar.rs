use fission_core::{BuildCtx, View, Widget, Handler, WidgetNodeId};
use fission_core::ui::{Container, Node, Text, TextContent, Button, ButtonVariant, Row, Column, Switch};
use fission_core::op::Color;
use fission_widgets::{
    VStack, HStack, Card, Divider, Icon, Calendar, Menu, MenuItem, Stat, CircularProgress, Stepper, Tooltip, Skeleton,
};
use crate::model::{InboxState, Folder, SetMeetCameraOn, SetMeetMicOn, SetCalendarSelected, ShowToast};
use fission_icons::material;
use chrono::{Datelike, Local};
use serde_json;

pub struct RightSidebar;

impl Widget<InboxState> for RightSidebar {
    fn build(&self, ctx: &mut BuildCtx<InboxState>, view: &View<InboxState>) -> Node {
        let today = Local::now().date_naive();

        let meet_camera_id = ctx.bind(SetMeetCameraOn(false), (|s: &mut InboxState, a: SetMeetCameraOn, _| s.meet_camera_on = a.0) as Handler<InboxState, SetMeetCameraOn>).id;
        let meet_mic_id = ctx.bind(SetMeetMicOn(false), (|s: &mut InboxState, a: SetMeetMicOn, _| s.meet_mic_on = a.0) as Handler<InboxState, SetMeetMicOn>).id;
        let calendar_id = ctx.bind(SetCalendarSelected(today), (|s: &mut InboxState, a: SetCalendarSelected, _| s.calendar_selected = Some(a.0)) as Handler<InboxState, SetCalendarSelected>).id;
        let toast_id = ctx.bind(ShowToast("".into()), (|s: &mut InboxState, a: ShowToast, _| {
            s.toast_message = Some(a.0);
            s.show_toast = true;
        }) as Handler<InboxState, ShowToast>).id;

        let unread_total = view
            .state
            .emails
            .iter()
            .filter(|e| e.folders.contains(&Folder::Inbox) && !e.is_read)
            .count();
        let starred_total = view.state.emails.iter().filter(|e| e.is_flagged).count();

        let quick_actions = Menu {
            items: vec![
                MenuItem {
                    label: "New event".into(),
                    icon: None,
                    on_select: Some(fission_core::ActionEnvelope {
                        id: toast_id,
                        payload: serde_json::to_vec(&ShowToast("Created a new event".into())).unwrap(),
                    }),
                },
                MenuItem {
                    label: "New task".into(),
                    icon: None,
                    on_select: Some(fission_core::ActionEnvelope {
                        id: toast_id,
                        payload: serde_json::to_vec(&ShowToast("Created a new task".into())).unwrap(),
                    }),
                },
                MenuItem {
                    label: "Add reminder".into(),
                    icon: None,
                    on_select: Some(fission_core::ActionEnvelope {
                        id: toast_id,
                        payload: serde_json::to_vec(&ShowToast("Added a reminder".into())).unwrap(),
                    }),
                },
            ],
            width: Some(220.0),
            max_height: Some(200.0),
        }.build(ctx, view);

        Container::new(
            Column {
                children: vec![
                    Card {
                        child: Box::new(
                            Row {
                                gap: Some(8.0),
                                children: vec![
                                    CircularProgress { value: Some(0.6), size: 28.0, thickness: 3.0, ..Default::default() }.build(ctx, view),
                                    VStack {
                                        spacing: Some(2.0),
                                        children: vec![
                                            Text::new("Syncing").size(12.0).into_node(),
                                            Text::new("Last update 2 min ago").size(10.0).color(Color { r: 120, g: 120, b: 120, a: 255 }).into_node(),
                                            Skeleton {
                                                id: WidgetNodeId::explicit("sync_skeleton"),
                                                width: Some(120.0),
                                                height: Some(6.0),
                                                circle: false,
                                            }.build(ctx, view),
                                        ],
                                    }.into_node(),
                                ],
                                ..Default::default()
                            }.into_node()
                        ),
                        ..Default::default()
                    }.build(ctx, view),

                    Calendar {
                        year: today.year(),
                        month: today.month(),
                        selected_date: view.state.calendar_selected.or(Some(today)),
                        on_select: Some(std::sync::Arc::new(move |d| {
                            fission_core::ActionEnvelope {
                                id: calendar_id,
                                payload: serde_json::to_vec(&SetCalendarSelected(d)).unwrap(),
                            }
                        })),
                        on_navigate: None,
                    }.build(ctx, view),

                    Card {
                        child: Box::new(
                            VStack {
                                spacing: Some(8.0),
                                children: vec![
                                    Text::new("Quick actions").size(14.0).into_node(),
                                    quick_actions,
                                ],
                            }.into_node()
                        ),
                        ..Default::default()
                    }.build(ctx, view),

                    Card {
                        child: Box::new(
                            VStack {
                                spacing: Some(8.0),
                                children: vec![
                                    Text::new("Meet").size(14.0).into_node(),
                                    HStack {
                                        spacing: Some(8.0),
                                        children: vec![
                                            Tooltip {
                                                id: fission_core::WidgetNodeId::explicit("meet_camera_tip"),
                                                text: "Camera".into(),
                                                is_visible: false,
                                                child: Box::new(
                                                    Switch {
                                                        checked: view.state.meet_camera_on,
                                                        on_toggle: Some(fission_core::ActionEnvelope {
                                                            id: meet_camera_id,
                                                            payload: serde_json::to_vec(&SetMeetCameraOn(!view.state.meet_camera_on)).unwrap(),
                                                        }),
                                                        ..Default::default()
                                                    }.into_node()
                                                ),
                                            }.build(ctx, view),
                                            Tooltip {
                                                id: fission_core::WidgetNodeId::explicit("meet_mic_tip"),
                                                text: "Microphone".into(),
                                                is_visible: false,
                                                child: Box::new(
                                                    Switch {
                                                        checked: view.state.meet_mic_on,
                                                        on_toggle: Some(fission_core::ActionEnvelope {
                                                            id: meet_mic_id,
                                                            payload: serde_json::to_vec(&SetMeetMicOn(!view.state.meet_mic_on)).unwrap(),
                                                        }),
                                                        ..Default::default()
                                                    }.into_node()
                                                ),
                                            }.build(ctx, view),
                                        ],
                                    }.into_node(),
                                    Button {
                                        variant: ButtonVariant::Filled,
                                        child: Some(Box::new(
                                            HStack {
                                                spacing: Some(8.0),
                                                children: vec![
                                                    Icon::svg(material::av::video_call::regular()).size(18.0).into_node(),
                                                    Text::new("Start meeting").into_node(),
                                                ],
                                            }.into_node()
                                        )),
                                        on_press: None,
                                        ..Default::default()
                                    }.into_node(),
                                ],
                            }.into_node()
                        ),
                        ..Default::default()
                    }.build(ctx, view),

                    Card {
                        child: Box::new(
                            VStack {
                                spacing: Some(8.0),
                                children: vec![
                                    Text::new("Mailbox stats").size(14.0).into_node(),
                                    HStack {
                                        spacing: Some(8.0),
                                        children: vec![
                                            Stat { label: "Unread".into(), value: unread_total.to_string(), help_text: Some("In Inbox".into()) }.build(ctx, view),
                                            Stat { label: "Starred".into(), value: starred_total.to_string(), help_text: Some("All folders".into()) }.build(ctx, view),
                                        ],
                                    }.into_node(),
                                ],
                            }.into_node()
                        ),
                        ..Default::default()
                    }.build(ctx, view),

                    Card {
                        child: Box::new(
                            VStack {
                                spacing: Some(12.0),
                                children: vec![
                                    Text::new("Setup").size(14.0).into_node(),
                                    Stepper {
                                        steps: vec!["Import".into(), "Customize".into(), "Invite".into()],
                                        active_index: 1,
                                    }.build(ctx, view),
                                ],
                            }.into_node()
                        ),
                        ..Default::default()
                    }.build(ctx, view),
                ],
                ..Default::default()
            }.into_node()
        )
        .padding_all(16.0)
        .width(320.0)
        .bg(Color { r: 250, g: 250, b: 252, a: 255 })
        .into_node()
    }
}
