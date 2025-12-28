use fission_layout::{LayoutPoint, LayoutSize};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PointerButton {
    Primary,   // Left mouse button, primary touch
    Secondary, // Right mouse button, secondary touch
    Middle,    // Middle mouse button
    Other(u8), // Other buttons
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PointerEvent {
    Down {
        point: LayoutPoint,
        button: PointerButton,
    },
    Up {
        point: LayoutPoint,
        button: PointerButton,
    },
    Move {
        point: LayoutPoint,
    },
    Scroll {
        point: LayoutPoint,
        delta: LayoutPoint,
    }, // Added
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyCode {
    Space,
    Enter,
    Escape,
    Backspace,
    Tab,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    Char(char),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyEvent {
    Down { key_code: KeyCode, modifiers: u8 },
    Up { key_code: KeyCode, modifiers: u8 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LifecycleEvent {
    Init,
    Resume,
    Pause,
    Terminate,
    Resize { size: LayoutSize },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GestureEvent {
    Tap { point: LayoutPoint },
    DoubleTap { point: LayoutPoint },
    PanStart { point: LayoutPoint },
    PanUpdate { point: LayoutPoint, delta: LayoutPoint },
    PanEnd { point: LayoutPoint },
    LongPress { point: LayoutPoint },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    Pointer(PointerEvent),
    Keyboard(KeyEvent),
    Ime(ImeEvent),
    Gesture(GestureEvent),
    Lifecycle(LifecycleEvent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImeEvent {
    Preedit { text: String },
    Commit { text: String },
}
