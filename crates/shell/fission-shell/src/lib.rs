use anyhow::Result;
use fission_ir::NodeId;
use serde::{Deserialize, Serialize};

pub use fission_render::{LayoutPoint, LayoutRect, LayoutSize}; 
pub use fission_core::{InputEvent, PointerEvent, PointerButton, KeyEvent, KeyCode, LifecycleEvent};

// The Platform trait, implemented by concrete platform shells (desktop, mobile, web).
pub trait Platform {
    // Dispatches an input event to the Core Runtime. The Platform is responsible for
    // normalizing raw OS input into `InputEvent`.
    fn dispatch_event(&mut self, event: InputEvent) -> Result<()>;

    // A placeholder for getting the render surface, or rendering commands.
    // Actual rendering would happen via `fission-render` traits.
    fn present(&mut self, display_list_data: &[u8]) -> Result<()>; 
}
