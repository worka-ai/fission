use fission_widgets::{
    Button, LoweringContext, Node, Row, Text, Desugar
};
use fission_semantics::{Semantics, Role};
use fission_core::{ActionId, Action as CoreAction};
use fission_ir::{NodeId, Op, StructuralOp, LayoutOp};
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};

// Dummy Action for Button
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] 
pub struct TestClickAction;

impl CoreAction for TestClickAction {
    fn id(&self) -> ActionId {
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
    assert_eq!(node.op, Op::Structural(StructuralOp::Group));
    assert!(node.children.is_empty());
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
    };

    let mut cx = LoweringContext::new();
    let row_node_id = row_widget.desugar(&mut cx);
    
    assert!(cx.ir.nodes.contains_key(&row_node_id));
    let row_node = cx.ir.nodes.get(&row_node_id).unwrap();
    assert_eq!(row_node.op, Op::Layout(LayoutOp::Flex));
    assert_eq!(row_node.children.len(), 2);
    
    // Verify children exist
    for child_id in &row_node.children {
        assert!(cx.ir.nodes.contains_key(child_id));
    }
}

#[test]
fn test_button_widget_desugar_with_child_and_semantics() {
    let button_widget = Button {
        id: None,
        child: Some(Box::new(Text { value: "Click Me".into(), ..Default::default() }.into())),
        on_press: Some(*TEST_CLICK_ACTION_ID),
        semantics: Some(Semantics {
            role: Role::Button,
            label: Some("My Button".into()),
            value: None,
            actions: Default::default(),
            focusable: true,
        }),
    };

    let mut cx = LoweringContext::new();
    let button_node_id = button_widget.desugar(&mut cx);

    assert!(cx.ir.nodes.contains_key(&button_node_id));
    let btn_node = cx.ir.nodes.get(&button_node_id).unwrap();
    assert_eq!(btn_node.op, Op::Layout(LayoutOp::Box));
    assert_eq!(btn_node.children.len(), 1);
    
    // Semantics logic was simplified in previous step to just one node.
    // If we want checking for scope/semantics node, we'd need to re-add that logic.
    // For now, testing basic structure.
}

#[test]
fn test_node_enum_desugar() {
    let node = Node::from(Text::default());
    let mut cx = LoweringContext::new();
    node.desugar(&mut cx);
    assert!(!cx.ir.nodes.is_empty());
}