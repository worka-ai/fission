use downcast_rs::{Downcast, impl_downcast};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::any::Any;
use blake3;
use serde_json;

// ActionId is a stable, globally unique identifier for an Action type.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ActionId(u128);

impl ActionId {
    pub const fn from_u128(val: u128) -> Self {
        Self(val)
    }

    pub fn as_u128(&self) -> u128 {
        self.0
    }

    pub fn from_name(name: &str) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(name.as_bytes());
        let hash = hasher.finalize();
        ActionId(u128::from_le_bytes(hash.as_bytes()[0..16].try_into().unwrap()))
    }
}

// The Action trait for typed authoring.
// Must be Serializable/Deserializable to support the Envelope model.
pub trait Action: Serialize + DeserializeOwned + Any + Send + Sync + std::fmt::Debug {
    fn static_id() -> ActionId where Self: Sized;
    
    fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Action serialization failed")
    }
}

// The type-erased envelope stored in widgets and passed to reducers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionEnvelope {
    pub id: ActionId,
    // Payload is opaque bytes. serde_bytes could be used for optimization but Vec<u8> is fine for MVP.
    pub payload: Vec<u8>,
}

// Typed wrapper for ergonomic authoring.
// Users write: `on_press: Some(ActionRef(Increment { amount: 1 }).into())`
// Or simpler: `on_press: Some(Increment { amount: 1 }.into())` if we implement From<T> for ActionEnvelope?
// Implementing From<T> directly on ActionEnvelope for all T: Action is tricky due to orphan rules if T is local? 
// No, generic impls are allowed if the trait is local or type is local. ActionEnvelope is local.
// `impl<T: Action> From<T> for ActionEnvelope` works! 
// Then users just write `on_press: Some(Increment { ... }.into())`.
// The `ActionRef` wrapper suggested in the prompt is also good for explicit intent.
// Let's implement ActionRef as requested to be safe.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRef<T: Action>(pub T);

impl<T: Action> From<ActionRef<T>> for ActionEnvelope {
    fn from(action_ref: ActionRef<T>) -> Self {
        ActionEnvelope {
            id: T::static_id(),
            payload: action_ref.0.encode(),
        }
    }
}

// Also allow direct conversion for convenience if desired?
impl<T: Action> From<T> for ActionEnvelope {
    fn from(action: T) -> Self {
        ActionEnvelope {
            id: T::static_id(),
            payload: action.encode(),
        }
    }
}

// Trait for application state that can be managed by the Runtime.
pub trait AppState: Any + Send + Sync + std::fmt::Debug + Downcast {}

impl_downcast!(AppState);