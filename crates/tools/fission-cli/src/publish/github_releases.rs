use super::*;
use crate::release;
use anyhow::{bail, Context, Result};
use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{ACCEPT, CONTENT_LENGTH, CONTENT_TYPE, USER_AGENT};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const GITHUB_API: &str = "https://api.github.com";
const GITHUB_UPLOADS: &str = "https://uploads.github.com";
const GITHUB_API_VERSION: &str = "2022-11-28";

#[derive(Clone, Debug)]
struct ReleaseAsset {
    path: PathBuf,
    name: String,
    mime_type: String,
    sha256: Option<String>,
    size_bytes: u64,
}

pub(super) fn setup(options: &DistributeOptions, config: &PublishManifest) -> Result<()> {
    let cfg = github_releases_config(config, &options.site)?;
    let owner = cfg
        .owner
        .clone()
        .or_else(|| infer_github_owner(&options.project_dir))
        .unwrap_or_else(|| "<missing>".to_string());
    let repo = cfg
        .repo
        .clone()
        .or_else(|| infer_github_repo(&options.project_dir))
        .unwrap_or_else(|| "<missing>".to_string());
    println!("GitHub Releases setup checks for `{}`", options.site);
    println!("owner: {owner}");
    println!("repo: {repo}");
    println!(
        "tag: {}",
        cfg.tag
            .as_deref()
            .unwrap_or("<from --deploy or artifact version>")
    );
    println!(
        "Run `fission readiness distribute --provider github-releases --site {} --artifact <artifact-manifest> --project-dir {}` before publishing.",
        options.site,
        options.project_dir.display()
    );
    Ok(())
}

pub(super) fn readiness(
    project_dir: &Path,
    site: &str,
    artifact: Option<&Path>,
    config: &PublishManifest,
    checks: &mut Vec<ReadinessCheck>,
) -> Result<()> {
    let cfg = github_releases_config(config, site)?;
    let owner = cfg
        .owner
        .clone()
        .or_else(|| infer_github_owner(project_dir));
    let repo = cfg.repo.clone().or_else(|| infer_github_repo(project_dir));
    checks.push(required_value(
        "release.github_releases.owner_configured",
        owner.as_deref(),
        "GitHub release owner is configured or inferable from git remote",
        "Set distribution.github_releases.<site>.owner or configure an origin GitHub remote.",
    ));
    checks.push(required_value(
        "release.github_releases.repo_configured",
        repo.as_deref(),
        "GitHub release repository is configured or inferable from git remote",
        "Set distribution.github_releases.<site>.repo or configure an origin GitHub remote.",
    ));
    checks.push(required_provider_secret(
        "release.github_releases.token_available",
        DistributionProvider::GithubReleases,
        &["GH_TOKEN", "GITHUB_TOKEN"],
        "Create a GitHub token with repository Contents write permission and store it in CI secrets or the Fission release vault.",
    ));
    checks.push(check(
        "release.github_releases.replace_assets_explicit",
        CheckSeverity::Info,
        if cfg.replace_assets.is_some() {
            CheckStatus::Passed
        } else {
            CheckStatus::Skipped
        },
        "duplicate release asset behavior is explicit",
        cfg.replace_assets
            .map(|value| format!("replace_assets = {value}")),
        vec!["Set replace_assets = true to overwrite same-named release assets during republish, or false to fail safely."],
    ));
    if let Some(artifact) = artifact.filter(|path| path.exists()) {
        let manifest = read_artifact_manifest(artifact)?;
        let assets = release_assets(&manifest, artifact, &cfg)?;
        checks.push(check(
            "release.github_releases.assets_available",
            CheckSeverity::Error,
            if assets.is_empty() {
                CheckStatus::Missing
            } else {
                CheckStatus::Passed
            },
            "artifact manifest contains uploadable release assets",
            Some(format!(
                "{} uploadable assets for {} {}",
                assets.len(),
                manifest.target,
                manifest.format
            )),
            vec!["Package the app artifact first. GitHub Releases accepts any artifact file, including .run, .pkg, .exe, .msi, .msix, .apk, .aab, .ipa, archives, symbol files, and zipped static-site outputs."],
        ));
        let tag = release_tag(&cfg, &manifest, None);
        checks.push(check(
            "release.github_releases.tag_resolved",
            CheckSeverity::Error,
            if tag.is_some() {
                CheckStatus::Passed
            } else {
                CheckStatus::Missing
            },
            "release tag can be resolved",
            tag,
            vec!["Set distribution.github_releases.<site>.tag, pass --deploy <tag>, or ensure Cargo.toml has package.version."],
        ));
    }
    Ok(())
}

pub(super) fn status(
    options: &DistributeOptions,
    config: &PublishManifest,
) -> Result<DistributionReceipt> {
    let cfg = github_releases_config(config, &options.site)?;
    let (owner, repo) = github_repo(&options.project_dir, &cfg)?;
    let client = github_client()?;
    let token = release::provider_secret(
        DistributionProvider::GithubReleases,
        &["GH_TOKEN", "GITHUB_TOKEN"],
    )?;
    let release = if let Some(tag) = options.deploy.as_deref().or(cfg.tag.as_deref()) {
        github_json(
            github_request(
                &client,
                token.as_deref(),
                reqwest::Method::GET,
                format!(
                    "{GITHUB_API}/repos/{owner}/{repo}/releases/tags/{}",
                    url_segment(tag)
                ),
            ),
            "GitHub release status",
        )?
    } else {
        github_json(
            github_request(
                &client,
                token.as_deref(),
                reqwest::Method::GET,
                format!("{GITHUB_API}/repos/{owner}/{repo}/releases/latest"),
            ),
            "GitHub latest release status",
        )?
    };
    Ok(DistributionReceipt {
        schema_version: 1,
        created_at_unix_seconds: now_unix_seconds(),
        provider: "github-releases".to_string(),
        site: options.site.clone(),
        action: "status".to_string(),
        artifact_manifest: None,
        deployment_id: release
            .get("id")
            .and_then(Value::as_i64)
            .map(|id| id.to_string())
            .or_else(|| {
                release
                    .get("tag_name")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            }),
        canonical_url: release
            .get("html_url")
            .and_then(Value::as_str)
            .map(str::to_string),
        preview_url: None,
        custom_domain: None,
        status: github_release_state(&release),
        stdout: Some(serde_json::to_string_pretty(&release)?),
        stderr: None,
        manual_follow_up: Vec::new(),
    })
}

pub(super) fn publish(
    options: &DistributeOptions,
    config: &PublishManifest,
    artifact_path: &Path,
    manifest: &ArtifactManifest,
) -> Result<DistributionReceipt> {
    let cfg = github_releases_config(config, &options.site)?;
    let (owner, repo) = github_repo(&options.project_dir, &cfg)?;
    let tag = release_tag(&cfg, manifest, options.deploy.as_deref()).context(
        "GitHub Releases publishing requires a tag from --deploy, config tag, or artifact version",
    )?;
    let assets = release_assets(manifest, artifact_path, &cfg)?;
    if assets.is_empty() {
        bail!("GitHub Releases publishing requires at least one artifact file");
    }
    if options.dry_run {
        return Ok(DistributionReceipt {
            schema_version: 1,
            created_at_unix_seconds: now_unix_seconds(),
            provider: "github-releases".to_string(),
            site: options.site.clone(),
            action: "publish".to_string(),
            artifact_manifest: Some(artifact_path.display().to_string()),
            deployment_id: Some(tag.clone()),
            canonical_url: Some(format!(
                "https://github.com/{owner}/{repo}/releases/tag/{}",
                url_segment(&tag)
            )),
            preview_url: None,
            custom_domain: None,
            status: "dry-run".to_string(),
            stdout: Some(serde_json::to_string_pretty(&json!({
                "tag": tag,
                "assets": assets.iter().map(asset_json).collect::<Vec<_>>()
            }))?),
            stderr: None,
            manual_follow_up: Vec::new(),
        });
    }

    let token = release::provider_secret(
        DistributionProvider::GithubReleases,
        &["GH_TOKEN", "GITHUB_TOKEN"],
    )?
    .context(
        "GH_TOKEN, GITHUB_TOKEN, or Fission vault credentials are required for GitHub Releases",
    )?;
    let client = github_client()?;
    let release = upsert_release(
        &client,
        &token,
        &owner,
        &repo,
        &tag,
        &cfg,
        &options.project_dir,
    )?;
    let release_id = release
        .get("id")
        .and_then(Value::as_i64)
        .context("GitHub release response did not contain id")?;
    let existing = list_assets(&client, &token, &owner, &repo, release_id)?;
    let replace_assets = cfg.replace_assets.unwrap_or(false);
    let mut uploaded = Vec::new();
    for asset in assets {
        if let Some(existing_asset) = existing
            .iter()
            .find(|item| item.get("name").and_then(Value::as_str) == Some(asset.name.as_str()))
        {
            if !replace_assets {
                bail!(
                    "GitHub release asset `{}` already exists; set replace_assets = true to overwrite it",
                    asset.name
                );
            }
            delete_asset(&client, &token, &owner, &repo, existing_asset)?;
        }
        uploaded.push(upload_asset(
            &client, &token, &owner, &repo, release_id, &asset,
        )?);
    }
    let stdout = json!({
        "release": release,
        "uploaded_assets": uploaded,
    });
    Ok(DistributionReceipt {
        schema_version: 1,
        created_at_unix_seconds: now_unix_seconds(),
        provider: "github-releases".to_string(),
        site: options.site.clone(),
        action: "publish".to_string(),
        artifact_manifest: Some(artifact_path.display().to_string()),
        deployment_id: Some(release_id.to_string()),
        canonical_url: release
            .get("html_url")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                Some(format!(
                    "https://github.com/{owner}/{repo}/releases/tag/{tag}"
                ))
            }),
        preview_url: None,
        custom_domain: None,
        status: github_release_state(&release),
        stdout: Some(serde_json::to_string_pretty(&stdout)?),
        stderr: None,
        manual_follow_up: cfg
            .draft
            .unwrap_or(false)
            .then(|| {
                "Publish the draft release from GitHub when it is ready for users.".to_string()
            })
            .into_iter()
            .collect(),
    })
}

fn github_repo(project_dir: &Path, cfg: &GithubReleasesConfig) -> Result<(String, String)> {
    let owner = cfg
        .owner
        .clone()
        .or_else(|| infer_github_owner(project_dir))
        .context("distribution.github_releases.<site>.owner or GitHub origin remote is required")?;
    let repo = cfg
        .repo
        .clone()
        .or_else(|| infer_github_repo(project_dir))
        .context("distribution.github_releases.<site>.repo or GitHub origin remote is required")?;
    Ok((owner, repo))
}

fn release_tag(
    cfg: &GithubReleasesConfig,
    manifest: &ArtifactManifest,
    override_tag: Option<&str>,
) -> Option<String> {
    override_tag
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .or_else(|| cfg.tag.clone().filter(|value| !value.trim().is_empty()))
        .or_else(|| {
            manifest
                .project
                .version
                .as_ref()
                .map(|version| format!("v{version}"))
        })
}

fn upsert_release(
    client: &Client,
    token: &str,
    owner: &str,
    repo: &str,
    tag: &str,
    cfg: &GithubReleasesConfig,
    project_dir: &Path,
) -> Result<Value> {
    let get = github_request(
        client,
        Some(token),
        reqwest::Method::GET,
        format!(
            "{GITHUB_API}/repos/{owner}/{repo}/releases/tags/{}",
            url_segment(tag)
        ),
    )
    .send()
    .context("failed to query GitHub release by tag")?;
    if get.status().is_success() {
        let release = github_json_from_response(get, "GitHub release lookup")?;
        let release_id = release
            .get("id")
            .and_then(Value::as_i64)
            .context("GitHub release response did not contain id")?;
        return github_json(
            github_request(
                client,
                Some(token),
                reqwest::Method::PATCH,
                format!("{GITHUB_API}/repos/{owner}/{repo}/releases/{release_id}"),
            )
            .json(&release_payload(tag, cfg, project_dir)?),
            "GitHub release update",
        );
    }
    if get.status().as_u16() != 404 {
        return github_json_from_response(get, "GitHub release lookup");
    }
    github_json(
        github_request(
            client,
            Some(token),
            reqwest::Method::POST,
            format!("{GITHUB_API}/repos/{owner}/{repo}/releases"),
        )
        .json(&release_payload(tag, cfg, project_dir)?),
        "GitHub release create",
    )
}

fn release_payload(tag: &str, cfg: &GithubReleasesConfig, project_dir: &Path) -> Result<Value> {
    let mut payload = json!({
        "tag_name": tag,
        "name": cfg.name.as_deref().unwrap_or(tag),
        "draft": cfg.draft.unwrap_or(false),
        "prerelease": cfg.prerelease.unwrap_or(false),
    });
    if let Some(target_commitish) = cfg
        .target_commitish
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        payload["target_commitish"] = json!(target_commitish);
    }
    if let Some(make_latest) = cfg
        .make_latest
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        payload["make_latest"] = json!(make_latest);
    }
    if let Some(body) = release_notes(cfg, project_dir)? {
        payload["body"] = json!(body);
    }
    Ok(payload)
}

fn release_notes(cfg: &GithubReleasesConfig, project_dir: &Path) -> Result<Option<String>> {
    if let Some(notes) = cfg.notes.as_ref().filter(|value| !value.trim().is_empty()) {
        return Ok(Some(notes.clone()));
    }
    let Some(path) = cfg
        .notes_file
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(None);
    };
    let path = resolve_project_path(project_dir, path.clone());
    fs::read_to_string(&path)
        .map(Some)
        .with_context(|| format!("failed to read GitHub release notes {}", path.display()))
}

fn release_assets(
    manifest: &ArtifactManifest,
    artifact_path: &Path,
    cfg: &GithubReleasesConfig,
) -> Result<Vec<ReleaseAsset>> {
    let mut assets = manifest
        .artifacts
        .iter()
        .filter_map(release_asset_from_manifest_file)
        .collect::<Vec<_>>();
    if cfg.upload_artifact_manifest.unwrap_or(true) {
        let metadata = fs::metadata(artifact_path)
            .with_context(|| format!("failed to read {}", artifact_path.display()))?;
        assets.push(ReleaseAsset {
            path: artifact_path.to_path_buf(),
            name: artifact_path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or(ARTIFACT_MANIFEST)
                .to_string(),
            mime_type: content_type(artifact_path).to_string(),
            sha256: None,
            size_bytes: metadata.len(),
        });
    }
    Ok(assets)
}

fn release_asset_from_manifest_file(file: &ArtifactFile) -> Option<ReleaseAsset> {
    let path = PathBuf::from(&file.path);
    if !path.is_file() {
        return None;
    }
    let name = path.file_name()?.to_str()?.to_string();
    Some(ReleaseAsset {
        path,
        name,
        mime_type: file.mime_type.clone(),
        sha256: Some(file.sha256.clone()),
        size_bytes: file.size_bytes,
    })
}

fn list_assets(
    client: &Client,
    token: &str,
    owner: &str,
    repo: &str,
    release_id: i64,
) -> Result<Vec<Value>> {
    let value = github_json(
        github_request(
            client,
            Some(token),
            reqwest::Method::GET,
            format!("{GITHUB_API}/repos/{owner}/{repo}/releases/{release_id}/assets?per_page=100"),
        ),
        "GitHub release assets list",
    )?;
    Ok(value.as_array().cloned().unwrap_or_default())
}

fn delete_asset(
    client: &Client,
    token: &str,
    owner: &str,
    repo: &str,
    asset: &Value,
) -> Result<()> {
    let asset_id = asset
        .get("id")
        .and_then(Value::as_i64)
        .context("GitHub release asset did not contain id")?;
    let response = github_request(
        client,
        Some(token),
        reqwest::Method::DELETE,
        format!("{GITHUB_API}/repos/{owner}/{repo}/releases/assets/{asset_id}"),
    )
    .send()
    .context("failed to delete GitHub release asset")?;
    if response.status().is_success() {
        Ok(())
    } else {
        github_json_from_response(response, "GitHub release asset delete").map(|_| ())
    }
}

fn upload_asset(
    client: &Client,
    token: &str,
    owner: &str,
    repo: &str,
    release_id: i64,
    asset: &ReleaseAsset,
) -> Result<Value> {
    let bytes = fs::read(&asset.path)
        .with_context(|| format!("failed to read release asset {}", asset.path.display()))?;
    github_json(
        github_request(
            client,
            Some(token),
            reqwest::Method::POST,
            format!(
                "{GITHUB_UPLOADS}/repos/{owner}/{repo}/releases/{release_id}/assets?name={}",
                query_value(&asset.name)
            ),
        )
        .header(CONTENT_TYPE, asset.mime_type.as_str())
        .header(CONTENT_LENGTH, bytes.len().to_string())
        .body(bytes),
        "GitHub release asset upload",
    )
}

fn github_client() -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(300))
        .build()
        .context("failed to build GitHub HTTP client")
}

fn github_request(
    client: &Client,
    token: Option<&str>,
    method: reqwest::Method,
    url: String,
) -> RequestBuilder {
    let mut request = client
        .request(method, url)
        .header(USER_AGENT, "fission-cli-release/0.1")
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", GITHUB_API_VERSION);
    if let Some(token) = token.filter(|value| !value.trim().is_empty()) {
        request = request.bearer_auth(token.trim());
    }
    request
}

fn github_json(request: RequestBuilder, operation: &str) -> Result<Value> {
    github_json_from_response(
        request
            .send()
            .with_context(|| format!("failed to send {operation} request"))?,
        operation,
    )
}

fn github_json_from_response(response: Response, operation: &str) -> Result<Value> {
    let status = response.status();
    let text = response.text()?;
    if !status.is_success() {
        bail!("{operation} failed with {status}: {text}");
    }
    serde_json::from_str(&text).with_context(|| format!("failed to parse {operation}: {text}"))
}

fn github_release_state(value: &Value) -> String {
    if value.get("draft").and_then(Value::as_bool).unwrap_or(false) {
        "draft".to_string()
    } else if value
        .get("prerelease")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        "prerelease".to_string()
    } else {
        value
            .get("tag_name")
            .and_then(Value::as_str)
            .map(|tag| format!("published:{tag}"))
            .unwrap_or_else(|| "published".to_string())
    }
}

fn asset_json(asset: &ReleaseAsset) -> Value {
    json!({
        "name": asset.name,
        "path": asset.path,
        "mime_type": asset.mime_type,
        "sha256": asset.sha256,
        "size_bytes": asset.size_bytes,
    })
}

fn url_segment(value: &str) -> String {
    percent_encode(value, false)
}

fn query_value(value: &str) -> String {
    percent_encode(value, true)
}

fn percent_encode(value: &str, encode_slash: bool) -> String {
    let mut out = String::new();
    for byte in value.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            b'/' if !encode_slash => out.push('/'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact(path: &Path, relative_path: &str, kind: &str) -> ArtifactFile {
        ArtifactFile {
            kind: kind.to_string(),
            purpose: None,
            platform: None,
            upload_provider: None,
            path: path.display().to_string(),
            relative_path: relative_path.to_string(),
            sha256: "abc".to_string(),
            size_bytes: 3,
            mime_type: content_type(path).to_string(),
        }
    }

    fn manifest(files: Vec<ArtifactFile>) -> ArtifactManifest {
        ArtifactManifest {
            schema_version: 1,
            created_at_unix_seconds: 0,
            project: ArtifactProject {
                app_id: "com.example.app".to_string(),
                name: "app".to_string(),
                version: Some("1.2.3".to_string()),
            },
            target: "linux".to_string(),
            format: "run".to_string(),
            profile: "release".to_string(),
            root_dir: "/tmp".to_string(),
            artifacts: files,
            validation: ArtifactValidation {
                state: "passed".to_string(),
                checks: Vec::new(),
            },
        }
    }

    #[test]
    fn release_tag_defaults_to_version() {
        let manifest = manifest(Vec::new());
        assert_eq!(
            release_tag(&GithubReleasesConfig::default(), &manifest, None),
            Some("v1.2.3".to_string())
        );
        assert_eq!(
            release_tag(&GithubReleasesConfig::default(), &manifest, Some("nightly")),
            Some("nightly".to_string())
        );
    }

    #[test]
    fn release_assets_include_every_manifest_file_and_manifest() {
        let dir = std::env::temp_dir().join(format!(
            "fission-github-release-assets-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let run = dir.join("app.run");
        let html = dir.join("index.html");
        fs::write(&run, b"run").unwrap();
        fs::write(&html, b"html").unwrap();
        let manifest = manifest(vec![
            artifact(&run, "app.run", "asset"),
            artifact(&html, "index.html", "entry"),
        ]);
        let manifest_path = dir.join("artifact-manifest.json");
        fs::write(&manifest_path, b"{}").unwrap();
        let assets =
            release_assets(&manifest, &manifest_path, &GithubReleasesConfig::default()).unwrap();
        let names = assets
            .iter()
            .map(|asset| asset.name.as_str())
            .collect::<Vec<_>>();
        assert!(names.contains(&"app.run"));
        assert!(names.contains(&"index.html"));
        assert!(names.contains(&"artifact-manifest.json"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn upload_asset_names_are_query_encoded() {
        assert_eq!(
            query_value("App 1.0.0 (macOS).zip"),
            "App%201.0.0%20%28macOS%29.zip"
        );
    }
}
