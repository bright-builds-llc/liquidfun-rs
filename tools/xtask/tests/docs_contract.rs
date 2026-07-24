//! Command-level coverage for the machine-audited testing-layer contract.

use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const LAYERS: [&str; 12] = [
    "unit",
    "integration/API",
    "doctest",
    "upstream compatibility",
    "differential",
    "property",
    "checked-in regression",
    "fuzz",
    "Miri/UB-aliasing",
    "native sanitizer",
    "benchmark",
    "coverage",
];
const CONTRACT_DOCUMENTS: [&str; 7] = [
    "ARCHITECTURE.md",
    "CONTRIBUTING.md",
    "TESTING.md",
    "COMPATIBILITY.md",
    "README.md",
    "RELEASE.md",
    "SAFETY.md",
];
const CONTRACT_SUPPORT_FILES: [&str; 3] = [
    "crates/liquidfun/src/lib.rs",
    "protocol/fixtures/accepted/rigid-world-request.jsonl",
    "reference/compatibility.json",
];
static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

type TestResult = Result<(), Box<dyn Error>>;

struct DocsFixture {
    root: PathBuf,
}

impl DocsFixture {
    fn new() -> io::Result<Self> {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = workspace_root().join(format!(
            "target/xtask-docs-fixtures/{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&root)?;
        for document in CONTRACT_DOCUMENTS {
            let source = workspace_root().join(document);
            if source.is_file() {
                fs::copy(source, root.join(document))?;
            }
        }
        for support_file in CONTRACT_SUPPORT_FILES {
            let destination = root.join(support_file);
            let Some(parent) = destination.parent() else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "contract support file must have a parent directory",
                ));
            };
            fs::create_dir_all(parent)?;
            fs::copy(workspace_root().join(support_file), destination)?;
        }
        Ok(Self { root })
    }

    fn command(&self) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["docs", "check"])
            .env("LIQUIDFUN_XTASK_ROOT", &self.root)
            .output()
    }

    fn remove_layer(&self, layer: &str) -> io::Result<()> {
        self.rewrite(|line| (!table_row_has_layer(line, layer)).then(|| line.to_owned()))
    }

    fn duplicate_layer(&self, layer: &str) -> io::Result<()> {
        self.rewrite(|line| {
            if table_row_has_layer(line, layer) {
                Some(format!("{line}\n{line}"))
            } else {
                Some(line.to_owned())
            }
        })
    }

    fn replace_cell(&self, layer: &str, index: usize, value: &str) -> io::Result<()> {
        let mut found = false;
        self.rewrite(|line| {
            if !table_row_has_layer(line, layer) {
                return Some(line.to_owned());
            }
            found = true;
            let mut cells = parse_row(line).ok()?;
            let cell = cells.get_mut(index)?;
            value.clone_into(cell);
            Some(format!("| {} |", cells.join(" | ")))
        })?;
        if !found {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("testing layer `{layer}` is absent from fixture"),
            ));
        }
        Ok(())
    }

    fn rewrite(&self, mut map_line: impl FnMut(&str) -> Option<String>) -> io::Result<()> {
        let path = self.root.join("TESTING.md");
        let contents = fs::read_to_string(&path)?;
        let rewritten = contents
            .lines()
            .filter_map(&mut map_line)
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(path, format!("{rewritten}\n"))
    }

    fn cleanup(self) -> io::Result<()> {
        fs::remove_dir_all(self.root)
    }

    fn replace_document_text(&self, document: &str, from: &str, to: &str) -> io::Result<()> {
        let path = self.root.join(document);
        let contents = fs::read_to_string(&path)?;
        if !contents.contains(from) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{document} does not contain expected contract text `{from}`"),
            ));
        }
        fs::write(path, contents.replace(from, to))
    }
}

#[test]
fn check_accepts_repository_testing_contract() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_each_missing_required_layer() -> TestResult {
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

#[test]
fn check_rejects_duplicate_layer() -> TestResult {
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

#[test]
fn check_rejects_every_blank_required_column() -> TestResult {
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

#[test]
fn check_rejects_invalid_status() -> TestResult {
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

#[test]
fn check_rejects_invalid_placement() -> TestResult {
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

#[test]
fn check_rejects_missing_semantic_markers() -> TestResult {
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

#[test]
fn check_rejects_forbidden_placeholder_terms() -> TestResult {
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

#[test]
fn check_rejects_missing_phase4_contract_in_each_document() -> TestResult {
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

#[test]
fn phase5_contract_accepts_repository_documents() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn phase5_contract_rejects_missing_contract_in_each_document() -> TestResult {
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

#[test]
fn phase5_contract_rejects_false_surface_and_maturity_claims() -> TestResult {
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

#[test]
fn phase5_compatibility_report_matches_authoritative_ledger() -> TestResult {
    // Arrange
    let mut command = Command::new(env!("CARGO_BIN_EXE_xtask"));
    command.args(["inventory", "check-report"]);

    // Act
    let output = command.output()?;

    // Assert
    assert_success(&output);
    Ok(())
}

#[test]
fn phase7_contract_rejects_missing_solver_promotion() -> TestResult {
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

#[test]
fn phase6_contract_accepts_repository_documents() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn phase6_contract_rejects_missing_contract_in_each_document() -> TestResult {
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

#[test]
fn phase6_contract_rejects_missing_non_dynamic_admission_witnesses() -> TestResult {
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

#[test]
fn phase6_contract_rejects_missing_sanitizer_execution_evidence() -> TestResult {
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

#[test]
fn phase6_contract_rejects_missing_verifier_gap_closure() -> TestResult {
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

#[test]
fn phase6_contract_rejects_missing_second_round_boundary_contracts() -> TestResult {
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

#[test]
fn phase6_contract_rejects_deferred_surface_and_identity_overclaims() -> TestResult {
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

#[test]
fn phase7_contract_accepts_repository_documents() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn phase7_contract_rejects_missing_contract_in_each_document() -> TestResult {
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

#[test]
fn phase7_contract_rejects_unreviewed_maturity_and_private_state_claims() -> TestResult {
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

#[test]
fn phase8_contract_accepts_repository_documents() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn phase8_contract_rejects_missing_evidence_identity_in_each_document() -> TestResult {
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

#[test]
fn phase8_contract_rejects_platform_demotion() -> TestResult {
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

#[test]
fn phase8_contract_rejects_platform_evidence_drift() -> TestResult {
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

#[test]
fn phase8_contract_rejects_broader_maturity_claims() -> TestResult {
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

#[test]
fn phase12_publication_contract_accepts_repository_documents() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn phase12_publication_contract_rejects_missing_contract_in_each_document() -> TestResult {
    // Arrange, Act, Assert
    for (document, marker) in [
        ("README.md", "## Maturity and evidence"),
        ("CONTRIBUTING.md", "### Markdown"),
        ("RELEASE.md", "## Freeze the source candidate"),
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

#[test]
fn phase12_publication_contract_rejects_stale_maturity_claims() -> TestResult {
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

#[test]
fn check_rejects_absolute_user_paths() -> TestResult {
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

#[test]
fn oracle_workflow_only_cancels_superseded_code_change_runs() -> TestResult {
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

#[test]
fn oracle_workflow_fails_when_failure_evidence_is_missing() -> TestResult {
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

#[test]
fn oracle_workflow_bounds_sanitizer_failure_artifacts() -> TestResult {
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

#[test]
fn oracle_workflow_fetches_full_history_for_every_checkout() -> TestResult {
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

#[test]
fn windows_oracle_step_fails_fast_on_native_command_errors() -> TestResult {
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

fn parse_row(line: &str) -> io::Result<Vec<String>> {
    let trimmed = line.trim();
    let Some(contents) = trimmed
        .strip_prefix('|')
        .and_then(|contents| contents.strip_suffix('|'))
    else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid Markdown table row `{trimmed}`"),
        ));
    };
    Ok(contents
        .split('|')
        .map(|cell| cell.trim().to_owned())
        .collect())
}

fn table_row_has_layer(line: &str, layer: &str) -> bool {
    parse_row(line)
        .ok()
        .and_then(|cells| cells.into_iter().next())
        .is_some_and(|first| first == layer)
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure(output: &Output, category: &str) {
    assert!(!output.status.success(), "command unexpectedly succeeded");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(category),
        "expected `{category}` in stderr:\n{stderr}"
    );
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask manifest directory must be nested beneath the workspace root")
}
