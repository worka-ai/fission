use fission_core::action::{Action, AppState};
use fission_core::ui::{Button, Column, Overlay, Row, Scroll, Text, TextContent};
use fission_core::{BuildCtx, Node, View, Widget, LoweringContext, NodeId, Op, LayoutOp, LowerDyn, NodeBuilder, op::StructuralOp};
use fission_macros::Action;
use fission_shell_desktop::DesktopApp;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// --- SIZING WIDGET ---

#[derive(Debug)]
struct SizedBoxLowerer {
    width: Option<f32>,
    height: Option<f32>,
    child: Node,
}

impl LowerDyn for SizedBoxLowerer {
    fn lower_dyn(&self, cx: &mut LoweringContext) -> NodeId {
        let child_id = self.child.lower(cx);
        let mut box_node = NodeBuilder::new(
            cx.next_node_id(),
            Op::Layout(LayoutOp::Box {
                width: self.width,
                height: self.height,
                min_width: None,
                max_width: None,
                min_height: None,
                max_height: None,
                padding: [0.0; 4],
            }),
        );
        box_node.add_child(child_id);
        box_node.build(cx)
    }
}

fn sized_box(child: Node, width: Option<f32>, height: Option<f32>) -> Node {
    Node::Custom(fission_core::CustomNode {
        debug_tag: "SizedBox".into(),
        lowerer: Some(Arc::new(SizedBoxLowerer { width, height, child })),
    })
}


// --- STATE ---

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InboxAppState {
    pub dropdown_open: bool,
    pub selected_option: Option<String>,
}

impl AppState for InboxAppState {}

// --- ACTIONS ---

#[derive(Action, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct OnToggleDropDown;

#[derive(Action, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct OnSelectOption(String);

// --- WIDGETS ---

#[derive(Default)]
pub struct InboxApp;

impl Widget<InboxAppState> for InboxApp {
    fn build(&self, ctx: &mut BuildCtx<InboxAppState>, view: &View<InboxAppState>) -> Node {
        let options = vec![
            "High Priority".to_string(),
            "Medium Priority".to_string(),
            "Low Priority".to_string(),
        ];

        let dropdown_button = Button {
            child: Some(Box::new(
                Text {
                    content: TextContent::Literal(view.state.selected_option.as_deref().unwrap_or("Select an option").into()),
                    ..Default::default()
                }
                .into(),
            )),
            on_press: Some(ctx.bind(
                OnToggleDropDown,
                |state: &mut InboxAppState, _action: OnToggleDropDown| {
                    state.dropdown_open = !state.dropdown_open;
                },
            )),
            ..Default::default()
        }
        .into();

        let dropdown_options = if view.state.dropdown_open {
            let mut option_nodes = vec![];
            for option in &options {
                option_nodes.push(
                    Button {
                        child: Some(Box::new(Text { content: TextContent::Literal(option.clone().into()), ..Default::default() }.into())),
                        on_press: Some(ctx.bind(
                            OnSelectOption(option.clone()),
                            |state: &mut InboxAppState, action: OnSelectOption| {
                                state.selected_option = Some(action.0.clone());
                                state.dropdown_open = false;
                            },
                        )),
                        ..Default::default()
                    }
                    .into(),
                );
            }
            Column { children: option_nodes, ..Default::default() }.into()
        } else {
            Text { content: TextContent::Literal("".into()), ..Default::default() }.into()
        };

        let email_content = Overlay {
            id: None,
            content: Box::new(
                Column {
                    children: vec![
                        Text { content: TextContent::Literal("Email Content".into()), ..Default::default() }.into(),
                        Text { content: TextContent::Literal("From: ...".into()), ..Default::default() }.into(),
                        Text { content: TextContent::Literal("Subject: ...".into()), ..Default::default() }.into(),
                        dropdown_button,
                        Text { content: TextContent::Literal("Body: ...".into()), ..Default::default() }.into(),
                    ],
                    ..Default::default()
                }
                .into(),
            ),
            overlay: Box::new(dropdown_options),
        }
        .into();

        let sidebar = sized_box(
            Column {
                children: vec![
                    Text { content: TextContent::Literal("Folders".into()), ..Default::default() }.into(),
                    Button { child: Some(Box::new(Text{content: TextContent::Literal("Inbox".into()), ..Default::default()}.into())), ..Default::default() }.into(),
                    Button { child: Some(Box::new(Text{content: TextContent::Literal("Sent".into()), ..Default::default()}.into())), ..Default::default() }.into(),
                    Button { child: Some(Box::new(Text{content: TextContent::Literal("Trash".into()), ..Default::default()}.into())), ..Default::default() }.into(),
                ],
                ..Default::default()
            }
            .into(),
            Some(200.0),
            None
        );

        let email_list = sized_box(
            Scroll {
                child: Some(Box::new(Column {
                    children: vec![
                        Text { content: TextContent::Literal("Emails".into()), ..Default::default() }.into(),
                        Button { child: Some(Box::new(Text{content: TextContent::Literal("Email 1".into()), ..Default::default()}.into())), ..Default::default() }.into(),
                        Button { child: Some(Box::new(Text{content: TextContent::Literal("Email 2".into()), ..Default::default()}.into())), ..Default::default() }.into(),
                        Button { child: Some(Box::new(Text{content: TextContent::Literal("Email 3".into()), ..Default::default()}.into())), ..Default::default() }.into(),
                        Button { child: Some(Box::new(Text{content: TextContent::Literal("Email 4".into()), ..Default::default()}.into())), ..Default::default() }.into(),
                        Button { child: Some(Box::new(Text{content: TextContent::Literal("Email 5".into()), ..Default::default()}.into())), ..Default::default() }.into(),
                        Button { child: Some(Box::new(Text{content: TextContent::Literal("Email 6".into()), ..Default::default()}.into())), ..Default::default() }.into(),
                        Button { child: Some(Box::new(Text{content: TextContent::Literal("Email 7".into()), ..Default::default()}.into())), ..Default::default() }.into(),
                        Button { child: Some(Box::new(Text{content: TextContent::Literal("Email 8".into()), ..Default::default()}.into())), ..Default::default() }.into(),
                        Button { child: Some(Box::new(Text{content: TextContent::Literal("Email 9".into()), ..Default::default()}.into())), ..Default::default() }.into(),
                        Button { child: Some(Box::new(Text{content: TextContent::Literal("Email 10".into()), ..Default::default()}.into())), ..Default::default() }.into(),
                    ],
                    ..Default::default()
                }.into())),
                ..Default::default()
            }
            .into(),
            Some(300.0),
            None
        );

        Row {
            children: vec![
                sidebar,
                email_list,
                email_content,
            ],
            ..Default::default()
        }
        .into()
    }
}

fn main() -> anyhow::Result<()> {
    DesktopApp::new(InboxApp::default()).run()
}