use super::*;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Deserialize, Default)]
struct ContentToml {
    release: Option<ReleaseContentRoot>,
}

#[derive(Debug, Deserialize, Default)]
struct ReleaseContentRoot {
    screenshots: Option<ScreenshotConfig>,
    assets: Option<ProviderAssets>,
}

#[derive(Debug, Deserialize, Default)]
struct ScreenshotConfig {
    raw_dir: Option<String>,
    rendered_dir: Option<String>,
    #[serde(default)]
    scenarios: Vec<ScreenshotScenario>,
}

#[derive(Debug, Deserialize, Default)]
struct ScreenshotScenario {
    id: Option<String>,
    name: Option<String>,
    #[serde(default)]
    targets: Vec<String>,
    script: Option<String>,
    wait_for: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ProviderAssets {
    app_store: Option<AppStoreAssets>,
    play_store: Option<PlayStoreAssets>,
    microsoft_store: Option<MicrosoftStoreAssets>,
}

#[derive(Debug, Deserialize, Default)]
struct AppStoreAssets {
    screenshot_sets_dir: Option<String>,
    app_previews_dir: Option<String>,
    #[serde(default)]
    review_attachments: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct PlayStoreAssets {
    screenshot_sets_dir: Option<String>,
    preview_video_dir: Option<String>,
    feature_graphic: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct MicrosoftStoreAssets {
    screenshot_sets_dir: Option<String>,
    trailers_dir: Option<String>,
    logo_dir: Option<String>,
}

#[derive(Debug, Serialize)]
struct RenderManifest {
    schema_version: u32,
    created_at_unix_seconds: u64,
    provider: String,
    source_dir: String,
    output_dir: String,
    assets: Vec<RenderedAsset>,
}

#[derive(Debug, Serialize)]
struct RenderedAsset {
    kind: String,
    source: String,
    output: String,
    size_bytes: u64,
}

pub(super) fn validate_release_content_model(
    project_dir: &Path,
    provider: Option<publish::DistributionProvider>,
) -> LifecycleReport {
    let mut report = base_report("release-content.validate", provider, None);
    report.checks.push(path_check(
        "release_content.root_exists",
        project_dir.join("release-content"),
        "release-content directory exists",
    ));
    let config = match load_content_config(project_dir) {
        Ok(config) => {
            report.checks.push(ok_check(
                "release_content.config_parses",
                "fission.toml release content config parses",
            ));
            config
        }
        Err(error) => {
            report.checks.push(failed_check(
                "release_content.config_parses",
                error.to_string(),
            ));
            finalize_status(&mut report);
            return report;
        }
    };
    validate_screenshots(project_dir, &config, provider, &mut report.checks);
    validate_provider_assets(project_dir, &config, provider, &mut report.checks);
    finalize_status(&mut report);
    report
}

pub(super) fn capture_release_content(
    project_dir: &Path,
    target: Target,
    set: &str,
) -> Result<LifecycleReport> {
    let config = load_content_config(project_dir)?;
    let mut report = base_report("release-content.capture", None, Some(target));
    let screenshots = config
        .release
        .as_ref()
        .and_then(|release| release.screenshots.as_ref())
        .context("release.screenshots must be configured before capture")?;
    let raw_dir = project_dir.join(
        screenshots
            .raw_dir
            .as_deref()
            .unwrap_or("release-content/screenshots/raw"),
    );
    fs::create_dir_all(&raw_dir)?;
    let scenarios = screenshots
        .scenarios
        .iter()
        .filter(|scenario| scenario.targets.iter().any(|item| item == target.as_str()))
        .collect::<Vec<_>>();
    if scenarios.is_empty() {
        report.checks.push(failed_check(
            "release_content.capture.scenarios_available",
            format!(
                "no screenshot scenarios target {} for set {set}",
                target.as_str()
            ),
        ));
        finalize_status(&mut report);
        return Ok(report);
    }
    for scenario in scenarios {
        capture_scenario(
            project_dir,
            &raw_dir,
            target,
            set,
            scenario,
            &mut report.checks,
        )?;
    }
    finalize_status(&mut report);
    Ok(report)
}

pub(super) fn render_release_content(
    project_dir: &Path,
    provider: publish::DistributionProvider,
) -> Result<LifecycleReport> {
    let config = load_content_config(project_dir)?;
    let mut report = base_report("release-content.render", Some(provider), None);
    let screenshots = config
        .release
        .as_ref()
        .and_then(|release| release.screenshots.as_ref())
        .context("release.screenshots must be configured before render")?;
    let raw_dir = project_dir.join(
        screenshots
            .raw_dir
            .as_deref()
            .unwrap_or("release-content/screenshots/raw"),
    );
    let rendered_root = project_dir.join(
        screenshots
            .rendered_dir
            .as_deref()
            .unwrap_or("release-content/screenshots/rendered"),
    );
    let output_dir = rendered_root.join(provider.as_str());
    fs::create_dir_all(&output_dir)?;
    let mut assets = Vec::new();
    collect_render_assets(&raw_dir, &raw_dir, &output_dir, &mut assets)?;
    let manifest = RenderManifest {
        schema_version: 1,
        created_at_unix_seconds: now_unix_seconds(),
        provider: provider.as_str().to_string(),
        source_dir: raw_dir.display().to_string(),
        output_dir: output_dir.display().to_string(),
        assets,
    };
    let manifest_path = output_dir.join("release-content-manifest.json");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
    report.checks.push(LifecycleCheck {
        id: "release_content.render.manifest_written".to_string(),
        status: "passed".to_string(),
        summary: "render manifest was written".to_string(),
        details: Some(manifest_path.display().to_string()),
        remediation: Vec::new(),
    });
    report.checks.push(LifecycleCheck {
        id: "release_content.render.assets_present".to_string(),
        status: if manifest.assets.is_empty() {
            "missing"
        } else {
            "passed"
        }
        .to_string(),
        summary: "rendered release assets exist".to_string(),
        details: Some(format!("{} assets", manifest.assets.len())),
        remediation: vec![
            "Run release-content capture or add raw screenshots/videos before rendering."
                .to_string(),
        ],
    });
    finalize_status(&mut report);
    Ok(report)
}

fn capture_scenario(
    project_dir: &Path,
    raw_dir: &Path,
    target: Target,
    set: &str,
    scenario: &ScreenshotScenario,
    checks: &mut Vec<LifecycleCheck>,
) -> Result<()> {
    let id = scenario.id.as_deref().unwrap_or("scenario");
    checks.push(required_text_check(
        &format!("release_content.capture.{id}.id"),
        scenario.id.as_deref(),
        "scenario id is set",
    ));
    checks.push(required_text_check(
        &format!("release_content.capture.{id}.name"),
        scenario.name.as_deref(),
        "scenario name is set",
    ));
    checks.push(required_text_check(
        &format!("release_content.capture.{id}.wait_for"),
        scenario.wait_for.as_deref(),
        "scenario wait selector is set",
    ));
    let Some(script) = scenario.script.as_deref() else {
        checks.push(failed_check(
            &format!("release_content.capture.{id}.script"),
            "scenario script is missing".to_string(),
        ));
        return Ok(());
    };
    let script_path = project_dir.join(script);
    checks.push(path_check(
        &format!("release_content.capture.{id}.script_exists"),
        script_path.clone(),
        "scenario script exists",
    ));
    if !script_path.exists() {
        return Ok(());
    }
    match script_path.extension().and_then(|value| value.to_str()) {
        Some("sh") => run_capture_script(
            "bash",
            &[script_path.to_string_lossy().as_ref()],
            project_dir,
            raw_dir,
            target,
            set,
            id,
            checks,
        ),
        Some("ps1") => run_capture_script(
            "pwsh",
            &["-File", script_path.to_string_lossy().as_ref()],
            project_dir,
            raw_dir,
            target,
            set,
            id,
            checks,
        ),
        _ => {
            let receipt = raw_dir.join(format!("{set}-{id}-capture-plan.json"));
            let body = serde_json::json!({
                "schema_version": 1,
                "target": target.as_str(),
                "set": set,
                "scenario": id,
                "script": script_path,
                "wait_for": scenario.wait_for,
                "status": "planned",
                "note": "Non-shell scenario files are validated and recorded; execution is handled by the Fission platform test runner."
            });
            fs::write(&receipt, serde_json::to_vec_pretty(&body)?)?;
            checks.push(ok_check(
                &format!("release_content.capture.{id}.plan_written"),
                receipt.display().to_string(),
            ));
            Ok(())
        }
    }
}

fn run_capture_script(
    program: &str,
    args: &[&str],
    project_dir: &Path,
    raw_dir: &Path,
    target: Target,
    set: &str,
    id: &str,
    checks: &mut Vec<LifecycleCheck>,
) -> Result<()> {
    let output = Command::new(program)
        .args(args)
        .current_dir(project_dir)
        .env("FISSION_CAPTURE_OUTPUT", raw_dir)
        .env("FISSION_CAPTURE_TARGET", target.as_str())
        .env("FISSION_CAPTURE_SET", set)
        .env("FISSION_CAPTURE_SCENARIO", id)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("failed to run capture script through {program}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    checks.push(LifecycleCheck {
        id: format!("release_content.capture.{id}.script_ran"),
        status: if output.status.success() {
            "passed"
        } else {
            "failed"
        }
        .to_string(),
        summary: "capture script completed".to_string(),
        details: Some(format!(
            "stdout: {}; stderr: {}",
            stdout.trim(),
            stderr.trim()
        )),
        remediation: vec![
            "Fix the scenario script or run it manually with the printed environment variables."
                .to_string(),
        ],
    });
    Ok(())
}

fn validate_screenshots(
    project_dir: &Path,
    config: &ContentToml,
    provider: Option<publish::DistributionProvider>,
    checks: &mut Vec<LifecycleCheck>,
) {
    let screenshots = config
        .release
        .as_ref()
        .and_then(|release| release.screenshots.as_ref());
    let Some(screenshots) = screenshots else {
        checks.push(failed_check(
            "release_content.screenshots_configured",
            "[release.screenshots] is missing".to_string(),
        ));
        return;
    };
    let raw_dir = project_dir.join(
        screenshots
            .raw_dir
            .as_deref()
            .unwrap_or("release-content/screenshots/raw"),
    );
    let rendered_dir = project_dir.join(
        screenshots
            .rendered_dir
            .as_deref()
            .unwrap_or("release-content/screenshots/rendered"),
    );
    checks.push(path_check(
        "release_content.screenshots.raw_dir_exists",
        raw_dir,
        "raw screenshot directory exists",
    ));
    checks.push(path_check(
        "release_content.screenshots.rendered_dir_exists",
        rendered_dir.clone(),
        "rendered screenshot directory exists",
    ));
    checks.push(LifecycleCheck {
        id: "release_content.screenshots.scenarios_configured".to_string(),
        status: if screenshots.scenarios.is_empty() {
            "missing"
        } else {
            "passed"
        }
        .to_string(),
        summary: "screenshot scenarios are configured".to_string(),
        details: Some(format!("{} scenarios", screenshots.scenarios.len())),
        remediation: vec![
            "Add [[release.screenshots.scenarios]] entries with id, targets, script, and wait_for."
                .to_string(),
        ],
    });
    for scenario in &screenshots.scenarios {
        let id = scenario.id.as_deref().unwrap_or("scenario");
        if let Some(script) = scenario.script.as_deref() {
            checks.push(path_check(
                &format!("release_content.screenshots.{id}.script_exists"),
                project_dir.join(script),
                "scenario script exists",
            ));
        }
    }
    if let Some(provider) = provider {
        let provider_dir = rendered_dir.join(provider.as_str());
        let count = count_assets(&provider_dir).unwrap_or(0);
        checks.push(LifecycleCheck {
            id: format!("release_content.{}.rendered_assets", provider.as_str()),
            status: if count > 0 { "passed" } else { "missing" }.to_string(),
            summary: "provider rendered release assets exist".to_string(),
            details: Some(format!("{} assets in {}", count, provider_dir.display())),
            remediation: vec![format!(
                "Run `fission release-content render --provider {}` after capture.",
                provider.as_str()
            )],
        });
    }
}

fn validate_provider_assets(
    project_dir: &Path,
    config: &ContentToml,
    provider: Option<publish::DistributionProvider>,
    checks: &mut Vec<LifecycleCheck>,
) {
    let assets = config
        .release
        .as_ref()
        .and_then(|release| release.assets.as_ref());
    match provider {
        Some(publish::DistributionProvider::PlayStore) => {
            if let Some(play) = assets.and_then(|assets| assets.play_store.as_ref()) {
                check_optional_path(
                    project_dir,
                    "release_content.play_store.feature_graphic",
                    play.feature_graphic.as_deref(),
                    "Play Store feature graphic exists",
                    checks,
                );
                check_optional_path(
                    project_dir,
                    "release_content.play_store.screenshot_sets_dir",
                    play.screenshot_sets_dir.as_deref(),
                    "Play Store screenshot set directory exists",
                    checks,
                );
                check_optional_path(
                    project_dir,
                    "release_content.play_store.preview_video_dir",
                    play.preview_video_dir.as_deref(),
                    "Play Store preview video directory exists",
                    checks,
                );
            }
        }
        Some(publish::DistributionProvider::AppStore) => {
            if let Some(app) = assets.and_then(|assets| assets.app_store.as_ref()) {
                check_optional_path(
                    project_dir,
                    "release_content.app_store.screenshot_sets_dir",
                    app.screenshot_sets_dir.as_deref(),
                    "App Store screenshot set directory exists",
                    checks,
                );
                check_optional_path(
                    project_dir,
                    "release_content.app_store.app_previews_dir",
                    app.app_previews_dir.as_deref(),
                    "App Store preview video directory exists",
                    checks,
                );
                for path in &app.review_attachments {
                    checks.push(path_check(
                        "release_content.app_store.review_attachment",
                        project_dir.join(path),
                        "App Review attachment exists",
                    ));
                }
            }
        }
        Some(publish::DistributionProvider::MicrosoftStore) => {
            if let Some(ms) = assets.and_then(|assets| assets.microsoft_store.as_ref()) {
                check_optional_path(
                    project_dir,
                    "release_content.microsoft_store.screenshot_sets_dir",
                    ms.screenshot_sets_dir.as_deref(),
                    "Microsoft Store screenshot directory exists",
                    checks,
                );
                check_optional_path(
                    project_dir,
                    "release_content.microsoft_store.trailers_dir",
                    ms.trailers_dir.as_deref(),
                    "Microsoft Store trailers directory exists",
                    checks,
                );
                check_optional_path(
                    project_dir,
                    "release_content.microsoft_store.logo_dir",
                    ms.logo_dir.as_deref(),
                    "Microsoft Store logo directory exists",
                    checks,
                );
            }
        }
        _ => {}
    }
}

fn collect_render_assets(
    root: &Path,
    current: &Path,
    output_root: &Path,
    assets: &mut Vec<RenderedAsset>,
) -> Result<()> {
    if !current.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_render_assets(root, &path, output_root, assets)?;
            continue;
        }
        if !is_release_asset(&path) {
            continue;
        }
        let relative = path.strip_prefix(root).unwrap_or(&path);
        let dest = output_root.join(relative);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&path, &dest)?;
        let size = fs::metadata(&dest)?.len();
        assets.push(RenderedAsset {
            kind: asset_kind(&dest).to_string(),
            source: path.display().to_string(),
            output: dest.display().to_string(),
            size_bytes: size,
        });
    }
    Ok(())
}

fn check_optional_path(
    project_dir: &Path,
    id: &str,
    value: Option<&str>,
    summary: &str,
    checks: &mut Vec<LifecycleCheck>,
) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        checks.push(path_check(id, project_dir.join(value), summary));
    } else {
        checks.push(LifecycleCheck {
            id: id.to_string(),
            status: "missing".to_string(),
            summary: summary.to_string(),
            details: None,
            remediation: vec!["Configure the provider asset path in fission.toml.".to_string()],
        });
    }
}

fn count_assets(path: &Path) -> Result<usize> {
    let mut count = 0;
    if !path.exists() {
        return Ok(0);
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            count += count_assets(&path)?;
        } else if is_release_asset(&path) {
            count += 1;
        }
    }
    Ok(count)
}

fn is_release_asset(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "webp" | "mp4" | "mov" | "m4v")
    )
}

fn asset_kind(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "mp4" | "mov" | "m4v" => "video",
        _ => "image",
    }
}

fn required_text_check(id: &str, value: Option<&str>, summary: &str) -> LifecycleCheck {
    LifecycleCheck {
        id: id.to_string(),
        status: if value.is_some_and(|value| !value.trim().is_empty()) {
            "passed"
        } else {
            "missing"
        }
        .to_string(),
        summary: summary.to_string(),
        details: value.map(str::to_string),
        remediation: vec!["Set the missing scenario field in fission.toml.".to_string()],
    }
}

fn load_content_config(project_dir: &Path) -> Result<ContentToml> {
    let path = project_dir.join("fission.toml");
    let data =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&data).with_context(|| format!("failed to parse {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn unique_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "fission-release-content-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_content_project(dir: &Path) {
        fs::create_dir_all(dir.join("release-content/screenshots/raw/en-US")).unwrap();
        fs::write(
            dir.join("release-content/screenshots/raw/en-US/home.png"),
            b"png",
        )
        .unwrap();
        fs::create_dir_all(dir.join("tests/release_screenshots")).unwrap();
        fs::write(
            dir.join("tests/release_screenshots/home.toml"),
            "wait = true\n",
        )
        .unwrap();
        fs::write(
            dir.join("fission.toml"),
            r#"[app]
name = "content-demo"
app_id = "com.example.content_demo"

[release.screenshots]
raw_dir = "release-content/screenshots/raw"
rendered_dir = "release-content/screenshots/rendered"

[[release.screenshots.scenarios]]
id = "home"
name = "Home"
targets = ["web"]
script = "tests/release_screenshots/home.toml"
wait_for = "semantic:home"

[release.assets.play_store]
screenshot_sets_dir = "release-content/screenshots/rendered/play-store"
feature_graphic = "release-content/screenshots/raw/en-US/home.png"
"#,
        )
        .unwrap();
    }

    #[test]
    fn render_release_content_copies_raw_assets_and_writes_manifest() {
        let dir = unique_dir("render");
        write_content_project(&dir);
        let report =
            render_release_content(&dir, publish::DistributionProvider::PlayStore).unwrap();
        assert_ne!(report.status, "blocked");
        assert!(dir
            .join("release-content/screenshots/rendered/play-store/en-US/home.png")
            .exists());
        assert!(dir
            .join("release-content/screenshots/rendered/play-store/release-content-manifest.json")
            .exists());
    }
}
