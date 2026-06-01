use crate::ui::Container;
use crate::{
    AnyWidget, AppState, BoxConstraints, BuildCtx, IntoWidget, Node, NodeId, View, Widget,
    WidgetNodeId,
};
use std::sync::Arc;

/// A closure-based widget for small inline composition points.
pub struct Builder<S: AppState> {
    builder: Arc<dyn Fn(&mut BuildCtx<S>, &View<S>) -> AnyWidget<S> + Send + Sync>,
}

impl<S: AppState> Clone for Builder<S> {
    fn clone(&self) -> Self {
        Self {
            builder: Arc::clone(&self.builder),
        }
    }
}

impl<S: AppState> Builder<S> {
    pub fn new<F, W>(builder: F) -> Self
    where
        F: Fn(&mut BuildCtx<S>, &View<S>) -> W + Send + Sync + 'static,
        W: IntoWidget<S>,
    {
        Self {
            builder: Arc::new(move |ctx, view| builder(ctx, view).into_widget()),
        }
    }
}

impl<S: AppState> Widget<S> for Builder<S> {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> impl IntoWidget<S> {
        crate::view::internal_node_widget((self.builder)(ctx, view).lower_to_node(ctx, view))
    }
}

/// A closure-based widget that receives its parent's [`BoxConstraints`].
pub struct LayoutBuilder<S: AppState> {
    pub id: Option<WidgetNodeId>,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    builder: Arc<dyn Fn(&mut BuildCtx<S>, &View<S>, BoxConstraints) -> AnyWidget<S> + Send + Sync>,
}

impl<S: AppState> Clone for LayoutBuilder<S> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            flex_grow: self.flex_grow,
            flex_shrink: self.flex_shrink,
            builder: Arc::clone(&self.builder),
        }
    }
}

impl<S: AppState> LayoutBuilder<S> {
    pub fn new<F, W>(builder: F) -> Self
    where
        F: Fn(&mut BuildCtx<S>, &View<S>, BoxConstraints) -> W + Send + Sync + 'static,
        W: IntoWidget<S>,
    {
        Self {
            id: None,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            builder: Arc::new(move |ctx, view, constraints| {
                builder(ctx, view, constraints).into_widget()
            }),
        }
    }

    pub fn id(mut self, id: WidgetNodeId) -> Self {
        self.id = Some(id);
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
}

impl<S: AppState> Widget<S> for LayoutBuilder<S> {
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> impl IntoWidget<S> {
        let viewport = view
            .layout
            .map(|layout| layout.viewport_size)
            .unwrap_or_else(|| view.viewport_size());
        let mut max_w = viewport.width;
        let mut max_h = viewport.height;
        if !max_w.is_finite() || max_w <= 0.0 {
            max_w = f32::INFINITY;
        }
        if !max_h.is_finite() || max_h <= 0.0 {
            max_h = f32::INFINITY;
        }
        let fallback = BoxConstraints::loose(max_w, max_h);
        let constraints = self
            .id
            .and_then(|id| view.get_constraints(id))
            .unwrap_or(fallback);
        let child = (self.builder)(ctx, view, constraints).lower_to_node(ctx, view);
        crate::view::internal_node_widget(if let Some(id) = self.id {
            Container::<Node>::lowered(child)
                .id(NodeId::from(id))
                .flex_grow(self.flex_grow)
                .flex_shrink(self.flex_shrink)
                .into_node()
        } else {
            child
        })
    }
}
