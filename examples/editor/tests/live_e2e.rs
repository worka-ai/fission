/// Live E2E test for the Fission Editor.
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

#[test]
#[ignore]
fn editor_initial_render() {
    let mut child = launch_editor();
    let client = LiveTestClient::connect(CONTROL_PORT);
    client.wait_for_ready(20_000).expect("editor did not start");
    client.wait(2000).expect("wait");

    let dir = std::env::var("FISSION_SCREENSHOT_DIR")
        .unwrap_or_else(|_| "test_screenshots/editor_e2e".into());
    std::fs::create_dir_all(&dir).ok();

    // Initial screenshot
    client.screenshot(&format!("{}/01_initial.png", dir)).expect("screenshot");

    // Verify key UI elements
    let texts = client.get_text().expect("get_text");
    let all: Vec<&str> = texts.iter().map(|t| t.text.as_str()).collect();
    println!("Found {} text items", texts.len());

    let has = |needle: &str| all.iter().any(|t| t.contains(needle));
    assert!(has("EXPLORER"), "Explorer header missing");
    assert!(has("TERMINAL"), "Terminal panel missing");
    assert!(has("main"), "Git branch missing from status bar");
    assert!(has("Fission Editor"), "Welcome text missing");

    // Get tree
    let tree = client.get_tree().expect("get_tree");
    println!("Semantic tree: {} nodes", tree.len());

    // Try clicking a .rs file in the tree
    let rs_file = texts.iter().find(|t| t.text.ends_with(".rs"));
    if let Some(file) = rs_file {
        println!("Clicking file: {}", file.text);
        client.tap(file.x + file.width / 2.0, file.y + file.height / 2.0).expect("tap");
        client.pump().expect("pump");
        client.wait(500).expect("wait");
        client.screenshot(&format!("{}/02_file_open.png", dir)).expect("screenshot");

        // Verify tab appeared
        let texts_after = client.get_text().expect("get_text");
        let has_tab = texts_after.iter().any(|t| t.text.contains(&file.text));
        assert!(has_tab, "Tab for opened file should appear");
    }

    client.quit().expect("quit");
    let _ = child.wait();
    println!("Screenshots saved to {}/", dir);
}
