//! Content-addressed node identity.
//!
//! Every node in the IR graph is identified by a [`NodeId`] -- a 128-bit value
//! derived from a BLAKE3 hash. Two construction strategies are available:
//!
//! * **Explicit** -- hash a user-provided string key. Stable across rebuilds as
//!   long as the key string does not change.
//! * **Derived** -- hash a parent ID plus a child-index path. Gives every node in
//!   a subtree a deterministic identity based on its structural position.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A content-addressed 128-bit node identity.
///
/// `NodeId` is the primary key used throughout the Fission pipeline to refer to a
/// specific node. Because it is derived from BLAKE3 hashes, two nodes with the same
/// derivation inputs always produce the same `NodeId`, which makes tree diffing cheap.
///
/// # Construction
///
/// ```rust
/// use fission_ir::NodeId;
///
/// // From a stable string key (good for well-known, named nodes):
/// let id = NodeId::explicit("sidebar");
///
/// // From a parent ID and a position path (good for list items):
/// let parent = NodeId::explicit("list");
/// let item_3 = NodeId::derived(parent.as_u128(), &[3]);
/// ```
///
/// # Equality and hashing
///
/// `NodeId` implements `Eq`, `Hash`, and `Ord`, so it can be used as a key in
/// `HashMap`, `BTreeMap`, and `HashSet`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct NodeId(u128);

impl NodeId {
    /// Creates a `NodeId` from a raw 128-bit value.
    ///
    /// This is intended for internal use or deserialization. In most cases you
    /// should use [`NodeId::explicit`] or [`NodeId::derived`] instead.
    pub const fn from_u128(val: u128) -> Self {
        Self(val)
    }

    /// Returns the underlying 128-bit value.
    ///
    /// Useful when you need to feed a node's identity into another hash (e.g.,
    /// when deriving child IDs with [`NodeId::derived`]).
    pub fn as_u128(&self) -> u128 {
        self.0
    }

    /// Creates a `NodeId` from a user-provided string key.
    ///
    /// The key is hashed with BLAKE3 (prefixed with `"explicit:"`), producing a
    /// deterministic ID that is stable across rebuilds as long as the key string
    /// does not change. Use this for well-known, named nodes like `"root"` or
    /// `"toolbar"`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fission_ir::NodeId;
    /// let a = NodeId::explicit("header");
    /// let b = NodeId::explicit("header");
    /// assert_eq!(a, b); // same key -> same ID
    /// ```
    pub fn explicit(key: &str) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"explicit:");
        hasher.update(key.as_bytes());
        let hash = hasher.finalize();
        Self(u128::from_le_bytes(
            hash.as_bytes()[0..16].try_into().unwrap(),
        ))
    }

    /// Creates a `NodeId` derived from a parent ID and a child-index path.
    ///
    /// This implements *structural identity*: a node's ID is determined by its
    /// position in the tree rather than by a user-provided name. Useful for
    /// dynamically generated children like list items.
    ///
    /// # Arguments
    ///
    /// * `parent` -- The parent node's raw `u128` value (see [`NodeId::as_u128`]).
    /// * `path` -- One or more child indices describing the path from the parent.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fission_ir::NodeId;
    /// let parent = NodeId::explicit("list");
    /// let item_0 = NodeId::derived(parent.as_u128(), &[0]);
    /// let item_1 = NodeId::derived(parent.as_u128(), &[1]);
    /// assert_ne!(item_0, item_1);
    /// ```
    pub fn derived(parent: u128, path: &[u32]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"derived:");
        hasher.update(&parent.to_le_bytes());
        for index in path {
            hasher.update(&index.to_le_bytes());
        }
        let hash = hasher.finalize();
        Self(u128::from_le_bytes(
            hash.as_bytes()[0..16].try_into().unwrap(),
        ))
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({:032x})", self.0)
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}
