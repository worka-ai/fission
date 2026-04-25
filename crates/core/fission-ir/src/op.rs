//! Operations that IR nodes can perform.
//!
//! Every [`CoreNode`](crate::CoreNode) carries exactly one [`Op`]. The four
//! categories cover everything a node can *do*:
//!
//! | Category | Type | Purpose |
//! |----------|------|---------|
//! | Structure | [`StructuralOp`] | Grouping nodes without visual effect. |
//! | Layout | [`LayoutOp`] | Sizing and positioning children (Box, Flex, Grid, ...). |
//! | Paint | [`PaintOp`] | Drawing rectangles, text, images, paths, and SVGs. |
//! | Semantics | [`Semantics`] | Accessibility roles, labels, and action bindings. |
//!
//! Supporting types for colors, fills, strokes, text styles, flex parameters, and
//! grid tracks are also defined here.

use super::semantics::Semantics;
use super::widget_id::WidgetNodeId;
use crate::NodeId;
use serde::{Deserialize, Serialize};

/// The operation a node performs.
///
/// `Op` is the heart of the IR: it says what a [`CoreNode`](crate::CoreNode) *does*.
/// There are exactly four categories, each wrapping a more specific enum or struct.
///
/// # Example
///
/// ```rust
/// use fission_ir::{Op, LayoutOp};
///
/// let op = Op::Layout(LayoutOp::Box {
///     width: Some(100.0), height: Some(50.0),
///     min_width: None, max_width: None,
///     min_height: None, max_height: None,
///     padding: [0.0; 4], flex_grow: 0.0, flex_shrink: 1.0,
///     aspect_ratio: None,
/// });
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Op {
    /// A grouping node with no visual or layout effect. See [`StructuralOp`].
    Structural(StructuralOp),
    /// A layout node that sizes and positions its children. See [`LayoutOp`].
    Layout(LayoutOp),
    /// A paint node that draws something on screen. See [`PaintOp`].
    Paint(PaintOp),
    /// A semantics node that declares accessibility and interaction metadata.
    /// See [`Semantics`](crate::Semantics).
    Semantics(Semantics),
}

impl std::hash::Hash for Op {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Structural(s) => { 0.hash(state); s.hash(state); }
            Self::Layout(l) => { 1.hash(state); l.hash(state); }
            Self::Paint(p) => { 2.hash(state); p.hash(state); }
            Self::Semantics(s) => { 3.hash(state); s.hash(state); }
        }
    }
}

/// A structural operation that groups child nodes without any visual or layout effect.
///
/// Structural nodes exist so that the widget compiler can preserve logical grouping
/// boundaries in the IR. They are transparent to the layout engine and the renderer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash)]
pub enum StructuralOp {
    /// Groups children under a single parent. The `stable_hash` is a content hash
    /// of the group's children, used for efficient diffing.
    Group {
        /// Content hash of the grouped subtree. Two groups with the same children
        /// produce the same hash.
        stable_hash: u64,
    },
}

/// The scalar type used for all layout measurements (widths, heights, padding, etc.).
///
/// Currently `f32`. Using a type alias makes it easy to change precision globally.
pub type LayoutUnit = f32;

/// The primary axis direction for a flex or scroll container.
///
/// Determines whether children are laid out horizontally or vertically.
///
/// Defaults to [`Row`](FlexDirection::Row).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Hash)]
pub enum FlexDirection {
    /// Children are laid out left-to-right along the horizontal axis.
    Row,
    /// Children are laid out top-to-bottom along the vertical axis.
    Column,
}

impl Default for FlexDirection {
    fn default() -> Self {
        FlexDirection::Row
    }
}

/// The kind of platform-native surface embedded in the UI.
///
/// Used by [`LayoutOp::Embed`] to tell the platform layer what type of native
/// view to create and manage.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Hash)]
pub enum EmbedKind {
    /// A video playback surface.
    Video,
    /// A web browser view (e.g., WKWebView, WebView2).
    Web,
    /// A custom platform-native view not covered by the other variants.
    Custom,
}

/// A track sizing function for CSS Grid-style columns or rows.
///
/// Grid tracks define how available space is divided among columns and rows in a
/// [`LayoutOp::Grid`]. They work like the CSS `grid-template-columns` /
/// `grid-template-rows` values.
///
/// # Example
///
/// A three-column grid: 200px fixed, 1fr flexible, auto-sized:
///
/// ```rust
/// use fission_ir::op::GridTrack;
/// let columns = vec![GridTrack::Points(200.0), GridTrack::Fr(1.0), GridTrack::Auto];
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GridTrack {
    /// A fixed size in logical pixels.
    Points(LayoutUnit),
    /// A percentage of the grid container's available space (0.0 to 100.0).
    Percent(f32),
    /// A fractional unit. Remaining space after fixed and percent tracks is divided
    /// proportionally among `Fr` tracks.
    Fr(f32),
    /// Size to fit the content, with no minimum or maximum constraint.
    Auto,
    /// Size to the minimum content width/height (the narrowest the content can be
    /// without overflow).
    MinContent,
    /// Size to the maximum content width/height (the widest the content wants to be).
    MaxContent,
}

impl std::hash::Hash for GridTrack {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Points(u) => { 0.hash(state); u.to_bits().hash(state); }
            Self::Percent(f) => { 1.hash(state); f.to_bits().hash(state); }
            Self::Fr(f) => { 2.hash(state); f.to_bits().hash(state); }
            Self::Auto => { 3.hash(state); }
            Self::MinContent => { 4.hash(state); }
            Self::MaxContent => { 5.hash(state); }
        }
    }
}

/// Where a grid item is placed within its grid container.
///
/// Used by [`LayoutOp::GridItem`] to specify which row/column a child occupies.
/// Works like the CSS `grid-row-start` / `grid-column-start` properties.
///
/// Defaults to [`Auto`](GridPlacement::Auto).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Hash)]
pub enum GridPlacement {
    /// Let the grid auto-placement algorithm choose the position.
    Auto,
    /// Place at a specific grid line (1-indexed, matching CSS convention).
    Line(i16),
    /// Span across the given number of tracks from the start position.
    Span(u16),
}

impl Default for GridPlacement {
    fn default() -> Self { Self::Auto }
}

/// Whether a flex container wraps children onto multiple lines.
///
/// Equivalent to the CSS `flex-wrap` property.
///
/// Defaults to [`NoWrap`](FlexWrap::NoWrap).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Hash)]
pub enum FlexWrap {
    /// All children stay on a single line, potentially overflowing.
    NoWrap,
    /// Children wrap onto additional lines in the normal direction.
    Wrap,
    /// Children wrap onto additional lines in the reverse direction.
    WrapReverse,
}

impl Default for FlexWrap {
    fn default() -> Self {
        FlexWrap::NoWrap
    }
}

/// How children are aligned on the cross axis of a flex container.
///
/// Equivalent to the CSS `align-items` property.
///
/// Defaults to [`Stretch`](AlignItems::Stretch).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Hash)]
pub enum AlignItems {
    /// Align children to the start of the cross axis.
    Start,
    /// Align children to the end of the cross axis.
    End,
    /// Center children on the cross axis.
    Center,
    /// Stretch children to fill the cross axis. This is the default.
    Stretch,
    /// Align children so their text baselines line up.
    Baseline,
}

impl Default for AlignItems {
    fn default() -> Self {
        AlignItems::Stretch
    }
}

/// How children are distributed along the main axis of a flex container.
///
/// Equivalent to the CSS `justify-content` property.
///
/// Defaults to [`Start`](JustifyContent::Start).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Hash)]
pub enum JustifyContent {
    /// Pack children toward the start of the main axis.
    Start,
    /// Pack children toward the end of the main axis.
    End,
    /// Center children along the main axis.
    Center,
    /// Distribute children so the first is at the start, the last at the end,
    /// and equal space between each pair.
    SpaceBetween,
    /// Distribute children with equal space around each child (half-size spaces
    /// on the edges).
    SpaceAround,
    /// Distribute children with exactly equal space between and around them.
    SpaceEvenly,
}

impl Default for JustifyContent {
    fn default() -> Self {
        JustifyContent::Start
    }
}

/// A layout operation that sizes and positions a node and its children.
///
/// `LayoutOp` covers every layout model in Fission: constrained boxes, flexbox,
/// CSS Grid, scroll containers, absolute positioning, z-stacking, flyout menus,
/// transforms, and clipping. Each variant maps to a distinct layout algorithm in
/// the [`fission_layout`] crate.
///
/// # Padding convention
///
/// All `padding` fields use `[left, right, top, bottom]` order.
///
/// # Flex participation
///
/// Variants that have `flex_grow` and `flex_shrink` fields participate in flex
/// layout when placed inside a `Flex` parent. `flex_grow` controls how much extra
/// space the node claims; `flex_shrink` controls how much it gives up when the
/// container overflows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LayoutOp {
    /// A constrained box that sizes itself and stacks its children.
    ///
    /// This is the most common layout node. It applies optional fixed dimensions,
    /// min/max constraints, padding, and aspect ratio. Children are stacked on top
    /// of each other (like a single-child container).
    ///
    /// Use `Box` when you need a container with specific size constraints, padding,
    /// or an aspect ratio, but do not need flex or grid distribution.
    Box {
        /// Fixed width in logical pixels, or `None` to size-to-content.
        width: Option<LayoutUnit>,
        /// Fixed height in logical pixels, or `None` to size-to-content.
        height: Option<LayoutUnit>,
        /// Minimum width. The node will never be narrower than this.
        min_width: Option<LayoutUnit>,
        /// Maximum width. The node will never be wider than this.
        max_width: Option<LayoutUnit>,
        /// Minimum height. The node will never be shorter than this.
        min_height: Option<LayoutUnit>,
        /// Maximum height. The node will never be taller than this.
        max_height: Option<LayoutUnit>,
        /// Inner padding: `[left, right, top, bottom]`.
        padding: [LayoutUnit; 4],
        /// How much extra space this node claims when inside a flex container.
        /// `0.0` means it does not grow. Default: `0.0`.
        flex_grow: LayoutUnit,
        /// How much this node shrinks when a flex container overflows.
        /// `1.0` means it shrinks proportionally. Default: `1.0`.
        flex_shrink: LayoutUnit,
        /// If set, the node maintains this width/height ratio. For example,
        /// `Some(16.0 / 9.0)` gives a widescreen aspect ratio.
        aspect_ratio: Option<f32>,
    },
    /// A flex container that distributes children along a main axis.
    ///
    /// Implements CSS Flexbox semantics: children are measured, flex-grow/shrink is
    /// applied, and then children are positioned according to `justify_content` and
    /// `align_items`.
    Flex {
        /// Whether children flow horizontally ([`Row`](FlexDirection::Row)) or
        /// vertically ([`Column`](FlexDirection::Column)).
        direction: FlexDirection,
        /// Whether children wrap onto multiple lines. Default: [`NoWrap`](FlexWrap::NoWrap).
        wrap: FlexWrap,
        /// How much extra space this flex container claims from *its* parent flex.
        flex_grow: LayoutUnit,
        /// How much this flex container shrinks when its parent overflows.
        flex_shrink: LayoutUnit,
        /// Inner padding: `[left, right, top, bottom]`.
        padding: [LayoutUnit; 4],
        /// Space between children along the main axis. `None` means `0.0`.
        gap: Option<LayoutUnit>,
        /// Cross-axis alignment of children. Default: [`Stretch`](AlignItems::Stretch).
        align_items: AlignItems,
        /// Main-axis distribution of children. Default: [`Start`](JustifyContent::Start).
        justify_content: JustifyContent,
    },
    /// A CSS Grid container that places children into a row/column matrix.
    ///
    /// Columns and rows are defined by [`GridTrack`] sizing functions. Children are
    /// placed either automatically (in source order) or explicitly via
    /// [`GridItem`](LayoutOp::GridItem).
    Grid {
        /// Column track definitions. If empty, a single auto-width column is used.
        columns: Vec<GridTrack>,
        /// Row track definitions. If empty, rows are created automatically as needed.
        rows: Vec<GridTrack>,
        /// Horizontal gap between columns in logical pixels.
        column_gap: Option<LayoutUnit>,
        /// Vertical gap between rows in logical pixels.
        row_gap: Option<LayoutUnit>,
        /// Inner padding: `[left, right, top, bottom]`.
        padding: [LayoutUnit; 4],
    },
    /// A child of a [`Grid`](LayoutOp::Grid) that specifies its row/column placement.
    ///
    /// If a grid child does not use `GridItem`, the grid auto-placement algorithm
    /// assigns it the next available cell.
    GridItem {
        /// Which row line this item starts at. Default: [`Auto`](GridPlacement::Auto).
        row_start: GridPlacement,
        /// Which row line this item ends at. Default: [`Auto`](GridPlacement::Auto).
        row_end: GridPlacement,
        /// Which column line this item starts at. Default: [`Auto`](GridPlacement::Auto).
        col_start: GridPlacement,
        /// Which column line this item ends at. Default: [`Auto`](GridPlacement::Auto).
        col_end: GridPlacement,
    },
    /// A scrollable container.
    ///
    /// The scroll container clips its content and shifts it by a scroll offset
    /// obtained from a [`ScrollDataSource`](fission_layout). The layout engine
    /// gives the content infinite space along the scroll axis so it can measure
    /// its natural size.
    Scroll {
        /// Scroll axis: horizontal ([`Row`](FlexDirection::Row)) or vertical
        /// ([`Column`](FlexDirection::Column)).
        direction: FlexDirection,
        /// Whether to render a scrollbar indicator.
        show_scrollbar: bool,
        /// Fixed width, or `None` to size from constraints.
        width: Option<LayoutUnit>,
        /// Fixed height, or `None` to size from constraints.
        height: Option<LayoutUnit>,
        /// Minimum width constraint.
        min_width: Option<LayoutUnit>,
        /// Maximum width constraint.
        max_width: Option<LayoutUnit>,
        /// Minimum height constraint.
        min_height: Option<LayoutUnit>,
        /// Maximum height constraint.
        max_height: Option<LayoutUnit>,
        /// Inner padding: `[left, right, top, bottom]`.
        padding: [LayoutUnit; 4],
        /// Flex grow factor when inside a flex parent.
        flex_grow: LayoutUnit,
        /// Flex shrink factor when inside a flex parent.
        flex_shrink: LayoutUnit,
    },
    /// A placeholder for a platform-native surface (video, web view, etc.).
    ///
    /// The layout engine allocates space for the embed; the platform layer is
    /// responsible for creating and positioning the actual native view.
    Embed {
        /// What kind of native surface to create.
        kind: EmbedKind,
        /// The widget that owns this native surface.
        widget_id: WidgetNodeId,
        /// Fixed width, or `None` to use available space.
        width: Option<LayoutUnit>,
        /// Fixed height, or `None` to use available space.
        height: Option<LayoutUnit>,
    },
    /// A child that fills its parent's entire bounds.
    ///
    /// Equivalent to `Positioned { left: 0, top: 0, right: 0, bottom: 0 }` but
    /// expressed as a zero-field variant for clarity. Commonly used for overlays,
    /// backgrounds, and hit-test areas.
    AbsoluteFill,
    /// A child positioned absolutely within its parent.
    ///
    /// At least one of `left`/`right` and one of `top`/`bottom` should be set.
    /// If both `left` and `right` are set (and `width` is not), the width is
    /// inferred from the parent's width minus both offsets.
    Positioned {
        /// Offset from the parent's left edge.
        left: Option<LayoutUnit>,
        /// Offset from the parent's top edge.
        top: Option<LayoutUnit>,
        /// Offset from the parent's right edge.
        right: Option<LayoutUnit>,
        /// Offset from the parent's bottom edge.
        bottom: Option<LayoutUnit>,
        /// Fixed width. If `None`, width is inferred from `left`/`right`.
        width: Option<LayoutUnit>,
        /// Fixed height. If `None`, height is inferred from `top`/`bottom`.
        height: Option<LayoutUnit>,
    },
    /// A container that stacks all children on top of each other.
    ///
    /// Each child occupies the full size of the stack; later children paint on
    /// top of earlier ones. The stack's own size is the union of its children.
    ZStack,
    /// A container that centers its single child within the available space.
    Align,
    /// An anchored popup container (dropdown menu, tooltip, etc.).
    ///
    /// The `content` node is positioned relative to the `anchor` node's screen
    /// location, typically directly below it. The layout engine resolves anchor
    /// positions after the main layout pass.
    Flyout {
        /// The node that the flyout is anchored to.
        anchor: NodeId,
        /// The node containing the flyout content.
        content: NodeId,
    },
    /// Applies a 4x4 affine transform matrix to its child.
    ///
    /// The matrix is column-major, matching OpenGL/wgpu convention. The transform
    /// does not affect layout; it is applied during painting.
    Transform {
        /// A 4x4 column-major transform matrix.
        transform: [f32; 16],
    },
    /// Clips its child to a rectangular or path-defined region.
    ///
    /// If `path` is `None`, the clip is the node's layout rectangle. If `path` is
    /// set, it is an SVG-style path string.
    Clip {
        /// An optional SVG path string. `None` means clip to the layout rect.
        path: Option<String>,
    },
}

impl std::hash::Hash for LayoutOp {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let hash_unit = |u: LayoutUnit, h: &mut H| u.to_bits().hash(h);
        let hash_opt_unit = |u: Option<LayoutUnit>, h: &mut H| u.map(|v| v.to_bits()).hash(h);
        let hash_units = |us: [LayoutUnit; 4], h: &mut H| { for u in us { u.to_bits().hash(h); } };

        match self {
            Self::Box { width, height, min_width, max_width, min_height, max_height, padding, flex_grow, flex_shrink, aspect_ratio } => {
                0.hash(state); hash_opt_unit(*width, state); hash_opt_unit(*height, state);
                hash_opt_unit(*min_width, state); hash_opt_unit(*max_width, state);
                hash_opt_unit(*min_height, state); hash_opt_unit(*max_height, state);
                hash_units(*padding, state); hash_unit(*flex_grow, state); hash_unit(*flex_shrink, state);
                aspect_ratio.map(|f| f.to_bits()).hash(state);
            }
            Self::Flex { direction, wrap, flex_grow, flex_shrink, padding, gap, align_items, justify_content } => {
                1.hash(state); direction.hash(state); wrap.hash(state);
                hash_unit(*flex_grow, state); hash_unit(*flex_shrink, state);
                hash_units(*padding, state); hash_opt_unit(*gap, state);
                align_items.hash(state); justify_content.hash(state);
            }
            Self::Grid { columns, rows, column_gap, row_gap, padding } => {
                2.hash(state); columns.hash(state); rows.hash(state);
                hash_opt_unit(*column_gap, state); hash_opt_unit(*row_gap, state);
                hash_units(*padding, state);
            }
            Self::GridItem { row_start, row_end, col_start, col_end } => {
                3.hash(state); row_start.hash(state); row_end.hash(state); col_start.hash(state); col_end.hash(state);
            }
            Self::Scroll { direction, show_scrollbar, width, height, min_width, max_width, min_height, max_height, padding, flex_grow, flex_shrink } => {
                4.hash(state); direction.hash(state); show_scrollbar.hash(state);
                hash_opt_unit(*width, state); hash_opt_unit(*height, state);
                hash_opt_unit(*min_width, state); hash_opt_unit(*max_width, state);
                hash_opt_unit(*min_height, state); hash_opt_unit(*max_height, state);
                hash_units(*padding, state); hash_unit(*flex_grow, state); hash_unit(*flex_shrink, state);
            }
            Self::Embed { kind, widget_id, width, height } => {
                5.hash(state); kind.hash(state); widget_id.hash(state);
                hash_opt_unit(*width, state); hash_opt_unit(*height, state);
            }
            Self::AbsoluteFill => { 6.hash(state); }
            Self::Positioned { left, top, right, bottom, width, height } => {
                7.hash(state); hash_opt_unit(*left, state); hash_opt_unit(*top, state);
                hash_opt_unit(*right, state); hash_opt_unit(*bottom, state);
                hash_opt_unit(*width, state); hash_opt_unit(*height, state);
            }
            Self::ZStack => { 8.hash(state); }
            Self::Align => { 9.hash(state); }
            Self::Flyout { anchor, content } => { 10.hash(state); anchor.hash(state); content.hash(state); }
            Self::Transform { transform } => { 11.hash(state); for v in transform { v.to_bits().hash(state); } }
            Self::Clip { path } => { 12.hash(state); path.hash(state); }
        }
    }
}

/// An RGBA color with 8-bit channels.
///
/// Colors are used throughout the IR for fills, strokes, text, and shadows.
/// Several named constants are provided for common colors.
///
/// # Example
///
/// ```rust
/// use fission_ir::op::Color;
///
/// let semi_transparent_red = Color::RED.with_alpha(128);
/// assert_eq!(semi_transparent_red.a, 128);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Hash)]
pub struct Color {
    /// Red channel (0-255).
    pub r: u8,
    /// Green channel (0-255).
    pub g: u8,
    /// Blue channel (0-255).
    pub b: u8,
    /// Alpha channel (0 = fully transparent, 255 = fully opaque).
    pub a: u8,
}

impl Color {
    /// Opaque black: `rgba(0, 0, 0, 255)`.
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    /// Opaque white: `rgba(255, 255, 255, 255)`.
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    /// Opaque red: `rgba(255, 0, 0, 255)`.
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    /// Opaque green: `rgba(0, 255, 0, 255)`.
    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    /// Opaque blue: `rgba(0, 0, 255, 255)`.
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };

    /// Returns a copy of this color with a different alpha value.
    ///
    /// Useful for creating semi-transparent variants of existing colors without
    /// constructing a new `Color` from scratch.
    pub fn with_alpha(mut self, a: u8) -> Self {
        self.a = a;
        self
    }
}

/// A solid color fill.
///
/// Used by [`PaintOp::DrawRect`] and [`PaintOp::DrawPath`] to fill shapes with
/// a single color.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Hash)]
pub struct Fill {
    /// The fill color.
    pub color: Color,
}

/// A colored stroke (outline) with a line width.
///
/// Used by [`PaintOp::DrawRect`] and [`PaintOp::DrawPath`] to draw shape borders.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    /// The stroke color.
    pub color: Color,
    /// The stroke width in logical pixels.
    pub width: LayoutUnit,
}

impl std::hash::Hash for Stroke {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.color.hash(state);
        self.width.to_bits().hash(state);
    }
}

/// A drop shadow rendered behind a rectangle.
///
/// Used by [`PaintOp::DrawRect`] to add depth and elevation effects.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoxShadow {
    /// The shadow color (typically semi-transparent black).
    pub color: Color,
    /// The Gaussian blur radius in logical pixels. Larger values produce softer shadows.
    pub blur_radius: LayoutUnit,
    /// The horizontal and vertical offset of the shadow from the rectangle:
    /// `(dx, dy)` in logical pixels.
    pub offset: (LayoutUnit, LayoutUnit),
}

impl std::hash::Hash for BoxShadow {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.color.hash(state);
        self.blur_radius.to_bits().hash(state);
        self.offset.0.to_bits().hash(state);
        self.offset.1.to_bits().hash(state);
    }
}

/// How an image scales to fit its layout box.
///
/// Equivalent to the CSS `object-fit` property.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Hash)]
pub enum ImageFit {
    /// Scale the image uniformly so it fits entirely within the box, preserving
    /// aspect ratio. The image may be letter-boxed.
    Contain,
    /// Scale the image uniformly so it covers the entire box, preserving aspect
    /// ratio. Parts of the image may be clipped.
    Cover,
    /// Stretch the image to fill the box exactly, ignoring aspect ratio.
    Fill,
    /// Display the image at its natural size, without any scaling.
    None,
}

/// Styling properties for a run of text.
///
/// `TextStyle` controls how a segment of text is rendered: font size, color, underline,
/// and an optional background highlight (used for search-match highlighting, error
/// squiggles, etc.).
///
/// # Example
///
/// ```rust
/// use fission_ir::op::{TextStyle, Color};
///
/// let style = TextStyle {
///     font_size: 14.0,
///     color: Color::BLACK,
///     underline: false,
///     background_color: None,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    /// Font size in logical pixels.
    pub font_size: LayoutUnit,
    /// Text foreground color.
    pub color: Color,
    /// Whether to draw an underline beneath the text.
    pub underline: bool,
    /// Optional background highlight color for this run (find matches, error
    /// squiggles, selected text, etc.).
    pub background_color: Option<Color>,
}

impl std::hash::Hash for TextStyle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.font_size.to_bits().hash(state);
        self.color.hash(state);
        self.underline.hash(state);
        self.background_color.hash(state);
    }
}

/// A contiguous run of text with a uniform style.
///
/// Rich text is represented as a sequence of `TextRun`s. Each run has its own
/// [`TextStyle`], so different parts of a paragraph can have different colors,
/// sizes, or underline states.
///
/// # Example
///
/// ```rust
/// use fission_ir::op::{TextRun, TextStyle, Color};
///
/// let runs = vec![
///     TextRun {
///         text: "Hello, ".into(),
///         style: TextStyle { font_size: 14.0, color: Color::BLACK, underline: false, background_color: None },
///     },
///     TextRun {
///         text: "world!".into(),
///         style: TextStyle { font_size: 14.0, color: Color::BLUE, underline: true, background_color: None },
///     },
/// ];
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Hash)]
pub struct TextRun {
    /// The text content of this run.
    pub text: String,
    /// The style applied to every character in this run.
    pub style: TextStyle,
}

/// A paint operation that draws something on screen.
///
/// Paint nodes do not participate in layout sizing -- their visual output is
/// painted into the bounding box determined by their parent layout node. The
/// renderer walks paint ops to build the final [`DisplayList`](fission_render).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PaintOp {
    /// Draws a filled and/or stroked rectangle with optional rounded corners and shadow.
    ///
    /// This is the workhorse of the paint pipeline -- backgrounds, borders, cards,
    /// buttons, and dividers all compile down to `DrawRect`.
    DrawRect {
        /// The interior fill color. `None` means the rectangle has no fill (transparent).
        fill: Option<Fill>,
        /// The border stroke. `None` means no border.
        stroke: Option<Stroke>,
        /// Corner radius in logical pixels. `0.0` means sharp corners.
        corner_radius: LayoutUnit,
        /// An optional drop shadow behind the rectangle.
        shadow: Option<BoxShadow>,
    },
    /// Draws a single-style text string.
    ///
    /// Use this for simple labels where the entire string shares one style. For
    /// mixed-style text (e.g., syntax highlighting), use [`DrawRichText`](PaintOp::DrawRichText).
    DrawText {
        /// The text content to render.
        text: String,
        /// Font size in logical pixels.
        size: LayoutUnit,
        /// Text foreground color.
        color: Color,
        /// Whether to underline the text.
        underline: bool,
        /// If set, the renderer draws a text cursor at this byte index.
        caret_index: Option<usize>,
    },
    /// Draws multi-style (rich) text composed of [`TextRun`]s.
    ///
    /// Each run can have a different font size, color, underline, and background
    /// highlight. Used for code editors, formatted messages, and any text where
    /// inline styling varies.
    DrawRichText {
        /// The styled text runs, in order.
        runs: Vec<TextRun>,
        /// If set, the renderer draws a text cursor at this byte index
        /// (relative to the concatenated run text).
        caret_index: Option<usize>,
    },
    /// Draws a raster image from a URI or asset path.
    DrawImage {
        /// The image source: a file path, HTTP URL, or asset identifier.
        source: String,
        /// How the image scales to fit its layout box.
        fit: ImageFit,
    },
    /// Draws an SVG-style path string, optionally filled and/or stroked.
    ///
    /// The `path` uses SVG path data syntax (e.g., `"M 0 0 L 10 10 Z"`).
    DrawPath {
        /// SVG path data string.
        path: String,
        /// Optional fill color for the path interior.
        fill: Option<Fill>,
        /// Optional stroke for the path outline.
        stroke: Option<Stroke>,
    },
    /// Draws inline SVG content, optionally overriding fill and stroke colors.
    DrawSvg {
        /// The raw SVG markup as a string.
        content: String,
        /// Optional fill color override applied to the SVG elements.
        fill: Option<Fill>,
        /// Optional stroke color override applied to the SVG elements.
        stroke: Option<Stroke>,
    },
}

impl std::hash::Hash for PaintOp {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::DrawRect { fill, stroke, corner_radius, shadow } => {
                0.hash(state); fill.hash(state); stroke.hash(state);
                corner_radius.to_bits().hash(state); shadow.hash(state);
            }
            Self::DrawText { text, size, color, underline, caret_index } => {
                1.hash(state); text.hash(state); size.to_bits().hash(state);
                color.hash(state); underline.hash(state); caret_index.hash(state);
            }
            Self::DrawRichText { runs, caret_index } => {
                2.hash(state); runs.hash(state); caret_index.hash(state);
            }
            Self::DrawImage { source, fit } => {
                3.hash(state); source.hash(state); fit.hash(state);
            }
            Self::DrawPath { path, fill, stroke } => {
                4.hash(state); path.hash(state); fill.hash(state); stroke.hash(state);
            }
            Self::DrawSvg { content, fill, stroke } => {
                5.hash(state); content.hash(state); fill.hash(state); stroke.hash(state);
            }
        }
    }
}
