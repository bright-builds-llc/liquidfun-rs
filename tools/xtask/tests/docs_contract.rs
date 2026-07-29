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
    core_contracts::check_accepts_repository_testing_contract()
}

#[test]
fn check_rejects_each_missing_required_layer() -> TestResult {
    core_contracts::check_rejects_each_missing_required_layer()
}

#[test]
fn check_rejects_duplicate_layer() -> TestResult {
    core_contracts::check_rejects_duplicate_layer()
}

#[test]
fn check_rejects_every_blank_required_column() -> TestResult {
    core_contracts::check_rejects_every_blank_required_column()
}

#[test]
fn check_rejects_invalid_status() -> TestResult {
    core_contracts::check_rejects_invalid_status()
}

#[test]
fn check_rejects_invalid_placement() -> TestResult {
    core_contracts::check_rejects_invalid_placement()
}

#[test]
fn check_rejects_missing_semantic_markers() -> TestResult {
    core_contracts::check_rejects_missing_semantic_markers()
}

#[test]
fn check_rejects_forbidden_placeholder_terms() -> TestResult {
    core_contracts::check_rejects_forbidden_placeholder_terms()
}

#[test]
fn check_rejects_missing_phase4_contract_in_each_document() -> TestResult {
    core_contracts::check_rejects_missing_phase4_contract_in_each_document()
}

#[test]
fn phase5_contract_accepts_repository_documents() -> TestResult {
    core_contracts::phase5_contract_accepts_repository_documents()
}

#[test]
fn phase5_contract_rejects_missing_contract_in_each_document() -> TestResult {
    core_contracts::phase5_contract_rejects_missing_contract_in_each_document()
}

#[test]
fn phase5_contract_rejects_false_surface_and_maturity_claims() -> TestResult {
    core_contracts::phase5_contract_rejects_false_surface_and_maturity_claims()
}

#[test]
fn phase5_compatibility_report_matches_authoritative_ledger() -> TestResult {
    core_contracts::phase5_compatibility_report_matches_authoritative_ledger()
}

#[test]
fn phase7_contract_rejects_missing_solver_promotion() -> TestResult {
    core_contracts::phase7_contract_rejects_missing_solver_promotion()
}

#[test]
fn phase6_contract_accepts_repository_documents() -> TestResult {
    core_contracts::phase6_contract_accepts_repository_documents()
}

#[test]
fn phase6_contract_rejects_missing_contract_in_each_document() -> TestResult {
    core_contracts::phase6_contract_rejects_missing_contract_in_each_document()
}

#[test]
fn phase6_contract_rejects_missing_non_dynamic_admission_witnesses() -> TestResult {
    core_contracts::phase6_contract_rejects_missing_non_dynamic_admission_witnesses()
}

#[test]
fn phase6_contract_rejects_missing_sanitizer_execution_evidence() -> TestResult {
    core_contracts::phase6_contract_rejects_missing_sanitizer_execution_evidence()
}

#[test]
fn phase6_contract_rejects_missing_verifier_gap_closure() -> TestResult {
    core_contracts::phase6_contract_rejects_missing_verifier_gap_closure()
}

#[test]
fn phase6_contract_rejects_missing_second_round_boundary_contracts() -> TestResult {
    core_contracts::phase6_contract_rejects_missing_second_round_boundary_contracts()
}

#[test]
fn phase6_contract_rejects_deferred_surface_and_identity_overclaims() -> TestResult {
    core_contracts::phase6_contract_rejects_deferred_surface_and_identity_overclaims()
}

#[test]
fn phase7_contract_accepts_repository_documents() -> TestResult {
    promotion_and_workflow::phase7_contract_accepts_repository_documents()
}

#[test]
fn phase7_contract_rejects_missing_contract_in_each_document() -> TestResult {
    promotion_and_workflow::phase7_contract_rejects_missing_contract_in_each_document()
}

#[test]
fn phase7_contract_rejects_unreviewed_maturity_and_private_state_claims() -> TestResult {
    promotion_and_workflow::phase7_contract_rejects_unreviewed_maturity_and_private_state_claims()
}

#[test]
fn phase8_contract_accepts_repository_documents() -> TestResult {
    promotion_and_workflow::phase8_contract_accepts_repository_documents()
}

#[test]
fn phase8_contract_rejects_missing_evidence_identity_in_each_document() -> TestResult {
    promotion_and_workflow::phase8_contract_rejects_missing_evidence_identity_in_each_document()
}

#[test]
fn phase8_contract_rejects_platform_demotion() -> TestResult {
    promotion_and_workflow::phase8_contract_rejects_platform_demotion()
}

#[test]
fn phase8_contract_rejects_platform_evidence_drift() -> TestResult {
    promotion_and_workflow::phase8_contract_rejects_platform_evidence_drift()
}

#[test]
fn phase8_contract_rejects_broader_maturity_claims() -> TestResult {
    promotion_and_workflow::phase8_contract_rejects_broader_maturity_claims()
}

#[test]
fn phase12_publication_contract_accepts_repository_documents() -> TestResult {
    promotion_and_workflow::phase12_publication_contract_accepts_repository_documents()
}

#[test]
fn phase12_publication_contract_rejects_missing_contract_in_each_document() -> TestResult {
    promotion_and_workflow::phase12_publication_contract_rejects_missing_contract_in_each_document()
}

#[test]
fn phase12_publication_contract_rejects_stale_maturity_claims() -> TestResult {
    promotion_and_workflow::phase12_publication_contract_rejects_stale_maturity_claims()
}

#[test]
fn check_rejects_absolute_user_paths() -> TestResult {
    promotion_and_workflow::check_rejects_absolute_user_paths()
}

#[test]
fn oracle_workflow_only_cancels_superseded_code_change_runs() -> TestResult {
    promotion_and_workflow::oracle_workflow_only_cancels_superseded_code_change_runs()
}

#[test]
fn oracle_workflow_fails_when_failure_evidence_is_missing() -> TestResult {
    promotion_and_workflow::oracle_workflow_fails_when_failure_evidence_is_missing()
}

#[test]
fn oracle_workflow_bounds_sanitizer_failure_artifacts() -> TestResult {
    promotion_and_workflow::oracle_workflow_bounds_sanitizer_failure_artifacts()
}

#[test]
fn oracle_workflow_fetches_full_history_for_every_checkout() -> TestResult {
    promotion_and_workflow::oracle_workflow_fetches_full_history_for_every_checkout()
}

#[test]
fn windows_oracle_step_fails_fast_on_native_command_errors() -> TestResult {
    promotion_and_workflow::windows_oracle_step_fails_fast_on_native_command_errors()
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

#[path = "docs_contract/core_contracts.rs"]
mod core_contracts;
#[path = "docs_contract/promotion_and_workflow.rs"]
mod promotion_and_workflow;
