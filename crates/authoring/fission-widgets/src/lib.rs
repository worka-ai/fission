use fission_ir::{NodeId, Op, LayoutOp, StructuralOp, FlexDirection};
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
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Desugar for Text {
    fn desugar(&self, cx: &mut LoweringContext) -> NodeId {
        let node_id = self.id.unwrap_or_else(|| cx.next_node_id());
        // Text is a Box for layout purposes
        cx.add_node(node_id, Op::Layout(LayoutOp::Box { width: self.width, height: self.height }), vec![]);
        node_id
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Row {
    pub id: Option<WidgetNodeId>,
    pub children: Vec<Node>,
    pub semantics: Option<Semantics>,
    pub direction: FlexDirection,
    pub flex_grow: f32,
    pub flex_shrink: f32,
}

impl Desugar for Row {
    fn desugar(&self, cx: &mut LoweringContext) -> NodeId {
        let node_id = self.id.unwrap_or_else(|| cx.next_node_id());
        let mut child_ids = Vec::new();
        for child in &self.children {
            child_ids.push(child.desugar(cx));
        }
        cx.add_node(node_id, Op::Layout(LayoutOp::Flex { 
            direction: self.direction,
            flex_grow: self.flex_grow,
            flex_shrink: self.flex_shrink,
        }), child_ids);
        node_id
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Button {
    pub id: Option<WidgetNodeId>,
    pub child: Option<Box<Node>>,
    pub on_press: Option<ActionId>,
    pub semantics: Option<Semantics>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Desugar for Button {
    fn desugar(&self, cx: &mut LoweringContext) -> NodeId {
        let node_id = self.id.unwrap_or_else(|| cx.next_node_id());
        let mut child_ids = Vec::new();
        
        if let Some(child) = &self.child {
            child_ids.push(child.desugar(cx));
        }
        
        cx.add_node(node_id, Op::Layout(LayoutOp::Box { width: self.width, height: self.height }), child_ids);
        
        if let Some(_s) = &self.semantics {
             cx.add_node(node_id, Op::Structural(StructuralOp::Scope), vec![]); // This is not correctly adding a semantics node, just an extra structural op on the button ID.
        }
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

// Implement From for easier conversion to Node
impl From<Text> for Node { fn from(widget: Text) -> Self { Node::Text(widget) } }
impl From<Row> for Node { fn from(widget: Row) -> Self { Node::Row(widget) } }
impl From<Button> for Node { fn from(widget: Button) -> Self { Node::Button(widget) } }

impl Default for Node {
    fn default() -> Self {
        Node::Text(Text::default())
    }
}