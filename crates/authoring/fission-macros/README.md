# fission-macros

Procedural macros for the Fission UI framework.

## `#[derive(Action)]`

Generates the `Action` trait implementation for a struct, enabling it to be used as a dispatchable action in the Fission runtime.

### What it generates

For a struct `MyAction`, the macro produces:

1. A `lazy_static` constant `MYACTION_ACTION_ID` of type `ActionId`, computed by hashing the fully qualified type path (`module_path!() + "::" + type_name`).
2. An `impl Action for MyAction` block with a `static_id()` method that returns the deterministic `ActionId`.

### Generated code (conceptual)

```rust
#[derive(Action)]
struct IncrementCounter {
    amount: i32,
}

// Expands to (approximately):
lazy_static::lazy_static! {
    pub static ref INCREMENTCOUNTER_ACTION_ID: ActionId =
        ActionId::from_name(concat!(module_path!(), "::", "IncrementCounter"));
}

impl Action for IncrementCounter {
    fn static_id() -> ActionId {
        *INCREMENTCOUNTER_ACTION_ID
    }
}
```

### Requirements

- The struct must be serializable (typically via `#[derive(Serialize, Deserialize)]`) since actions are wrapped in `ActionEnvelope` for dispatch.
- The `lazy_static` crate must be available in the consumer's dependency tree.
- `fission_core::action::ActionId` and `fission_core::action::Action` must be in scope.

### Usage

```rust
use fission_macros::Action;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize, Action)]
struct ToggleMenu {
    menu_id: String,
}

// Register a reducer for this action
registry.register::<ToggleMenu, _>(|state, envelope, _node_id| {
    let action: ToggleMenu = envelope.decode()?;
    state.menu_open = !state.menu_open;
    Ok(())
});
```

## `#[derive(Widget)]`

Reserved derive macro for future widget code generation. Currently a no-op that produces an empty token stream.
