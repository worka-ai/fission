//! The serialisable widget-tree node enum.
//!
//! [`Node`] is the data structure returned by [`Widget::build`](crate::Widget::build).
//! It has one variant per built-in widget type plus a [`Custom`](Node::Custom)
//! escape hatch for application-defined widgets that need custom lowering.

use super::traits::{Lower, LowerDyn};
use super::widgets::{
    Align, Button, Checkbox, Clip, Column, Container, GestureDetector, FocusScope, Grid, GridItem, Icon, Image, LazyColumn, Overlay, Positioned, Radio, Row, SafeArea, Scroll, Slider, Spacer,
    Switch, Text, TextInput, Transform, Video, ZStack,
};
use crate::lowering::LoweringContext;
use fission_ir::{NodeId, Op, StructuralOp};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// A serialisable node in the declarative widget tree.
///
/// Every variant wraps one of the built-in widget structs. The tree is
/// constructed by [`Widget::build`](crate::Widget::build) and lowered into
/// the `fission-ir` intermediate representation for layout and rendering.
///
/// # Example
///
/// ```rust,ignore
/// let tree = Node::Column(Column {
///     children: vec![
///         Node::Text(Text::new("Hello")),
///         Node::Button(Button {
///             child: Some(Box::new(Node::Text(Text::new("Click me")))),
///             on_press: Some(envelope),
///             ..Default::default()
///         }),
///     ],
///     ..Default::default()
/// });
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Node {
    /// Horizontal flex container. See [`Row`].
    Row(Row),
    /// Vertical flex container. See [`Column`].
    Column(Column),
    /// Center-aligns its child within the available space. See [`Align`].
    Align(Align),
    /// Limits focus traversal to a subtree. See [`FocusScope`].
    FocusScope(FocusScope),
    /// Clips child content to a rounded rectangle. See [`Clip`].
    Clip(Clip),
    /// Static or i18n text label. See [`Text`].
    Text(Text),
    /// Applies a 4x4 matrix transform. See [`Transform`].
    Transform(Transform),
    /// Pressable button with variant styling. See [`Button`].
    Button(Button),
    /// Editable text field with optional syntax-highlighting. See [`TextInput`].
    TextInput(TextInput),
    /// Scrollable container. See [`Scroll`].
    Scroll(Scroll),
    /// Raster image. See [`Image`].
    Image(Image),
    /// Platform-native video player. See [`Video`].
    Video(Video),
    /// Z-axis stacking container (children layered on top of each other). See [`ZStack`].
    ZStack(ZStack),
    /// Content with an overlay layer on top. See [`Overlay`].
    Overlay(Overlay),
    /// Universal wrapper with background, border, padding, and size. See [`Container`].
    Container(Container),
    /// Gesture handler (tap, drag, hover, drop). See [`GestureDetector`].
    GestureDetector(GestureDetector),
    /// CSS-grid-style layout. See [`Grid`].
    Grid(Grid),
    /// A child placed within a [`Grid`]. See [`GridItem`].
    GridItem(GridItem),
    /// Boolean toggle with a square indicator. See [`Checkbox`].
    Checkbox(Checkbox),
    /// Boolean toggle with a sliding thumb. See [`Switch`].
    Switch(Switch),
    /// Single-select radio button. See [`Radio`].
    Radio(Radio),
    /// Insets content to avoid system chrome (notch, status bar). See [`SafeArea`].
    SafeArea(SafeArea),
    /// Absolutely positioned child within a [`ZStack`]. See [`Positioned`].
    Positioned(Positioned),
    /// Flexible or fixed-size empty space. See [`Spacer`].
    Spacer(Spacer),
    /// Continuous value selector with a draggable thumb. See [`Slider`].
    Slider(Slider),
    /// Virtualized vertical list for large data sets. See [`LazyColumn`].
    LazyColumn(LazyColumn),
    /// Vector icon rendered from an SVG path, file, or inline content. See [`Icon`].
    Icon(Icon),
    /// Escape hatch for application-defined widgets with custom lowering. See [`CustomNode`].
    Custom(CustomNode),
}

impl Node {
    pub fn lower(&self, cx: &mut LoweringContext) -> NodeId {
        match self {
            Node::Row(w) => w.lower(cx),
            Node::Column(w) => w.lower(cx),
            Node::Align(w) => w.lower(cx),
            Node::FocusScope(w) => w.lower(cx),
            Node::Clip(w) => w.lower(cx),
            Node::Text(w) => w.lower(cx),
            Node::Transform(w) => w.lower(cx),
            Node::Button(w) => w.lower(cx),
            Node::TextInput(w) => w.lower(cx),
            Node::Scroll(w) => w.lower(cx),
            Node::Image(w) => w.lower(cx),
            Node::Video(w) => w.lower(cx),
            Node::ZStack(w) => w.lower(cx),
            Node::Overlay(w) => w.lower(cx),
            Node::Container(w) => w.lower(cx),
            Node::GestureDetector(w) => w.lower(cx),
            Node::Grid(w) => w.lower(cx),
            Node::GridItem(w) => w.lower(cx),
            Node::Checkbox(w) => w.lower(cx),
            Node::Switch(w) => w.lower(cx),
            Node::Radio(w) => w.lower(cx),
            Node::SafeArea(w) => w.lower(cx),
            Node::Positioned(w) => w.lower(cx),
            Node::Spacer(w) => w.lower(cx),
            Node::Slider(w) => w.lower(cx),
            Node::LazyColumn(w) => w.lower(cx),
            Node::Icon(w) => w.lower(cx),
            Node::Custom(w) => {
                let lowerer = w.lowerer.as_ref().expect("CustomNode lowerer must be set");
                let child_id = lowerer.lower_dyn(cx);
                let wrapper = cx.next_node_id();
                let mut builder = crate::lowering::NodeBuilder::new(
                    wrapper,
                    Op::Structural(StructuralOp::Group {
                        stable_hash: lowerer.stable_key(),
                    }),
                );
                builder.add_child(child_id);
                builder.build(cx)
            }
        }
    }
}

impl From<Row> for Node {
    fn from(w: Row) -> Self {
        Node::Row(w)
    }
}
impl From<Column> for Node {
    fn from(w: Column) -> Self {
        Node::Column(w)
    }
}
impl From<Align> for Node {
    fn from(w: Align) -> Self {
        Node::Align(w)
    }
}
impl From<FocusScope> for Node {
    fn from(w: FocusScope) -> Self {
        Node::FocusScope(w)
    }
}
impl From<Clip> for Node {
    fn from(w: Clip) -> Self {
        Node::Clip(w)
    }
}
impl From<Text> for Node {
    fn from(w: Text) -> Self {
        Node::Text(w)
    }
}
impl From<Transform> for Node {
    fn from(w: Transform) -> Self {
        Node::Transform(w)
    }
}
impl From<Button> for Node {
    fn from(w: Button) -> Self {
        Node::Button(w)
    }
}
impl From<TextInput> for Node {
    fn from(w: TextInput) -> Self {
        Node::TextInput(w)
    }
}
impl From<Scroll> for Node {
    fn from(w: Scroll) -> Self {
        Node::Scroll(w)
    }
}
impl From<Image> for Node {
    fn from(w: Image) -> Self {
        Node::Image(w)
    }
}
impl From<ZStack> for Node {
    fn from(w: ZStack) -> Self {
        Node::ZStack(w)
    }
}
impl From<Overlay> for Node {
    fn from(w: Overlay) -> Self {
        Node::Overlay(w)
    }
}
impl From<Container> for Node {
    fn from(w: Container) -> Self {
        Node::Container(w)
    }
}
impl From<GestureDetector> for Node {
    fn from(w: GestureDetector) -> Self {
        Node::GestureDetector(w)
    }
}
impl From<Grid> for Node {
    fn from(w: Grid) -> Self {
        Node::Grid(w)
    }
}
impl From<GridItem> for Node {
    fn from(w: GridItem) -> Self {
        Node::GridItem(w)
    }
}
impl From<Checkbox> for Node {
    fn from(w: Checkbox) -> Self {
        Node::Checkbox(w)
    }
}
impl From<Switch> for Node {
    fn from(w: Switch) -> Self {
        Node::Switch(w)
    }
}
impl From<Radio> for Node {
    fn from(w: Radio) -> Self {
        Node::Radio(w)
    }
}
impl From<SafeArea> for Node {
    fn from(w: SafeArea) -> Self {
        Node::SafeArea(w)
    }
}
impl From<Positioned> for Node {
    fn from(w: Positioned) -> Self {
        Node::Positioned(w)
    }
}
impl From<Spacer> for Node {
    fn from(w: Spacer) -> Self {
        Node::Spacer(w)
    }
}
impl From<Slider> for Node {
    fn from(w: Slider) -> Self {
        Node::Slider(w)
    }
}
impl From<LazyColumn> for Node {
    fn from(w: LazyColumn) -> Self {
        Node::LazyColumn(w)
    }
}
impl From<Icon> for Node {
    fn from(w: Icon) -> Self {
        Node::Icon(w)
    }
}

/// An application-defined node with custom lowering logic.
///
/// `CustomNode` is the escape hatch for widgets that cannot be expressed with
/// the built-in primitives. Provide a [`LowerDyn`] implementation to emit
/// arbitrary `fission-ir` operations.
///
/// # Example
///
/// ```rust,ignore
/// let custom = CustomNode {
///     debug_tag: "MyCanvasWidget".to_string(),
///     lowerer: Some(Arc::new(MyCanvasLowerer)),
/// };
/// Node::Custom(custom)
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CustomNode {
    /// Human-readable name for debugging and diagnostics.
    pub debug_tag: String,
    /// The lowering implementation (skipped during serialisation).
    #[serde(skip)]
    pub lowerer: Option<Arc<dyn LowerDyn>>,
}
