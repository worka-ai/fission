use crate::lowering::{LoweringContext, NodeBuilder};
use crate::ui::traits::Lower;
use crate::ui::Node;
use crate::{AnyWidget, AppState, BuildCtx, IntoWidget, View, Widget};
use fission_ir::{
    op::{LayoutOp, Op},
    NodeId,
};
use serde::{Deserialize, Serialize};

/// Absolutely positions a child within a [`ZStack`](super::ZStack).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Positioned<Child = Node> {
    pub id: Option<NodeId>,
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub child: Option<Box<Child>>,
}

impl<Child> Default for Positioned<Child> {
    fn default() -> Self {
        Self {
            id: None,
            left: None,
            top: None,
            right: None,
            bottom: None,
            width: None,
            height: None,
            child: None,
        }
    }
}

impl<S: AppState> Positioned<AnyWidget<S>> {
    pub fn new(child: impl IntoWidget<S>) -> Self {
        Self {
            child: Some(Box::new(child.into_widget())),
            ..Default::default()
        }
    }

    pub fn child(mut self, child: impl IntoWidget<S>) -> Self {
        self.child = Some(Box::new(child.into_widget()));
        self
    }
}

impl<Child> Positioned<Child> {
    pub fn left(mut self, value: f32) -> Self {
        self.left = Some(value);
        self
    }

    pub fn top(mut self, value: f32) -> Self {
        self.top = Some(value);
        self
    }

    pub fn right(mut self, value: f32) -> Self {
        self.right = Some(value);
        self
    }

    pub fn bottom(mut self, value: f32) -> Self {
        self.bottom = Some(value);
        self
    }

    pub fn width(mut self, value: f32) -> Self {
        self.width = Some(value);
        self
    }

    pub fn height(mut self, value: f32) -> Self {
        self.height = Some(value);
        self
    }
}

impl Positioned<Node> {
    #[doc(hidden)]
    pub fn into_node(self) -> Node {
        Node::Positioned(self)
    }
}

impl<S: AppState> Widget<S> for Positioned<AnyWidget<S>> {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> impl IntoWidget<S> {
        crate::view::internal_node_widget(Node::Positioned(Positioned {
            id: self.id,
            left: self.left,
            top: self.top,
            right: self.right,
            bottom: self.bottom,
            width: self.width,
            height: self.height,
            child: self
                .child
                .as_ref()
                .map(|child| child.lower_to_node(ctx, view))
                .map(Box::new),
        }))
    }
}

impl Lower for Positioned<Node> {
    fn lower(&self, cx: &mut LoweringContext) -> NodeId {
        let id = self.id.unwrap_or_else(|| cx.next_node_id());
        cx.push_scope(id);

        let child_id = self.child.as_ref().map(|child| child.lower(cx));

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
