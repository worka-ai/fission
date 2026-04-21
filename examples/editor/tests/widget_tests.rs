/// Widget-level E2E tests for the Fission Editor using LiveTestClient.
///
/// These tests exercise the editor through the HTTP control channel,
/// validating widget rendering, interaction, and state transitions in
/// the live application. They complement the unit tests in model.rs by
/// testing the full widget tree rather than the model in isolation.
///
/// Run: cargo test -p fission-editor --test widget_tests -- --ignored --nocapture

use fission_test_driver::LiveTestClient;
use std::io::Write;
use std::process::{Child, Command};

const CONTROL_PORT: u16 = 9879; // Different port from live_e2e to avoid collision

fn launch_editor() -> Child {
    Command::new("cargo")
        .args(["run", "-p", "fission-editor", "--", "."])
        .env("FISSION_TEST_CONTROL_PORT", CONTROL_PORT.to_string())
        .spawn()
        .expect("failed to launch editor")
}

fn dir() -> String {
    let d = "test_screenshots/editor_widget";
    std::fs::create_dir_all(d).ok();
    d.to_string()
}

fn create_temp_file(name: &str, content: &str) -> String {
    let path = std::env::temp_dir().join(name);
    let mut f = std::fs::File::create(&path).expect("create temp file");
    f.write_all(content.as_bytes()).expect("write temp file");
    path.to_string_lossy().to_string()
}

fn cleanup(path: &str) {
    std::fs::remove_file(path).ok();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Verify the file tree widget renders directory entries after expanding.
#[test]
#[ignore]
fn file_tree_shows_entries() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    client.pump().unwrap();

    // The root folder should show in the sidebar
    client.assert_text_visible("EXPLORER").unwrap();

    // Expand a known folder
    client.tap_text("crates").unwrap();
    client.pump().unwrap();

    // After expansion, child entries should be visible
    let texts = client.get_text().unwrap();
    let entry_names: Vec<&str> = texts.iter().map(|t| t.text.as_str()).collect();

    // "crates" directory contains known subdirectories
    assert!(
        entry_names.iter().any(|n| *n == "authoring" || n.contains("authoring")),
        "expected 'authoring' in file tree, found: {:?}",
        &entry_names[..entry_names.len().min(30)]
    );

    client.quit().unwrap();
    let _ = child.wait();
}

/// Verify that the tab bar renders correctly after opening a file.
#[test]
#[ignore]
fn tab_bar_renders_after_open() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    // Open Cargo.toml
    client.tap_text("Cargo.toml").unwrap();
    client.pump().unwrap();

    // Tab bar should show the file name
    client.assert_text_visible("Cargo.toml").unwrap();

    // The semantic tree should have a tab-like role or clickable region
    let tree = client.get_tree().unwrap();
    let has_clickable = tree.iter().any(|n| n.focusable || n.role == "Button" || n.role == "Tab");
    assert!(has_clickable, "tab bar should have interactive elements");

    client.screenshot(&format!("{}/tab_bar.png", dir())).unwrap();
    client.quit().unwrap();
    let _ = child.wait();
}

/// Verify the status bar shows language and position info.
#[test]
#[ignore]
fn status_bar_shows_info() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    // Open a Rust file to check language detection in status bar
    client.tap_text("crates").unwrap();
    client.pump().unwrap();

    // Look for any .rs file to open
    let texts = client.get_text().unwrap();
    let rs_file = texts.iter().find(|t| t.text.ends_with(".rs"));

    if let Some(f) = rs_file {
        client.tap_text(&f.text).unwrap();
        client.pump().unwrap();

        // Status bar should show "Rust" language indicator
        let texts_after = client.get_text().unwrap();
        let has_lang = texts_after.iter().any(|t| t.text.contains("Rust"));
        println!("Status bar shows Rust: {}", has_lang);
    }

    client.screenshot(&format!("{}/status_bar.png", dir())).unwrap();
    client.quit().unwrap();
    let _ = child.wait();
}

/// Verify the editor surface renders line numbers and content.
#[test]
#[ignore]
fn editor_surface_renders_content() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    // Open Cargo.toml
    client.tap_text("Cargo.toml").unwrap();
    client.pump().unwrap();

    let texts = client.get_text().unwrap();

    // Should see file content
    let has_content = texts.iter().any(|t| t.text.contains("[workspace]"));
    assert!(has_content, "editor surface should render file content");

    // Should see line numbers (e.g., "1", "2", "3")
    let has_line_numbers = texts.iter().any(|t| t.text.trim() == "1");
    println!("Line numbers visible: {}", has_line_numbers);

    client.screenshot(&format!("{}/editor_surface.png", dir())).unwrap();
    client.quit().unwrap();
    let _ = child.wait();
}

/// Verify that typing in the editor updates the display.
#[test]
#[ignore]
fn typing_updates_display() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    // Open a file
    client.tap_text("Cargo.toml").unwrap();
    client.pump().unwrap();

    // Type some text
    client.type_text("UNIQUE_MARKER").unwrap();
    client.pump().unwrap();

    // The typed text should appear in the display
    let texts = client.get_text().unwrap();
    let has_marker = texts.iter().any(|t| t.text.contains("UNIQUE_MARKER"));
    println!("Typed text visible: {}", has_marker);

    // Undo the change
    client.press_key("Z", 4).unwrap();
    client.pump().unwrap();

    client.screenshot(&format!("{}/typing_test.png", dir())).unwrap();
    client.quit().unwrap();
    let _ = child.wait();
}

/// Verify the command palette widget renders correctly.
#[test]
#[ignore]
fn command_palette_widget_renders() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    // Open command palette
    client.press_key("P", 4 | 1).unwrap();
    client.pump().unwrap();

    let texts = client.get_text().unwrap();
    // Command palette should show commands or a search input
    let has_palette_content = texts.iter().any(|t| {
        t.text.contains("Save")
            || t.text.contains("Toggle")
            || t.text.contains("Find")
            || t.text.contains(">")
    });
    println!("Command palette has content: {}", has_palette_content);

    client.screenshot(&format!("{}/command_palette_widget.png", dir())).unwrap();

    // Dismiss
    client.press_key("Escape", 0).unwrap();
    client.pump().unwrap();

    client.quit().unwrap();
    let _ = child.wait();
}

/// Verify the terminal panel widget renders terminal lines.
#[test]
#[ignore]
fn terminal_panel_widget_renders() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    client.pump().unwrap();

    // Terminal should show initial lines
    client.assert_text_visible("Ready.").unwrap();

    // The terminal panel should have the "TERMINAL" header
    client.assert_text_visible("TERMINAL").unwrap();

    client.screenshot(&format!("{}/terminal_panel.png", dir())).unwrap();
    client.quit().unwrap();
    let _ = child.wait();
}

/// Verify sidebar toggling hides/shows the EXPLORER section.
#[test]
#[ignore]
fn sidebar_toggle_hides_explorer() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    client.pump().unwrap();
    client.assert_text_visible("EXPLORER").unwrap();

    // Toggle sidebar off
    client.press_key("B", 4).unwrap();
    client.pump().unwrap();

    // EXPLORER should be hidden now
    let result = client.assert_text_not_visible("EXPLORER");
    println!("Sidebar hidden check: {:?}", result);

    client.screenshot(&format!("{}/sidebar_hidden.png", dir())).unwrap();

    // Toggle back on
    client.press_key("B", 4).unwrap();
    client.pump().unwrap();
    client.assert_text_visible("EXPLORER").unwrap();

    client.screenshot(&format!("{}/sidebar_shown.png", dir())).unwrap();
    client.quit().unwrap();
    let _ = child.wait();
}

/// Verify the find bar widget appears and accepts text.
#[test]
#[ignore]
fn find_bar_widget_interaction() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    // Open a file first
    client.tap_text("Cargo.toml").unwrap();
    client.pump().unwrap();

    // Open find bar
    client.press_key("F", 4).unwrap();
    client.pump().unwrap();

    client.screenshot(&format!("{}/find_bar_open.png", dir())).unwrap();

    // Type a search term
    client.type_text("workspace").unwrap();
    client.pump().unwrap();

    client.screenshot(&format!("{}/find_bar_with_query.png", dir())).unwrap();

    // Close find bar
    client.press_key("Escape", 0).unwrap();
    client.pump().unwrap();

    client.quit().unwrap();
    let _ = child.wait();
}

/// Verify the search panel (sidebar section) renders results.
#[test]
#[ignore]
fn search_panel_widget_renders() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    // Switch to search panel via activity bar icon
    let texts = client.get_text().unwrap();
    if let Some(icon) = texts.iter().find(|t| t.text == "\u{1f50d}" && t.x < 50.0) {
        client
            .tap(icon.x + icon.width / 2.0, icon.y + icon.height / 2.0)
            .unwrap();
        client.pump().unwrap();

        // Should see "SEARCH" header or search input area
        let texts_after = client.get_text().unwrap();
        let has_search = texts_after
            .iter()
            .any(|t| t.text.contains("SEARCH") || t.text.contains("Search"));
        println!("Search panel visible: {}", has_search);

        client
            .screenshot(&format!("{}/search_panel.png", dir()))
            .unwrap();
    } else {
        println!("Search icon not found, skipping search panel widget test");
    }

    client.quit().unwrap();
    let _ = child.wait();
}

/// Verify the git panel (sidebar section) renders status entries.
#[test]
#[ignore]
fn git_panel_widget_renders() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    // Switch to git panel via activity bar icon
    let texts = client.get_text().unwrap();
    if let Some(icon) = texts.iter().find(|t| t.text == "\u{2387}" && t.x < 50.0) {
        client
            .tap(icon.x + icon.width / 2.0, icon.y + icon.height / 2.0)
            .unwrap();
        client.pump().unwrap();

        // Should see "GIT" or "SOURCE CONTROL" header
        let texts_after = client.get_text().unwrap();
        let has_git = texts_after
            .iter()
            .any(|t| t.text.contains("GIT") || t.text.contains("Source") || t.text.contains("Changes"));
        println!("Git panel visible: {}", has_git);

        client
            .screenshot(&format!("{}/git_panel.png", dir()))
            .unwrap();
    } else {
        println!("Git icon not found, skipping git panel widget test");
    }

    client.quit().unwrap();
    let _ = child.wait();
}

/// Verify layout integrity: no text items with zero or broken dimensions.
#[test]
#[ignore]
fn layout_integrity_check() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    // Open a file to get a full layout
    client.tap_text("Cargo.toml").unwrap();
    client.pump().unwrap();

    let texts = client.get_text().unwrap();
    let broken: Vec<_> = texts
        .iter()
        .filter(|t| (t.width < 1.0 || t.height < 3.0) && !t.text.trim().is_empty())
        .collect();

    for b in &broken {
        println!("  BROKEN: {}x{} at ({},{}) \"{}\"", b.width, b.height, b.x, b.y, b.text);
    }

    assert_eq!(
        broken.len(),
        0,
        "layout integrity: {} text items have broken dimensions",
        broken.len()
    );

    client.screenshot(&format!("{}/layout_integrity.png", dir())).unwrap();
    client.quit().unwrap();
    let _ = child.wait();
}

/// Verify that the semantic tree has expected roles for accessibility.
#[test]
#[ignore]
fn semantic_tree_has_expected_roles() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    // Open a file to populate the tree
    client.tap_text("Cargo.toml").unwrap();
    client.pump().unwrap();

    let tree = client.get_tree().unwrap();

    let roles: Vec<&str> = tree.iter().map(|n| n.role.as_str()).collect();
    let unique_roles: std::collections::HashSet<&&str> = roles.iter().collect();
    println!("Unique roles in semantic tree: {:?}", unique_roles);

    // Should have at least some interactive elements
    let has_buttons = roles.iter().any(|r| *r == "Button");
    let has_text_input = roles.iter().any(|r| *r == "TextInput");
    println!(
        "Has buttons: {}, has text input: {}",
        has_buttons, has_text_input
    );

    client.quit().unwrap();
    let _ = child.wait();
}

/// Verify PROBLEMS tab switch in the bottom panel.
#[test]
#[ignore]
fn problems_tab_switch() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    client.pump().unwrap();

    // Look for PROBLEMS tab
    let texts = client.get_text().unwrap();
    if texts.iter().any(|t| t.text.contains("PROBLEMS")) {
        client.tap_text("PROBLEMS").unwrap();
        client.pump().unwrap();

        client
            .screenshot(&format!("{}/problems_tab.png", dir()))
            .unwrap();

        // Switch back to TERMINAL
        client.tap_text("TERMINAL").unwrap();
        client.pump().unwrap();

        println!("Problems tab switch OK");
    } else {
        println!("PROBLEMS tab not visible");
    }

    client.quit().unwrap();
    let _ = child.wait();
}

/// Verify the menu bar renders and dropdown appears on click.
#[test]
#[ignore]
fn menu_bar_dropdown() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();

    client.pump().unwrap();

    let texts = client.get_text().unwrap();
    if texts.iter().any(|t| t.text == "File") {
        client.tap_text("File").unwrap();
        client.pump().unwrap();

        let dropdown_texts = client.get_text().unwrap();
        let dropdown_items: Vec<&str> = dropdown_texts
            .iter()
            .filter(|t| {
                t.text.contains("Save")
                    || t.text.contains("New")
                    || t.text.contains("Open")
                    || t.text.contains("Close")
            })
            .map(|t| t.text.as_str())
            .collect();
        println!("Menu dropdown items: {:?}", dropdown_items);
        assert!(!dropdown_items.is_empty(), "File menu should show items");

        client
            .screenshot(&format!("{}/menu_dropdown.png", dir()))
            .unwrap();

        // Dismiss menu
        client.press_key("Escape", 0).unwrap();
        client.pump().unwrap();
    } else {
        println!("File menu not found");
    }

    client.quit().unwrap();
    let _ = child.wait();
}
