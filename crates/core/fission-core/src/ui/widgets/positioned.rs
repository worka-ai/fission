use crate::lowering::{LoweringContext, NodeBuilder};
use crate::ui::traits::Lower;
use crate::ui::Node;
use fission_ir::{op::{LayoutOp, Op}, NodeId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Positioned {
    pub id: Option<NodeId>,
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub child: Option<Box<Node>>,
}

impl Positioned {
    pub fn into_node(self) -> Node {
        Node::Positioned(self)
    }
}

impl Lower for Positioned {
    fn lower(&self, cx: &mut LoweringContext) -> NodeId {
        let id = self.id.unwrap_or_else(|| cx.next_node_id());
        cx.push_scope(id);

        let child_id = if let Some(child) = &self.child {
            Some(child.lower(cx))
        } else {
            None
        };

        let mut builder = NodeBuilder::new(
            id,
            Op::Layout(LayoutOp::Positioned {
                left: self.left,
                top: self.top,
                right: self.right,
                bottom: self.bottom,
                width: self.width,
                height: self.height,
            }),
        );
        
        if let Some(cid) = child_id {
            builder.add_child(cid);
        }
        
        cx.pop_scope();
        builder.build(cx)
    }
}


