use downcast_rs::{Downcast, impl_downcast};
use serde::{Deserialize, Serialize};
use std::any::Any;
use blake3;

// ActionId is a stable, globally unique identifier for an Action type.
// It's conceptually similar to NodeId but for actions.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ActionId(u128);

impl ActionId {
    /// Creates an ActionId from a string, e.g., for built-in or test actions.
    pub fn from_name(name: &str) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(name.as_bytes());
        let hash = hasher.finalize();
        ActionId(u128::from_le_bytes(hash.as_bytes()[0..16].try_into().unwrap()))
    }
}

pub trait Action: Any + Send + Sync + std::fmt::Debug + Downcast {
    fn id(&self) -> ActionId;
    // Actions are expected to be serializable, but `dyn Trait` cannot be directly serialized.
    // This will be handled by the `#[derive(Action)]` macro eventually,
    // or by custom `Runtime` serialization logic.
}

// This macro implements Downcast for `dyn Action` and also adds `downcast_ref` etc. methods.
impl_downcast!(Action);

// Trait for application state that can be managed by the Runtime.
// This allows a single Runtime to manage multiple, distinct state types.
pub trait AppState: Any + Send + Sync + std::fmt::Debug + Downcast {
    // This trait intentionally doesn't define `as_any` or `as_any_mut`.
    // The `impl_downcast!(AppState);` macro generates these for `dyn AppState`.
}

// Same for AppState.
impl_downcast!(AppState);
