use fission_widgets::{
    Button, LoweringContext, Node, Row, Text, Desugar
};
use fission_ir::{NodeId, Op, StructuralOp, LayoutOp, Semantics, Role};
use fission_core::{ActionId, Action as CoreAction};
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
    // Default Text has no semantics, so it maps directly to LayoutOp::Box (or whatever Text maps to)
    // In lib.rs: Op::Layout(LayoutOp::Box)
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
        on_press: Some(*TEST_CLICK_ACTION_ID),
        semantics: Some(Semantics {
            role: Role::Button,
            label: Some("My Button".into()),
            value: None,
            actions: Default::default(),
            focusable: true,
        }),
        ..Default::default()
    };

    let mut cx = LoweringContext::new();
    let button_node_id = button_widget.desugar(&mut cx);

    // Should return the semantics node ID
    assert!(cx.ir.nodes.contains_key(&button_node_id));
    let semantics_node = cx.ir.nodes.get(&button_node_id).unwrap();
    assert!(matches!(semantics_node.op, Op::Semantics(_)));
    
    // Semantics node should have 1 child (the layout node)
    assert_eq!(semantics_node.children.len(), 1);
    let layout_node_id = semantics_node.children[0];
    let layout_node = cx.ir.nodes.get(&layout_node_id).unwrap();
    assert!(matches!(layout_node.op, Op::Layout(LayoutOp::Box { .. })));
}
