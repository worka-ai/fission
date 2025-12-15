use crate::ui::{Node, traits::Lower};
use crate::lowering::{LoweringContext, NodeBuilder};
use fission_ir::{NodeId, Op, LayoutOp, FlexDirection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Column {
    pub id: Option<NodeId>,
    pub children: Vec<Node>,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub padding: [f32; 4],
}

impl Lower for Column {
    fn lower(&self, cx: &mut LoweringContext) -> NodeId {
        let id = self.id.unwrap_or_else(|| cx.next_node_id());
        let mut builder = NodeBuilder::new(
            id,
            Op::Layout(LayoutOp::Flex {
                direction: FlexDirection::Column,
                flex_grow: self.flex_grow,
                flex_shrink: self.flex_shrink,
                padding: self.padding,
            }),
        );
        for child in &self.children {
            builder.add_child(child.lower(cx));
        }
        builder.build(cx)
    }
}
