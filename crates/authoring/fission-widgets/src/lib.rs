use fission_ir::{NodeId, Op, LayoutOp, StructuralOp, FlexDirection, Semantics, ActionEntry, ActionSet, PaintOp, op::Color as IrColor};
use fission_core::{Action as CoreAction, ActionId, ActionEnvelope};
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
    pub font_size: Option<f32>,
    pub color: Option<IrColor>,
}

impl Desugar for Text {
    fn desugar(&self, cx: &mut LoweringContext) -> NodeId {
        let layout_node_id = self.id.unwrap_or_else(|| cx.next_node_id());
        
        // 1. Create a LayoutOp::Box node for the text's bounding box.
        // This node defines the size/position of the text.
        cx.add_node(layout_node_id, Op::Layout(LayoutOp::Box { width: self.width, height: self.height }), vec![]);

        // 2. Create a PaintOp::DrawText node as a child of the layout node.
        // This node carries the actual text rendering information.
        let paint_node_id = cx.next_node_id();
        cx.add_node(paint_node_id, Op::Paint(PaintOp::DrawText { 
            text: self.value.clone(), 
            size: self.font_size.unwrap_or(14.0), // Default font size
            color: self.color.unwrap_or(IrColor::BLACK),
        }), vec![]);

        // Attach the paint node to the layout node as a child
        if let Some(layout_node) = cx.ir.nodes.get_mut(&layout_node_id) {
            layout_node.children.push(paint_node_id);
            // Update parent of paint_node
            if let Some(paint_node) = cx.ir.nodes.get_mut(&paint_node_id) {
                paint_node.parent = Some(layout_node_id);
            }
        }

        // If semantics are present, they wrap the layout node.
        if let Some(s) = &self.semantics {
            let semantics_id = cx.next_node_id();
            cx.add_node(semantics_id, Op::Semantics(s.clone()), vec![layout_node_id]);
            return semantics_id;
        }
        
        layout_node_id
    }
}

// Removed impl Default for FlexDirection, as it's now in fission-ir/src/op.rs

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)] // Default derive is fine now
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
        let mut child_ids = Vec::new();
        for child in &self.children {
            child_ids.push(child.desugar(cx));
        }

        let layout_id = if self.semantics.is_some() { cx.next_node_id() } else { self.id.unwrap_or_else(|| cx.next_node_id()) };
        
        cx.add_node(layout_id, Op::Layout(LayoutOp::Flex { 
            direction: self.direction,
            flex_grow: self.flex_grow,
            flex_shrink: self.flex_shrink,
        }), child_ids);

        if let Some(s) = &self.semantics {
            let semantics_id = cx.next_node_id();
            cx.add_node(semantics_id, Op::Semantics(s.clone()), vec![layout_id]);
            return semantics_id;
        }

        layout_id
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Button {
    pub id: Option<WidgetNodeId>,
    pub child: Option<Box<Node>>,
    pub on_press: Option<ActionEnvelope>, // Using ActionEnvelope
    pub semantics: Option<Semantics>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Desugar for Button {
    fn desugar(&self, cx: &mut LoweringContext) -> NodeId {
        let mut child_ids = Vec::new();
        if let Some(child) = &self.child {
            child_ids.push(child.desugar(cx));
        }
        
        let layout_id = if self.semantics.is_some() { cx.next_node_id() } else { self.id.unwrap_or_else(|| cx.next_node_id()) };

        cx.add_node(layout_id, Op::Layout(LayoutOp::Box { width: self.width, height: self.height }), child_ids);
        
        if let Some(s_widget) = &self.semantics {
            let semantics_id = cx.next_node_id();
            let mut s_op = s_widget.clone();

            if let Some(action_envelope) = &self.on_press {
                let action_entry = ActionEntry {
                    action_id: action_envelope.id.as_u128(),
                    payload_data: Some(action_envelope.payload.clone()),
                };
                s_op.actions.entries.push(action_entry);
            }

            cx.add_node(semantics_id, Op::Semantics(s_op), vec![layout_id]);
            return semantics_id;
        }
        
        layout_id
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
