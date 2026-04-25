use crate::lowering::{LoweringContext, NodeBuilder};
use crate::ui::traits::Lower;
use crate::ui::Node;
use fission_ir::{LayoutOp, NodeId, Op};
use serde::{Deserialize, Serialize};

/// Centers its child within the available parent space.
///
/// `Align` is a convenience wrapper that applies center alignment on both
/// axes. It expands to fill the parent and places the child at the centre.
///
/// # Example
///
/// ```rust,ignore
/// Align::new(Text::new("Centered!").into_node())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Align {
    /// Explicit node identity.
    pub id: Option<NodeId>,
    /// The child widget to center.
    pub child: Box<Node>,
}

impl Align {
    pub fn new(child: Node) -> Self {
        Self {
            child: Box::new(child),
            id: None,
        }
    }

    pub fn into_node(self) -> Node {
        Node::Align(self)
    }
}

impl Lower for Align {
    fn lower(&self, cx: &mut LoweringContext) -> NodeId {
        let id = self.id.unwrap_or_else(|| cx.next_node_id());
        cx.push_scope(id);
        let child_id = self.child.lower(cx);
        cx.pop_scope();

        let mut builder = NodeBuilder::new(id, Op::Layout(LayoutOp::Align));
        builder.add_child(child_id);
        builder.build(cx)
    }
}
