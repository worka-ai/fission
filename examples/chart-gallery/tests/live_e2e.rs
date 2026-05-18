use fission_test_driver::LiveTestClient;
use std::net::TcpListener;
use std::process::{Child, Command};

fn reserve_control_port() -> u16 {
    TcpListener::bind(("127.0.0.1", 0))
        .expect("bind ephemeral test port")
        .local_addr()
        .expect("read ephemeral test port")
        .port()
}

fn launch_chart_gallery(control_port: u16) -> Child {
    let bin = std::env::var("CARGO_BIN_EXE_chart-gallery")
        .or_else(|_| std::env::var("CARGO_BIN_EXE_chart_gallery"))
        .unwrap_or_else(|_| "target/debug/chart-gallery".to_string());
    Command::new(bin)
        .env("FISSION_TEST_CONTROL_PORT", control_port.to_string())
        .env("FISSION_BACKGROUND_TEST", "1")
        .spawn()
        .expect("failed to launch chart-gallery")
}

fn launch_chart_gallery_doc_capture(control_port: u16, slug: &str) -> Child {
    let bin = std::env::var("CARGO_BIN_EXE_chart-gallery")
        .or_else(|_| std::env::var("CARGO_BIN_EXE_chart_gallery"))
        .unwrap_or_else(|_| "target/debug/chart-gallery".to_string());
    Command::new(bin)
        .env("FISSION_TEST_CONTROL_PORT", control_port.to_string())
        .env("FISSION_BACKGROUND_TEST", "1")
        .env("FISSION_CHART_DOC_SLUG", slug)
        .spawn()
        .expect("failed to launch chart-gallery doc capture")
}

fn screenshot_dir() -> String {
    let dir = std::env::var("FISSION_SCREENSHOT_DIR").unwrap_or_else(|_| {
        format!(
            "{}/../../.artifacts/screenshots/examples/chart-gallery/chart_gallery_live",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    std::fs::create_dir_all(&dir).ok();
    dir
}

fn chart_doc_slugs() -> Vec<String> {
    let catalog_path = format!(
        "{}/../../website/src/data/chartCatalog.ts",
        env!("CARGO_MANIFEST_DIR")
    );
    let catalog = std::fs::read_to_string(&catalog_path)
        .unwrap_or_else(|err| panic!("read chart catalog at {catalog_path}: {err}"));
    let mut slugs = Vec::new();
    let mut rest = catalog.as_str();
    let needle = "\"slug\": \"";
    while let Some(index) = rest.find(needle) {
        let after = &rest[index + needle.len()..];
        let end = after
            .find('"')
            .unwrap_or_else(|| panic!("unterminated slug in chart catalog near {after:?}"));
        slugs.push(after[..end].to_string());
        rest = &after[end + 1..];
    }
    assert!(!slugs.is_empty(), "chart catalog did not contain any slugs");
    slugs
}

#[test]
#[ignore]
fn sidebar_scroll_reaches_lower_entries() {
    let control_port = reserve_control_port();
    let mut child = launch_chart_gallery(control_port);
    let client = LiveTestClient::connect(control_port);
    client
        .wait_for_ready(20_000)
        .expect("chart-gallery did not start");
    client.wait(1_500).expect("initial wait");

    let dir = screenshot_dir();
    let before = format!("{}/01_sidebar_before_scroll.png", dir);
    let after = format!("{}/02_sidebar_after_scroll.png", dir);
    client.screenshot(&before).expect("before screenshot");

    for _ in 0..4 {
        client
            .scroll(120.0, 420.0, 0.0, 180.0)
            .expect("sidebar scroll");
        client.pump().expect("pump after scroll");
        client.wait(200).expect("wait after scroll");
    }
    client.screenshot(&after).expect("after screenshot");

    client
        .assert_text_visible("Liquidfill")
        .expect("scrolling the sidebar should reveal lower navigation entries");

    client.quit().expect("quit");
    let _ = child.wait();
}

#[test]
#[ignore]
fn generate_real_chart_doc_screenshots() {
    if std::env::var("FISSION_UPDATE_CHART_DOCS").ok().as_deref() != Some("1") {
        eprintln!("set FISSION_UPDATE_CHART_DOCS=1 to refresh website chart screenshots");
        return;
    }

    let output_dir = format!(
        "{}/../../website/static/img/charts",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::create_dir_all(&output_dir).expect("create website chart screenshot dir");

    for slug in chart_doc_slugs() {
        let control_port = reserve_control_port();
        let mut child = launch_chart_gallery_doc_capture(control_port, &slug);
        let client = LiveTestClient::connect(control_port);
        client
            .wait_for_ready(20_000)
            .unwrap_or_else(|err| panic!("{slug} did not start: {err}"));
        client.wait(500).expect("initial doc capture wait");
        client
            .screenshot(&format!("{output_dir}/{slug}.png"))
            .unwrap_or_else(|err| panic!("capture {slug}: {err}"));
        client.quit().expect("quit doc capture");
        let _ = child.wait();
    }
}
