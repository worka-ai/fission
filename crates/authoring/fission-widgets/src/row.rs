use crate::{node::Node, Desugar, LoweringContext, WidgetNodeId};
use fission_ir::{FlexDirection, LayoutOp, NodeId, Op, Semantics};
use serde::{Deserialize, Serialize};

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
        let mut child_ids = Vec::new();
        for child in &self.children {
            child_ids.push(child.desugar(cx));
        }

        let layout_id = if self.semantics.is_some() {
            cx.next_node_id()
        } else {
            self.id.unwrap_or_else(|| cx.next_node_id())
        };

        cx.add_node(
            layout_id,
            Op::Layout(LayoutOp::Flex {
                direction: self.direction,
                flex_grow: self.flex_grow,
                flex_shrink: self.flex_shrink,
            }),
            child_ids,
        );

        if let Some(s) = &self.semantics {
            let semantics_id = cx.next_node_id();
            cx.add_node(semantics_id, Op::Semantics(s.clone()), vec![layout_id]);
            return semantics_id;
        }

        layout_id
    }
}
