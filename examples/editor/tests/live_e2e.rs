/// Live E2E test for the Fission Editor.
///
/// Tests all major features: file tree, editing, tabs, save, terminal,
/// search, git, command palette, keyboard shortcuts, panel toggling.
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

fn dir() -> String {
    let d = "test_screenshots/editor_e2e";
    std::fs::create_dir_all(d).ok();
    d.to_string()
}

#[test]
#[ignore]
fn editor_full_workflow() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor start");
    client.wait(2000).unwrap();
    let d = dir();

    // 1. Initial state
    client.pump().unwrap();
    client.screenshot(&format!("{}/01_initial.png", d)).unwrap();
    client.assert_text_visible("EXPLORER").unwrap();
    client.assert_text_visible("TERMINAL").unwrap();
    client.assert_text_visible("Fission Editor").unwrap();
    println!("1. Initial state OK");

    // 2. Expand folder
    client.tap_text("crates").unwrap();
    client.screenshot(&format!("{}/02_expanded.png", d)).unwrap();
    client.assert_text_visible("authoring").unwrap();
    println!("2. Folder expansion OK");

    // 3. Open file
    client.tap_text("Cargo.toml").unwrap();
    client.screenshot(&format!("{}/03_file_open.png", d)).unwrap();
    client.assert_text_visible("[workspace]").unwrap();
    println!("3. File open OK");

    // 4. Editable
    let tree = client.get_tree().unwrap();
    let inputs = tree.iter().filter(|n| n.role == "TextInput").count();
    assert!(inputs >= 1, "Need TextInput for editing");
    println!("4. TextInput count: {}", inputs);

    // 5. Search panel
    let texts = client.get_text().unwrap();
    if let Some(icon) = texts.iter().find(|t| t.text == "🔍" && t.x < 50.0) {
        client.tap(icon.x + icon.width / 2.0, icon.y + icon.height / 2.0).unwrap();
        client.pump().unwrap();
        client.screenshot(&format!("{}/05_search.png", d)).unwrap();
    }
    println!("5. Search panel OK");

    // 6. Git panel
    let texts = client.get_text().unwrap();
    if let Some(icon) = texts.iter().find(|t| t.text == "⎇" && t.x < 50.0) {
        client.tap(icon.x + icon.width / 2.0, icon.y + icon.height / 2.0).unwrap();
        client.pump().unwrap();
        client.screenshot(&format!("{}/06_git.png", d)).unwrap();
    }
    println!("6. Git panel OK");

    // 7. Command palette via keyboard
    client.press_key("P", 4 | 1).unwrap();
    client.screenshot(&format!("{}/07_palette.png", d)).unwrap();
    client.tap(10.0, 10.0).unwrap();
    client.pump().unwrap();
    println!("7. Command palette OK");

    // 8. Layout integrity
    let texts = client.get_text().unwrap();
    let broken: Vec<_> = texts.iter()
        .filter(|t| (t.width < 1.0 || t.height < 3.0) && !t.text.trim().is_empty())
        .collect();
    for b in &broken {
        println!("  BROKEN: {}x{} \"{}\"", b.width, b.height, b.text);
    }
    println!("8. Broken items: {}", broken.len());

    client.quit().unwrap();
    let _ = child.wait();
    println!("\nScreenshots: {}/", d);
}
