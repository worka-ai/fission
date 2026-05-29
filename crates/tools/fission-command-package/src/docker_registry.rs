use super::*;

pub(super) fn publish(
    options: &DistributeOptions,
    config: &PublishManifest,
    artifact_path: &Path,
    manifest: &ArtifactManifest,
) -> Result<DistributionReceipt> {
    let cfg = docker_registry_config(config, &options.site)?;
    let metadata_tags = docker_metadata_tags(manifest)?;
    let push_tags = options
        .deploy
        .as_ref()
        .filter(|tag| !tag.trim().is_empty())
        .map(|tag| vec![tag.clone()])
        .or_else(|| non_empty_vec(cfg.tags))
        .unwrap_or_else(|| metadata_tags.clone());
    if push_tags.is_empty() {
        bail!("docker-registry publish requires [package.docker].tags, [distribution.docker_registry.<site>].tags, or image-metadata.json tags");
    }
    if find_in_path("docker").is_none() {
        bail!("docker was not found on PATH; run readiness distribute for remediation");
    }

    if options.dry_run {
        println!("Would push Docker image tags: {}", push_tags.join(", "));
        return Ok(DistributionReceipt {
            schema_version: 1,
            created_at_unix_seconds: now_unix_seconds(),
            provider: "docker-registry".to_string(),
            site: options.site.clone(),
            action: "publish".to_string(),
            artifact_manifest: Some(artifact_path.display().to_string()),
            deployment_id: None,
            canonical_url: push_tags.first().cloned(),
            preview_url: None,
            custom_domain: None,
            status: "dry-run".to_string(),
            stdout: None,
            stderr: None,
            manual_follow_up: Vec::new(),
        });
    }

    let source_tag = metadata_tags
        .first()
        .cloned()
        .or_else(|| push_tags.first().cloned())
        .context("docker image metadata did not contain a usable source tag")?;
    let mut stdout = String::new();
    let mut stderr = String::new();
    for tag in &push_tags {
        if tag != &source_tag && !metadata_tags.iter().any(|item| item == tag) {
            let output = Command::new("docker")
                .args(["tag", source_tag.as_str(), tag.as_str()])
                .output()
                .with_context(|| format!("failed to run docker tag {source_tag} {tag}"))?;
            stdout.push_str(&String::from_utf8_lossy(&output.stdout));
            stderr.push_str(&String::from_utf8_lossy(&output.stderr));
            if !output.status.success() {
                bail!("docker tag failed with {}", output.status);
            }
        }
        let output = Command::new("docker")
            .args(["push", tag.as_str()])
            .output()
            .with_context(|| format!("failed to run docker push {tag}"))?;
        stdout.push_str(&String::from_utf8_lossy(&output.stdout));
        stderr.push_str(&String::from_utf8_lossy(&output.stderr));
        if !output.status.success() {
            bail!("docker push {tag} failed with {}", output.status);
        }
    }

    Ok(DistributionReceipt {
        schema_version: 1,
        created_at_unix_seconds: now_unix_seconds(),
        provider: "docker-registry".to_string(),
        site: options.site.clone(),
        action: "publish".to_string(),
        artifact_manifest: Some(artifact_path.display().to_string()),
        deployment_id: push_tags.first().cloned(),
        canonical_url: push_tags.first().cloned(),
        preview_url: None,
        custom_domain: None,
        status: "published".to_string(),
        stdout: Some(stdout),
        stderr: (!stderr.trim().is_empty()).then_some(stderr),
        manual_follow_up: vec![
            "Keep the artifact manifest and pushed image digest with the release record."
                .to_string(),
        ],
    })
}

pub(super) fn status(
    options: &DistributeOptions,
    config: &PublishManifest,
) -> Result<DistributionReceipt> {
    let cfg = docker_registry_config(config, &options.site)?;
    let tag = cfg
        .tags
        .as_ref()
        .and_then(|tags| tags.iter().find(|tag| !tag.trim().is_empty()))
        .context("docker-registry status requires distribution.docker_registry.<site>.tags")?;
    command_status_receipt(
        options,
        "docker-registry",
        "docker",
        vec![
            "manifest".to_string(),
            "inspect".to_string(),
            tag.to_string(),
        ],
    )
}

pub(super) fn readiness(
    site: &str,
    artifact: Option<&Path>,
    config: &PublishManifest,
    checks: &mut Vec<ReadinessCheck>,
) -> Result<()> {
    let cfg = docker_registry_config(config, site)?;
    checks.push(check_tool(
        "release.docker_registry.docker_available",
        "docker",
        "Install Docker, authenticate with `docker login`, and ensure the Docker engine is reachable.",
    ));
    let configured_tags = cfg
        .tags
        .as_ref()
        .map(|tags| tags.iter().filter(|tag| !tag.trim().is_empty()).count())
        .unwrap_or(0);
    let metadata_tags = artifact
        .and_then(|artifact| read_artifact_manifest(artifact).ok())
        .and_then(|manifest| docker_metadata_tags(&manifest).ok())
        .unwrap_or_default();
    checks.push(check(
        "release.docker_registry.tags_available",
        CheckSeverity::Error,
        if configured_tags > 0 || !metadata_tags.is_empty() {
            CheckStatus::Passed
        } else {
            CheckStatus::Missing
        },
        "Docker image tags are available",
        Some(format!(
            "configured tags: {configured_tags}, package tags: {}",
            metadata_tags.len()
        )),
        vec!["Set [package.docker].tags before packaging, set [distribution.docker_registry.<site>].tags, or rebuild the Docker image package."],
    ));
    if let Some(path) = artifact {
        let manifest = read_artifact_manifest(path)?;
        checks.push(check(
            "release.docker_registry.artifact_is_docker_image",
            CheckSeverity::Error,
            if manifest.format == PackageFormat::DockerImage.as_str() {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            },
            "artifact manifest describes a docker-image package",
            Some(format!("format = {}", manifest.format)),
            vec!["Run `fission package --target server --format docker-image --release` or `fission package --target site --format docker-image --release`."],
        ));
        checks.push(check_path(
            "release.docker_registry.image_metadata_exists",
            Path::new(&manifest.root_dir).join("image-metadata.json"),
            "image metadata exists",
            "Rebuild the Docker image package so image-metadata.json is present.",
        ));
    }
    Ok(())
}

fn docker_metadata_tags(manifest: &ArtifactManifest) -> Result<Vec<String>> {
    let metadata_path = Path::new(&manifest.root_dir).join("image-metadata.json");
    let data = fs::read_to_string(&metadata_path)
        .with_context(|| format!("failed to read {}", metadata_path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&data)
        .with_context(|| format!("failed to parse {}", metadata_path.display()))?;
    Ok(value
        .get("tags")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .filter(|tag| !tag.trim().is_empty())
        .map(str::to_string)
        .collect())
}

fn non_empty_vec(values: Option<Vec<String>>) -> Option<Vec<String>> {
    let values = values?
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();
    (!values.is_empty()).then_some(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_tags_are_read_from_image_metadata() {
        let root = std::env::temp_dir().join(format!(
            "fission-docker-registry-metadata-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("image-metadata.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "schema_version": 1,
                "tags": ["registry.example.com/app:1.2.3", "registry.example.com/app:latest"]
            }))
            .unwrap(),
        )
        .unwrap();
        let manifest = ArtifactManifest {
            schema_version: 1,
            created_at_unix_seconds: 0,
            project: ArtifactProject {
                app_id: "com.example.app".to_string(),
                name: "app".to_string(),
                version: Some("1.2.3".to_string()),
            },
            target: "server".to_string(),
            format: "docker-image".to_string(),
            profile: "release".to_string(),
            root_dir: root.display().to_string(),
            artifacts: Vec::new(),
            validation: ArtifactValidation {
                state: "passed".to_string(),
                checks: Vec::new(),
            },
        };

        let tags = docker_metadata_tags(&manifest).unwrap();
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0], "registry.example.com/app:1.2.3");

        let _ = fs::remove_dir_all(root);
    }
}
