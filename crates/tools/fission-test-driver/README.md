# fission-test-driver

Automated UI testing client and protocol for Fission applications.

This crate provides both the JSON protocol types (shared between the test client and the desktop shell server) and a `LiveTestClient` that drives a running Fission application over HTTP.

## Architecture

```
Test process                        Application process
+-----------------+                 +-------------------------+
| LiveTestClient  | ---HTTP/JSON--> | test_control server     |
|   .tap(x, y)    |                 |   (fission-shell-desktop)|
|   .type_text()  |                 |   dispatches to Runtime |
|   .screenshot() | <--HTTP/JSON--- |   returns TestResponse  |
+-----------------+                 +-------------------------+
```

The application must be launched with `FISSION_TEST_CONTROL_PORT=<port>` to enable the test control server. The `LiveTestClient` connects to `http://127.0.0.1:<port>` and sends `TestCommand` JSON payloads to `/cmd`.

## Protocol types

### `TestCommand`

All commands are serialized with `#[serde(tag = "cmd")]`:

| Command | Fields | Description |
|---------|--------|-------------|
| `Tap` | `x: f32, y: f32` | Simulate a pointer down + up at the given coordinates. |
| `TapText` | `text: String` | Find visible text matching the string and tap its center. |
| `Scroll` | `x, y, dx, dy: f32` | Simulate a scroll event at (x, y) with delta (dx, dy). |
| `TypeText` | `text: String` | Type each character as a keyboard event into the focused input. |
| `PressKey` | `key: String, modifiers: u8` | Press a named key (e.g., `"Enter"`, `"Escape"`, `"Tab"`, `"a"`) with modifier flags. |
| `Screenshot` | `path: String` | Capture the current frame to a PNG file at the given path. |
| `GetText` | (none) | Return all visible text items with their bounding rects. |
| `GetTree` | (none) | Return the semantic accessibility tree. |
| `Wait` | `ms: u64` | Sleep for the given duration (server-side). |
| `Pump` | (none) | Force a frame render and wait for it to complete. |
| `Quit` | (none) | Exit the application. |

### `TestResponse`

| Variant | Fields | Description |
|---------|--------|-------------|
| `Ok` | (none) | Command succeeded. |
| `Text` | `items: Vec<TextItem>` | Response to `GetText`. |
| `Tree` | `nodes: Vec<SemanticNode>` | Response to `GetTree`. |
| `Error` | `message: String` | Command failed with a reason. |

### `TextItem`

```rust
pub struct TextItem {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

### `SemanticNode`

```rust
pub struct SemanticNode {
    pub role: String,       // e.g., "Button", "TextInput", "Generic"
    pub label: Option<String>,
    pub value: Option<String>,
    pub focusable: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
```

## `LiveTestClient`

The client provides both low-level command methods and high-level convenience helpers.

### Connection

```rust
use fission_test_driver::LiveTestClient;

let client = LiveTestClient::connect(9876);
client.wait_for_ready(5000)?; // Wait up to 5s for the app to start
```

### Low-level methods

| Method | Description |
|--------|-------------|
| `tap(x, y)` | Tap at coordinates. |
| `tap_text(text)` | Find and tap text (pumps before and after). |
| `scroll(x, y, dx, dy)` | Scroll at coordinates. |
| `type_text(text)` | Type characters into the focused input. |
| `press_key(key, modifiers)` | Press a key with modifiers (pumps after). |
| `screenshot(path)` | Save a screenshot PNG. |
| `get_text()` | Get all visible text items. |
| `get_tree()` | Get the semantic tree. |
| `wait(ms)` | Server-side sleep. |
| `pump()` | Force a frame and wait for completion. |
| `quit()` | Exit the application. |

### High-level helpers

| Method | Description |
|--------|-------------|
| `tap_text_and_wait(text, ms)` | Tap text then wait. |
| `assert_text_visible(needle)` | Assert that text containing `needle` is on screen. |
| `assert_text_not_visible(needle)` | Assert that text containing `needle` is not on screen. |

## Usage example

```rust
use fission_test_driver::LiveTestClient;

#[test]
fn test_login_flow() {
    let client = LiveTestClient::connect(9876);
    client.wait_for_ready(10_000).unwrap();

    // Type into the email field
    client.tap_text("Email").unwrap();
    client.type_text("user@example.com").unwrap();
    client.pump().unwrap();

    // Click the login button
    client.tap_text("Log In").unwrap();

    // Verify navigation
    client.assert_text_visible("Dashboard").unwrap();
    client.assert_text_not_visible("Log In").unwrap();

    // Take a screenshot for visual regression
    client.screenshot("/tmp/dashboard.png").unwrap();

    client.quit().unwrap();
}
```

## Modifier flags

The `modifiers` parameter is a bitmask:

| Bit | Modifier |
|-----|----------|
| `0x01` | Shift |
| `0x02` | Alt/Option |
| `0x04` | Control |
| `0x08` | Super/Command |
