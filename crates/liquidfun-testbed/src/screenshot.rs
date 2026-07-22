//! Deterministic diagnostic capture metadata outside comparison and evidence authority.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{CapabilityOptions, run_capability_check};

const MAXIMUM_REPORT_BYTES: usize = 128 * 1024;
const DIAGNOSTIC_AUTHORITY: &str = "Diagnostic only — screenshots do not prove compatibility.";

/// Inputs for the deterministic visual-contract diagnostic command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualContractOptions {
    fixture: PathBuf,
    output: PathBuf,
    commit: Box<str>,
}

impl VisualContractOptions {
    /// Creates validated visual-contract inputs.
    ///
    /// # Errors
    ///
    /// Rejects commit identities outside the public short/full lowercase hexadecimal form.
    pub fn new(
        fixture: PathBuf,
        output: PathBuf,
        commit: &str,
    ) -> Result<Self, VisualContractError> {
        if commit != "Unavailable"
            && (!(7..=40).contains(&commit.len())
                || !commit
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()))
        {
            return Err(VisualContractError::InvalidProvenance);
        }
        Ok(Self {
            fixture,
            output,
            commit: commit.into(),
        })
    }
}

/// One post-write regular diagnostic artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiagnosticArtifact {
    path: Box<str>,
    sha256: Box<str>,
    regular: bool,
}

impl DiagnosticArtifact {
    /// Returns whether post-write metadata proves a non-link regular file.
    #[must_use]
    pub const fn is_regular(&self) -> bool {
        self.regular
    }
}

/// Complete deterministic diagnostic report with explicit authority exclusions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VisualContractReport {
    schema_version: u32,
    fixture_sha256: Box<str>,
    commit: Box<str>,
    authority: &'static str,
    contributes_to_comparison: bool,
    contributes_to_evidence: bool,
    capability_passed: bool,
    artifacts: Box<[DiagnosticArtifact]>,
}

impl VisualContractReport {
    /// Returns true when renderer capabilities and every diagnostic artifact passed.
    #[must_use]
    pub fn all_passed(&self) -> bool {
        self.capability_passed && self.artifacts.iter().all(DiagnosticArtifact::is_regular)
    }

    /// Returns the exact immutable fixture SHA-256 used for the diagnostic capture.
    #[must_use]
    pub fn resolved_sha256(&self) -> &str {
        &self.fixture_sha256
    }

    /// Returns the bounded Rust commit provenance or literal `Unavailable`.
    #[must_use]
    pub fn commit(&self) -> &str {
        &self.commit
    }

    /// Returns the explicit diagnostic-only authority label.
    #[must_use]
    pub const fn authority(&self) -> &'static str {
        self.authority
    }

    /// Returns false because pixels cannot enter semantic comparison.
    #[must_use]
    pub const fn contributes_to_comparison(&self) -> bool {
        self.contributes_to_comparison
    }

    /// Returns false because screenshots cannot authorize compatibility evidence.
    #[must_use]
    pub const fn contributes_to_evidence(&self) -> bool {
        self.contributes_to_evidence
    }

    /// Returns every confined diagnostic image artifact.
    #[must_use]
    pub fn artifacts(&self) -> &[DiagnosticArtifact] {
        &self.artifacts
    }
}

/// Runs the retained renderer matrix and writes deterministic diagnostic metadata.
///
/// # Errors
///
/// Returns bounded categories for capability, fixture, filesystem, or report failures.
pub fn run_visual_contract_check(
    options: &VisualContractOptions,
) -> Result<VisualContractReport, VisualContractError> {
    let capability = run_capability_check(&CapabilityOptions::new(
        options.fixture.clone(),
        options.output.clone(),
    ))
    .map_err(|_| VisualContractError::Capability)?;
    let fixture_bytes = fs::read(&options.fixture).map_err(|_| VisualContractError::Filesystem)?;
    let fixture_sha256 = hex_sha256(&fixture_bytes);
    let artifacts = capability
        .artifacts()
        .iter()
        .map(|artifact| DiagnosticArtifact {
            path: artifact.path().into(),
            sha256: artifact.sha256().into(),
            regular: artifact.is_regular(),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let report = VisualContractReport {
        schema_version: 1,
        fixture_sha256: fixture_sha256.into_boxed_str(),
        commit: options.commit.clone(),
        authority: DIAGNOSTIC_AUTHORITY,
        contributes_to_comparison: false,
        contributes_to_evidence: false,
        capability_passed: capability.all_passed(),
        artifacts,
    };
    write_report(&options.output, &report)?;
    Ok(report)
}

fn write_report(
    relative_output: &Path,
    report: &VisualContractReport,
) -> Result<(), VisualContractError> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or(VisualContractError::Filesystem)?;
    let report_path = repository
        .join(relative_output)
        .join("visual-contract-report.json");
    reject_link(&report_path)?;
    let encoded = serde_json::to_vec_pretty(report).map_err(|_| VisualContractError::Encoding)?;
    if encoded.len() > MAXIMUM_REPORT_BYTES {
        return Err(VisualContractError::Encoding);
    }
    fs::write(&report_path, encoded).map_err(|_| VisualContractError::Filesystem)?;
    let metadata =
        fs::symlink_metadata(report_path).map_err(|_| VisualContractError::Filesystem)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(VisualContractError::Filesystem);
    }
    Ok(())
}

fn reject_link(path: &Path) -> Result<(), VisualContractError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(VisualContractError::Filesystem)
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(VisualContractError::Filesystem),
    }
}

fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Bounded visual-contract diagnostic error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum VisualContractError {
    /// Commit or provenance fields were invalid.
    #[error("visual contract provenance is invalid")]
    InvalidProvenance,
    /// The retained renderer capability failed.
    #[error("visual contract capability failed")]
    Capability,
    /// A confined file operation failed.
    #[error("visual contract filesystem operation failed")]
    Filesystem,
    /// The bounded deterministic report could not be encoded.
    #[error("visual contract report encoding failed")]
    Encoding,
}

impl VisualContractError {
    /// Returns a stable non-sensitive category.
    #[must_use]
    pub const fn category(self) -> &'static str {
        match self {
            Self::InvalidProvenance => "invalid_provenance",
            Self::Capability => "capability",
            Self::Filesystem => "filesystem",
            Self::Encoding => "encoding",
        }
    }
}
