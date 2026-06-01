use crate::lowering::{wrap_zstack_child, LoweringContext, NodeBuilder};
use crate::ui::traits::Lower;
use crate::ui::Node;
use crate::{AnyWidget, AppState, BuildCtx, IntoWidget, View, Widget};
use fission_ir::{LayoutOp, NodeId, Op};
use serde::{Deserialize, Serialize};

/// A z-axis stacking container that layers children on top of each other.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZStack<Child = Node> {
    pub id: Option<NodeId>,
    pub children: Vec<Child>,
}

impl<Child> Default for ZStack<Child> {
    fn default() -> Self {
        Self {
            id: None,
            children: Vec::new(),
        }
    }
}

impl<S: AppState> ZStack<AnyWidget<S>> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn child(mut self, child: impl IntoWidget<S>) -> Self {
        self.children.push(child.into_widget());
        self
    }

    pub fn children<I, W>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = W>,
        W: IntoWidget<S>,
    {
        self.children = children.into_iter().map(IntoWidget::into_widget).collect();
        self
    }
}

impl ZStack<Node> {
    #[doc(hidden)]
    pub fn children(mut self, children: Vec<Node>) -> Self {
        self.children = children;
        self
    }

    #[doc(hidden)]
    pub fn into_node(self) -> Node {
        Node::ZStack(self)
    }
}

impl<S: AppState> Widget<S> for ZStack<AnyWidget<S>> {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> impl IntoWidget<S> {
        crate::view::internal_node_widget(Node::ZStack(ZStack {
            id: self.id,
            children: self
                .children
                .iter()
                .map(|child| child.lower_to_node(ctx, view))
                .collect(),
        }))
    }
}

impl Lower for ZStack<Node> {
    fn lower(&self, cx: &mut LoweringContext) -> NodeId {
        let id = self.id.unwrap_or_else(|| cx.next_node_id());

        cx.push_scope(id);

        let mut builder = NodeBuilder::new(id, Op::Layout(LayoutOp::ZStack));
        for child in &self.children {
            let child_id = child.lower(cx);
            builder.add_child(wrap_zstack_child(cx, child_id));
        }

        cx.pop_scope();

        builder.build(cx)
    }
}
