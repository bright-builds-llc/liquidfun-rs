use std::{
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::FailureSignature;

pub(super) const CANDIDATE_SCHEMA_VERSION: u32 = 1;
pub(super) const MANIFEST_FIELDS: [&str; 27] = [
    "artifact_kind",
    "path",
    "sha256",
    "generator_revision",
    "request_sha256",
    "scenario_content_sha256",
    "scenario_sha256",
    "protocol_version",
    "scenario_schema_version",
    "trace_schema_version",
    "tolerance_profile_version",
    "tolerance_profile_sha256",
    "oracle_revision",
    "adapter_revision",
    "adapter_content_sha256",
    "build_identity_sha256",
    "preset",
    "compiler",
    "target",
    "flags",
    "source",
    "trace_payload_sha256",
    "failure_signature",
    "notice_refs",
    "reviewer",
    "reviewed_at",
    "review_status",
];
pub(super) const REQUIRED_FILES: [&str; 6] = [
    "request.jsonl",
    "trace.jsonl",
    "report.json",
    "identity.jsonl",
    "stderr.txt",
    "scenario.json",
];
pub(super) const MAX_REPORT_BYTES: usize = 1024 * 1024;
pub(super) const MAX_REPLAY_EPOCH: u64 = 100;

/// Accepted artifact class, which owns the derived destination path and replay expectation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    /// A provenance-bound oracle trace that semantically matches native Rust.
    ReviewedTrace,
    /// A canonical minimized scenario that retains one exact first-divergence signature.
    MinimizedRegression,
}

/// Borrowed generated evidence supplied to the confined staging boundary.
#[derive(Clone, Copy)]
pub struct StageRequest<'a> {
    /// Stable candidate identity, never a path.
    pub artifact_id: &'a str,
    /// Typed artifact class used to derive the accepted destination.
    pub artifact_kind: ArtifactKind,
    /// Expected validated scenario identity.
    pub scenario_id: &'a str,
    /// Reviewed oracle build preset identity.
    pub preset: &'a str,
    /// Reviewed one-shot, reuse, or sanitizer profile identity.
    pub session_profile: &'a str,
    /// Full Git revision of the generator implementation.
    pub generator_revision: &'a str,
    /// Exact newline-complete scenario request record.
    pub request_bytes: &'a [u8],
    /// Exact newline-complete handshake and trace records.
    pub trace_bytes: &'a [u8],
    /// Supervisor-bounded stderr evidence.
    pub stderr_bytes: &'a [u8],
    /// Optional expected mismatch signature from a preceding minimization run.
    pub maybe_failure_signature: Option<&'a FailureSignature>,
}

/// Explicit human review metadata; successful generation never implies approval.
#[derive(Debug, Clone, Copy)]
pub struct ReviewMetadata<'a> {
    pub(super) reviewer: &'a str,
    pub(super) reviewed_at: &'a str,
    pub(super) review_status: ReviewStatus,
}

impl<'a> ReviewMetadata<'a> {
    /// Creates an explicit approval record.
    #[must_use]
    pub const fn approved(reviewer: &'a str, reviewed_at: &'a str) -> Self {
        Self {
            reviewer,
            reviewed_at,
            review_status: ReviewStatus::Approved,
        }
    }

    /// Creates an explicit rejection record that cannot be promoted.
    #[must_use]
    pub const fn rejected(reviewer: &'a str, reviewed_at: &'a str) -> Self {
        Self {
            reviewer,
            reviewed_at,
            review_status: ReviewStatus::Rejected,
        }
    }
}

/// Newly staged candidate below the canonical ignored staging root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactCandidate {
    pub(super) artifact_id: Box<str>,
    pub(super) directory: PathBuf,
}

impl ArtifactCandidate {
    /// Returns the validated candidate identity.
    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    /// Returns the canonical staging directory for audit and focused tests.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }
}

/// Deterministic replay and review result written only below the staging root.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReviewReceipt {
    pub(super) artifact_id: Box<str>,
    pub(super) review_status: ReviewStatus,
    pub(super) diff: Box<str>,
}

impl ReviewReceipt {
    /// Returns `approved` or `rejected` exactly as recorded.
    #[must_use]
    pub const fn review_status(&self) -> &'static str {
        self.review_status.as_str()
    }

    /// Returns a deterministic line-oriented diff against existing accepted bytes.
    #[must_use]
    pub fn diff(&self) -> &str {
        &self.diff
    }
}

/// Successful no-clobber artifact and manifest promotion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PromotionReceipt {
    pub(super) artifact_id: Box<str>,
    pub(super) artifact_path: PathBuf,
    pub(super) manifest_path: PathBuf,
    pub(super) sha256: Box<str>,
    pub(super) post_commit_warnings: Vec<Box<str>>,
}

impl PromotionReceipt {
    /// Returns the derived accepted artifact path.
    #[must_use]
    pub fn artifact_path(&self) -> &Path {
        &self.artifact_path
    }

    /// Returns the atomically replaced manifest path.
    #[must_use]
    pub fn manifest_path(&self) -> &Path {
        &self.manifest_path
    }

    /// Returns the accepted content SHA-256.
    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    /// Returns diagnostics for cleanup failures that occurred after the manifest committed.
    #[must_use]
    pub fn post_commit_warnings(&self) -> &[Box<str>] {
        &self.post_commit_warnings
    }
}

/// Fail-closed staging, replay, confinement, review, or promotion error.
#[derive(Debug, thiserror::Error)]
pub enum FixtureError {
    /// A caller-controlled identifier was not one normalized bounded component.
    #[error("invalid {field} identifier `{value}`")]
    InvalidIdentifier {
        /// Identifier role.
        field: &'static str,
        /// Rejected raw value.
        value: String,
    },
    /// A filesystem boundary contains a symbolic link.
    #[error("fixture path contains symbolic link {}", path.display())]
    Symlink {
        /// Rejected symbolic-link path.
        path: PathBuf,
    },
    /// A resolved path escaped the canonical repository or staging boundary.
    #[error("fixture path escaped its confined root: {}", path.display())]
    PathEscape {
        /// Rejected escaping path.
        path: PathBuf,
    },
    /// A required candidate file was absent or not regular.
    #[error("candidate file `{file}` is missing or not regular")]
    MissingCandidateFile {
        /// Required candidate filename.
        file: &'static str,
    },
    /// Candidate bytes exceeded their named reviewed limit.
    #[error("candidate field `{field}` exceeds {limit} bytes")]
    SizeLimit {
        /// Bounded candidate field.
        field: &'static str,
        /// Maximum accepted bytes.
        limit: usize,
    },
    /// Staged bytes differ from their recorded digest.
    #[error("candidate file `{file}` SHA-256 mismatch")]
    HashMismatch {
        /// Dirty candidate filename.
        file: String,
    },
    /// Candidate bytes, identity, schema, profile, or scenario failed typed replay.
    #[error("candidate replay validation failed: {0}")]
    Replay(String),
    /// A reviewed regression no longer reproduces the recorded first divergence.
    #[error("candidate regression failure signature changed")]
    SignatureMismatch,
    /// Promotion has no explicit bound approval receipt.
    #[error("candidate requires an explicit successful replay review")]
    ReviewRequired,
    /// The derived accepted destination already exists.
    #[error("accepted destination already exists: {}", path.display())]
    DestinationExists {
        /// Derived destination that already exists.
        path: PathBuf,
    },
    /// A candidate ID or review receipt already exists.
    #[error("candidate staging entry already exists: {}", path.display())]
    CandidateExists {
        /// Existing candidate or review receipt path.
        path: PathBuf,
    },
    /// Artifact manifest content is incompatible with the required schema.
    #[error("artifact manifest is invalid: {0}")]
    Manifest(String),
    /// Filesystem operation failed.
    #[error(transparent)]
    Io(#[from] io::Error),
    /// Strict TOML parsing or rendering failed.
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    /// TOML rendering failed.
    #[error(transparent)]
    TomlSerialize(#[from] toml::ser::Error),
    /// Deterministic JSON rendering failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
}

impl ReviewStatus {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CandidateMetadata {
    pub(super) schema_version: u32,
    pub(super) artifact_id: String,
    pub(super) artifact_kind: ArtifactKind,
    pub(super) scenario_id: String,
    pub(super) scenario_sha256: String,
    pub(super) source_json: String,
    pub(super) protocol_version: u32,
    pub(super) scenario_schema_version: u32,
    pub(super) trace_schema_version: u32,
    pub(super) tolerance_profile_version: u32,
    pub(super) tolerance_profile_sha256: String,
    pub(super) oracle_revision: String,
    pub(super) adapter_revision: String,
    pub(super) adapter_content_sha256: String,
    pub(super) build_identity_sha256: String,
    pub(super) preset: String,
    pub(super) session_profile: String,
    pub(super) compiler: String,
    pub(super) target: String,
    pub(super) flags: Vec<String>,
    pub(super) generator_revision: String,
    pub(super) review_status: ReviewStatus,
    pub(super) request_sha256: String,
    pub(super) trace_sha256: String,
    pub(super) report_sha256: String,
    pub(super) identity_sha256: String,
    pub(super) stderr_sha256: String,
    pub(super) scenario_bytes_sha256: String,
    pub(super) trace_payload_sha256: String,
    pub(super) failure_signature_json: Option<String>,
    pub(super) candidate_sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StoredReview {
    pub(super) schema_version: u32,
    pub(super) artifact_id: String,
    pub(super) candidate_sha256: String,
    pub(super) reviewer: String,
    pub(super) reviewed_at: String,
    pub(super) review_status: ReviewStatus,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ArtifactManifest {
    pub(super) schema_version: u32,
    pub(super) record_schema_version: u32,
    pub(super) oracle_revision: String,
    pub(super) record_fields: Vec<String>,
    pub(super) artifacts: Vec<ArtifactRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ArtifactRecord {
    pub(super) artifact_kind: ManifestArtifactKind,
    pub(super) path: String,
    pub(super) sha256: String,
    pub(super) generator_revision: String,
    pub(super) request_sha256: String,
    pub(super) scenario_content_sha256: String,
    pub(super) scenario_sha256: String,
    pub(super) protocol_version: u32,
    pub(super) scenario_schema_version: u32,
    pub(super) trace_schema_version: u32,
    pub(super) tolerance_profile_version: u32,
    pub(super) tolerance_profile_sha256: String,
    pub(super) oracle_revision: String,
    pub(super) adapter_revision: String,
    pub(super) adapter_content_sha256: String,
    pub(super) build_identity_sha256: String,
    pub(super) preset: String,
    pub(super) compiler: String,
    pub(super) target: String,
    pub(super) flags: Vec<String>,
    pub(super) source: ManifestSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) trace_payload_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) failure_signature: Option<ManifestFailureSignature>,
    pub(super) notice_refs: Vec<String>,
    pub(super) reviewer: String,
    pub(super) reviewed_at: String,
    pub(super) review_status: ManifestReviewStatus,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ManifestArtifactKind {
    Trace,
    Regression,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum ManifestSource {
    Named {
        name: String,
    },
    Seeded {
        generator_id: String,
        generator_version: u32,
        seed: u64,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ManifestFailureSignature {
    pub(super) checkpoint_id: String,
    pub(super) phase: String,
    pub(super) semantic_path: serde_json::Value,
    pub(super) kind: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ManifestReviewStatus {
    Reviewed,
}

pub(super) struct ReplayedCandidate {
    pub(super) directory: PathBuf,
    pub(super) metadata: CandidateMetadata,
    pub(super) accepted_bytes: Vec<u8>,
}
