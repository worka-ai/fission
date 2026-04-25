//! Display list and rendering abstraction for the Fission UI framework.
//!
//! After the widget tree has been compiled to an IR and the layout engine has
//! positioned every node, the framework flattens the result into a [`DisplayList`]
//! -- an ordered sequence of [`DisplayOp`] commands that describe exactly what to
//! draw and in what order.
//!
//! Platform backends implement the [`Renderer`] trait to consume a display list and
//! produce pixels using whatever GPU or software rasterizer is available.
//!
//! # Pipeline
//!
//! ```text
//! CoreIR  -->  LayoutSnapshot  -->  DisplayList  -->  Renderer (Metal, Vulkan, ...)
//! ```
//!
//! # Graphics state model
//!
//! The display list uses a save/restore stack model (like HTML Canvas or
//! CoreGraphics). [`DisplayOp::Save`] pushes the current clip and transform onto a
//! stack, and [`DisplayOp::Restore`] pops it. Drawing operations between a
//! save/restore pair are affected by the clips and transforms set within that scope.

use fission_ir::NodeId;
pub use fission_layout::{LayoutPoint, LayoutRect, LayoutSize, LayoutUnit};
use serde::{Deserialize, Serialize};

/// An RGBA color with 8-bit channels.
///
/// This is the render-side mirror of [`fission_ir::op::Color`]. A separate type is
/// defined here so that `fission-render` can be used without pulling in the full IR
/// crate's type system.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

/// A solid color fill for shapes.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Fill {
    /// The fill color.
    pub color: Color,
}

/// A colored stroke (outline) with a line width.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    /// The stroke color.
    pub color: Color,
    /// The stroke width in logical pixels.
    pub width: LayoutUnit,
}

/// A drop shadow rendered behind a rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoxShadow {
    /// The shadow color (typically semi-transparent black).
    pub color: Color,
    /// The Gaussian blur radius in logical pixels.
    pub blur_radius: LayoutUnit,
    /// The horizontal and vertical offset: `(dx, dy)`.
    pub offset: (LayoutUnit, LayoutUnit),
}

/// How an image scales to fit its layout box.
///
/// Equivalent to the CSS `object-fit` property.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ImageFit {
    /// Scale uniformly to fit inside the box (may letter-box).
    Contain,
    /// Scale uniformly to cover the entire box (may clip).
    Cover,
    /// Stretch to fill exactly (ignores aspect ratio).
    Fill,
    /// Display at natural size, no scaling.
    None,
}

/// Styling properties for a run of text.
///
/// Controls font size, color, underline, and optional background highlight
/// (used for search matches, selections, etc.).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    /// Font size in logical pixels.
    pub font_size: LayoutUnit,
    /// Text foreground color.
    pub color: Color,
    /// Whether to draw an underline beneath the text.
    pub underline: bool,
    /// Optional background highlight color for this run.
    pub background_color: Option<Color>,
}

/// A contiguous run of text with a uniform style.
///
/// Rich text is represented as a `Vec<TextRun>`. Each run carries its own
/// [`TextStyle`], allowing mixed colors, sizes, and underlines within a single
/// text block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextRun {
    /// The text content of this run.
    pub text: String,
    /// The style applied to this run.
    pub style: TextStyle,
}

/// A single rendering command in a [`DisplayList`].
///
/// Display ops are executed in order by a [`Renderer`] backend. The save/restore
/// stack model means that clip and transform ops affect all subsequent draw ops
/// until the matching `Restore`.
///
/// Most draw ops carry a `bounds` field (the layout rectangle of the owning node)
/// and an optional `node_id` for hit-testing and debugging.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DisplayOp {
    /// Pushes the current graphics state (clip region, transform) onto the stack.
    Save,
    /// Pops the graphics state, reverting clip and transform to the last `Save`.
    Restore,
    /// Sets a rectangular clip region. Drawing outside this rectangle is discarded.
    ClipRect(LayoutRect),
    /// Sets a rounded-rectangle clip region.
    ClipRoundedRect {
        /// The clip rectangle.
        rect: LayoutRect,
        /// The corner radius for the rounded clip.
        radius: LayoutUnit,
    },
    /// Translates the coordinate origin by the given offset.
    Translate(LayoutPoint),
    /// Applies a 4x4 column-major transform matrix to the coordinate space.
    Transform([LayoutUnit; 16]),
    /// Draws a filled and/or stroked rectangle with optional rounded corners and shadow.
    DrawRect {
        /// The rectangle to draw.
        rect: LayoutRect,
        /// Optional interior fill.
        fill: Option<Fill>,
        /// Optional border stroke.
        stroke: Option<Stroke>,
        /// Corner radius (0.0 for sharp corners).
        corner_radius: LayoutUnit,
        /// Optional drop shadow.
        shadow: Option<BoxShadow>,
        /// The layout bounds of the owning node (used for hit-testing).
        bounds: LayoutRect,
        /// The IR node that produced this draw call (for debugging).
        node_id: Option<NodeId>,
    },
    /// Draws a single-style text string.
    DrawText {
        /// The text to render.
        text: String,
        /// The top-left position of the text.
        position: LayoutPoint,
        /// Font size in logical pixels.
        size: LayoutUnit,
        /// Text foreground color.
        color: Color,
        /// The layout bounds of the owning node.
        bounds: LayoutRect,
        /// The IR node that produced this draw call.
        node_id: Option<NodeId>,
        /// Whether to underline the text.
        underline: bool,
        /// If set, a text cursor is drawn at this byte index.
        caret_index: Option<usize>,
    },
    /// Draws multi-style (rich) text.
    DrawRichText {
        /// The styled text runs, in order.
        runs: Vec<TextRun>,
        /// The top-left position of the text block.
        position: LayoutPoint,
        /// The layout bounds of the owning node.
        bounds: LayoutRect,
        /// The IR node that produced this draw call.
        node_id: Option<NodeId>,
        /// If set, a text cursor is drawn at this byte index.
        caret_index: Option<usize>,
    },
    /// Draws a raster image.
    DrawImage {
        /// The rectangle to draw the image into.
        rect: LayoutRect,
        /// Image source: file path, URL, or asset identifier.
        source: String,
        /// How the image scales to fit `rect`.
        fit: ImageFit,
        /// The layout bounds of the owning node.
        bounds: LayoutRect,
        /// The IR node that produced this draw call.
        node_id: Option<NodeId>,
    },
    /// Draws an SVG-style path string.
    DrawPath {
        /// SVG path data (e.g., `"M 0 0 L 10 10 Z"`).
        path: String,
        /// Optional fill for the path interior.
        fill: Option<Fill>,
        /// Optional stroke for the path outline.
        stroke: Option<Stroke>,
        /// The layout bounds of the owning node.
        bounds: LayoutRect,
        /// The IR node that produced this draw call.
        node_id: Option<NodeId>,
    },
    /// Draws inline SVG content.
    DrawSvg {
        /// The raw SVG markup.
        content: String,
        /// Optional fill color override.
        fill: Option<Fill>,
        /// Optional stroke color override.
        stroke: Option<Stroke>,
        /// The layout bounds of the owning node.
        bounds: LayoutRect,
        /// The IR node that produced this draw call.
        node_id: Option<NodeId>,
    },
    /// Blits an external surface (video frame, embedded web view, etc.).
    DrawSurface {
        /// The rectangle to draw the surface into.
        rect: LayoutRect,
        /// Platform-specific surface identifier.
        surface_id: u64,
        /// Position/frame index within the surface (e.g., video frame number).
        position: u64,
        /// The layout bounds of the owning node.
        bounds: LayoutRect,
        /// The IR node that produced this draw call.
        node_id: Option<NodeId>,
    },
}

/// An ordered list of [`DisplayOp`]s ready to be consumed by a [`Renderer`].
///
/// The display list is the final output of the Fission pipeline before pixels.
/// It is serializable, so it can be sent to a separate render thread or process.
///
/// # Example
///
/// ```rust
/// use fission_render::*;
///
/// let bounds = LayoutRect::new(0.0, 0.0, 800.0, 600.0);
/// let mut list = DisplayList::new(bounds);
///
/// list.push(DisplayOp::Save);
/// list.push(DisplayOp::DrawRect {
///     rect: LayoutRect::new(10.0, 10.0, 100.0, 50.0),
///     fill: Some(Fill { color: Color { r: 0, g: 0, b: 0, a: 255 } }),
///     stroke: None,
///     corner_radius: 0.0,
///     shadow: None,
///     bounds: LayoutRect::new(10.0, 10.0, 100.0, 50.0),
///     node_id: None,
/// });
/// list.push(DisplayOp::Restore);
///
/// assert_eq!(list.ops.len(), 3);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayList {
    /// The ordered sequence of rendering commands.
    pub ops: Vec<DisplayOp>,
    /// The bounding rectangle of the entire scene (typically the viewport).
    pub bounds: LayoutRect,
}

impl DisplayList {
    /// Creates an empty display list with the given scene bounds.
    pub fn new(bounds: LayoutRect) -> Self {
        Self {
            ops: Vec::new(),
            bounds,
        }
    }

    /// Appends a display operation to the end of the list.
    pub fn push(&mut self, op: DisplayOp) {
        self.ops.push(op);
    }
}

/// A backend that consumes a [`DisplayList`] and produces pixels.
///
/// Implement this trait for each platform rendering backend (Metal, Vulkan, wgpu,
/// Skia, software rasterizer, etc.). The framework calls
/// [`render`](Renderer::render) once per frame with the display list for the
/// current scene.
///
/// # Example
///
/// ```rust,ignore
/// use fission_render::{Renderer, DisplayList, DisplayOp};
///
/// struct SoftwareRenderer { buffer: Vec<u8> }
///
/// impl Renderer for SoftwareRenderer {
///     fn render(&mut self, display_list: &DisplayList) -> anyhow::Result<()> {
///         for op in &display_list.ops {
///             match op {
///                 DisplayOp::DrawRect { rect, fill, .. } => { /* rasterize */ }
///                 _ => {}
///             }
///         }
///         Ok(())
///     }
/// }
/// ```
pub trait Renderer {
    /// Renders the given display list. Called once per frame.
    fn render(&mut self, display_list: &DisplayList) -> anyhow::Result<()>;
}
