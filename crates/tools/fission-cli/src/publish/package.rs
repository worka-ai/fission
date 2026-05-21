use super::*;
use crate::{cargo_package_name, read_project_config, workflow, FissionProject, Target};
use anyhow::{bail, Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Deserialize;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Builder as TarBuilder;

#[derive(Debug, Deserialize, Default)]
struct PackageManifest {
    package: Option<PackageRoot>,
}

#[derive(Debug, Deserialize, Default)]
struct PackageRoot {
    macos: Option<MacosPackageConfig>,
}

#[derive(Debug, Deserialize, Default)]
struct MacosPackageConfig {
    bundle_id: Option<String>,
    minimum_os: Option<String>,
    entitlements: Option<String>,
    signing_identity: Option<String>,
    installer_identity: Option<String>,
    notarize: Option<bool>,
}

pub(super) fn package_artifact(options: &PackageOptions) -> Result<ArtifactManifest> {
    match options.format {
        PackageFormat::Static => package_static(options),
        PackageFormat::Run => package_linux_run(options),
        PackageFormat::App => package_macos_app(options),
        PackageFormat::Pkg => package_macos_pkg(options),
        PackageFormat::Exe => package_windows_exe(options),
        PackageFormat::Apk => package_android_apk(options),
        PackageFormat::Aab => package_with_project_script(
            options,
            Target::Android,
            "platforms/android/package-aab.sh",
            "aab",
        ),
        PackageFormat::Ipa => {
            package_with_project_script(options, Target::Ios, "platforms/ios/package-ipa.sh", "ipa")
        }
        PackageFormat::Msi => package_with_project_script(
            options,
            Target::Windows,
            "platforms/windows/package-msi.ps1",
            "msi",
        ),
        PackageFormat::Msix => package_with_project_script(
            options,
            Target::Windows,
            "platforms/windows/package-msix.ps1",
            "msix",
        ),
    }
}

pub(super) fn package_static(options: &PackageOptions) -> Result<ArtifactManifest> {
    if options.format != PackageFormat::Static {
        bail!("only --format static is currently supported");
    }
    let project = read_project_config(&options.project_dir)?;
    if !project.targets.contains(&options.target) {
        bail!(
            "target `{}` is not configured for this app; run `cargo fission add-target {} --project-dir {}`",
            options.target.as_str(),
            options.target.as_str(),
            options.project_dir.display()
        );
    }

    let source_dir = match options.target {
        Target::Site => {
            workflow::site_build(&options.project_dir, options.release)?;
            site_output_dir(&options.project_dir)?
        }
        Target::Web => {
            workflow::build_app(workflow::BuildOptions {
                project_dir: options.project_dir.clone(),
                target: Some(Target::Web),
                release: options.release,
            })?;
            options.project_dir.join("platforms/web")
        }
        other => bail!(
            "static packaging currently supports site and web targets, not `{}`",
            other.as_str()
        ),
    };

    if !source_dir.join("index.html").exists() {
        bail!(
            "static package source {} does not contain index.html",
            source_dir.display()
        );
    }

    let profile = profile_name(options.release);
    let staging_dir = clean_package_dir(options)?;
    copy_dir_contents(&source_dir, &staging_dir)?;

    finish_artifact_manifest(&project, options, &staging_dir, profile)
}

fn package_linux_run(options: &PackageOptions) -> Result<ArtifactManifest> {
    ensure_package_target(options, Target::Linux, PackageFormat::Run)?;
    require_host_os(Target::Linux)?;
    let project = read_project_config(&options.project_dir)?;
    let profile = profile_name(options.release);
    let staging_dir = clean_package_dir(options)?;
    let payload_dir = staging_dir.join("payload");
    fs::create_dir_all(&payload_dir)?;
    let binary = build_desktop_binary(&options.project_dir, options.release)?;
    let executable_name = binary
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("app")
        .to_string();
    fs::copy(&binary, payload_dir.join(&executable_name)).with_context(|| {
        format!(
            "failed to copy {} to {}",
            binary.display(),
            payload_dir.display()
        )
    })?;
    copy_optional_assets(&options.project_dir, &payload_dir)?;

    let package_name = sanitize_file_stem(&project.app.name);
    let run_path = staging_dir.join(format!(
        "{package_name}-{}-{}.run",
        cargo_package_version(&options.project_dir).unwrap_or_else(|| "0.0.0".to_string()),
        profile
    ));
    write_linux_run(&payload_dir, &run_path, &project.app.name, &executable_name)?;
    fs::remove_dir_all(&payload_dir).ok();
    finish_artifact_manifest(&project, options, &staging_dir, profile)
}

fn package_macos_app(options: &PackageOptions) -> Result<ArtifactManifest> {
    ensure_package_target(options, Target::Macos, PackageFormat::App)?;
    require_host_os(Target::Macos)?;
    let project = read_project_config(&options.project_dir)?;
    let profile = profile_name(options.release);
    let staging_dir = clean_package_dir(options)?;
    let macos = macos_package_config(&options.project_dir)?;
    let app_bundle = create_macos_app_bundle(options, &project, &staging_dir, &macos)?;
    sign_macos_app_if_configured(&options.project_dir, &app_bundle, &macos)?;
    println!("{}", app_bundle.display());
    finish_artifact_manifest(&project, options, &staging_dir, profile)
}

fn package_macos_pkg(options: &PackageOptions) -> Result<ArtifactManifest> {
    ensure_package_target(options, Target::Macos, PackageFormat::Pkg)?;
    require_host_os(Target::Macos)?;
    let project = read_project_config(&options.project_dir)?;
    let profile = profile_name(options.release);
    let staging_dir = clean_package_dir(options)?;
    let app_staging = staging_dir.join("app-staging");
    let macos = macos_package_config(&options.project_dir)?;
    let app_bundle = create_macos_app_bundle(options, &project, &app_staging, &macos)?;
    sign_macos_app_if_configured(&options.project_dir, &app_bundle, &macos)?;
    let pkg_path = staging_dir.join(format!(
        "{}-{}.pkg",
        sanitize_file_stem(&project.app.name),
        cargo_package_version(&options.project_dir).unwrap_or_else(|| "0.0.0".to_string())
    ));
    if find_in_path("pkgbuild").is_none() {
        bail!("pkgbuild was not found; install Xcode command line tools to create macOS .pkg packages");
    }
    let status = Command::new("pkgbuild")
        .arg("--component")
        .arg(&app_bundle)
        .arg("--install-location")
        .arg("/Applications")
        .args(pkgbuild_signing_args(&macos))
        .arg(&pkg_path)
        .status()
        .context("failed to run pkgbuild")?;
    if !status.success() {
        bail!("pkgbuild failed with {status}");
    }
    notarize_macos_artifact_if_configured(&pkg_path, &macos)?;
    fs::remove_dir_all(&app_staging).ok();
    finish_artifact_manifest(&project, options, &staging_dir, profile)
}

fn package_windows_exe(options: &PackageOptions) -> Result<ArtifactManifest> {
    ensure_package_target(options, Target::Windows, PackageFormat::Exe)?;
    require_host_os(Target::Windows)?;
    let project = read_project_config(&options.project_dir)?;
    let profile = profile_name(options.release);
    let staging_dir = clean_package_dir(options)?;
    let binary = build_desktop_binary(&options.project_dir, options.release)?;
    let dest = staging_dir.join(binary.file_name().unwrap_or_else(|| OsStr::new("app.exe")));
    fs::copy(&binary, &dest)
        .with_context(|| format!("failed to copy {} to {}", binary.display(), dest.display()))?;
    copy_optional_assets(&options.project_dir, &staging_dir)?;
    finish_artifact_manifest(&project, options, &staging_dir, profile)
}

fn package_android_apk(options: &PackageOptions) -> Result<ArtifactManifest> {
    ensure_package_target(options, Target::Android, PackageFormat::Apk)?;
    let project = read_project_config(&options.project_dir)?;
    let profile = profile_name(options.release);
    let staging_dir = clean_package_dir(options)?;
    let script = options.project_dir.join("platforms/android/package-apk.sh");
    let output_path = run_packaging_script(&options.project_dir, &script, options.release)?
        .with_context(|| format!("{} did not print an .apk path", script.display()))?;
    if output_path.extension().and_then(OsStr::to_str) != Some("apk") {
        bail!(
            "{} printed {}, expected an .apk artifact",
            script.display(),
            output_path.display()
        );
    }
    let dest = staging_dir.join(
        output_path
            .file_name()
            .unwrap_or_else(|| OsStr::new("app.apk")),
    );
    fs::copy(&output_path, &dest).with_context(|| {
        format!(
            "failed to copy Android APK {} to {}",
            output_path.display(),
            dest.display()
        )
    })?;
    finish_artifact_manifest(&project, options, &staging_dir, profile)
}

fn package_with_project_script(
    options: &PackageOptions,
    target: Target,
    relative_script: &str,
    expected_extension: &str,
) -> Result<ArtifactManifest> {
    ensure_package_target(options, target, options.format)?;
    let project = read_project_config(&options.project_dir)?;
    let profile = profile_name(options.release);
    let staging_dir = clean_package_dir(options)?;
    let script = options.project_dir.join(relative_script);
    if !script.exists() {
        bail!(
            "{} packaging requires {}; this target packaging flow has not been configured for this project yet",
            options.format.as_str(),
            script.display()
        );
    }
    let output_path = run_packaging_script(&options.project_dir, &script, options.release)?
        .with_context(|| format!("{} did not print a package path", script.display()))?;
    if output_path.extension().and_then(OsStr::to_str) != Some(expected_extension) {
        bail!(
            "{} printed {}, expected a .{} artifact",
            script.display(),
            output_path.display(),
            expected_extension
        );
    }
    let dest = staging_dir.join(
        output_path
            .file_name()
            .unwrap_or_else(|| OsStr::new("artifact")),
    );
    fs::copy(&output_path, &dest).with_context(|| {
        format!(
            "failed to copy package {} to {}",
            output_path.display(),
            dest.display()
        )
    })?;
    finish_artifact_manifest(&project, options, &staging_dir, profile)
}

fn finish_artifact_manifest(
    project: &FissionProject,
    options: &PackageOptions,
    staging_dir: &Path,
    profile: &str,
) -> Result<ArtifactManifest> {
    let manifest = build_artifact_manifest(project, options, staging_dir, profile)?;
    let manifest_path = staging_dir.join(ARTIFACT_MANIFEST);
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?).with_context(|| {
        format!(
            "failed to write artifact manifest {}",
            manifest_path.display()
        )
    })?;
    Ok(manifest)
}

fn ensure_package_target(
    options: &PackageOptions,
    expected_target: Target,
    expected_format: PackageFormat,
) -> Result<()> {
    if options.target != expected_target || options.format != expected_format {
        bail!(
            "--target {} --format {} is required for this package path",
            expected_target.as_str(),
            expected_format.as_str()
        );
    }
    let project = read_project_config(&options.project_dir)?;
    if !project.targets.contains(&options.target) {
        bail!(
            "target `{}` is not configured for this app; run `cargo fission add-target {} --project-dir {}`",
            options.target.as_str(),
            options.target.as_str(),
            options.project_dir.display()
        );
    }
    Ok(())
}

fn profile_name(release: bool) -> &'static str {
    if release {
        "release"
    } else {
        "debug"
    }
}

fn clean_package_dir(options: &PackageOptions) -> Result<PathBuf> {
    let staging_dir = options
        .project_dir
        .join("target/fission")
        .join(profile_name(options.release))
        .join(options.target.as_str())
        .join(options.format.as_str());
    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir)
            .with_context(|| format!("failed to clean {}", staging_dir.display()))?;
    }
    fs::create_dir_all(&staging_dir)
        .with_context(|| format!("failed to create {}", staging_dir.display()))?;
    Ok(staging_dir)
}

fn require_host_os(target: Target) -> Result<()> {
    let ok = match target {
        Target::Linux => cfg!(target_os = "linux"),
        Target::Macos => cfg!(target_os = "macos"),
        Target::Windows => cfg!(target_os = "windows"),
        _ => true,
    };
    if ok {
        Ok(())
    } else {
        bail!(
            "{} packaging must run on a {} host for now",
            target.as_str(),
            target.as_str()
        )
    }
}

fn build_desktop_binary(project_dir: &Path, release: bool) -> Result<PathBuf> {
    let mut command = Command::new("cargo");
    command.arg("build").current_dir(project_dir);
    if release {
        command.arg("--release");
    }
    let status = command.status().context("failed to run cargo build")?;
    if !status.success() {
        bail!("desktop build failed with {status}");
    }
    let name = cargo_package_name(project_dir).context("Cargo.toml package.name is required")?;
    let executable = if cfg!(target_os = "windows") {
        format!("{name}.exe")
    } else {
        name
    };
    let path = project_dir
        .join("target")
        .join(profile_name(release))
        .join(executable);
    if !path.exists() {
        bail!("expected built binary at {}", path.display());
    }
    Ok(path)
}

fn create_macos_app_bundle(
    options: &PackageOptions,
    project: &FissionProject,
    staging_dir: &Path,
    macos: &MacosPackageConfig,
) -> Result<PathBuf> {
    let binary = build_desktop_binary(&options.project_dir, options.release)?;
    let executable = binary
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("app")
        .to_string();
    let app_name = display_app_name(&project.app.name);
    let app_bundle = staging_dir.join(format!("{app_name}.app"));
    let contents = app_bundle.join("Contents");
    let macos_dir = contents.join("MacOS");
    let resources = contents.join("Resources");
    fs::create_dir_all(&macos_dir)?;
    fs::create_dir_all(&resources)?;
    fs::copy(&binary, macos_dir.join(&executable)).with_context(|| {
        format!(
            "failed to copy {} into {}",
            binary.display(),
            app_bundle.display()
        )
    })?;
    if let Some(icon) = app_icon_path(&options.project_dir) {
        let _ = fs::copy(icon, resources.join("AppIcon.png"));
    }
    let version =
        cargo_package_version(&options.project_dir).unwrap_or_else(|| "0.1.0".to_string());
    let plist = render_info_plist(project, &app_name, &executable, macos, &version);
    fs::write(contents.join("Info.plist"), plist)?;
    fs::write(contents.join("PkgInfo"), "APPL????")?;
    Ok(app_bundle)
}

fn render_info_plist(
    project: &FissionProject,
    app_name: &str,
    executable: &str,
    macos: &MacosPackageConfig,
    version: &str,
) -> String {
    let bundle_id = macos
        .bundle_id
        .as_deref()
        .unwrap_or(project.app.app_id.as_str());
    let minimum_os = macos.minimum_os.as_deref().unwrap_or("13.0");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key>
  <string>{}</string>
  <key>CFBundleName</key>
  <string>{}</string>
  <key>CFBundleDisplayName</key>
  <string>{}</string>
  <key>CFBundleExecutable</key>
  <string>{}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>{}</string>
  <key>CFBundleVersion</key>
  <string>{}</string>
  <key>LSMinimumSystemVersion</key>
  <string>{}</string>
</dict>
</plist>
"#,
        escape_xml(bundle_id),
        escape_xml(app_name),
        escape_xml(app_name),
        escape_xml(executable),
        escape_xml(version),
        escape_xml(version),
        escape_xml(minimum_os)
    )
}

fn macos_package_config(project_dir: &Path) -> Result<MacosPackageConfig> {
    let path = project_dir.join("fission.toml");
    let data =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let manifest: PackageManifest =
        toml::from_str(&data).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(manifest
        .package
        .and_then(|package| package.macos)
        .unwrap_or_default())
}

fn sign_macos_app_if_configured(
    project_dir: &Path,
    app_bundle: &Path,
    macos: &MacosPackageConfig,
) -> Result<()> {
    let Some(identity) = macos
        .signing_identity
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(());
    };
    let mut command = Command::new("codesign");
    command
        .arg("--force")
        .arg("--timestamp")
        .arg("--options")
        .arg("runtime")
        .arg("--sign")
        .arg(identity);
    if let Some(entitlements) = macos
        .entitlements
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        command
            .arg("--entitlements")
            .arg(resolve_project_path(project_dir, entitlements.to_string()));
    }
    let status = command
        .arg(app_bundle)
        .status()
        .context("failed to run codesign")?;
    if !status.success() {
        bail!("codesign failed with {status}");
    }
    let verify = Command::new("codesign")
        .args(["--verify", "--deep", "--strict", "--verbose=2"])
        .arg(app_bundle)
        .status()
        .context("failed to verify macOS code signature")?;
    if !verify.success() {
        bail!("codesign verification failed with {verify}");
    }
    Ok(())
}

fn pkgbuild_signing_args(macos: &MacosPackageConfig) -> Vec<String> {
    macos
        .installer_identity
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(|identity| vec!["--sign".to_string(), identity.to_string()])
        .unwrap_or_default()
}

fn notarize_macos_artifact_if_configured(
    artifact: &Path,
    macos: &MacosPackageConfig,
) -> Result<()> {
    if !macos.notarize.unwrap_or(false) {
        return Ok(());
    }
    let key = env::var("APP_STORE_CONNECT_API_KEY_PATH")
        .context("APP_STORE_CONNECT_API_KEY_PATH is required when package.macos.notarize = true")?;
    let key_id = env::var("APP_STORE_CONNECT_KEY_ID")
        .context("APP_STORE_CONNECT_KEY_ID is required when package.macos.notarize = true")?;
    let issuer = env::var("APP_STORE_CONNECT_ISSUER_ID")
        .context("APP_STORE_CONNECT_ISSUER_ID is required when package.macos.notarize = true")?;
    let submit = Command::new("xcrun")
        .args([
            "notarytool",
            "submit",
            artifact.to_string_lossy().as_ref(),
            "--key",
            &key,
            "--key-id",
            &key_id,
            "--issuer",
            &issuer,
            "--wait",
        ])
        .status()
        .context("failed to run xcrun notarytool")?;
    if !submit.success() {
        bail!("notarytool submit failed with {submit}");
    }
    let staple = Command::new("xcrun")
        .args(["stapler", "staple"])
        .arg(artifact)
        .status()
        .context("failed to run xcrun stapler")?;
    if !staple.success() {
        bail!("stapler failed with {staple}");
    }
    Ok(())
}

fn write_linux_run(
    payload_dir: &Path,
    run_path: &Path,
    app_name: &str,
    executable_name: &str,
) -> Result<()> {
    let mut archive = Vec::new();
    {
        let encoder = GzEncoder::new(&mut archive, Compression::default());
        let mut tar = TarBuilder::new(encoder);
        tar.append_dir_all(".", payload_dir)?;
        let encoder = tar.into_inner()?;
        encoder.finish()?;
    }
    let mut file = fs::File::create(run_path)?;
    writeln!(
        file,
        r#"#!/bin/sh
set -eu
APP_NAME="{app_name}"
EXECUTABLE="{executable_name}"
DEST="${{FISSION_INSTALL_DIR:-$HOME/.local/opt/$APP_NAME}}"
mkdir -p "$DEST"
ARCHIVE_LINE=$(awk '/^__FISSION_ARCHIVE_BELOW__$/ {{ print NR + 1; exit 0; }}' "$0")
tail -n +"$ARCHIVE_LINE" "$0" | tar -xz -C "$DEST"
chmod +x "$DEST/$EXECUTABLE" 2>/dev/null || true
echo "Installed $APP_NAME to $DEST"
echo "Run: $DEST/$EXECUTABLE"
exit 0
__FISSION_ARCHIVE_BELOW__"#
    )?;
    file.write_all(&archive)?;
    set_executable(run_path)?;
    Ok(())
}

fn set_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(perms.mode() | 0o755);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}

fn copy_optional_assets(project_dir: &Path, dest: &Path) -> Result<()> {
    let assets = project_dir.join("assets");
    if assets.exists() {
        copy_dir_contents(&assets, &dest.join("assets"))?;
    }
    Ok(())
}

fn run_packaging_script(
    project_dir: &Path,
    script: &Path,
    release: bool,
) -> Result<Option<PathBuf>> {
    if !script.exists() {
        bail!("packaging script is missing at {}", script.display());
    }
    let extension = script.extension().and_then(OsStr::to_str);
    let mut command = if extension == Some("ps1") {
        let program = if cfg!(windows) {
            "powershell"
        } else if find_in_path("pwsh").is_some() {
            "pwsh"
        } else {
            bail!(
                "{} requires PowerShell; install pwsh or run this package format on Windows",
                script.display()
            );
        };
        let mut command = Command::new(program);
        if cfg!(windows) {
            command.args(["-ExecutionPolicy", "Bypass", "-File"]);
        } else {
            command.arg("-File");
        }
        command.arg(script);
        command
    } else if cfg!(windows) || extension == Some("sh") {
        let mut command = Command::new("bash");
        command.arg(script);
        command
    } else {
        Command::new(script)
    };
    command.current_dir(project_dir);
    if release {
        command.env("ANDROID_PROFILE", "release");
        command.env("IOS_PROFILE", "release");
        command.env("WINDOWS_PROFILE", "release");
    }
    let output = command
        .output()
        .with_context(|| format!("failed to run {}", script.display()))?;
    io::stderr().write_all(&output.stderr).ok();
    if !output.status.success() {
        bail!("{} failed with {}", script.display(), output.status);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .rev()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let path = PathBuf::from(line);
            if path.is_absolute() {
                path
            } else {
                project_dir.join(path)
            }
        })
        .find(|path| path.exists()))
}

fn sanitize_file_stem(value: &str) -> String {
    let stem = value
        .chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' => ch,
            _ => '-',
        })
        .collect::<String>()
        .trim_matches(['-', '.', '_'])
        .to_string();
    if stem.is_empty() {
        "app".to_string()
    } else {
        stem
    }
}

fn display_app_name(value: &str) -> String {
    let mut out = String::new();
    let mut uppercase_next = true;
    for ch in value.chars() {
        match ch {
            '-' | '_' | '.' | ' ' => {
                if !out.ends_with(' ') && !out.is_empty() {
                    out.push(' ');
                }
                uppercase_next = true;
            }
            _ if uppercase_next => {
                out.extend(ch.to_uppercase());
                uppercase_next = false;
            }
            _ => out.push(ch),
        }
    }
    if out.trim().is_empty() {
        "Fission App".to_string()
    } else {
        out.trim().to_string()
    }
}

fn app_icon_path(project_dir: &Path) -> Option<PathBuf> {
    [
        "assets/app-icon.icns",
        "assets/AppIcon.icns",
        "assets/app-icon.png",
        "assets/icon.png",
    ]
    .into_iter()
    .map(|relative| project_dir.join(relative))
    .find(|path| path.exists())
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
