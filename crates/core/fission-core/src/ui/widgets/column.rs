use crate::{
    AnyWidget, AppState, BuildCtx, IntoWidget, Lower, LoweringContext, Node, NodeBuilder, View,
    Widget,
};
use fission_ir::op::{AlignItems, FlexWrap, JustifyContent};
use fission_ir::{FlexDirection, LayoutOp, NodeId, Op, Semantics};
use serde::{Deserialize, Serialize};

/// A vertical flex container that lays out child widgets top-to-bottom.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column<Child = Node> {
    pub id: Option<NodeId>,
    pub children: Vec<Child>,
    pub semantics: Option<Semantics>,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub gap: Option<f32>,
    pub wrap: FlexWrap,
    pub align_items: AlignItems,
    pub justify_content: JustifyContent,
}

impl<Child> Default for Column<Child> {
    fn default() -> Self {
        Self {
            id: None,
            children: Vec::new(),
            gap: None,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            semantics: None,
            wrap: FlexWrap::NoWrap,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Start,
        }
    }
}

impl<S: AppState> Column<AnyWidget<S>> {
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

impl<Child> Column<Child> {
    pub fn flex_grow(mut self, flex_grow: f32) -> Self {
        self.flex_grow = flex_grow;
        self
    }

    pub fn gap(mut self, gap: Option<f32>) -> Self {
        self.gap = gap;
        self
    }

    pub fn align_items(mut self, align: AlignItems) -> Self {
        self.align_items = align;
        self
    }

    pub fn justify_content(mut self, justify: JustifyContent) -> Self {
        self.justify_content = justify;
        self
    }
}

impl Column<Node> {
    #[doc(hidden)]
    pub fn children(mut self, children: Vec<Node>) -> Self {
        self.children = children;
        self
    }

    #[doc(hidden)]
    pub fn into_node(self) -> Node {
        Node::Column(self)
    }
}

impl<S: AppState> Widget<S> for Column<AnyWidget<S>> {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> impl IntoWidget<S> {
        crate::view::internal_node_widget(Node::Column(Column {
            id: self.id,
            children: self
                .children
                .iter()
                .map(|child| child.lower_to_node(ctx, view))
                .collect(),
            gap: self.gap,
            flex_grow: self.flex_grow,
            flex_shrink: self.flex_shrink,
            semantics: self.semantics.clone(),
            wrap: self.wrap,
            align_items: self.align_items,
            justify_content: self.justify_content,
        }))
    }
}

impl Lower for Column<Node> {
    fn lower(&self, cx: &mut LoweringContext) -> NodeId {
        let layout_id = self.id.unwrap_or_else(|| cx.next_node_id());

        cx.push_scope(layout_id);

        let mut builder = NodeBuilder::new(
            layout_id,
            Op::Layout(LayoutOp::Flex {
                direction: FlexDirection::Column,
                wrap: self.wrap,
                flex_grow: self.flex_grow,
                flex_shrink: self.flex_shrink,
                padding: [0.0; 4],
                gap: self.gap,
                align_items: self.align_items,
                justify_content: self.justify_content,
            }),
        );
        for child in &self.children {
            builder.add_child(child.lower(cx));
        }

        cx.pop_scope();

        let layout_id = builder.build(cx);

        if let Some(semantics) = &self.semantics {
            let wrapper_id = cx.next_node_id();
            let mut semantics_builder =
                NodeBuilder::new(wrapper_id, Op::Semantics(semantics.clone()));
            semantics_builder.add_child(layout_id);
            semantics_builder.build(cx)
        } else {
            layout_id
        }
    }
}
