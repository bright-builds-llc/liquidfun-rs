use super::{
    CheckpointRecord, EngineKind, HandshakeRecord, ProtocolSessionValidator, TraceBegin,
    TraceDecodeError, TraceEnd, TraceRecord, TraceValidator, WorldCounts,
    decode_math_probe_end_jsonl, decode_math_probe_result_jsonl, decode_trace_record_jsonl,
    trace_payload_sha256,
};
use crate::{
    BuildIdentity, BuildIdentityFields, CheckpointId, FloatBits, HarnessFailureKind, HarnessLimits,
    Phase4BuildIdentityFields, RequestId, Sha256Hex, decode_handshake_jsonl,
    decode_scenario_request_jsonl,
};

const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";

fn identity(revision: &str) -> BuildIdentity {
    BuildIdentity::new(BuildIdentityFields::new(
        revision,
        "adapter-v1",
        "c7f36eaf2f184a36b9c9a04636d3e22785d815c4948d55d0b3cbf44ee7245fc8",
        "oracle-debug",
        "Clang",
        "22.1.8",
        "aarch64-apple-darwin",
        "Debug",
        "-O0 -g",
        "-lc++",
        "none",
    ))
    .expect("identity fixture should validate")
}

#[test]
fn math_probe_result_decoder_bounds_and_validates_float_metadata() {
    // Arrange
    let valid = b"{\"case_id\":\"abs-negative-zero\",\"operation\":\"abs\",\"policy_path\":\"math.operation.abs\",\"horizon\":{\"kind\":\"operation\"},\"values\":[{\"field\":\"value\",\"bits\":0,\"class\":\"zero\",\"negative\":false}],\"discrete\":[]}\n";
    let wrong_class = String::from_utf8(valid.to_vec())
        .expect("fixture is UTF-8")
        .replace("\"class\":\"zero\"", "\"class\":\"normal\"");
    let oversized = format!(
        "{{\"case_id\":\"{}\",\"operation\":\"abs\",\"policy_path\":\"math.operation.abs\",\"horizon\":{{\"kind\":\"operation\"}},\"values\":[],\"discrete\":[]}}\n",
        "x".repeat(129)
    );
    let limits = HarnessLimits::phase2_default_v1();

    // Act
    let decoded = decode_math_probe_result_jsonl(valid, &limits);
    let metadata_error = decode_math_probe_result_jsonl(wrong_class.as_bytes(), &limits);
    let bound_error = decode_math_probe_result_jsonl(oversized.as_bytes(), &limits);

    // Assert
    assert_eq!(
        decoded.expect("valid result should decode").case_id(),
        "abs-negative-zero"
    );
    assert!(metadata_error.is_err());
    assert!(bound_error.is_err());
}

#[test]
fn math_probe_end_decoder_requires_reset_proof() {
    // Arrange
    let valid = b"{\"protocol_version\":1,\"record_kind\":\"math_probe_end\",\"request_id\":\"phase-04-math-probe-request\",\"result_count\":39,\"reset_epoch\":1,\"reset_verified\":true}\n";
    let invalid = String::from_utf8(valid.to_vec())
        .expect("fixture is UTF-8")
        .replace("\"reset_verified\":true", "\"reset_verified\":false");
    let limits = HarnessLimits::phase2_default_v1();

    // Act
    let decoded = decode_math_probe_end_jsonl(valid, &limits);
    let invalid = decode_math_probe_end_jsonl(invalid.as_bytes(), &limits);

    // Assert
    assert_eq!(decoded.expect("valid end should decode").result_count(), 39);
    assert!(invalid.is_err());
}

fn request(checkpoint_count: usize) -> crate::ScenarioRequestRecord {
    let checkpoints = (0..checkpoint_count)
        .map(|index| format!(
            "{{\"checkpoint_id\":\"checkpoint-{index}\",\"after_command_id\":\"step-1\",\"phase\":\"phase-{index}\",\"observables\":[\"world_counts\",\"simulation_time\"]}}"
        ))
        .collect::<Vec<_>>()
        .join(",");
    let record = format!(
        concat!(
            "{{\"protocol_version\":1,\"record_kind\":\"scenario_request\",",
            "\"request_id\":\"request-1\",\"scenario_schema_version\":1,",
            "\"requested_trace_schema_version\":1,\"tolerance_profile_version\":1,",
            "\"tolerance_profile_sha256\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",",
            "\"scenario\":{{\"scenario_id\":\"empty-world\",\"source\":{{\"kind\":\"named\",\"name\":\"empty-world\"}},",
            "\"gravity_x_bits\":0,\"gravity_y_bits\":0,\"entities\":[],",
            "\"commands\":[{{\"kind\":\"step\",\"command_id\":\"step-1\",\"timestep_bits\":100,",
            "\"velocity_iterations\":8,\"position_iterations\":3,\"particle_iterations\":1}}],",
            "\"checkpoints\":[{}]}}}}\n"
        ),
        checkpoints
    );
    decode_scenario_request_jsonl(record.as_bytes(), &HarnessLimits::phase2_default_v1())
        .expect("request fixture should validate")
}

fn valid_records(checkpoint_count: usize, identity: &BuildIdentity) -> Vec<TraceRecord> {
    let request = request(checkpoint_count);
    let checkpoints = (0..checkpoint_count)
        .map(|index| {
            CheckpointRecord::new(
                RequestId::new("request-1").expect("request ID should validate"),
                CheckpointId::new(format!("checkpoint-{index}"))
                    .expect("checkpoint ID should validate"),
                u32::try_from(index).expect("small fixture ordinal should fit"),
                format!("phase-{index}"),
                FloatBits::new(u32::try_from(index).expect("small fixture time should fit")),
                WorldCounts::zero(),
                identity.identity_sha256().clone(),
            )
            .expect("checkpoint should validate")
        })
        .collect::<Vec<_>>();
    let payload_sha256 = trace_payload_sha256(&checkpoints)
        .expect("checkpoint payload should hash deterministically");
    let mut records = vec![TraceRecord::Begin(
        TraceBegin::for_request(&request, EngineKind::CppOracle, identity)
            .expect("trace begin should build"),
    )];
    records.extend(checkpoints.into_iter().map(TraceRecord::Checkpoint));
    records.push(TraceRecord::End(TraceEnd::new(
        RequestId::new("request-1").expect("request ID should validate"),
        u32::try_from(checkpoint_count).expect("small fixture count should fit"),
        payload_sha256,
        1,
        true,
        identity.identity_sha256().clone(),
    )));
    records
}

#[test]
fn handshake_must_precede_requests_and_match_expected_provenance() {
    // Arrange
    let good_identity = identity(REVISION);
    let wrong_identity = identity("1111111111111111111111111111111111111111");
    let mut session = ProtocolSessionValidator::new(REVISION);

    // Act
    let before_handshake = session.begin_request(&request(0));
    let wrong = session.accept_handshake(HandshakeRecord::phase2(wrong_identity));
    let accepted = session.accept_handshake(HandshakeRecord::phase2(good_identity));
    let duplicate = session.accept_handshake(HandshakeRecord::phase2(identity(REVISION)));

    // Assert
    assert_eq!(
        before_handshake
            .expect_err("request before handshake should fail")
            .kind(),
        HarnessFailureKind::HandshakeMalformed
    );
    assert_eq!(
        wrong.expect_err("wrong provenance should fail").kind(),
        HarnessFailureKind::WrongProvenance
    );
    assert!(accepted.is_ok());
    assert_eq!(
        duplicate
            .expect_err("duplicate handshake should fail")
            .kind(),
        HarnessFailureKind::SequenceViolation
    );
}

#[test]
fn trace_validator_accepts_zero_one_and_multiple_ordered_checkpoints() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let identity = identity(REVISION);

    // Act
    let results = [0, 1, 2].map(|count| {
        TraceValidator::validate(
            &request(count),
            &identity,
            1,
            valid_records(count, &identity),
            &limits,
        )
    });

    // Assert
    for (count, result) in results.into_iter().enumerate() {
        assert_eq!(
            result.expect("valid trace should pass").checkpoints().len(),
            count
        );
    }
}

#[test]
fn trace_validator_rejects_sequence_request_identity_and_reset_failures() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let identity = identity(REVISION);
    let request = request(1);
    let mut out_of_order = valid_records(1, &identity);
    out_of_order.swap(0, 1);
    let mut wrong_request = valid_records(1, &identity);
    if let TraceRecord::Checkpoint(checkpoint) = &mut wrong_request[1] {
        checkpoint.set_request_id_for_test(
            RequestId::new("request-other").expect("request ID should validate"),
        );
    }
    let mut false_reset = valid_records(1, &identity);
    if let TraceRecord::End(end) = &mut false_reset[2] {
        end.set_reset_verified_for_test(false);
    }

    // Act
    let sequence_error = TraceValidator::validate(&request, &identity, 1, out_of_order, &limits)
        .expect_err("out-of-order trace should fail");
    let request_error = TraceValidator::validate(&request, &identity, 1, wrong_request, &limits)
        .expect_err("request mismatch should fail");
    let reset_error = TraceValidator::validate(&request, &identity, 1, false_reset, &limits)
        .expect_err("false reset proof should fail");

    // Assert
    assert_eq!(sequence_error.kind(), HarnessFailureKind::SequenceViolation);
    assert_eq!(request_error.kind(), HarnessFailureKind::RequestIdMismatch);
    assert_eq!(reset_error.kind(), HarnessFailureKind::AdapterResetFailure);
}

#[test]
fn trace_validator_rejects_identity_count_hash_missing_and_trailing_failures() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let identity = identity(REVISION);
    let request = request(1);
    let mut wrong_identity = valid_records(1, &identity);
    if let TraceRecord::Checkpoint(checkpoint) = &mut wrong_identity[1] {
        checkpoint
            .set_identity_for_test(Sha256Hex::new("11".repeat(32)).expect("hash should validate"));
    }
    let mut wrong_count = valid_records(1, &identity);
    if let TraceRecord::End(end) = &mut wrong_count[2] {
        end.set_checkpoint_count_for_test(2);
    }
    let mut wrong_hash = valid_records(1, &identity);
    if let TraceRecord::End(end) = &mut wrong_hash[2] {
        end.set_payload_hash_for_test(
            Sha256Hex::new("22".repeat(32)).expect("hash should validate"),
        );
    }
    let mut wrong_epoch = valid_records(1, &identity);
    if let TraceRecord::End(end) = &mut wrong_epoch[2] {
        end.set_reset_epoch_for_test(2);
    }
    let mut missing_end = valid_records(1, &identity);
    missing_end.pop();
    let mut trailing = valid_records(1, &identity);
    trailing.push(TraceRecord::End(TraceEnd::new(
        RequestId::new("request-1").expect("request ID should validate"),
        0,
        Sha256Hex::new("00".repeat(32)).expect("hash should validate"),
        2,
        true,
        identity.identity_sha256().clone(),
    )));

    // Act
    let kinds = [
        wrong_identity,
        wrong_count,
        wrong_hash,
        wrong_epoch,
        missing_end,
        trailing,
    ]
    .map(|records| {
        TraceValidator::validate(&request, &identity, 1, records, &limits)
            .expect_err("invalid trace should fail")
            .kind()
    });

    // Assert
    assert_eq!(kinds[0], HarnessFailureKind::TraceIdentityMismatch);
    assert_eq!(kinds[1], HarnessFailureKind::SequenceViolation);
    assert_eq!(kinds[2], HarnessFailureKind::SequenceViolation);
    assert_eq!(kinds[3], HarnessFailureKind::AdapterResetFailure);
    assert_eq!(kinds[4], HarnessFailureKind::UnexpectedEof);
    assert_eq!(kinds[5], HarnessFailureKind::SequenceViolation);
}

#[test]
fn trace_validator_rejects_begin_end_and_checkpoint_transition_permutations() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let identity = identity(REVISION);
    let request = request(1);
    let valid = valid_records(1, &identity);
    let begin = valid[0].clone();
    let checkpoint = valid[1].clone();
    let end = valid[2].clone();
    let cases = [
        vec![checkpoint.clone(), begin.clone(), end.clone()],
        vec![end.clone()],
        vec![begin.clone(), begin, checkpoint, end],
    ];

    // Act
    let kinds = cases.map(|records| {
        TraceValidator::validate(&request, &identity, 1, records, &limits)
            .expect_err("invalid transition should fail")
            .kind()
    });

    // Assert
    assert_eq!(kinds, [HarnessFailureKind::SequenceViolation; 3]);
}

#[test]
fn world_counts_expose_every_exact_empty_world_field() {
    // Arrange and Act
    let counts = WorldCounts::zero();

    // Assert
    assert_eq!(counts.bodies(), 0);
    assert_eq!(counts.fixtures(), 0);
    assert_eq!(counts.joints(), 0);
    assert_eq!(counts.contacts(), 0);
    assert_eq!(counts.particle_systems(), 0);
    assert_eq!(counts.particle_groups(), 0);
    assert_eq!(counts.particles(), 0);
}

#[test]
fn trace_decoder_rejects_an_invalid_named_source_before_validation() {
    // Arrange
    let identity = Sha256Hex::new("11".repeat(32)).expect("hash should validate");
    let record = format!(
        concat!(
            "{{\"record_kind\":\"trace_begin\",\"protocol_version\":1,",
            "\"request_id\":\"request-1\",\"trace_schema_version\":1,",
            "\"scenario_id\":\"empty-world\",",
            "\"scenario_sha256\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",",
            "\"source\":{{\"kind\":\"named\",\"name\":\"\"}},",
            "\"tolerance_profile_version\":1,",
            "\"tolerance_profile_sha256\":\"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\",",
            "\"engine_kind\":\"cpp_oracle\",\"identity_sha256\":\"{}\"}}\n"
        ),
        identity.as_str()
    );

    // Act
    let error = decode_trace_record_jsonl(record.as_bytes(), &HarnessLimits::phase2_default_v1())
        .expect_err("empty source name should fail at the boundary");

    // Assert
    let TraceDecodeError::Validation(error) = error else {
        panic!("expected typed trace validation failure");
    };
    assert_eq!(error.kind(), HarnessFailureKind::MalformedRecord);
}

#[test]
fn raw_build_identity_decodes_all_phase4_fields() {
    // Arrange
    let phase4 = Phase4BuildIdentityFields::new(
        "33".repeat(32),
        "AppleClang",
        "21.0.0",
        "arm64-apple-darwin",
        "baseline",
        "<none>",
        "macos-sdk",
        "O0",
        "precise",
        "off",
        "ieee",
        "scalar baseline",
        "macos",
        "libSystem",
        "libSystem",
        "nearest_ties_even",
        true,
    );
    let identity = BuildIdentity::new(
        BuildIdentityFields::new(
            REVISION,
            "adapter-v1",
            "c7f36eaf2f184a36b9c9a04636d3e22785d815c4948d55d0b3cbf44ee7245fc8",
            "oracle-debug",
            "AppleClang",
            "21.0.0",
            "arm64-apple-darwin",
            "Debug",
            "-O0",
            "<none>",
            "none",
        )
        .with_phase4(phase4),
    )
    .expect("full identity should validate");
    let record = serde_json::json!({
        "protocol_version": 1,
        "record_kind": "handshake",
        "supported_scenario_versions": [1],
        "supported_trace_versions": [1],
        "supported_tolerance_versions": [1],
        "build_identity": {
            "oracle_revision": REVISION,
            "adapter_revision": "adapter-v1",
            "adapter_content_sha256": "c7f36eaf2f184a36b9c9a04636d3e22785d815c4948d55d0b3cbf44ee7245fc8",
            "cmake_preset": "oracle-debug",
            "compiler_id": "AppleClang",
            "compiler_version": "21.0.0",
            "target": "arm64-apple-darwin",
            "build_type": "Debug",
            "effective_compile_flags": "-O0",
            "effective_link_flags": "<none>",
            "sanitizer_mode": "none",
            "compile_command_sha256": "33".repeat(32),
            "target_triple": "arm64-apple-darwin",
            "target_cpu": "baseline",
            "target_features": "<none>",
            "sdk_or_sysroot": "macos-sdk",
            "optimization": "O0",
            "fp_model": "precise",
            "fp_contract": "off",
            "denormal_mode": "ieee",
            "feature_set": "scalar baseline",
            "os": "macos",
            "libc": "libSystem",
            "libm": "libSystem",
            "rounding_mode": "nearest_ties_even",
            "gradual_underflow": true
        },
        "identity_sha256": identity.identity_sha256().as_str()
    });
    let mut jsonl = serde_json::to_vec(&record).expect("handshake should encode");
    jsonl.push(b'\n');

    // Act
    let decoded = decode_handshake_jsonl(&jsonl, &HarnessLimits::phase2_default_v1())
        .expect("full Phase 4 handshake should decode");

    // Assert
    let decoded_phase4 = decoded
        .build_identity()
        .maybe_phase4()
        .expect("Phase 4 extension should be present");
    assert_eq!(decoded_phase4.compile_command_sha256(), "33".repeat(32));
    assert_eq!(decoded_phase4.compiler_id(), "AppleClang");
    assert_eq!(decoded_phase4.target_triple(), "arm64-apple-darwin");
    assert!(decoded_phase4.gradual_underflow());
}
