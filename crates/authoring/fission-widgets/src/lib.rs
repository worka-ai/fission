use fission_ir::{NodeId, Op, LayoutOp, StructuralOp};
use fission_semantics::{Semantics, Role};
use fission_core::{Action as CoreAction, ActionId};
use serde::{Serialize, Deserialize};
use anyhow::Result;
use lazy_static::lazy_static;
use std::fmt::Debug;

pub use fission_core::{Desugar, LoweringContext}; 

pub type WidgetNodeId = NodeId;

// --- Basic Widgets --- //

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub id: Option<WidgetNodeId>,
    pub value: String,
    pub semantics: Option<Semantics>,
}

impl Desugar for Text {
    fn desugar(&self, cx: &mut LoweringContext) -> NodeId {
        let node_id = self.id.unwrap_or_else(|| cx.next_node_id());
        cx.add_node(node_id, Op::Structural(StructuralOp::Group), vec![]);
        node_id
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Row {
    pub id: Option<WidgetNodeId>,
    pub children: Vec<Node>,
    pub semantics: Option<Semantics>,
}

impl Desugar for Row {
    fn desugar(&self, cx: &mut LoweringContext) -> NodeId {
        let node_id = self.id.unwrap_or_else(|| cx.next_node_id());
        let mut child_ids = Vec::new();
        for child in &self.children {
            child_ids.push(child.desugar(cx));
        }
        cx.add_node(node_id, Op::Layout(LayoutOp::Flex), child_ids);
        node_id
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Button {
    pub id: Option<WidgetNodeId>,
    pub child: Option<Box<Node>>,
    pub on_press: Option<ActionId>,
    pub semantics: Option<Semantics>,
}

impl Desugar for Button {
    fn desugar(&self, cx: &mut LoweringContext) -> NodeId {
        let node_id = self.id.unwrap_or_else(|| cx.next_node_id());
        let mut child_ids = Vec::new();
        
        // Scope logic: if semantics are present, we might wrap the child in a Scope node?
        // Or Button itself is the Scope/Box?
        // For simplicity: Button -> LayoutOp::Box.
        // If semantics exist, they attach to this node (in a real system).
        // Here we just replicate the previous structure: Button -> Box -> Children.
        // If we want a separate Semantics node, it would be a child or parent.
        // Let's stick to Button node = Box op.
        
        if let Some(child) = &self.child {
            child_ids.push(child.desugar(cx));
        }
        
        cx.add_node(node_id, Op::Layout(LayoutOp::Box), child_ids);
        
        // If semantics were separate ops, we'd need to link them.
        // Previous code pushed StructuralOp::Scope. Let's assume Button IS the Scope/Box combo.
        // Or we create a parent wrapper?
        // Let's keep it simple: Button lowers to one Core Node (Box) for now.
        
        node_id
    }
}

// Node enum
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Node {
    Text(Text),
    Row(Row),
    Button(Button),
}

impl Desugar for Node {
    fn desugar(&self, cx: &mut LoweringContext) -> NodeId {
        match self {
            Node::Text(w) => w.desugar(cx),
            Node::Row(w) => w.desugar(cx),
            Node::Button(w) => w.desugar(cx),
        }
    }
}

impl From<Text> for Node { fn from(widget: Text) -> Self { Node::Text(widget) } }
impl From<Row> for Node { fn from(widget: Row) -> Self { Node::Row(widget) } }
impl From<Button> for Node { fn from(widget: Button) -> Self { Node::Button(widget) } }

impl Default for Node {
    fn default() -> Self {
        Node::Text(Text::default())
    }
}
