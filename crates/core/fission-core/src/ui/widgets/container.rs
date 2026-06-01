use crate::lowering::{LoweringContext, NodeBuilder};
use crate::ui::traits::Lower;
use crate::ui::Node;
use crate::{AnyWidget, AppState, BuildCtx, IntoWidget, View, Widget};
use fission_ir::{
    op::{BoxShadow, Color, Fill, LayoutOp, Op, PaintOp, Stroke},
    NodeId,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container<Child = Node> {
    pub id: Option<NodeId>,
    pub child: Option<Box<Child>>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub padding: [f32; 4],
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub background_fill: Option<Fill>,
    pub background_color: Option<Color>,
    pub border_color: Option<Color>,
    pub border_width: f32,
    pub border_radius: f32,
    pub shadow: Option<BoxShadow>,
    pub shadows: Vec<BoxShadow>,
}

impl<Child> Default for Container<Child> {
    fn default() -> Self {
        Self {
            id: None,
            child: None,
            width: None,
            height: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            padding: [0.0; 4],
            flex_grow: 0.0,
            flex_shrink: 1.0,
            background_fill: None,
            background_color: None,
            border_color: None,
            border_width: 0.0,
            border_radius: 0.0,
            shadow: None,
            shadows: Vec::new(),
        }
    }
}

impl<S: AppState> Container<AnyWidget<S>> {
    pub fn new(child: impl IntoWidget<S>) -> Self {
        Self {
            child: Some(Box::new(child.into_widget())),
            ..Default::default()
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn child(mut self, child: impl IntoWidget<S>) -> Self {
        self.child = Some(Box::new(child.into_widget()));
        self
    }
}

impl<Child> Container<Child> {
    pub fn id(mut self, id: NodeId) -> Self {
        self.id = Some(id);
        self
    }
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = Some(w);
        self.height = Some(h);
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self
    }
    pub fn min_width(mut self, w: f32) -> Self {
        self.min_width = Some(w);
        self
    }
    pub fn max_width(mut self, w: f32) -> Self {
        self.max_width = Some(w);
        self
    }
    pub fn min_height(mut self, h: f32) -> Self {
        self.min_height = Some(h);
        self
    }
    pub fn max_height(mut self, h: f32) -> Self {
        self.max_height = Some(h);
        self
    }
    pub fn padding_all(mut self, p: f32) -> Self {
        self.padding = [p; 4];
        self
    }
    pub fn padding(mut self, padding: [f32; 4]) -> Self {
        self.padding = padding;
        self
    }
    pub fn flex_grow(mut self, grow: f32) -> Self {
        self.flex_grow = grow;
        self
    }
    pub fn flex_shrink(mut self, shrink: f32) -> Self {
        self.flex_shrink = shrink;
        self
    }
    pub fn bg(mut self, color: Color) -> Self {
        self.background_fill = Some(Fill::Solid(color));
        self.background_color = Some(color);
        self
    }
    pub fn bg_fill(mut self, fill: Fill) -> Self {
        self.background_fill = Some(fill);
        self.background_color = None;
        self
    }
    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = width;
        self
    }
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.border_radius = radius;
        self
    }
    pub fn shadow(mut self, shadow: BoxShadow) -> Self {
        self.shadow = Some(shadow);
        self
    }
    pub fn shadows(mut self, shadows: Vec<BoxShadow>) -> Self {
        self.shadows = shadows;
        self
    }
}

impl Container<Node> {
    pub fn lowered(child: Node) -> Self {
        Self {
            child: Some(Box::new(child)),
            ..Default::default()
        }
    }

    #[doc(hidden)]
    pub fn into_node(self) -> Node {
        Node::Container(self)
    }
}

impl<S: AppState> Widget<S> for Container<AnyWidget<S>> {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> impl IntoWidget<S> {
        crate::view::internal_node_widget(Node::Container(Container {
            id: self.id,
            child: self
                .child
                .as_ref()
                .map(|child| child.lower_to_node(ctx, view))
                .map(Box::new),
            width: self.width,
            height: self.height,
            min_width: self.min_width,
            max_width: self.max_width,
            min_height: self.min_height,
            max_height: self.max_height,
            padding: self.padding,
            flex_grow: self.flex_grow,
            flex_shrink: self.flex_shrink,
            background_fill: self.background_fill.clone(),
            background_color: self.background_color,
            border_color: self.border_color,
            border_width: self.border_width,
            border_radius: self.border_radius,
            shadow: self.shadow,
            shadows: self.shadows.clone(),
        }))
    }
}

impl Lower for Container<Node> {
    fn lower(&self, cx: &mut LoweringContext) -> NodeId {
        let id = self.id.unwrap_or_else(|| cx.next_node_id());
        cx.push_scope(id);

        let mut children_ids = Vec::new();

        if self.background_fill.is_some()
            || self.background_color.is_some()
            || self.border_color.is_some()
            || self.shadow.is_some()
            || !self.shadows.is_empty()
        {
            for shadow in &self.shadows {
                let paint = NodeBuilder::new(
                    cx.next_node_id(),
                    Op::Paint(PaintOp::DrawRect {
                        fill: None,
                        stroke: None,
                        corner_radius: self.border_radius,
                        shadow: Some(*shadow),
                    }),
                )
                .build(cx);
                children_ids.push(paint);
            }
            let paint = NodeBuilder::new(
                cx.next_node_id(),
                Op::Paint(PaintOp::DrawRect {
                    fill: self
                        .background_fill
                        .clone()
                        .or_else(|| self.background_color.map(Fill::Solid)),
                    stroke: self.border_color.map(|c| Stroke {
                        fill: Fill::Solid(c),
                        width: self.border_width,
                        dash_array: None,
                        line_cap: fission_ir::op::LineCap::Butt,
                        line_join: fission_ir::op::LineJoin::Miter,
                    }),
                    corner_radius: self.border_radius,
                    shadow: self.shadow,
                }),
            )
            .build(cx);
            children_ids.push(paint);
        }

        if let Some(child) = &self.child {
            children_ids.push(child.lower(cx));
        }

        cx.pop_scope();

        let mut layout = NodeBuilder::new(
            id,
            Op::Layout(LayoutOp::Box {
                width: self.width,
                height: self.height,
                min_width: self.min_width,
                max_width: self.max_width,
                min_height: self.min_height,
                max_height: self.max_height,
                padding: self.padding,
                flex_grow: self.flex_grow,
                flex_shrink: self.flex_shrink,
                aspect_ratio: None,
            }),
        );

        for cid in children_ids {
            layout.add_child(cid);
        }

        layout.build(cx)
    }
}
