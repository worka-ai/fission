//! Widget-level identity, separate from IR node identity.
//!
//! A [`WidgetNodeId`] identifies a *widget* rather than an IR node. Widgets may
//! compile to multiple IR nodes, but they share a single `WidgetNodeId`. This is
//! used primarily by [`LayoutOp::Embed`](crate::op::LayoutOp::Embed) to reference
//! a platform-native surface (video player, web view, etc.) that the framework
//! does not render itself.

use blake3;
use serde::{Deserialize, Serialize};

/// A 128-bit identity for a widget.
///
/// Like [`NodeId`](crate::NodeId), this is derived from a BLAKE3 hash, but it uses
/// the `"widget:"` prefix so that widget IDs and node IDs never collide. Widget IDs
/// are interconvertible with node IDs via `From` impls.
///
/// # Example
///
/// ```rust
/// use fission_ir::WidgetNodeId;
/// let wid = WidgetNodeId::explicit("video-player");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord, Debug)]
pub struct WidgetNodeId(u128);

impl WidgetNodeId {
    /// Creates a `WidgetNodeId` from a raw 128-bit value.
    ///
    /// Intended for internal use or deserialization.
    pub const fn from_u128(val: u128) -> Self {
        Self(val)
    }

    /// Returns the underlying 128-bit value.
    pub fn as_u128(&self) -> u128 {
        self.0
    }

    /// Creates a `WidgetNodeId` from a user-provided name string.
    ///
    /// The name is hashed with BLAKE3 (prefixed with `"widget:"`), producing a
    /// deterministic ID. Use this to give stable identities to platform-embedded
    /// widgets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fission_ir::WidgetNodeId;
    /// let a = WidgetNodeId::explicit("camera-preview");
    /// let b = WidgetNodeId::explicit("camera-preview");
    /// assert_eq!(a, b);
    /// ```
    pub fn explicit(name: &str) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"widget:");
        hasher.update(name.as_bytes());
        let hash = hasher.finalize();
        Self(u128::from_le_bytes(
            hash.as_bytes()[0..16].try_into().unwrap(),
        ))
    }
}

impl From<crate::node_id::NodeId> for WidgetNodeId {
    fn from(node: crate::node_id::NodeId) -> Self {
        Self(node.as_u128())
    }
}

impl From<WidgetNodeId> for crate::node_id::NodeId {
    fn from(id: WidgetNodeId) -> Self {
        crate::node_id::NodeId::from_u128(id.0)
    }
}
