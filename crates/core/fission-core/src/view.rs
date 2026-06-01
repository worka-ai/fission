//! Read-only view, widget trait, and selector pattern.
//!
//! During [`Widget::build`], the framework provides a [`View`] that gives
//! read-only access to the current [`AppState`], theme, i18n registry,
//! layout snapshot, and animation values. Widgets use this to decide what
//! to render without any side-effects.

use crate::{
    env::VideoState,
    registry::{AnimationPropertyId, VideoRegistration},
    ui::{
        ActionScope, Align, Checkbox, Clip, Composite, FocusScope, GestureDetector, Grid, GridItem,
        Icon, Image, LazyColumn, Node, Overlay, Radio, RichText, SafeArea, Scroll, SemanticsRegion,
        Slider, Spacer, Switch, Text, TextInput, Transform, Video,
    },
    AppState, BuildCtx, Env, LayoutRect, LayoutSize, LayoutSnapshot, RuntimeState,
};
use fission_i18n::I18nRegistry;
use fission_ir::{NodeId, WidgetNodeId};
use fission_layout::BoxConstraints;
use fission_theme::Theme;
use std::{any::Any, sync::Arc};

/// Read-only access to application state and environment during widget building.
///
/// `View` is the primary way widgets read data. It is parameterised over the
/// concrete [`AppState`] type `S`, giving type-safe access to `state` while
/// also exposing the theme, i18n registry, layout snapshot from the previous
/// frame, and animation values.
///
/// # Example
///
/// ```rust,ignore
/// fn build(
///     &self,
///     _ctx: &mut BuildCtx<MyState>,
///     view: &View<MyState>,
/// ) -> impl IntoWidget<MyState> {
///     let name = &view.state.user_name;
///     let theme = view.theme();
///     Text::new(format!("Hello, {}!", name))
///         .color(theme.tokens.colors.primary)
/// }
/// ```
pub struct View<'a, S: AppState> {
    /// Reference to the current application state.
    pub state: &'a S,
    /// Runtime interaction, scroll, text-edit, and animation state.
    pub runtime: &'a RuntimeState,
    /// Environment (theme, i18n, viewport size, locale).
    pub env: &'a Env,
    /// Layout snapshot from the previous frame, if available.
    pub layout: Option<&'a LayoutSnapshot>,
}

impl<'a, S: AppState> View<'a, S> {
    pub fn new(
        state: &'a S,
        runtime: &'a RuntimeState,
        env: &'a Env,
        layout: Option<&'a LayoutSnapshot>,
    ) -> Self {
        Self {
            state,
            runtime,
            env,
            layout,
        }
    }

    pub fn theme(&self) -> &Theme {
        &self.env.theme
    }
    pub fn i18n(&self) -> &I18nRegistry {
        &self.env.i18n
    }

    pub fn get_rect(&self, id: WidgetNodeId) -> Option<LayoutRect> {
        let node_id: NodeId = id.into();
        self.layout.and_then(|l| l.get_node_rect(node_id))
    }

    pub fn get_constraints(&self, id: WidgetNodeId) -> Option<BoxConstraints> {
        let node_id: NodeId = id.into();
        self.layout.and_then(|l| l.get_node_constraints(node_id))
    }

    pub fn viewport_size(&self) -> LayoutSize {
        self.env.viewport_size
    }

    pub fn select<T: Selector<S>>(&self) -> T::Output {
        T::select(self)
    }

    pub fn animation_value(&self, widget_id: WidgetNodeId, property: &AnimationPropertyId) -> f32 {
        self.runtime
            .animation
            .values
            .get(&(widget_id, property.clone()))
            .copied()
            .unwrap_or_else(|| property.default_value())
    }

    pub fn video_state(&self, widget_id: WidgetNodeId) -> Option<&VideoState> {
        self.runtime.video.states.get(&widget_id)
    }
}

/// A selector that derives a value from the [`View`].
///
/// Selectors extract and transform data from state so widgets can depend on
/// derived values without coupling to the full state shape.
///
/// # Example
///
/// ```rust,ignore
/// struct ItemCount;
/// impl Selector<MyState> for ItemCount {
///     type Output = usize;
///     fn select(view: &View<MyState>) -> usize {
///         view.state.items.len()
///     }
/// }
///
/// // In a widget:
/// let count: usize = view.select::<ItemCount>();
/// ```
pub trait Selector<S: AppState> {
    /// The type produced by the selector.
    type Output;
    /// Extract the value from the given view.
    fn select(view: &View<S>) -> Self::Output;
}

/// Type-erased storage for a Fission widget.
///
/// Application authors should rarely need to name this type directly. It exists
/// because widget composition needs to store heterogeneous child widgets in
/// collections such as columns, rows, routes, overlays, and slots while still
/// allowing each widget's `build` method to return a concrete Rust type.
///
/// A tempting alternative is `Vec<Box<dyn Widget<S>>>`, but that would force
/// [`Widget`] itself to be object-safe. Fission deliberately keeps [`Widget`]
/// as a normal Rust authoring trait whose only surface is [`Widget::build`].
/// Internal erasure and lowering live here instead of on the public trait.
pub struct AnyWidget<S: AppState> {
    inner: AnyWidgetInner<S>,
}

enum AnyWidgetInner<S: AppState> {
    Widget(Arc<dyn ErasedWidget<S>>),
}

impl<S: AppState> Clone for AnyWidget<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<S: AppState> Clone for AnyWidgetInner<S> {
    fn clone(&self) -> Self {
        match self {
            Self::Widget(widget) => Self::Widget(Arc::clone(widget)),
        }
    }
}

trait ErasedWidget<S: AppState>: Send + Sync {
    fn lower_to_node(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> Node;
}

impl<S, W> ErasedWidget<S> for W
where
    S: AppState,
    W: Widget<S> + Send + Sync + Any + 'static,
{
    fn lower_to_node(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> Node {
        if let Some(node) = (self as &dyn Any).downcast_ref::<InternalNodeWidget>() {
            return node.node.clone();
        }
        self.build(ctx, view).into_widget().lower_to_node(ctx, view)
    }
}

impl<S: AppState> AnyWidget<S> {
    /// Create an erased widget from a normal Fission widget.
    pub fn new<W>(widget: W) -> Self
    where
        W: Widget<S> + Send + Sync + 'static,
    {
        Self {
            inner: AnyWidgetInner::Widget(Arc::new(widget)),
        }
    }

    /// Lower this erased widget to Fission's internal node representation.
    #[doc(hidden)]
    pub fn lower_to_node(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> Node {
        match &self.inner {
            AnyWidgetInner::Widget(widget) => widget.lower_to_node(ctx, view),
        }
    }
}

/// Converts a value into Fission's erased widget storage.
///
/// This is Fission-specific rather than `Into<Widget>` for two reasons:
///
/// - `Widget` is a trait, not a concrete storage type, so `Into<Widget>` is not
///   a valid Rust target. `Into<Box<dyn Widget<S>>>` would require an
///   object-safe [`Widget`] trait, which conflicts with the ergonomic
///   `build -> impl IntoWidget<S>` model.
/// - Fission owns this trait, so the framework can preserve clear diagnostics
///   and attach future metadata such as stable identity, source labels, hot
///   reload hints, and devtools information without exposing the internal node
///   representation as the authoring API.
///
/// Normal application code does not need to call this trait directly. Use
/// `impl Widget<S>` structs and pass concrete widget values to child slots.
pub trait IntoWidget<S: AppState>: Send + Sync + 'static {
    /// Convert this value into Fission's erased widget storage.
    fn into_widget(self) -> AnyWidget<S>;
}

impl<S, W> IntoWidget<S> for W
where
    S: AppState,
    W: Widget<S> + Send + Sync + 'static,
{
    fn into_widget(self) -> AnyWidget<S> {
        AnyWidget::new(self)
    }
}

/// A hidden framework-owned widget that terminates recursive build lowering at
/// the internal node representation.
#[doc(hidden)]
#[derive(Clone)]
pub struct InternalNodeWidget {
    node: Node,
}

/// Convert a concrete framework-owned node into the internal widget pipeline.
///
/// This is hidden because application code should return widgets, not nodes.
#[doc(hidden)]
pub fn internal_node_widget(node: Node) -> InternalNodeWidget {
    InternalNodeWidget { node }
}

impl<S: AppState> Widget<S> for InternalNodeWidget {
    fn build(&self, _ctx: &mut BuildCtx<S>, _view: &View<S>) -> impl IntoWidget<S> {
        self.clone()
    }
}

/// Lower a widget to Fission's internal node representation.
///
/// Runtime, testing, and shell code use this as the boundary between the public
/// widget authoring model and the internal node/IR pipeline. It is deliberately
/// a free function rather than a method on [`Widget`], so the Widget API surface
/// remains strictly `build`.
#[doc(hidden)]
pub fn lower_widget_to_node<S, W>(widget: &W, ctx: &mut BuildCtx<S>, view: &View<S>) -> Node
where
    S: AppState,
    W: Widget<S> + ?Sized,
{
    widget
        .build(ctx, view)
        .into_widget()
        .lower_to_node(ctx, view)
}

/// The core trait for composable UI components.
///
/// A `Widget` produces another widget-like value given read-only access to
/// state ([`View`]) and a mutable build context ([`BuildCtx`]) for binding
/// actions, registering portals, and requesting animations. The returned value
/// is converted into Fission's private node tree by [`IntoWidget`].
///
/// # Example
///
/// ```rust,ignore
/// struct Greeting;
///
/// impl Widget<AppState> for Greeting {
///     fn build(
///         &self,
///         ctx: &mut BuildCtx<AppState>,
///         view: &View<AppState>,
///     ) -> impl IntoWidget<AppState> {
///         Text::new(format!("Hello, {}", view.state.name))
///     }
/// }
/// ```
pub trait Widget<S: AppState> {
    /// Build this widget.
    ///
    /// Called once per frame. Implementations must be pure -- all side-effects
    /// go through `ctx` (action binding, portals, animations).
    fn build(&self, ctx: &mut BuildCtx<S>, view: &View<S>) -> impl IntoWidget<S>;
}

macro_rules! impl_widget_for_primitive {
    ($t:ty, $v:ident) => {
        impl<S: AppState> Widget<S> for $t {
            fn build(&self, _ctx: &mut BuildCtx<S>, _view: &View<S>) -> impl crate::IntoWidget<S> {
                crate::view::internal_node_widget(Node::$v(self.clone()))
            }
        }
    };
}

impl_widget_for_primitive!(ActionScope, ActionScope);
impl_widget_for_primitive!(Align, Align);
impl_widget_for_primitive!(FocusScope, FocusScope);
impl_widget_for_primitive!(Clip, Clip);
impl_widget_for_primitive!(Text, Text);
impl_widget_for_primitive!(RichText, RichText);
impl_widget_for_primitive!(Transform, Transform);
impl_widget_for_primitive!(TextInput, TextInput);
impl_widget_for_primitive!(Scroll, Scroll);
impl_widget_for_primitive!(SemanticsRegion, SemanticsRegion);
impl_widget_for_primitive!(Image, Image);
impl_widget_for_primitive!(Overlay, Overlay);
impl_widget_for_primitive!(GestureDetector, GestureDetector);
impl_widget_for_primitive!(Grid, Grid);
impl_widget_for_primitive!(GridItem, GridItem);
impl_widget_for_primitive!(Checkbox, Checkbox);
impl_widget_for_primitive!(Switch, Switch);
impl_widget_for_primitive!(Radio, Radio);
impl_widget_for_primitive!(SafeArea, SafeArea);
impl_widget_for_primitive!(Composite, Composite);
impl_widget_for_primitive!(Spacer, Spacer);
impl_widget_for_primitive!(Slider, Slider);
impl_widget_for_primitive!(LazyColumn, LazyColumn);
impl_widget_for_primitive!(Icon, Icon);

impl<S: AppState> Widget<S> for Video {
    fn build(&self, ctx: &mut BuildCtx<S>, _view: &View<S>) -> impl crate::IntoWidget<S> {
        let mut video = self.clone();
        let id = video
            .id
            .unwrap_or_else(|| WidgetNodeId::explicit(&video.source));
        video.id = Some(id);

        ctx.register_video(VideoRegistration {
            node_id: id,
            source: video.source.clone(),
            autoplay: video.autoplay,
            loop_playback: video.loop_playback,
        });

        crate::view::internal_node_widget(Node::Video(video))
    }
}
