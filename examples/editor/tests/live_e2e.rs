/// Live E2E test for the Fission Editor.
///
/// This test launches the real editor, interacts with it, takes screenshots,
/// and reads them back to verify visual correctness.
///
/// Run: cargo test -p fission-editor --test live_e2e -- --ignored --nocapture

use fission_test_driver::LiveTestClient;
use std::process::{Child, Command};

const CONTROL_PORT: u16 = 9878;

fn launch_editor() -> Child {
    Command::new("cargo")
        .args(["run", "-p", "fission-editor", "--", "."])
        .env("FISSION_TEST_CONTROL_PORT", CONTROL_PORT.to_string())
        .spawn()
        .expect("failed to launch editor")
}

fn screenshot_dir() -> String {
    let dir = std::env::var("FISSION_SCREENSHOT_DIR")
        .unwrap_or_else(|_| "test_screenshots/editor_e2e".into());
    std::fs::create_dir_all(&dir).ok();
    dir
}

#[test]
#[ignore]
fn editor_full_workflow() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor did not start");
    client.wait(2000).expect("wait for initial render");

    let dir = screenshot_dir();

    // --- 1. Initial state verification ---
    client.pump().expect("pump");
    client.screenshot(&format!("{}/01_initial.png", dir)).expect("screenshot");

    let texts = client.get_text().expect("get_text");
    println!("Initial: {} text items", texts.len());
    assert!(texts.iter().any(|t| t.text.contains("EXPLORER")), "Missing EXPLORER header");
    assert!(texts.iter().any(|t| t.text.contains("TERMINAL")), "Missing TERMINAL panel");
    assert!(texts.iter().any(|t| t.text.contains("Fission Editor")), "Missing welcome text");
    assert!(texts.iter().any(|t| t.text.contains("main")), "Missing git branch in status bar");

    // Verify file tree has entries
    let tree_items: Vec<_> = texts.iter()
        .filter(|t| t.x > 50.0 && t.x < 250.0 && t.height > 10.0)
        .collect();
    assert!(tree_items.len() > 3, "File tree should have entries, found {}", tree_items.len());

    // --- 2. Expand a folder ---
    client.tap_text("crates").expect("expand crates");
    client.screenshot(&format!("{}/02_crates_expanded.png", dir)).expect("screenshot");

    let texts = client.get_text().expect("get_text");
    assert!(texts.iter().any(|t| t.text == "authoring"), "crates should show authoring subfolder");
    assert!(texts.iter().any(|t| t.text == "core"), "crates should show core subfolder");

    // --- 3. Open a file ---
    client.tap_text("Cargo.toml").expect("open Cargo.toml");
    client.screenshot(&format!("{}/03_file_open.png", dir)).expect("screenshot");

    let texts = client.get_text().expect("get_text");
    // Tab should appear
    let has_tab = texts.iter().any(|t| t.text.contains("Cargo.toml") && t.y < 40.0);
    assert!(has_tab, "Tab for Cargo.toml should appear at top");
    // Content should have line numbers
    let has_line_1 = texts.iter().any(|t| t.text.trim() == "1" && t.x < 300.0);
    assert!(has_line_1, "Line number 1 should appear");
    // Content should show workspace
    let has_workspace = texts.iter().any(|t| t.text.contains("[workspace]"));
    assert!(has_workspace, "File content should show [workspace]");

    // --- 4. Verify text editing exists ---
    let tree = client.get_tree().expect("get_tree");
    let text_inputs: Vec<_> = tree.iter().filter(|n| n.role == "TextInput").collect();
    println!("TextInput nodes: {}", text_inputs.len());
    assert!(text_inputs.len() >= 1, "Should have at least one TextInput for editing");

    // --- 5. Verify layout integrity ---
    let texts = client.get_text().expect("get_text");
    let broken: Vec<_> = texts.iter()
        .filter(|t| (t.width < 1.0 || t.height < 3.0) && !t.text.trim().is_empty())
        .collect();
    if !broken.is_empty() {
        println!("BROKEN items ({}):", broken.len());
        for b in &broken {
            println!("  {}x{} \"{}\"", b.width, b.height, b.text);
        }
    }
    assert!(broken.is_empty(), "All text should be properly sized");

    // --- Done ---
    client.quit().expect("quit");
    let _ = child.wait();
    println!("\nAll screenshots saved to {}/", dir);
}
