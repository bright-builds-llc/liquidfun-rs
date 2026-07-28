use super::*;

pub(super) fn check_accepts_repository_testing_contract() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_each_missing_required_layer() -> TestResult {
    // Arrange, Act, Assert
    for layer in LAYERS {
        let fixture = DocsFixture::new()?;
        fixture.remove_layer(layer)?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/layer");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn check_rejects_duplicate_layer() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;
    fixture.duplicate_layer("unit")?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure(&output, "docs/layer");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_every_blank_required_column() -> TestResult {
    // Arrange, Act, Assert
    for index in 0..9 {
        let fixture = DocsFixture::new()?;
        fixture.replace_cell("unit", index, "")?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/schema");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn check_rejects_invalid_status() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;
    fixture.replace_cell("unit", 1, "active")?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure(&output, "docs/status");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_invalid_placement() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;
    fixture.replace_cell("unit", 7, "nightly")?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure(&output, "docs/placement");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn check_rejects_missing_semantic_markers() -> TestResult {
    // Arrange, Act, Assert
    for (index, value) in [
        (3, "Documented command."),
        (4, "Installed toolchain."),
        (5, "Standard diagnostics."),
        (6, "Run it again."),
        (8, "Some evidence."),
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_cell("unit", index, value)?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/content");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn check_rejects_forbidden_placeholder_terms() -> TestResult {
    // Arrange, Act, Assert
    for placeholder in ["TODO", "TBD", "placeholder", "REPLACE_ME", "replace with"] {
        let fixture = DocsFixture::new()?;
        fixture.replace_cell("unit", 2, placeholder)?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/placeholder");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn check_rejects_missing_phase4_contract_in_each_document() -> TestResult {
    // Arrange, Act, Assert
    for (document, marker) in [
        (
            "ARCHITECTURE.md",
            "## Phase 4 math and numerical boundaries",
        ),
        ("TESTING.md", "The four float policies are"),
        ("COMPATIBILITY.md", "`subsystem.common-math-and-settings`"),
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text(document, marker, "removed-contract-marker")?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/phase4-contract");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn phase5_contract_accepts_repository_documents() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn phase5_contract_rejects_missing_contract_in_each_document() -> TestResult {
    // Arrange, Act, Assert
    for (document, marker) in [
        ("ARCHITECTURE.md", "## Phase 5 collision boundaries"),
        ("TESTING.md", "## Phase 5 collision comparison policy"),
        ("COMPATIBILITY.md", "`subsystem.collision-broad-phase`"),
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text(document, marker, "removed-phase5-contract-marker")?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/phase5-contract");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn phase5_contract_rejects_false_surface_and_maturity_claims() -> TestResult {
    // Arrange, Act, Assert
    for claim in [
        "full parity",
        "production ready",
        "all platforms validated",
        "query order is guaranteed",
        "global epsilon",
        "cargo xtask differential d0",
        "packed contact keys are public",
        "DynamicTree exposes public iteration",
        "Phase 6 is complete",
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text(
            "README.md",
            "## Architecture and evidence",
            &format!("False claim: {claim}\n\n## Architecture and evidence"),
        )?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/phase5-overclaim");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn phase5_compatibility_report_matches_authoritative_ledger() -> TestResult {
    // Arrange
    let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
    command.args(["inventory", "check-report"]);

    // Act
    let output = command.output()?;

    // Assert
    assert_success(&output);
    Ok(())
}

pub(super) fn phase7_contract_rejects_missing_solver_promotion() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;
    fixture.replace_document_text(
        "COMPATIBILITY.md",
        "| `subsystem.rigid-islands-and-solver` | `liquidfun/Box2D/Box2D/Dynamics` | `liquidfun::dynamics` | applicable | yes | yes | yes | yes | yes | yes | yes | no |",
        "| `subsystem.rigid-islands-and-solver` | `liquidfun/Box2D/Box2D/Dynamics` | `liquidfun::dynamics` | applicable | yes | yes | no | no | no | no | no | no |",
    )?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure(&output, "docs/phase7-contract");
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn phase6_contract_accepts_repository_documents() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

pub(super) fn phase6_contract_rejects_missing_contract_in_each_document() -> TestResult {
    // Arrange, Act, Assert
    for (document, marker) in [
        ("ARCHITECTURE.md", "## Phase 6 rigid-world boundaries"),
        ("TESTING.md", "## Phase 6 rigid-world comparison policy"),
        (
            "COMPATIBILITY.md",
            "`public-api.liquidfun-box2d-box2d-dynamics-b2body-h`",
        ),
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text(document, marker, "removed-phase6-contract-marker")?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/phase6-contract");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn phase6_contract_rejects_missing_non_dynamic_admission_witnesses() -> TestResult {
    // Arrange, Act, Assert
    for witness in [
        "static_kinematic_overlap_rejected",
        "kinematic_kinematic_overlap_rejected",
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text(
            "protocol/fixtures/accepted/rigid-world-request.jsonl",
            witness,
            "removed-admission-witness",
        )?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/phase6-contract");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn phase6_contract_rejects_missing_sanitizer_execution_evidence() -> TestResult {
    // Arrange, Act, Assert
    for marker in [
        "ctest --test-dir target/reference/oracle-asan-ubsan --output-on-failure --no-tests=error -R '^liquidfun-reference-protocol$'",
        "cargo xtask differential compare --scenario rigid-world --preset oracle-asan-ubsan --session-profile one-shot",
        "retains failures for seven days",
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text("TESTING.md", marker, "removed-sanitizer-contract")?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/phase6-contract");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn phase6_contract_rejects_missing_verifier_gap_closure() -> TestResult {
    // Arrange, Act, Assert
    for gap_id in [
        "aggregate-mass-atomicity",
        "non-dynamic-contact-admission",
        "ignored-step-parameters",
        "rigid-action-bound-mismatch",
        "invalid-centered-inertia-boundary",
        "rigid-staging-not-integrated",
        "rigid-sanitizer-not-executed",
        "implicit-aggregate-mass-atomicity",
        "zero-centered-inertia-boundary",
        "rigid-fixture-checkout-provenance",
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text("TESTING.md", gap_id, "removed-gap-closure")?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/phase6-contract");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn phase6_contract_rejects_missing_second_round_boundary_contracts() -> TestResult {
    // Arrange, Act, Assert
    for (document, marker) in [
        (
            "ARCHITECTURE.md",
            "`BodyTypeChangeError` and `FixtureDestructionError`",
        ),
        (
            "ARCHITECTURE.md",
            "positive-origin inertia must remain finite and strictly positive",
        ),
        (
            "ARCHITECTURE.md",
            "adapter-source and effective compile-command digests",
        ),
        ("TESTING.md", "local debug/release and replay passes are D2"),
        ("TESTING.md", "same-build byte-identical runs are D0"),
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text(document, marker, "removed-boundary-contract")?;
        let output = fixture.command()?;
        assert_failure(&output, "docs/phase6-contract");
        fixture.cleanup()?;
    }
    Ok(())
}

pub(super) fn phase6_contract_rejects_deferred_surface_and_identity_overclaims() -> TestResult {
    // Arrange, Act, Assert
    for claim in [
        "full rigid parity",
        "public durable contacts",
        "mutable shapes",
        "global epsilon",
        "general solver is implemented",
        "complete island solver",
        "forces are implemented",
        "sleeping is implemented",
        "CCD is implemented",
        "world queries are implemented",
        "world configuration is implemented",
        "joint solving is implemented",
        "platform validated",
        "raw contact identity",
        "raw proxy identity",
    ] {
        let fixture = DocsFixture::new()?;
        fixture.replace_document_text(
            "README.md",
            "## Architecture and evidence",
            &format!("False claim: {claim}\n\n## Architecture and evidence"),
        )?;
        let output = fixture.command()?;
        let category = if claim == "global epsilon" {
            "docs/phase5-overclaim"
        } else {
            "docs/phase6-overclaim"
        };
        assert_failure(&output, category);
        fixture.cleanup()?;
    }
    Ok(())
}
