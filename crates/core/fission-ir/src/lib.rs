//! Intermediate representation for the Fission UI framework.
//!
//! `fission-ir` defines the node graph that sits between the high-level widget tree
//! and the low-level layout and paint pipelines. Every widget compiles down to one or
//! more [`CoreNode`]s stored inside a [`CoreIR`] container. Each node carries a single
//! [`Op`] that describes what it does: lay out children, draw something on screen,
//! group subtrees, or declare accessibility semantics.
//!
//! # Architecture
//!
//! ```text
//! Widget Tree  -->  fission-ir (CoreIR)  -->  Layout Engine  -->  Display List
//! ```
//!
//! The IR is platform-agnostic, serializable (via serde), and content-addressed
//! (every [`NodeId`] is a BLAKE3 hash). This makes it cheap to diff across frames
//! and safe to send across process boundaries.
//!
//! # Example
//!
//! ```rust
//! use fission_ir::{CoreIR, NodeId, Op, LayoutOp, FlexDirection, FlexWrap, AlignItems, JustifyContent};
//!
//! let mut ir = CoreIR::new();
//!
//! let child = NodeId::explicit("label");
//! ir.add_node(child, Op::Layout(LayoutOp::Box {
//!     width: Some(200.0), height: Some(40.0),
//!     min_width: None, max_width: None,
//!     min_height: None, max_height: None,
//!     padding: [4.0; 4],
//!     flex_grow: 0.0, flex_shrink: 1.0,
//!     aspect_ratio: None,
//! }), vec![]);
//!
//! let root = NodeId::explicit("root");
//! ir.add_node(root, Op::Layout(LayoutOp::Flex {
//!     direction: FlexDirection::Column,
//!     wrap: FlexWrap::NoWrap,
//!     flex_grow: 1.0, flex_shrink: 1.0,
//!     padding: [8.0; 4], gap: Some(4.0),
//!     align_items: AlignItems::Start,
//!     justify_content: JustifyContent::Start,
//! }), vec![child]);
//!
//! ir.set_root(root);
//! assert_eq!(ir.nodes.len(), 2);
//! ```

pub mod node_id;
pub mod op;
pub mod semantics;
pub mod widget_id;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use node_id::NodeId;
pub use op::{
    AlignItems, EmbedKind, FlexDirection, FlexWrap, GridPlacement, GridTrack, JustifyContent,
    LayoutOp, Op, PaintOp, StructuralOp,
};
pub use semantics::{ActionEntry, ActionSet, Role, Semantics};
pub use widget_id::WidgetNodeId;

/// The current version of the IR format.
///
/// Increment this when making breaking changes to the serialized representation
/// so that consumers can detect version mismatches.
pub const IR_VERSION: u32 = 1;

/// A single node in the intermediate representation graph.
///
/// Every element in a Fission UI compiles down to one or more `CoreNode`s. A node
/// carries an [`Op`] that says what it *does* (layout, paint, group, or declare
/// semantics), a list of children, and a content hash for efficient diffing.
///
/// # Fields
///
/// Nodes form a tree through `children` (downward links) and `parent` (upward link).
/// The `hash` field is a content hash of the node's operation and children, used to
/// skip unchanged subtrees during reconciliation.
///
/// # Example
///
/// You rarely construct `CoreNode` directly -- use [`CoreIR::add_node`] instead:
///
/// ```rust
/// use fission_ir::{CoreIR, NodeId, Op, LayoutOp};
///
/// let mut ir = CoreIR::new();
/// let id = NodeId::explicit("box");
/// ir.add_node(id, Op::Layout(LayoutOp::Box {
///     width: Some(100.0), height: Some(50.0),
///     min_width: None, max_width: None,
///     min_height: None, max_height: None,
///     padding: [0.0; 4], flex_grow: 0.0, flex_shrink: 1.0,
///     aspect_ratio: None,
/// }), vec![]);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoreNode {
    /// The unique, content-addressed identity of this node.
    pub id: NodeId,
    /// The operation this node performs (layout, paint, structural, or semantics).
    pub op: Op,
    /// Ordered list of child node IDs. Order matters for layout and paint order.
    pub children: Vec<NodeId>,
    /// The parent of this node, or `None` if this is the root.
    /// Set automatically by [`CoreIR::add_node`].
    pub parent: Option<NodeId>,
    /// A content hash of this node's operation and subtree, used for efficient
    /// diffing between frames. A value of `0` means the hash has not been computed.
    pub hash: u64,
}

/// The root container for an intermediate representation graph.
///
/// `CoreIR` owns all nodes in the tree and knows which one is the root. It is the
/// primary data structure you build when compiling a widget tree, and the primary
/// input to the layout engine.
///
/// # Example
///
/// ```rust
/// use fission_ir::{CoreIR, NodeId, Op, LayoutOp};
///
/// let mut ir = CoreIR::new();
/// let root = NodeId::explicit("root");
/// ir.add_node(root, Op::Layout(LayoutOp::Box {
///     width: None, height: None,
///     min_width: None, max_width: None,
///     min_height: None, max_height: None,
///     padding: [0.0; 4], flex_grow: 1.0, flex_shrink: 1.0,
///     aspect_ratio: None,
/// }), vec![]);
/// ir.set_root(root);
/// assert!(ir.root.is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CoreIR {
    /// All nodes in the graph, keyed by their [`NodeId`].
    pub nodes: HashMap<NodeId, CoreNode>,
    /// The root node of the tree, or `None` if the tree is empty.
    pub root: Option<NodeId>,
}

impl CoreIR {
    /// Creates an empty IR with no nodes and no root.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node to the graph and wires up parent-child relationships.
    ///
    /// Each child in `children` that already exists in the graph will have its
    /// `parent` field set to `id`. Add children before their parents to ensure
    /// the parent link is established.
    ///
    /// # Arguments
    ///
    /// * `id` -- The identity of the new node.
    /// * `op` -- What this node does (layout, paint, structural, or semantics).
    /// * `children` -- Ordered list of child [`NodeId`]s.
    pub fn add_node(&mut self, id: NodeId, op: Op, children: Vec<NodeId>) {
        let core_node = CoreNode {
            id,
            op,
            children: children.clone(),
            parent: None,
            hash: 0,
        };
        self.nodes.insert(id, core_node);

        for child_id in children {
            if let Some(child_node) = self.nodes.get_mut(&child_id) {
                child_node.parent = Some(id);
            }
        }
    }

    /// Designates a node as the root of the tree.
    ///
    /// The layout engine starts traversal from this node. There must be exactly
    /// one root; calling this again replaces the previous root.
    pub fn set_root(&mut self, id: NodeId) {
        self.root = Some(id);
    }
}
