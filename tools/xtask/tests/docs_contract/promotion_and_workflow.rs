use super::*;

pub(super) fn phase7_contract_accepts_repository_documents() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn phase7_contract_rejects_missing_contract_in_each_document() -> TestResult {
    // Arrange, Act, Assert
    for (document, marker) in [
        (
            "crates/liquidfun/src/lib.rs",
            "# Phase 7 checked rigid-world contract",
        ),
        (
            "ARCHITECTURE.md",
            "## Phase 7 rigid solver, world operations, and CCD boundaries",
        ),
        ("TESTING.md", "## Phase 7 rigid-world comparison policy"),
        (
            "COMPATIBILITY.md",
            "`public-api.liquidfun-box2d-box2d-dynamics-b2island-h`",
        ),
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text(document, marker, "removed-phase7-contract-marker")?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/phase7-contract");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn phase7_contract_rejects_unreviewed_maturity_and_private_state_claims() -> TestResult {
    // Arrange, Act, Assert
    for claim in [
        "complete rigid solver",
        "complete rigid-world parity",
        "D3-validated",
        "D3 validated",
        "Phase 7 platform validated",
        "multi-platform parity",
        "query callbacks are ordered",
        "ray callbacks are ordered",
        "public CCD cache",
        "public TOI counter",
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text(
            "README.md",
            "## Architecture and evidence",
            &format!("False claim: {claim}\n\n## Architecture and evidence"),
        )?;
        let output = fixture.command()?;
        let category = if claim == "Phase 7 platform validated" {
            "docs/phase6-overclaim"
        } else {
            "docs/phase7-overclaim"
        };
        assert_failure(&output, category);
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn phase8_contract_accepts_repository_documents() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn phase8_contract_rejects_missing_evidence_identity_in_each_document() -> TestResult {
    // Arrange, Act, Assert
    for (document, marker) in [
        (
            "crates/liquidfun/src/lib.rs",
            "# Phase 8 joints, rope, and callback contract",
        ),
        (
            "ARCHITECTURE.md",
            "phase8-canonical-29383445374-beb98bd74b1d26ab0a96c6be33ce1926d349abf0",
        ),
        ("TESTING.md", "## Phase 8 canonical rigid-world sign-off"),
        ("COMPATIBILITY.md", "`subsystem.joints`"),
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text(document, marker, "removed-phase8-contract-marker")?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/phase8-contract");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn phase8_contract_rejects_platform_demotion() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;
    fixture.replace_document_text(
        "COMPATIBILITY.md",
        "| `subsystem.joints` | `liquidfun/Box2D/Box2D/Dynamics/Joints` | `liquidfun::dynamics::joints` | applicable | yes | yes | yes | yes | yes | yes | yes | no |",
        "| `subsystem.joints` | `liquidfun/Box2D/Box2D/Dynamics/Joints` | `liquidfun::dynamics::joints` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
    )?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure(&output, "docs/phase8-contract");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn phase8_contract_rejects_platform_evidence_drift() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;
    fixture.replace_document_text(
        "reference/compatibility.json",
        "phase8-canonical-29383445374-beb98bd74b1d26ab0a96c6be33ce1926d349abf0/identity.json",
        "phase8-canonical-29374708477-533c2ccf97b3921079baf7c339ddb4dad1a4038b/identity.json",
    )?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure(&output, "docs/phase8-evidence");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn phase8_contract_rejects_broader_maturity_claims() -> TestResult {
    // Arrange, Act, Assert
    for claim in [
        "RIGD-10 is complete",
        "particle parity is complete",
        "cross-platform parity is complete",
        "performance is validated",
        "the testbed is complete",
        "release ready",
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text(
            "README.md",
            "## Architecture and evidence",
            &format!("False claim: {claim}\n\n## Architecture and evidence"),
        )?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/phase8-overclaim");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn phase12_publication_contract_accepts_repository_documents() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn phase12_publication_contract_rejects_missing_contract_in_each_document() -> TestResult
{
    // Arrange, Act, Assert
    for (document, marker) in [
        ("README.md", "not release-ready"),
        ("CONTRIBUTING.md", "### Markdown"),
        ("COMPATIBILITY.md", "run-bound release attestation"),
        ("RELEASE.md", "## Current release status"),
        ("SAFETY.md", "## Renderer and oracle isolation"),
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text(document, marker, "removed-phase12-contract-marker")?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/phase12-public-contract");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn phase12_publication_contract_rejects_stale_maturity_claims() -> TestResult {
    // Arrange, Act, Assert
    for claim in [
        "early vertical-slice stage",
        "Do not use this crate for simulation yet",
        "particles remain pending",
        "the testbed remains pending",
        "release audit has passed",
        "faster than C++",
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text(
            "README.md",
            "## Architecture and evidence",
            &format!("Stale claim: {claim}\n\n## Architecture and evidence"),
        )?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/current-overclaim");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn check_rejects_absolute_user_paths() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;
    fixture.replace_document_text(
        "README.md",
        "## Architecture and evidence",
        "Local evidence: /Users/example/private\n\n## Architecture and evidence",
    )?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure(&output, "docs/local-path");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn oracle_workflow_only_cancels_superseded_code_change_runs() -> TestResult {
    // Arrange
    let workflow = fs::read_to_string(workspace_root().join(".github/workflows/oracle.yml"))?;

    // Act
    let maybe_policy = workflow
        .lines()
        .find(|line| line.trim_start().starts_with("cancel-in-progress:"));

    // Assert
    assert_eq!(
        maybe_policy.map(str::trim),
        Some(
            "cancel-in-progress: ${{ github.event_name == 'pull_request' || github.event_name == 'push' }}"
        )
    );
    Ok(())
}

pub(super) fn oracle_workflow_fails_when_failure_evidence_is_missing() -> TestResult {
    // Arrange
    let workflow = fs::read_to_string(workspace_root().join(".github/workflows/oracle.yml"))?;

    // Act
    let maybe_missing_file_policy = workflow
        .lines()
        .find(|line| line.trim_start().starts_with("if-no-files-found:"));

    // Assert
    assert_eq!(
        maybe_missing_file_policy.map(str::trim),
        Some("if-no-files-found: error")
    );
    Ok(())
}

pub(super) fn oracle_workflow_bounds_sanitizer_failure_artifacts() -> TestResult {
    // Arrange
    let workflow = fs::read_to_string(workspace_root().join(".github/workflows/oracle.yml"))?;
    let upload_step = workflow
        .split("      - name: Upload bounded differential failure evidence")
        .nth(1)
        .and_then(|suffix| suffix.split("\n      - name: ").next())
        .expect("sanitizer failure upload step must remain present");

    // Act
    let artifact_paths = upload_step
        .lines()
        .filter_map(|line| line.trim().strip_prefix("path:"))
        .map(str::trim)
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(artifact_paths, ["target/differential/failures"]);
    assert!(upload_step.contains("if: failure()"));
    assert!(upload_step.contains("if-no-files-found: error"));
    assert!(upload_step.contains("retention-days: 7"));
    Ok(())
}

pub(super) fn oracle_workflow_fetches_full_history_for_every_checkout() -> TestResult {
    // Arrange
    let workflow = fs::read_to_string(workspace_root().join(".github/workflows/oracle.yml"))?;

    // Act
    let checkout_steps = workflow
        .split("      - name: ")
        .filter(|step| step.contains("uses: actions/checkout@"))
        .collect::<Vec<_>>();

    // Assert
    assert!(
        !checkout_steps.is_empty(),
        "Oracle CI must check out sources"
    );
    assert!(
        checkout_steps
            .iter()
            .all(|step| step.lines().any(|line| line.trim() == "fetch-depth: 0")),
        "every Oracle checkout must fetch history for provenance validation"
    );
    Ok(())
}

pub(super) fn windows_oracle_step_fails_fast_on_native_command_errors() -> TestResult {
    // Arrange
    let workflow = fs::read_to_string(workspace_root().join(".github/workflows/oracle.yml"))?;
    let windows_job = workflow
        .split("  portability-windows:")
        .nth(1)
        .expect("Windows portability job must remain in the Oracle workflow");
    let build_step = windows_job
        .split("      - name: Verify and build with the native Visual Studio environment")
        .nth(1)
        .expect("Windows verify-and-build step must remain present");

    // Act
    let error_preference = build_step
        .find("$ErrorActionPreference = \"Stop\"")
        .expect("PowerShell errors must stop the Windows build step");
    let native_preference = build_step
        .find("$PSNativeCommandUseErrorActionPreference = $true")
        .expect("native command failures must stop the Windows build step");
    let first_command = build_step
        .find("cargo xtask upstream verify")
        .expect("Windows build step must verify the upstream checkout");

    // Assert
    assert!(error_preference < first_command);
    assert!(native_preference < first_command);
    Ok(())
}
