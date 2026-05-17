# fission-macros

Procedural macros for the Fission UI framework.

## `#[fission_action]`

Injects the standard Fission action derives in one attribute:

```rust
use fission_macros::fission_action;

#[fission_action]
struct Increment;
```

This expands to the standard Fission action implementation plus the common serialization, debug, clone, and equality derives. The generated `Action` implementation computes the deterministic action ID from `module_path!()` and the Rust type name, then caches it with `std::sync::OnceLock`.

For payloads that cannot implement `Eq`, use:

```rust
#[fission_action(no_eq)]
struct SetScale(f32);
```

## `#[derive(Action)]`

The derive macro remains available for lower-level crates that want to choose their own serialization and comparison derives explicitly. Application code should normally prefer `#[fission_action]`.

## `#[derive(Widget)]`

Reserved derive macro for future widget code generation. Currently a no-op that produces an empty token stream.
