//! Read-only contract tests for the checked-in Phase-2 scenario and protocol fixtures.

use std::{fs, path::PathBuf};

use liquidfun_test_protocol::{
    BuildIdentity, CodecErrorKind, HarnessLimits, ProtocolSessionValidator, ProtocolVersion,
    RecordLimit, RequestedObservable, ScenarioDecodeError, ScenarioErrorKind,
    ScenarioSchemaVersion, ScenarioSource, Sha256Hex, ToleranceProfile, ToleranceProfileVersion,
    TraceRecord, TraceSchemaVersion, TraceValidator, decode_handshake_jsonl,
    decode_scenario_request_jsonl, decode_trace_record_jsonl, encode_jsonl,
};
use serde::Serialize;

const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn read_fixture(relative: &str) -> Vec<u8> {
    fs::read(repository_path(relative)).expect("checked-in fixture should be readable")
}

fn accepted_request() -> liquidfun_test_protocol::ScenarioRequestRecord {
    decode_scenario_request_jsonl(
        &read_fixture("protocol/fixtures/accepted/empty-world-request.jsonl"),
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("accepted request fixture should validate")
}

fn assert_rejected_fixture(relative: &str, expected: CodecErrorKind) {
    let bytes = read_fixture(relative);
    if expected == CodecErrorKind::PartialRecord {
        assert!(!bytes.ends_with(b"\n"));
    } else {
        assert!(bytes.ends_with(b"\n"));
    }
    let error = decode_scenario_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect_err("rejected fixture should fail strict decoding");
    let ScenarioDecodeError::Codec(error) = error else {
        panic!("expected a strict codec rejection for {relative}");
    };
    assert_eq!(error.kind(), expected);
}

#[derive(Serialize)]
struct CanonicalHandshake<'a> {
    protocol_version: ProtocolVersion,
    record_kind: &'static str,
    supported_scenario_versions: [ScenarioSchemaVersion; 1],
    supported_trace_versions: [TraceSchemaVersion; 1],
    supported_tolerance_versions: [ToleranceProfileVersion; 1],
    build_identity: CanonicalBuildIdentity<'a>,
    identity_sha256: &'a Sha256Hex,
}

#[derive(Serialize)]
struct CanonicalBuildIdentity<'a> {
    oracle_revision: &'a str,
    adapter_revision: &'a str,
    adapter_content_sha256: &'a Sha256Hex,
    cmake_preset: &'a str,
    compiler_id: &'a str,
    compiler_version: &'a str,
    target: &'a str,
    build_type: &'a str,
    effective_compile_flags: &'a str,
    effective_link_flags: &'a str,
    sanitizer_mode: &'a str,
}

fn encode_handshake(identity: &BuildIdentity, limits: &HarnessLimits) -> Vec<u8> {
    let record = CanonicalHandshake {
        protocol_version: ProtocolVersion::CURRENT,
        record_kind: "handshake",
        supported_scenario_versions: [ScenarioSchemaVersion::CURRENT],
        supported_trace_versions: [TraceSchemaVersion::CURRENT],
        supported_tolerance_versions: [ToleranceProfileVersion::CURRENT],
        build_identity: CanonicalBuildIdentity {
            oracle_revision: identity.oracle_revision(),
            adapter_revision: identity.adapter_revision(),
            adapter_content_sha256: identity.adapter_content_sha256(),
            cmake_preset: identity.cmake_preset(),
            compiler_id: identity.compiler_id(),
            compiler_version: identity.compiler_version(),
            target: identity.target(),
            build_type: identity.build_type(),
            effective_compile_flags: identity.effective_compile_flags(),
            effective_link_flags: identity.effective_link_flags(),
            sanitizer_mode: identity.sanitizer_mode(),
        },
        identity_sha256: identity.identity_sha256(),
    };
    encode_jsonl(&record, limits, RecordLimit::Output)
        .expect("validated handshake should encode canonically")
}

#[test]
fn fixtures_named_scenario_is_bounded_ordered_and_exact_bit() {
    // Arrange
    let scenario_bytes = read_fixture("scenarios/phase-02/empty-world.json");
    let request = accepted_request();

    // Act
    let scenario_json: serde_json::Value =
        serde_json::from_slice(&scenario_bytes).expect("named scenario should be valid JSON");
    let mut canonical =
        serde_json::to_vec(request.scenario()).expect("validated scenario should serialize");
    canonical.push(b'\n');

    // Assert
    assert_eq!(scenario_bytes, canonical);
    assert_eq!(request.scenario().commands().len(), 2);
    assert_eq!(request.scenario().checkpoints().len(), 2);
    assert_eq!(scenario_json["entities"], serde_json::json!([]));
    assert_eq!(request.scenario().gravity_x_bits().bits(), 0);
    assert_eq!(request.scenario().gravity_y_bits().bits(), 3_240_099_840);
    assert_eq!(
        request.scenario().commands()[0].timestep_bits().bits(),
        1_056_964_608
    );
    assert_eq!(
        request.scenario().commands()[1].timestep_bits().bits(),
        1_056_964_608
    );
    assert_eq!(scenario_json["commands"][0]["velocity_iterations"], 8);
    assert_eq!(scenario_json["commands"][0]["position_iterations"], 3);
    assert_eq!(scenario_json["commands"][0]["particle_iterations"], 1);
    assert_eq!(
        request.scenario().checkpoints()[0]
            .after_command_id()
            .as_str(),
        "step-1"
    );
    assert_eq!(
        request.scenario().checkpoints()[1]
            .after_command_id()
            .as_str(),
        "step-2"
    );
    assert_eq!(
        request.scenario().checkpoints()[0].observables(),
        &[
            RequestedObservable::WorldCounts,
            RequestedObservable::SimulationTime,
        ]
    );
    assert!(matches!(
        request.scenario().source(),
        ScenarioSource::Named { name } if name.as_ref() == "empty-world"
    ));
}

#[test]
fn fixtures_accepted_request_reencodes_byte_identically() {
    // Arrange
    let bytes = read_fixture("protocol/fixtures/accepted/empty-world-request.jsonl");
    let limits = HarnessLimits::phase2_default_v1();

    // Act
    let request = decode_scenario_request_jsonl(&bytes, &limits)
        .expect("accepted request fixture should validate");
    let canonical = encode_jsonl(&request, &limits, RecordLimit::Input)
        .expect("validated request should encode canonically");

    // Assert
    assert_eq!(canonical, bytes);
    assert_eq!(
        request.tolerance_profile_sha256(),
        ToleranceProfile::phase2_v1().profile_sha256()
    );
}

#[test]
fn fixtures_accepted_trace_reencodes_and_validates_reset_proof() {
    // Arrange
    let bytes = read_fixture("protocol/fixtures/accepted/empty-world-trace.jsonl");
    let limits = HarnessLimits::phase2_default_v1();
    let request = accepted_request();
    let mut lines = bytes.split_inclusive(|byte| *byte == b'\n');
    let handshake_line = lines
        .next()
        .expect("trace fixture should start with handshake");

    // Act
    let handshake = decode_handshake_jsonl(handshake_line, &limits)
        .expect("accepted handshake should validate");
    let identity = handshake.build_identity().clone();
    let mut session = ProtocolSessionValidator::new(REVISION);
    session
        .accept_handshake(handshake)
        .expect("handshake should match pinned provenance");
    session
        .begin_request(&request)
        .expect("validated request should follow handshake");
    let records = lines
        .map(|line| {
            decode_trace_record_jsonl(line, &limits)
                .expect("accepted streamed trace record should decode")
        })
        .collect::<Vec<_>>();
    let mut canonical = encode_handshake(&identity, &limits);
    for record in &records {
        canonical.extend(
            encode_jsonl(record, &limits, RecordLimit::Output)
                .expect("validated trace record should encode canonically"),
        );
    }
    let trace = TraceValidator::validate(&request, &identity, 1, records, &limits)
        .expect("complete trace should validate");

    // Assert
    assert_eq!(canonical, bytes);
    assert_eq!(trace.checkpoints().len(), 2);
    assert_eq!(trace.reset_epoch(), 1);
    assert!(canonical.ends_with(b"\n"));
    assert!(matches!(
        decode_trace_record_jsonl(
            canonical
                .split_inclusive(|byte| *byte == b'\n')
                .nth(1)
                .expect("trace begin should exist"),
            &limits,
        ),
        Ok(TraceRecord::Begin(_))
    ));
}

#[test]
fn fixtures_duplicate_member_has_duplicate_category() {
    // Arrange
    let path = "protocol/fixtures/rejected/duplicate-member.jsonl";

    // Act and Assert
    assert_rejected_fixture(path, CodecErrorKind::DuplicateMember);
}

#[test]
fn fixtures_unknown_record_kind_has_unknown_kind_category() {
    // Arrange
    let path = "protocol/fixtures/rejected/unknown-record-kind.jsonl";

    // Act and Assert
    assert_rejected_fixture(path, CodecErrorKind::UnknownRecordKind);
}

#[test]
fn fixtures_partial_record_has_partial_category() {
    // Arrange
    let path = "protocol/fixtures/rejected/partial-record.jsonl";

    // Act and Assert
    assert_rejected_fixture(path, CodecErrorKind::PartialRecord);
}

#[test]
fn fixtures_unsupported_version_has_version_category() {
    // Arrange
    let path = "protocol/fixtures/rejected/unsupported-version.jsonl";

    // Act and Assert
    assert_rejected_fixture(path, CodecErrorKind::UnsupportedVersion);
}

#[test]
fn fixtures_oversized_id_has_boundary_category() {
    // Arrange
    let path = "protocol/fixtures/rejected/oversized-id.jsonl";

    // Act and Assert
    assert_rejected_fixture(path, CodecErrorKind::BoundaryLimitExceeded);
}

#[test]
fn fixtures_empty_checkpoint_phase_matches_schema_and_runtime_rejection() {
    // Arrange
    let bytes = read_fixture("protocol/fixtures/rejected/empty-checkpoint-phase.jsonl");
    let record: serde_json::Value =
        serde_json::from_slice(&bytes).expect("rejected fixture should remain valid JSON");
    let schema: serde_json::Value =
        serde_json::from_slice(&read_fixture("protocol/schemas/scenario-v1.schema.json"))
            .expect("scenario schema should remain valid JSON");

    // Act
    let error = decode_scenario_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect_err("empty checkpoint phase should fail typed decoding");

    // Assert
    assert_eq!(record["scenario"]["checkpoints"][0]["phase"], "");
    assert_eq!(
        schema["properties"]["checkpoints"]["items"]["properties"]["phase"]["minLength"],
        1
    );
    assert_eq!(
        error.scenario_kind(),
        Some(ScenarioErrorKind::EmptyCheckpointPhase)
    );
}
