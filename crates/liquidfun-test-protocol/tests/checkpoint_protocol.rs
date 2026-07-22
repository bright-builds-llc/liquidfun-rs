//! Integration coverage for resolved-run requests and canonical checkpoints.

use liquidfun_test_protocol::{
    CanonicalCheckpoint, CatalogDefinition, CatalogProgram, CatalogRunRequest, CatalogSlug,
    CheckpointDecodeError, CheckpointErrorKind, CheckpointId, CheckpointPosition,
    CheckpointProfileName, CheckpointSchemaVersion, CheckpointSet, CodecErrorKind, DebugColorBits,
    DebugFillBits, DebugLayerName, DebugOwnerId, DebugPrimitiveKey, DebugPrimitiveKindName,
    DebugPrimitiveOrder, DebugPrimitiveRecord, DebugStrokeBits, EvidenceTier, FloatBits,
    GeneratorId, GeneratorVersion, HarnessLimits, MathProbePolicyPath, NumericObservation,
    OccurrenceKind, OrderedOccurrence, PrimitivePoint, ProtocolVersion, RequestId, ResolveRequest,
    RunProvenanceRequirements, RunSettings, ScenarioEligibility, ScenarioId, ScenarioVersion,
    Sha256Hex, StructuralObservation, StructuralValue, Vec2Bits, WireDebugPrimitive,
    decode_canonical_checkpoint_jsonl, decode_catalog_run_request_jsonl,
    encode_canonical_checkpoint_jsonl, encode_catalog_run_request_jsonl, resolve_catalog,
};
use serde_json::{Value, json};

const TIMESTEP_BITS: u32 = 0x3c88_8889;

fn id(value: &str) -> ScenarioId {
    ScenarioId::new(value).expect("fixture semantic ID should validate")
}

fn resolved() -> liquidfun_test_protocol::ResolvedScenario {
    let definition = CatalogDefinition::new(
        CatalogSlug::new("checkpoint-world").expect("fixture slug should validate"),
        "Checkpoint world",
        ScenarioVersion::CURRENT,
        GeneratorId::new("static-checkpoint").expect("fixture generator should validate"),
        GeneratorVersion::CURRENT,
        ScenarioEligibility::NamedOnly,
        Vec::new(),
        CatalogProgram::exact_gravity(
            Vec2Bits {
                x_bits: FloatBits::new(0.0_f32.to_bits()),
                y_bits: FloatBits::new((-10.0_f32).to_bits()),
            },
            1,
        )
        .expect("fixture program should validate"),
    )
    .expect("fixture definition should validate");
    let settings = RunSettings::new(FloatBits::new(TIMESTEP_BITS), 8, 3, 1)
        .expect("fixture settings should validate");
    resolve_catalog(
        &[definition],
        &ResolveRequest::new(
            CatalogSlug::new("checkpoint-world").expect("fixture slug should validate"),
            None,
            settings,
        ),
    )
    .expect("fixture should resolve")
}

fn run_requirements() -> RunProvenanceRequirements {
    RunProvenanceRequirements::new(
        Sha256Hex::new("1".repeat(64)).expect("fixture identity hash should validate"),
        HarnessLimits::phase2_default_v1().profile_sha256(),
        EvidenceTier::D1Canonical,
    )
}

fn run_request() -> CatalogRunRequest {
    CatalogRunRequest::new(
        RequestId::new("run-request-1").expect("fixture request ID should validate"),
        resolved(),
        run_requirements(),
    )
    .expect("fixture run request should validate")
}

fn checkpoint() -> CanonicalCheckpoint {
    let stroke = DebugStrokeBits::new(
        DebugColorBits::rgba(0x58, 0xa6, 0xff, 0xff),
        FloatBits::new(0.125_f32.to_bits()),
    )
    .expect("fixture stroke should validate");
    let primitive = DebugPrimitiveRecord::new(
        DebugPrimitiveOrder::SourceSignificant,
        WireDebugPrimitive::Point(PrimitivePoint::new(
            DebugPrimitiveKey::new(
                DebugOwnerId::World,
                DebugLayerName::Contacts,
                DebugPrimitiveKindName::Point,
                0,
                0,
            ),
            stroke,
            Some(DebugFillBits::new(DebugColorBits::rgba(
                0xd2, 0x99, 0x22, 0xff,
            ))),
            Vec2Bits {
                x_bits: FloatBits::new(1.0_f32.to_bits()),
                y_bits: FloatBits::new(2.0_f32.to_bits()),
            },
            FloatBits::new(0.05_f32.to_bits()),
        )),
    );
    CanonicalCheckpoint::new(
        RequestId::new("run-request-1").expect("fixture request ID should validate"),
        resolved().identity().content_sha256().clone(),
        CheckpointId::new("checkpoint-0001").expect("fixture checkpoint ID should validate"),
        CheckpointPosition::LogicalStep { ordinal: 1 },
        FloatBits::new(TIMESTEP_BITS),
        vec![StructuralObservation::new(
            id("world.body-count"),
            StructuralValue::Count(1),
        )],
        vec![NumericObservation::new(
            id("world.contact-distance"),
            FloatBits::new(1.0_f32.to_bits()),
            MathProbePolicyPath::MathVectorLength,
        )],
        vec![OrderedOccurrence::new(
            id("contact.begin-0001"),
            OccurrenceKind::ContactBegin,
            id("contact-1"),
        )],
        vec![
            CheckpointSet::new(id("world.active-bodies"), vec![id("body-2"), id("body-1")])
                .expect("fixture set should canonicalize"),
        ],
        vec![primitive],
        vec![CheckpointProfileName::ContactLifecycle],
    )
    .expect("fixture checkpoint should validate")
}

fn json_line(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("fixture JSON should encode");
    bytes.push(b'\n');
    bytes
}

#[test]
fn resolved_request_round_trip_carries_exact_bytes_and_identity() {
    // Arrange
    let request = run_request();
    let limits = HarnessLimits::phase2_default_v1();

    // Act
    let bytes = encode_catalog_run_request_jsonl(&request, &limits)
        .expect("valid run request should encode");
    let decoded =
        decode_catalog_run_request_jsonl(&bytes, &limits).expect("valid run request should decode");

    // Assert
    assert_eq!(decoded, request);
    assert_eq!(
        decoded.resolved().canonical_bytes(),
        resolved().canonical_bytes()
    );
    assert_eq!(
        decoded.resolved().identity().content_sha256(),
        resolved().identity().content_sha256()
    );
    assert_eq!(
        decoded.provenance_requirements().evidence_tier(),
        EvidenceTier::D1Canonical
    );
}

#[test]
fn canonical_checkpoint_round_trip_preserves_declared_semantics() {
    // Arrange
    let checkpoint = checkpoint();
    let limits = HarnessLimits::phase2_default_v1();

    // Act
    let bytes = encode_canonical_checkpoint_jsonl(&checkpoint, &limits)
        .expect("valid checkpoint should encode");
    let decoded =
        decode_canonical_checkpoint_jsonl(&bytes, &limits).expect("valid checkpoint should decode");

    // Assert
    assert_eq!(decoded, checkpoint);
    assert_eq!(decoded.schema_version(), CheckpointSchemaVersion::CURRENT);
    assert_eq!(
        decoded.unordered_sets()[0].members(),
        &[id("body-1"), id("body-2")]
    );
    assert_eq!(
        decoded.profile_names(),
        &[CheckpointProfileName::ContactLifecycle]
    );
    assert!(
        !String::from_utf8(bytes)
            .expect("checkpoint should be UTF-8")
            .contains("duration")
    );
}

#[test]
fn run_request_rejects_unknown_duplicate_wrong_hash_and_wrong_version() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let encoded = encode_catalog_run_request_jsonl(&run_request(), &limits)
        .expect("fixture run request should encode");
    let mut value: Value = serde_json::from_slice(&encoded).expect("fixture JSON should parse");
    let mut unknown = value.clone();
    unknown["private_slot"] = json!(7);
    let duplicate = String::from_utf8(encoded.clone())
        .expect("fixture should be UTF-8")
        .replacen(
            "\"protocol_version\":1",
            "\"protocol_version\":1,\"protocol_version\":1",
            1,
        )
        .into_bytes();
    value["resolved_sha256"] = json!("f".repeat(64));
    let wrong_hash = json_line(&value);
    let mut wrong_version: Value =
        serde_json::from_slice(&encoded).expect("fixture JSON should parse");
    wrong_version["catalog_schema_version"] = json!(2);

    // Act
    let unknown_error = decode_catalog_run_request_jsonl(&json_line(&unknown), &limits)
        .expect_err("unknown fields must fail");
    let duplicate_error = decode_catalog_run_request_jsonl(&duplicate, &limits)
        .expect_err("duplicate fields must fail");
    let hash_error = decode_catalog_run_request_jsonl(&wrong_hash, &limits)
        .expect_err("wrong resolved hash must fail");
    let version_error = decode_catalog_run_request_jsonl(&json_line(&wrong_version), &limits)
        .expect_err("wrong catalog version must fail");

    // Assert
    assert_eq!(
        unknown_error.codec_kind(),
        Some(CodecErrorKind::UnknownField)
    );
    assert_eq!(
        duplicate_error.codec_kind(),
        Some(CodecErrorKind::DuplicateMember)
    );
    assert_eq!(
        hash_error.validation_kind(),
        Some(CheckpointErrorKind::HashMismatch)
    );
    assert_eq!(
        version_error.codec_kind(),
        Some(CodecErrorKind::UnsupportedVersion)
    );
}

#[test]
fn checkpoint_rejects_unknown_duplicate_nonfinite_dangling_and_order_violations() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let encoded = encode_canonical_checkpoint_jsonl(&checkpoint(), &limits)
        .expect("fixture checkpoint should encode");
    let original: Value = serde_json::from_slice(&encoded).expect("fixture JSON should parse");
    let mut unknown = original.clone();
    unknown["observations"][0]["value"]["kind"] = json!("private_storage_row");
    let duplicate = String::from_utf8(encoded.clone())
        .expect("fixture should be UTF-8")
        .replacen(
            "\"checkpoint_schema_version\":1",
            "\"checkpoint_schema_version\":1,\"checkpoint_schema_version\":1",
            1,
        )
        .into_bytes();
    let mut nonfinite = original.clone();
    nonfinite["numeric_observations"][0]["value_bits"] = json!(f32::INFINITY.to_bits());
    let mut dangling = original.clone();
    dangling["checkpoint_id"] = json!("checkpoint-9999");
    let mut order = original.clone();
    let duplicate_occurrence = order["ordered_occurrences"][0].clone();
    order["ordered_occurrences"]
        .as_array_mut()
        .expect("fixture occurrences should be an array")
        .push(duplicate_occurrence);

    // Act
    let errors = [
        decode_canonical_checkpoint_jsonl(&json_line(&unknown), &limits),
        decode_canonical_checkpoint_jsonl(&duplicate, &limits),
        decode_canonical_checkpoint_jsonl(&json_line(&nonfinite), &limits),
        decode_canonical_checkpoint_jsonl(&json_line(&dangling), &limits),
        decode_canonical_checkpoint_jsonl(&json_line(&order), &limits),
    ];

    // Assert
    assert!(matches!(errors[0], Err(CheckpointDecodeError::Codec(_))));
    assert_eq!(
        errors[1]
            .as_ref()
            .expect_err("duplicate must fail")
            .codec_kind(),
        Some(CodecErrorKind::DuplicateMember)
    );
    assert_eq!(
        errors[2]
            .as_ref()
            .expect_err("nonfinite must fail")
            .validation_kind(),
        Some(CheckpointErrorKind::InvalidFloat)
    );
    assert_eq!(
        errors[3]
            .as_ref()
            .expect_err("dangling must fail")
            .validation_kind(),
        Some(CheckpointErrorKind::CheckpointMismatch)
    );
    assert_eq!(
        errors[4]
            .as_ref()
            .expect_err("duplicate occurrence must fail")
            .validation_kind(),
        Some(CheckpointErrorKind::DuplicateSemanticId)
    );
}

#[test]
fn checkpoint_bounds_reject_the_first_excess_before_acceptance() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let encoded = encode_canonical_checkpoint_jsonl(&checkpoint(), &limits)
        .expect("fixture checkpoint should encode");
    let mut value: Value = serde_json::from_slice(&encoded).expect("fixture JSON should parse");
    let observation = value["observations"][0].clone();
    let observations = value["observations"]
        .as_array_mut()
        .expect("fixture observations should be an array");
    for ordinal in 1..=limits.observables_per_checkpoint() {
        let mut extra = observation.clone();
        extra["observation_id"] = json!(format!("world.extra-{ordinal:04}"));
        observations.push(extra);
    }

    // Act
    let error = decode_canonical_checkpoint_jsonl(&json_line(&value), &limits)
        .expect_err("N + 1 observations must fail");

    // Assert
    assert_eq!(
        error.codec_kind(),
        Some(CodecErrorKind::BoundaryLimitExceeded)
    );
}

#[test]
fn forbidden_private_and_render_fields_are_absent_from_the_wire_contract() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();

    // Act
    let run_text = String::from_utf8(
        encode_catalog_run_request_jsonl(&run_request(), &limits)
            .expect("fixture run request should encode"),
    )
    .expect("run request should be UTF-8");
    let checkpoint_text = String::from_utf8(
        encode_canonical_checkpoint_jsonl(&checkpoint(), &limits)
            .expect("fixture checkpoint should encode"),
    )
    .expect("checkpoint should be UTF-8");

    // Assert
    for forbidden in [
        "duration",
        "frame",
        "pixel",
        "render_order",
        "pointer",
        "arena_slot",
        "proxy_id",
        "dense_index",
    ] {
        assert!(!run_text.contains(forbidden));
        assert!(!checkpoint_text.contains(forbidden));
    }
    assert_eq!(ProtocolVersion::CURRENT.get(), 1);
}
