//! Closed release evidence domain types.

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

/// Every evidence class required by the release decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReleaseEvidenceKind {
    Package,
    Msrv,
    Platform,
    ConditionalPlatform,
    CanonicalDifferential,
    RustSafety,
    CppSanitizer,
    Fuzz,
    Regressions,
    RustCoverage,
    CppCoverage,
    Performance,
    Docs,
    Notices,
    CorpusClosure,
    CompatibilityClosure,
}

impl ReleaseEvidenceKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Package => "package",
            Self::Msrv => "msrv",
            Self::Platform => "platform",
            Self::ConditionalPlatform => "conditional_platform",
            Self::CanonicalDifferential => "canonical_differential",
            Self::RustSafety => "rust_safety",
            Self::CppSanitizer => "cpp_sanitizer",
            Self::Fuzz => "fuzz",
            Self::Regressions => "regressions",
            Self::RustCoverage => "rust_coverage",
            Self::CppCoverage => "cpp_coverage",
            Self::Performance => "performance",
            Self::Docs => "docs",
            Self::Notices => "notices",
            Self::CorpusClosure => "corpus_closure",
            Self::CompatibilityClosure => "compatibility_closure",
        }
    }
}

impl Display for ReleaseEvidenceKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Unique release evidence identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvidenceKey {
    pub(crate) kind: ReleaseEvidenceKind,
    pub(crate) target: String,
}

/// Tracked producer and toolchain allowlist entry.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RequiredEvidence {
    pub(crate) kind: ReleaseEvidenceKind,
    pub(crate) target: String,
    pub(crate) workflow: String,
    pub(crate) job: String,
    pub(crate) toolchain: String,
}

impl RequiredEvidence {
    pub(crate) fn key(&self) -> EvidenceKey {
        EvidenceKey {
            kind: self.kind,
            target: self.target.clone(),
        }
    }
}

/// Closed tracked required-evidence registry.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RequiredEvidenceManifest {
    pub(crate) schema_version: u8,
    pub(crate) evidence: Vec<RequiredEvidence>,
}

/// Untrusted release manifest parsed before validation.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReleaseManifest {
    pub(crate) schema_version: u8,
    pub(crate) candidate_commit: String,
    pub(crate) items: Vec<ReleaseEvidenceRecord>,
}

/// Untrusted producer identity.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProducerIdentity {
    pub(crate) workflow: String,
    pub(crate) job: String,
    pub(crate) run_id: String,
}

/// One untrusted evidence reference from the candidate manifest.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReleaseEvidenceRecord {
    pub(crate) kind: ReleaseEvidenceKind,
    pub(crate) target: String,
    pub(crate) candidate_commit: String,
    pub(crate) producer: ProducerIdentity,
    pub(crate) artifact_path: String,
    pub(crate) artifact_sha256: String,
    pub(crate) payload_sha256: String,
    pub(crate) toolchain: String,
    pub(crate) review_status: String,
    pub(crate) status: String,
}

impl ReleaseEvidenceRecord {
    pub(crate) fn key(&self) -> EvidenceKey {
        EvidenceKey {
            kind: self.kind,
            target: self.target.clone(),
        }
    }
}

/// Strict evidence artifact envelope.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EvidenceArtifact {
    pub(crate) schema_version: u8,
    pub(crate) kind: ReleaseEvidenceKind,
    pub(crate) target: String,
    pub(crate) candidate_commit: String,
    pub(crate) status: String,
    pub(crate) payload_sha256: String,
    pub(crate) claims: serde_json::Value,
}

/// Fully validated record safe to include in a ready decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ValidatedEvidence {
    pub(crate) kind: ReleaseEvidenceKind,
    pub(crate) target: String,
    pub(crate) workflow: String,
    pub(crate) job: String,
    pub(crate) run_id: String,
    pub(crate) toolchain: String,
    pub(crate) artifact_sha256: String,
    pub(crate) payload_sha256: String,
}

/// Complete immutable release decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReleaseReadiness {
    pub(crate) candidate_commit: String,
    pub(crate) evidence: Vec<ValidatedEvidence>,
}
