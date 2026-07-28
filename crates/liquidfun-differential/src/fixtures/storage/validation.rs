use sha2::{Digest, Sha256};

use crate::fixtures::domain::{ArtifactKind, CandidateMetadata, FixtureError, ReviewMetadata};

pub(in crate::fixtures) fn candidate_sha256(metadata: &CandidateMetadata) -> String {
    let mut digest = Sha256::new();
    let artifact_kind = match metadata.artifact_kind {
        ArtifactKind::ReviewedTrace => "reviewed_trace",
        ArtifactKind::MinimizedRegression => "minimized_regression",
    };
    for value in [
        metadata.artifact_id.as_str(),
        artifact_kind,
        metadata.scenario_id.as_str(),
        metadata.scenario_sha256.as_str(),
        metadata.source_json.as_str(),
        metadata.tolerance_profile_sha256.as_str(),
        metadata.oracle_revision.as_str(),
        metadata.adapter_revision.as_str(),
        metadata.adapter_content_sha256.as_str(),
        metadata.build_identity_sha256.as_str(),
        metadata.preset.as_str(),
        metadata.session_profile.as_str(),
        metadata.compiler.as_str(),
        metadata.target.as_str(),
        metadata.generator_revision.as_str(),
        metadata.review_status.as_str(),
        metadata.request_sha256.as_str(),
        metadata.trace_sha256.as_str(),
        metadata.report_sha256.as_str(),
        metadata.identity_sha256.as_str(),
        metadata.stderr_sha256.as_str(),
        metadata.scenario_bytes_sha256.as_str(),
        metadata.trace_payload_sha256.as_str(),
        metadata.failure_signature_json.as_deref().unwrap_or(""),
    ] {
        digest.update(value.len().to_be_bytes());
        digest.update(value.as_bytes());
    }
    for version in [
        metadata.schema_version,
        metadata.protocol_version,
        metadata.scenario_schema_version,
        metadata.trace_schema_version,
        metadata.tolerance_profile_version,
    ] {
        digest.update(version.to_be_bytes());
    }
    for flag in &metadata.flags {
        digest.update(flag.len().to_be_bytes());
        digest.update(flag.as_bytes());
    }
    format!("{:x}", digest.finalize())
}

pub(in crate::fixtures) fn validate_review(review: ReviewMetadata<'_>) -> Result<(), FixtureError> {
    validate_nonempty(review.reviewer, "reviewer")?;
    validate_nonempty(review.reviewed_at, "reviewed_at")?;
    if !review.reviewed_at.contains('T') || !review.reviewed_at.ends_with('Z') {
        return Err(FixtureError::Replay(
            "review timestamp must be explicit UTC RFC3339 form".to_owned(),
        ));
    }
    Ok(())
}

pub(in crate::fixtures) fn validate_identifier(
    value: &str,
    field: &'static str,
) -> Result<(), FixtureError> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value.bytes().enumerate().all(|(index, byte)| {
            (index > 0 && matches!(byte, b'.' | b'_' | b'-'))
                || byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
        });
    if valid {
        return Ok(());
    }
    Err(FixtureError::InvalidIdentifier {
        field,
        value: value.to_owned(),
    })
}

fn validate_nonempty(value: &str, field: &'static str) -> Result<(), FixtureError> {
    if !value.trim().is_empty() && !value.chars().any(char::is_control) {
        return Ok(());
    }
    Err(FixtureError::InvalidIdentifier {
        field,
        value: value.to_owned(),
    })
}

pub(in crate::fixtures) fn validate_revision(value: &str) -> Result<(), FixtureError> {
    if super::is_revision(value) {
        return Ok(());
    }
    Err(FixtureError::InvalidIdentifier {
        field: "generator revision",
        value: value.to_owned(),
    })
}

pub(in crate::fixtures) fn validate_preset_profile(
    preset: &str,
    profile: &str,
) -> Result<(), FixtureError> {
    if !matches!(
        preset,
        "oracle-debug" | "oracle-release" | "oracle-asan-ubsan"
    ) {
        return Err(FixtureError::InvalidIdentifier {
            field: "preset",
            value: preset.to_owned(),
        });
    }
    if !matches!(profile, "one-shot" | "reuse" | "sanitizer") {
        return Err(FixtureError::InvalidIdentifier {
            field: "session profile",
            value: profile.to_owned(),
        });
    }
    if profile == "sanitizer" && preset != "oracle-asan-ubsan" {
        return Err(FixtureError::Replay(
            "sanitizer profile requires the sanitizer preset".to_owned(),
        ));
    }
    Ok(())
}

pub(in crate::fixtures) fn enforce_size(
    field: &'static str,
    bytes: &[u8],
    limit: usize,
) -> Result<(), FixtureError> {
    if bytes.len() <= limit {
        return Ok(());
    }
    Err(FixtureError::SizeLimit { field, limit })
}
