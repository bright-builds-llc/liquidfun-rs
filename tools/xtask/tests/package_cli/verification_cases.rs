use super::*;

pub(super) fn verify_rejects_native_source_extensions() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::ForbiddenNativeSource)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/forbidden-content");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_rejects_graphics_assets() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::ForbiddenGraphics)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/forbidden-content");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_rejects_testbed_content() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::ForbiddenTestbed)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/forbidden-content");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_artifact_rejects_archive_hash_substitution() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    fixture.write_artifact_identity()?;
    fixture.write_platform_policy(&valid_platform_policy(None))?;
    fs::write(&fixture.archive, b"substituted archive bytes")?;

    // Act
    let output = fixture.artifact_command(
        &fixture.root.join("artifact-identity.json"),
        "1.97.0",
        "aarch64-apple-darwin",
    )?;

    // Assert
    assert_failure_category(&output, "package/artifact-hash");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_artifact_rejects_wrong_rust_version() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    fixture.write_artifact_identity()?;
    fixture.write_platform_policy(&valid_platform_policy(None))?;
    let identity_path = fixture.root.join("artifact-identity.json");
    let mut identity: serde_json::Value = serde_json::from_slice(&fs::read(&identity_path)?)?;
    identity["rust_version"] = serde_json::json!("1.93");
    fs::write(&identity_path, serde_json::to_vec_pretty(&identity)?)?;

    // Act
    let output = fixture.artifact_command(&identity_path, "1.97.0", "aarch64-apple-darwin")?;

    // Assert
    assert_failure_category(&output, "package/artifact-identity");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_artifact_rejects_missing_feature() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    fixture.write_artifact_identity()?;
    fixture.write_platform_policy(&valid_platform_policy(None))?;
    let identity_path = fixture.root.join("artifact-identity.json");
    let mut identity: serde_json::Value = serde_json::from_slice(&fs::read(&identity_path)?)?;
    identity["features"] = serde_json::json!(["default"]);
    fs::write(&identity_path, serde_json::to_vec_pretty(&identity)?)?;

    // Act
    let output = fixture.artifact_command(&identity_path, "1.97.0", "aarch64-apple-darwin")?;

    // Assert
    assert_failure_category(&output, "package/artifact-identity");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_artifact_rejects_d1_platform_promotion() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    fixture.write_artifact_identity()?;
    let mut support = valid_platform_policy(None);
    support["evidence_tier"] = serde_json::json!("d1_canonical");
    fixture.write_platform_policy(&support)?;

    // Act
    let output = fixture.artifact_command(
        &fixture.root.join("artifact-identity.json"),
        "1.97.0",
        "aarch64-apple-darwin",
    )?;

    // Assert
    assert_failure_category(&output, "package/platform-policy");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_artifact_rejects_fixture_promotion_capability() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    fixture.write_artifact_identity()?;
    let mut support = valid_platform_policy(None);
    support["fixture_promotion"] = serde_json::json!("d1_canonical");
    fixture.write_platform_policy(&support)?;

    // Act
    let output = fixture.artifact_command(
        &fixture.root.join("artifact-identity.json"),
        "1.97.0",
        "aarch64-apple-darwin",
    )?;

    // Assert
    assert_failure_category(&output, "package/platform-policy");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_artifact_builds_exact_bytes_on_a_durable_native_target() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    fixture.write_artifact_identity()?;
    fixture.write_platform_policy(&valid_platform_policy(None))?;

    // Act
    let output = fixture.artifact_command(
        &fixture.root.join("artifact-identity.json"),
        "1.97.0",
        "aarch64-apple-darwin",
    )?;

    // Assert
    assert_success(&output);
    assert!(fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_artifact_accepts_fresh_conditional_native_evidence() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    fixture.write_artifact_identity()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    fixture.write_platform_policy(&valid_platform_policy(Some((now, now + 7_776_000))))?;

    // Act
    let output = fixture.artifact_command(
        &fixture.root.join("artifact-identity.json"),
        "1.97.0",
        "x86_64-apple-darwin",
    )?;

    // Assert
    assert_success(&output);
    assert!(fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_artifact_rejects_stale_conditional_native_evidence() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    fixture.write_artifact_identity()?;
    fixture.write_platform_policy(&valid_platform_policy(Some((1, 7_776_001))))?;

    // Act
    let output = fixture.artifact_command(
        &fixture.root.join("artifact-identity.json"),
        "1.97.0",
        "x86_64-apple-darwin",
    )?;

    // Assert
    assert_failure_category(&output, "package/platform-evidence");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_artifact_rejects_missing_conditional_native_evidence() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    fixture.write_artifact_identity()?;
    fixture.write_platform_policy(&valid_platform_policy(None))?;

    // Act
    let output = fixture.artifact_command(
        &fixture.root.join("artifact-identity.json"),
        "1.97.0",
        "x86_64-apple-darwin",
    )?;

    // Assert
    assert_failure_category(&output, "package/platform-evidence");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn ci_keeps_the_focused_headless_gate_submodule_free_and_before_visual_work() {
    // Arrange
    let workflow = include_str!("../../../../.github/workflows/ci.yml");

    // Act
    let gate = workflow
        .find("Prove headless workflows and consumer package isolation")
        .expect("CI should name the focused gate");
    let package = workflow[gate..]
        .find("cargo xtask package verify")
        .expect("focused gate should verify the packaged crate");
    let maybe_visual = workflow.find("visual");

    // Assert
    assert!(workflow.matches("submodules: false").count() >= 3);
    assert!(workflow[gate..].contains("--test phase11_public_observability"));
    assert!(workflow[gate..].contains("--test headless_catalog"));
    assert!(workflow[gate..].contains("--test package_cli"));
    assert!(workflow.contains("cargo xtask inventory corpus check-snapshot"));
    assert!(workflow.contains("cargo xtask inventory corpus check-closure"));
    assert!(workflow.contains("cargo xtask inventory corpus generate-report"));
    assert!(workflow.contains("git diff --exit-code -- UPSTREAM-CORPUS.md"));
    assert!(workflow.contains("cargo build -p liquidfun-testbed --all-targets --all-features"));
    assert!(workflow.contains("cargo test -p liquidfun-testbed --all-features"));
    assert!(workflow.contains("DISPLAY: \"\""));
    assert!(workflow.contains("WAYLAND_DISPLAY: \"\""));
    assert!(maybe_visual.is_none_or(|visual| gate + package < visual));
}

pub(super) fn phase11_decisions_and_requirements_have_audited_evidence() {
    // Arrange
    let testing = include_str!("../../../../TESTING.md");

    // Act
    let maybe_audit = testing
        .split("## Phase 11 decision and requirement audit")
        .nth(1);

    // Assert
    let audit = maybe_audit.expect("TESTING.md should contain the final Phase 11 audit");
    for decision in 1..=26 {
        let id = format!("`D-{decision:02}`");
        assert!(audit.contains(&id), "missing {id} from the Phase 11 audit");
    }
    for requirement in [
        "RIGD-10", "TEST-03", "EXMP-01", "EXMP-02", "EXMP-03", "EXMP-04", "EXMP-05", "EXMP-06",
    ] {
        let id = format!("`{requirement}`");
        assert!(audit.contains(&id), "missing {id} from the Phase 11 audit");
    }
}

pub(super) fn advisory_policy_has_no_waiver_after_renderer_replacement() -> TestResult {
    // Arrange
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let deny: toml::Value =
        toml::from_str(&fs::read_to_string(repository_root.join("deny.toml"))?)?;
    let liquidfun: toml::Value = toml::from_str(&fs::read_to_string(
        repository_root.join("crates/liquidfun/Cargo.toml"),
    )?)?;
    let testbed: toml::Value = toml::from_str(&fs::read_to_string(
        repository_root.join("crates/liquidfun-testbed/Cargo.toml"),
    )?)?;

    // Act
    let ignored = deny["advisories"]["ignore"]
        .as_array()
        .expect("advisory ignores should be an array")
        .iter()
        .map(|value| value.as_str().expect("advisory IDs should be strings"))
        .collect::<Vec<_>>();
    let liquidfun_dependencies = liquidfun["dependencies"]
        .as_table()
        .expect("liquidfun dependencies should be a table");

    // Assert
    assert!(ignored.is_empty());
    assert!(!liquidfun_dependencies.contains_key("macroquad"));
    assert_eq!(testbed["package"]["publish"].as_bool(), Some(false));
    let testbed_dependencies = testbed["dependencies"]
        .as_table()
        .expect("testbed dependencies should be a table");
    assert_eq!(
        testbed_dependencies["eframe"]["version"].as_str(),
        Some("=0.35.0")
    );
    assert_eq!(
        testbed_dependencies["eframe"]["default-features"].as_bool(),
        Some(false)
    );
    assert!(testbed_dependencies["eframe"]["features"].is_array());
    assert!(testbed_dependencies["egui"].is_str());
    assert!(testbed_dependencies["tiny-skia"].is_str());
    assert!(!testbed_dependencies.contains_key("macroquad"));
    Ok(())
}

pub(super) fn verify_rejects_private_or_graphical_dependencies_from_consumer_metadata() -> TestResult
{
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    let mut metadata: serde_json::Value = serde_json::from_slice(&fs::read(&fixture.metadata)?)?;
    metadata["packages"][0]["dependencies"] = serde_json::json!([{
        "name": "liquidfun-differential",
        "kind": null
    }]);
    fs::write(&fixture.metadata, serde_json::to_vec(&metadata)?)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/dependency-graph");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_rejects_more_than_one_default_publishable_package() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    let mut metadata: serde_json::Value = serde_json::from_slice(&fs::read(&fixture.metadata)?)?;
    metadata["workspace_default_members"] = serde_json::json!([
        "liquidfun 0.0.0 (path+file:///fixture/liquidfun)",
        "private 0.0.0"
    ]);
    fs::write(&fixture.metadata, serde_json::to_vec(&metadata)?)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/default-members");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_rejects_a_second_publishable_workspace_package() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    let mut metadata: serde_json::Value = serde_json::from_slice(&fs::read(&fixture.metadata)?)?;
    metadata["packages"]
        .as_array_mut()
        .expect("fixture packages should be an array")
        .push(serde_json::json!({
            "id": "private 0.0.0 (path+file:///fixture/private)",
            "name": "private",
            "publish": null,
            "manifest_path": fixture.root.join("crates/private/Cargo.toml"),
            "dependencies": [],
            "features": {"default": []}
        }));
    metadata["workspace_members"] = serde_json::json!([
        "liquidfun 0.0.0 (path+file:///fixture/liquidfun)",
        "private 0.0.0 (path+file:///fixture/private)"
    ]);
    fs::write(&fixture.metadata, serde_json::to_vec(&metadata)?)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/publish-policy");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_accepts_archive_with_matching_license() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    assert!(fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_rejects_forbidden_package_files() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::ForbiddenContent)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/forbidden-content");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_rejects_parent_traversal_before_building() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::ParentTraversal)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/archive-path");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn verify_rejects_absolute_paths_before_building() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::AbsolutePath)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/archive-path");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}
