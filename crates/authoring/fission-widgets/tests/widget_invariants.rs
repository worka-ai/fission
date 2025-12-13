use fission_widgets::{
    Button, LoweringContext, Node, Row, Text, Desugar
};
use fission_ir::{NodeId, Op, StructuralOp, LayoutOp, Semantics, Role, ActionSet, ActionEntry};
use fission_core::{Action as CoreAction, ActionId};
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};

// Dummy Action for Button
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] 
pub struct TestClickAction { pub value: u32 }

// Manual impl since macro isn't used here to avoid recursion/deps in test file context (simplified)
// Wait, I can use fission_macros::Action if I add it to dev-dependencies.
// But for now manual impl is fine.
impl CoreAction for TestClickAction {
    fn static_id() -> ActionId {
        *TEST_CLICK_ACTION_ID
    }
}

lazy_static! {
    static ref TEST_CLICK_ACTION_ID: ActionId = ActionId::from_name("fission_widgets_test::TestClickAction");
}

#[test]
fn test_text_widget_default_and_desugar() {
    let text_widget = Text::default();
    assert_eq!(text_widget.value, "");

    let mut cx = LoweringContext::new();
    let node_id = text_widget.desugar(&mut cx);
    
    assert!(cx.ir.nodes.contains_key(&node_id));
    let node = cx.ir.nodes.get(&node_id).unwrap();
    assert!(matches!(node.op, Op::Layout(LayoutOp::Box { .. })));
}

#[test]
fn test_row_widget_children_desugar() {
    let row_widget = Row {
        id: None,
        children: vec![
            Text { value: "Hello".into(), ..Default::default() }.into(),
            Text { value: "World".into(), ..Default::default() }.into(),
        ],
        semantics: None,
        ..Default::default()
    };

    let mut cx = LoweringContext::new();
    let row_node_id = row_widget.desugar(&mut cx);
    
    assert!(cx.ir.nodes.contains_key(&row_node_id));
    let row_node = cx.ir.nodes.get(&row_node_id).unwrap();
    assert!(matches!(row_node.op, Op::Layout(LayoutOp::Flex { .. })));
    assert_eq!(row_node.children.len(), 2);
}

#[test]
fn test_button_widget_desugar_with_child_and_semantics() {
    let button_widget = Button {
        id: None,
        child: Some(Box::new(Text { value: "Click Me".into(), ..Default::default() }.into())),
        on_press: Some(TestClickAction { value: 1 }.into()), // Use .into() to create Envelope
        semantics: Some(Semantics {
            role: Role::Button,
            label: Some("My Button".into()),
            value: None,
            actions: ActionSet::default(), // on_press will be added by desugar
            focusable: true,
        }),
        ..Default::default()
    };

    let mut cx = LoweringContext::new();
    let button_node_id = button_widget.desugar(&mut cx);

    assert!(cx.ir.nodes.contains_key(&button_node_id));
    let semantics_node = cx.ir.nodes.get(&button_node_id).unwrap();
    assert!(matches!(semantics_node.op, Op::Semantics(_)));
    
    if let Op::Semantics(s_op) = &semantics_node.op {
        assert_eq!(s_op.actions.entries.len(), 1);
        assert_eq!(s_op.actions.entries[0].action_id, TEST_CLICK_ACTION_ID.as_u128());
        assert!(s_op.actions.entries[0].payload_data.is_some());
    }
}

#[test]
fn test_node_enum_desugar() {
    let node = Node::from(Text::default());
    let mut cx = LoweringContext::new();
    node.desugar(&mut cx);
    assert!(!cx.ir.nodes.is_empty());
}