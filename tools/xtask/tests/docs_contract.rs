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
const CONTRACT_DOCUMENTS: [&str; 4] = [
    "ARCHITECTURE.md",
    "TESTING.md",
    "COMPATIBILITY.md",
    "README.md",
];
const CONTRACT_SUPPORT_FILES: [&str; 1] = ["protocol/fixtures/accepted/rigid-world-request.jsonl"];
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
            fs::copy(workspace_root().join(document), root.join(document))?;
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
        let prefix = format!("| {layer} |");
        self.rewrite(|line| (!line.starts_with(&prefix)).then(|| line.to_owned()))
    }

    fn duplicate_layer(&self, layer: &str) -> io::Result<()> {
        let prefix = format!("| {layer} |");
        self.rewrite(|line| {
            if line.starts_with(&prefix) {
                Some(format!("{line}\n{line}"))
            } else {
                Some(line.to_owned())
            }
        })
    }

    fn replace_cell(&self, layer: &str, index: usize, value: &str) -> io::Result<()> {
        let prefix = format!("| {layer} |");
        let mut found = false;
        self.rewrite(|line| {
            if !line.starts_with(&prefix) {
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
        ("README.md", "bounded Phase 4 math"),
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
        ("README.md", "Phase 5 immutable shape/collision substrate"),
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
    command.args(["inventory", "check"]);

    // Act
    let output = command.output()?;

    // Assert
    assert_success(&output);
    Ok(())
}

#[test]
fn phase6_contract_rejects_broad_solver_promotion() -> TestResult {
    // Arrange
    let fixture = DocsFixture::new()?;
    fixture.replace_document_text(
        "COMPATIBILITY.md",
        "| `subsystem.rigid-islands-and-solver` | `liquidfun/Box2D/Box2D/Dynamics` | `liquidfun::dynamics` | applicable | yes | yes | no | no | no | no | no | no |",
        "| `subsystem.rigid-islands-and-solver` | `liquidfun/Box2D/Box2D/Dynamics` | `liquidfun::dynamics` | applicable | yes | yes | yes | yes | yes | no | yes | no |",
    )?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure(&output, "docs/phase6-contract");
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
        ("README.md", "Phase 6 minimal rigid-world vertical slice"),
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
