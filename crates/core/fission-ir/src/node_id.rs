use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct NodeId(u128);

impl NodeId {
    /// Create a NodeId from a known hash/value (internal use only).
    pub const fn from_u128(val: u128) -> Self {
        Self(val)
    }

    /// Create an explicit NodeId from a user-provided string key.
    /// This is stable across refactors if the string remains constant.
    pub fn explicit(key: &str) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"explicit:");
        hasher.update(key.as_bytes());
        let hash = hasher.finalize();
        Self(u128::from_le_bytes(hash.as_bytes()[0..16].try_into().unwrap()))
    }

    /// Create a derived NodeId from a parent ID and a path index.
    /// This implements the structural identity requirement.
    pub fn derived(parent: u128, path: &[u32]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"derived:");
        hasher.update(&parent.to_le_bytes());
        for index in path {
            hasher.update(&index.to_le_bytes());
        }
        let hash = hasher.finalize();
        Self(u128::from_le_bytes(hash.as_bytes()[0..16].try_into().unwrap()))
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
