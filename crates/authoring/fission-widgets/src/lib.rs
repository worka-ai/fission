#![allow(unused_imports)]

use fission_macros::Action;
use fission_core::{Action as CoreAction, ActionId};
use serde::{Serialize, Deserialize};

#[derive(Action, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct MyTestAppAction { pub value: u32 }

// We will add widgets here later

// Current widgets (from the previous step)
use fission_ir::{NodeId, Op, LayoutOp, StructuralOp};
use fission_semantics::{Semantics, Role};
use anyhow::Result;
use lazy_static::lazy_static;
use std::fmt::Debug;

pub type WidgetNodeId = NodeId;

// Placeholder for a lowering context. In reality, this would accumulate Core IR nodes.
pub struct LoweringContext {
    pub next_node_id_seed: u128,
    pub ops: Vec<(NodeId, Op)>, // Accumulate Core IR operations
}

impl LoweringContext {
    pub fn new() -> Self { LoweringContext { next_node_id_seed: 0, ops: Vec::new() } }

    pub fn next_node_id(&mut self) -> NodeId {
        self.next_node_id_seed += 1;
        NodeId::derived(0, &[self.next_node_id_seed as u32])
    }

    pub fn push_op(&mut self, node_id: NodeId, op: Op) {
        self.ops.push((node_id, op));
    }
}

// The Desugar trait as defined in 03-3-authoring-node-tree-model.md
pub trait Desugar: Send + Sync + Debug + PartialEq {
    fn desugar(&self, cx: &mut LoweringContext) -> NodeId;
}

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
        // For now, just push a dummy StructuralOp, will be more detailed later
        cx.push_op(node_id, Op::Structural(StructuralOp::Group));
        // In a real scenario, this would generate multiple ops for text layout, painting, etc.
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
        cx.push_op(node_id, Op::Layout(LayoutOp::Flex)); // Row implies Flex layout
        for child in &self.children {
            child.desugar(cx);
        }
        node_id
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Button {
    pub id: Option<WidgetNodeId>,
    pub child: Option<Box<Node>>,
    pub on_press: Option<ActionId>, // Reference an action ID directly for now
    pub semantics: Option<Semantics>,
}

impl Desugar for Button {
    fn desugar(&self, cx: &mut LoweringContext) -> NodeId {
        let node_id = self.id.unwrap_or_else(|| cx.next_node_id());
        cx.push_op(node_id, Op::Layout(LayoutOp::Box)); // Button implies a Box layout
        if let Some(child) = &self.child {
            child.desugar(cx);
        }
        // Simulate pushing semantics for the button
        if let Some(_s) = &self.semantics {
            // In real lowering, `fission-semantics` types would be converted to Core IR ops.
            // For now, just a dummy op to indicate semantics were processed.
             cx.push_op(node_id, Op::Structural(StructuralOp::Scope));
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
    // The Custom variant and its associated `dyn Desugar` serialization/cloning
    // will be handled with a more robust solution later.
}

impl Node {
    pub fn desugar(&self, cx: &mut LoweringContext) -> NodeId {
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