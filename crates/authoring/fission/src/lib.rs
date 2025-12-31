// Facade crate: re-export core + widget APIs for consumers.

pub mod core {
    pub use fission_core::*;
}

pub mod widgets {
    pub use fission_widgets::*;
}

// Common widget layer (includes core widgets re-exported by fission-widgets).
pub use fission_widgets::*;

// Core runtime/action APIs (avoid name collisions with widget exports).
pub use fission_core::{
    action::{Action, ActionEnvelope, ActionId, AppState},
    context::{Effects, ReducerContext},
    effect::{ActionInput, Effect, EffectEnvelope, EffectPayload, SystemEffect},
    env::{Clipboard, Env, InteractionStateMap, ImeHandler, RuntimeState, ScrollStateMap},
    event::{InputEvent, KeyCode, KeyEvent, LifecycleEvent, PointerButton, PointerEvent},
    op,
    LayoutEngine, LayoutOp, LayoutPoint, LayoutRect, LayoutSize, LayoutUnit, TextMeasurer,
    NodeId, Runtime, WidgetNodeId,
};

// Prelude for UI authoring: widgets + common types (not the entire API surface).
pub mod prelude {
    pub use fission_widgets::*;
    pub use fission_core::action::{Action, ActionEnvelope, ActionId, AppState};
    pub use fission_core::context::{Effects, ReducerContext};
}
