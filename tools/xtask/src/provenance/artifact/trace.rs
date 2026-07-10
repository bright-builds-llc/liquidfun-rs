//! Strict accepted-trace decoding and cross-record identity/hash validation.

use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{ArtifactSource, RawArtifactRecord, ScenarioDocument, validate_sha256};
use crate::provenance::{ProvenanceError, require_nonempty};

#[derive(Debug, Deserialize)]
struct RecordKindProbe {
    record_kind: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HandshakeRecord {
    protocol_version: u32,
    record_kind: String,
    supported_scenario_versions: Vec<u32>,
    supported_trace_versions: Vec<u32>,
    supported_tolerance_versions: Vec<u32>,
    build_identity: BuildIdentityRecord,
    identity_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildIdentityRecord {
    oracle_revision: String,
    adapter_revision: String,
    adapter_content_sha256: String,
    cmake_preset: String,
    compiler_id: String,
    compiler_version: String,
    target: String,
    build_type: String,
    effective_compile_flags: String,
    effective_link_flags: String,
    sanitizer_mode: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceBeginRecord {
    protocol_version: u32,
    record_kind: String,
    request_id: String,
    trace_schema_version: u32,
    scenario_id: String,
    scenario_sha256: String,
    source: ArtifactSource,
    tolerance_profile_version: u32,
    tolerance_profile_sha256: String,
    engine_kind: String,
    identity_sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CheckpointRecord {
    protocol_version: u32,
    record_kind: String,
    request_id: String,
    checkpoint_id: String,
    ordinal: usize,
    phase: String,
    simulation_time_bits: u32,
    world_counts: WorldCounts,
    identity_sha256: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WorldCounts {
    bodies: u64,
    fixtures: u64,
    joints: u64,
    contacts: u64,
    particle_systems: u64,
    particle_groups: u64,
    particles: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceEndRecord {
    protocol_version: u32,
    record_kind: String,
    request_id: String,
    checkpoint_count: usize,
    trace_payload_sha256: String,
    reset_epoch: u64,
    reset_verified: bool,
    identity_sha256: String,
}

pub(super) fn validate_trace(
    path: &Path,
    common: &RawArtifactRecord,
    expected_payload_sha256: &str,
    scenario: &ScenarioDocument,
) -> Result<(), ProvenanceError> {
    validate_sha256("trace_payload_sha256", expected_payload_sha256)?;
    let text = std::fs::read_to_string(path).map_err(|error| {
        ProvenanceError::new("filesystem", format!("failed to read trace: {error}"))
    })?;
    if !text.ends_with('\n') {
        return Err(ProvenanceError::new(
            "trace",
            "accepted trace must be newline complete",
        ));
    }
    let mut lines = text.lines();
    let handshake_line = lines
        .next()
        .ok_or_else(|| ProvenanceError::new("trace", "trace handshake is missing"))?;
    let handshake: HandshakeRecord = parse_trace_kind(handshake_line, "handshake")?;
    validate_handshake(common, &handshake)?;
    let begin_line = lines
        .next()
        .ok_or_else(|| ProvenanceError::new("trace", "trace begin is missing"))?;
    let begin: TraceBeginRecord = parse_trace_kind(begin_line, "trace_begin")?;
    validate_trace_begin(common, &handshake, &begin, scenario)?;
    let remaining = lines.collect::<Vec<_>>();
    let Some((end_line, checkpoint_lines)) = remaining.split_last() else {
        return Err(ProvenanceError::new("trace", "trace end is missing"));
    };
    let checkpoints = checkpoint_lines
        .iter()
        .enumerate()
        .map(|(ordinal, line)| {
            let checkpoint: CheckpointRecord = parse_trace_kind(line, "checkpoint")?;
            if checkpoint.ordinal != ordinal
                || checkpoint.protocol_version != common.protocol_version
                || checkpoint.request_id != begin.request_id
                || checkpoint.identity_sha256 != common.build_identity_sha256
            {
                return Err(ProvenanceError::new(
                    "trace",
                    "checkpoint order or identity mismatch",
                ));
            }
            Ok(checkpoint)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let end: TraceEndRecord = parse_trace_kind(end_line, "trace_end")?;
    let actual_payload_sha256 = checkpoint_payload_sha256(&checkpoints)?;
    if end.protocol_version != common.protocol_version
        || end.record_kind != "trace_end"
        || end.request_id != begin.request_id
        || end.checkpoint_count != checkpoints.len()
        || !end.reset_verified
        || end.reset_epoch == 0
        || end.identity_sha256 != common.build_identity_sha256
        || end.trace_payload_sha256 != expected_payload_sha256
        || actual_payload_sha256 != expected_payload_sha256
    {
        return Err(ProvenanceError::new(
            "trace",
            "trace end hash, count, reset, or identity mismatch",
        ));
    }
    Ok(())
}

fn parse_trace_kind<T: for<'de> Deserialize<'de>>(
    line: &str,
    expected: &str,
) -> Result<T, ProvenanceError> {
    let probe: RecordKindProbe = serde_json::from_str(line)
        .map_err(|error| ProvenanceError::new("trace", error.to_string()))?;
    if probe.record_kind != expected {
        return Err(ProvenanceError::new(
            "trace",
            format!("expected `{expected}`, actual `{}`", probe.record_kind),
        ));
    }
    serde_json::from_str(line).map_err(|error| ProvenanceError::new("trace", error.to_string()))
}

fn validate_handshake(
    common: &RawArtifactRecord,
    handshake: &HandshakeRecord,
) -> Result<(), ProvenanceError> {
    let identity = &handshake.build_identity;
    let compiler = format!("{} {}", identity.compiler_id, identity.compiler_version);
    if handshake.protocol_version != common.protocol_version
        || handshake.record_kind != "handshake"
        || !handshake
            .supported_scenario_versions
            .contains(&common.scenario_schema_version)
        || !handshake
            .supported_trace_versions
            .contains(&common.trace_schema_version)
        || !handshake
            .supported_tolerance_versions
            .contains(&common.tolerance_profile_version)
        || identity.oracle_revision != common.oracle_revision
        || identity.adapter_revision != common.adapter_revision
        || identity.adapter_content_sha256 != common.adapter_content_sha256
        || identity.cmake_preset != common.preset
        || compiler != common.compiler
        || identity.target != common.target
        || common.flags
            != [
                identity.effective_compile_flags.clone(),
                identity.effective_link_flags.clone(),
            ]
        || handshake.identity_sha256 != common.build_identity_sha256
        || build_identity_sha256(identity) != common.build_identity_sha256
    {
        return Err(ProvenanceError::new(
            "identity",
            "artifact build identity does not match the accepted trace handshake",
        ));
    }
    require_nonempty("build_type", &identity.build_type)?;
    require_nonempty("sanitizer_mode", &identity.sanitizer_mode)
}

fn validate_trace_begin(
    common: &RawArtifactRecord,
    handshake: &HandshakeRecord,
    begin: &TraceBeginRecord,
    scenario: &ScenarioDocument,
) -> Result<(), ProvenanceError> {
    if begin.protocol_version != common.protocol_version
        || begin.record_kind != "trace_begin"
        || begin.trace_schema_version != common.trace_schema_version
        || begin.scenario_id != scenario.scenario_id
        || begin.scenario_sha256 != common.scenario_sha256
        || begin.source != common.source
        || begin.tolerance_profile_version != common.tolerance_profile_version
        || begin.tolerance_profile_sha256 != common.tolerance_profile_sha256
        || begin.engine_kind != "cpp_oracle"
        || begin.identity_sha256 != handshake.identity_sha256
    {
        return Err(ProvenanceError::new(
            "identity",
            "artifact trace begin does not match scenario, policy, or build identity",
        ));
    }
    require_nonempty("request_id", &begin.request_id)
}

fn checkpoint_payload_sha256(checkpoints: &[CheckpointRecord]) -> Result<String, ProvenanceError> {
    let mut digest = Sha256::new();
    for checkpoint in checkpoints {
        let bytes = serde_json::to_vec(checkpoint).map_err(|error| {
            ProvenanceError::new("trace", format!("failed to encode checkpoint: {error}"))
        })?;
        digest.update(bytes.len().to_be_bytes());
        digest.update(bytes);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn build_identity_sha256(identity: &BuildIdentityRecord) -> String {
    let values = [
        ("oracle_revision", identity.oracle_revision.as_str()),
        ("adapter_revision", identity.adapter_revision.as_str()),
        (
            "adapter_content_sha256",
            identity.adapter_content_sha256.as_str(),
        ),
        ("cmake_preset", identity.cmake_preset.as_str()),
        ("compiler_id", identity.compiler_id.as_str()),
        ("compiler_version", identity.compiler_version.as_str()),
        ("target", identity.target.as_str()),
        ("build_type", identity.build_type.as_str()),
        (
            "effective_compile_flags",
            identity.effective_compile_flags.as_str(),
        ),
        (
            "effective_link_flags",
            identity.effective_link_flags.as_str(),
        ),
        ("sanitizer_mode", identity.sanitizer_mode.as_str()),
    ];
    let mut digest = Sha256::new();
    for (name, value) in values {
        digest.update(name.len().to_be_bytes());
        digest.update(name.as_bytes());
        digest.update(value.len().to_be_bytes());
        digest.update(value.as_bytes());
    }
    format!("{:x}", digest.finalize())
}
