use crate::ui::{Node, traits::Lower};
use crate::lowering::LoweringContext;
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
        let child_ids: Vec<NodeId> = self.children.iter().map(|c| c.lower(cx)).collect();
        let id = self.id.unwrap_or_else(|| cx.next_node_id());
        
        cx.add_node(
            id,
            Op::Layout(LayoutOp::Flex {
                direction: FlexDirection::Column,
                flex_grow: self.flex_grow,
                flex_shrink: self.flex_shrink,
                padding: self.padding,
            }),
            child_ids.clone(),
        );
        
        for child_id in child_ids {
            if let Some(node) = cx.ir.nodes.get_mut(&child_id) {
                node.parent = Some(id);
            }
        }
        id
    }
}
