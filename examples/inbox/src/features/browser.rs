use fission_core::{BuildCtx, View, Widget, WidgetNodeId, NodeId, Handler, ActionEnvelope, ActionId};
use fission_core::ui::{Text, Node, Container, Button as CoreButton, ButtonVariant};
use fission_core::op::Color;
use fission_widgets::{Modal, ModalAction, VStack, HStack, TextInput, WebView};
use crate::model::{
    InboxState, ToggleBrowserDemo, OpenSystemLink, OpenInAppLink, StartAuth
};
use std::sync::Arc;

pub struct BrowserModal;

impl Widget<InboxState> for BrowserModal {
    fn build(&self, ctx: &mut BuildCtx<InboxState>, view: &View<InboxState>) -> Node {
        Modal {
            id: WidgetNodeId::explicit("browser_modal"),
            title: "Browser & Links Demo".into(),
            is_open: true,
            on_dismiss: Some(ctx.bind(ToggleBrowserDemo(false), (|s: &mut InboxState, a, _| s.show_browser_demo = a.0) as Handler<InboxState, ToggleBrowserDemo>)),
            width: Some(700.0),
            content: Box::new(
                VStack {
                    spacing: Some(24.0),
                    children: vec![
                        // Section 1: Embedded
                        VStack {
                            spacing: Some(8.0),
                            children: vec![
                                Text::new("Mechanism 1: Embedded Widget").size(16.0).into_node(),
                                Text::new("A native WebView embedded directly in the layout.")
                                    .size(12.0)
                                    .color(Color { r: 128, g: 128, b: 128, a: 255 })
                                    .into_node(),
                                
                                Container::new(
                                    WebView {
                                        id: WidgetNodeId::explicit("demo_webview"),
                                        url: view.state.browser_url.clone(),
                                        user_agent: None,
                                    }.build(ctx, view)
                                )
                                .width(600.0)
                                .height(300.0)
                                .border(Color::BLACK, 1.0)
                                .into_node(),
                            ]
                        }.into_node(),

                        // Section 2: System / In-App
                        VStack {
                            spacing: Some(8.0),
                            children: vec![
                                Text::new("Mechanism 2: System / In-App").size(16.0).into_node(),
                                HStack {
                                    spacing: Some(16.0),
                                    children: vec![
                                        fission_widgets::Button {
                                            variant: fission_widgets::ButtonVariant::Outline,
                                            child: Some(Box::new(Text::new("Open System Browser").into_node())),
                                            on_press: Some(ctx.bind(OpenSystemLink("https://google.com".into()), (|_, _, _| {}) as Handler<InboxState, OpenSystemLink>)),
                                            ..Default::default()
                                        }.build(ctx, view),
                                        
                                        fission_widgets::Button {
                                            variant: fission_widgets::ButtonVariant::Filled,
                                            child: Some(Box::new(Text::new("Open In-App (Custom Tab)").color(Color::WHITE).into_node())),
                                            on_press: Some(ctx.bind(OpenInAppLink("https://fission.rs".into()), (|_, _, _| {}) as Handler<InboxState, OpenInAppLink>)),
                                            ..Default::default()
                                        }.build(ctx, view),
                                    ]
                                }.into_node()
                            ]
                        }.into_node(),

                        // Section 3: Auth
                        VStack {
                            spacing: Some(8.0),
                            children: vec![
                                Text::new("Mechanism 3: Secure Auth").size(16.0).into_node(),
                                fission_widgets::Button {
                                    variant: fission_widgets::ButtonVariant::Filled,
                                    child: Some(Box::new(Text::new("Log in with Provider").color(Color::WHITE).into_node())),
                                    on_press: Some(ctx.bind(StartAuth, (|_, _, _| {}) as Handler<InboxState, StartAuth>)),
                                    ..Default::default()
                                }.build(ctx, view),
                            ]
                        }.into_node(),
                    ]
                }.into_node()
            ),
            actions: vec![
                ModalAction { 
                    label: "Close".into(), 
                    is_primary: true, 
                    on_press: Some(ctx.bind(ToggleBrowserDemo(false), (|s: &mut InboxState, a, _| s.show_browser_demo = a.0) as Handler<InboxState, ToggleBrowserDemo>)) 
                }
            ]
        }.build(ctx, view)
    }
}