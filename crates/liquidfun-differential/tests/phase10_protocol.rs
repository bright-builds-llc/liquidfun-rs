//! Strict shared Phase 10 particle-group protocol contracts.

use liquidfun_differential::NativeRigidWorldExecutor;
use liquidfun_test_protocol::{
    CodecErrorKind, FloatBits, HarnessLimits, Phase10BehaviorLeaf, Phase10GroupDefinition,
    Phase10GroupDestination, Phase10GroupSource, Phase10Observation, Phase10Operation,
    Phase10PairSnapshot, Phase10Provenance, Phase10SemanticOutcome, Phase10Shape,
    Phase10StateObservation, Phase10TriadSnapshot, Phase10ValidationKind, Phase10Witness,
    Phase10WitnessObservation, RecordLimit, RigidWorldDecodeError, ScenarioId, TransformBits,
    Vec2Bits, WitnessRole, decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl,
    encode_jsonl, validate_phase10_operation,
};
use serde_json::{Value, json};

const PHASE8_REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
const PHASE9_REQUEST: &[u8] =
    include_bytes!("fixtures/rigid_world/phase9/cases/storage-systems-and-permutations.jsonl");
const SCENARIO_SCHEMA: &[u8] = include_bytes!("../../../protocol/schemas/scenario-v1.schema.json");
const TRACE_SCHEMA: &[u8] = include_bytes!("../../../protocol/schemas/trace-v1.schema.json");

fn id(value: &str) -> ScenarioId {
    ScenarioId::new(value).expect("test semantic ID should be valid")
}

fn bits(value: f32) -> FloatBits {
    FloatBits::from_f32(value)
}

fn vector(x: f32, y: f32) -> Vec2Bits {
    Vec2Bits {
        x_bits: bits(x),
        y_bits: bits(y),
    }
}

fn definition() -> Phase10GroupDefinition {
    Phase10GroupDefinition {
        provenance: Phase10Provenance {
            extension_version: 1,
            generator_id: id("phase10-test-generator"),
            generator_version: id("v1"),
            upstream_revision: id("upstream-revision"),
            toolchain_id: id("rust-test-toolchain"),
            seed: 42,
        },
        system_id: id("system-a"),
        group_id: id("group-a"),
        member_ids: vec![id("particle-a"), id("particle-b")].into_boxed_slice(),
        source: Phase10GroupSource::Filled {
            shapes: vec![Phase10Shape::Circle {
                center: vector(0.0, 0.0),
                radius_bits: bits(1.0),
            }]
            .into_boxed_slice(),
        },
        destination: Phase10GroupDestination::New,
        particle_flags_bits: 1 << 3,
        group_flags_bits: 1,
        transform: TransformBits {
            position: vector(2.0, 3.0),
            angle_bits: bits(0.25),
        },
        linear_velocity: vector(4.0, 5.0),
        angular_velocity_bits: bits(0.5),
        color: [1, 2, 3, 4],
        strength_bits: bits(0.75),
        maybe_stride_bits: Some(bits(0.25)),
        lifetime_bits: bits(8.0),
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "one literal fixture keeps the complete strict wire schema visible"
)]
fn phase10_request_value() -> Value {
    let mut value: Value =
        serde_json::from_slice(PHASE8_REQUEST).expect("Phase 8 fixture should be JSON");
    let timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .first_mut()
        .expect("fixture should contain a timeline");
    timeline["particle_systems"] = json!([{
        "system_id": "system-a",
        "buffer_mode": { "kind": "growable", "initial_capacity": 256 },
        "paused": false,
        "strict_contact_check": true,
        "stuck_threshold": 2,
        "density_bits": bits(1.0).bits(),
        "gravity_scale_bits": bits(1.0).bits(),
        "radius_bits": bits(0.25).bits(),
        "damping_bits": bits(0.0).bits(),
        "destruction_by_age": true,
        "lifetime_granularity_bits": bits(1.0 / 60.0).bits(),
        "maximum_count": null
    }]);
    timeline["particles"] = json!([]);
    let mut definition_a = serde_json::to_value(definition()).expect("definition should encode");
    definition_a["source"] = json!({
        "kind": "explicit",
        "positions": [
            { "x_bits": bits(0.0).bits(), "y_bits": bits(0.0).bits() },
            { "x_bits": bits(0.5).bits(), "y_bits": bits(0.0).bits() }
        ]
    });
    let mut append = definition_a.clone();
    append["member_ids"] = json!(["particle-c"]);
    append["source"] = json!({ "kind": "explicit", "positions": [
        { "x_bits": bits(1.0).bits(), "y_bits": bits(0.0).bits() }
    ] });
    append["destination"] = json!({ "kind": "append_to", "target_group_id": "group-a" });
    let mut definition_b = definition_a.clone();
    definition_b["group_id"] = json!("group-b");
    definition_b["member_ids"] = json!(["particle-d"]);
    definition_b["source"] = json!({ "kind": "explicit", "positions": [
        { "x_bits": bits(1.5).bits(), "y_bits": bits(0.0).bits() }
    ] });
    let actions = timeline["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    let operations = [
        (
            "p10-create-system",
            json!({ "kind": "particle", "action": { "kind": "create_system", "system_id": "system-a" } }),
        ),
        (
            "p10-create-a",
            json!({ "kind": "particle_group", "operation": { "kind": "create_group", "definition": definition_a } }),
        ),
        (
            "p10-append-a",
            json!({ "kind": "particle_group", "operation": { "kind": "create_group", "definition": append } }),
        ),
        (
            "p10-create-b",
            json!({ "kind": "particle_group", "operation": { "kind": "create_group", "definition": definition_b } }),
        ),
        (
            "p10-join",
            json!({ "kind": "particle_group", "operation": { "kind": "join_groups", "target_group_id": "group-a", "source_group_id": "group-b" } }),
        ),
        (
            "p10-split",
            json!({ "kind": "particle_group", "operation": { "kind": "split_group", "group_id": "group-a", "created_group_ids": ["group-c", "group-d"] } }),
        ),
        (
            "p10-flags",
            json!({ "kind": "particle_group", "operation": { "kind": "set_group_flags", "group_id": "group-a", "group_flags_bits": 3 } }),
        ),
        (
            "p10-step",
            json!({ "kind": "particle_group", "operation": { "kind": "step", "timestep_bits": bits(1.0 / 60.0).bits(), "velocity_iterations": 8, "position_iterations": 3, "particle_iterations": 2 } }),
        ),
        (
            "p10-inspect",
            json!({ "kind": "particle_group", "operation": { "kind": "inspect_state" } }),
        ),
        (
            "p10-destroy-c",
            json!({ "kind": "particle_group", "operation": { "kind": "destroy_group", "group_id": "group-c" } }),
        ),
        (
            "p10-destroy-d",
            json!({ "kind": "particle_group", "operation": { "kind": "destroy_group", "group_id": "group-d" } }),
        ),
        (
            "p10-destroy-a",
            json!({ "kind": "particle_group", "operation": { "kind": "destroy_group", "group_id": "group-a" } }),
        ),
        (
            "p10-destroy-system",
            json!({ "kind": "particle", "action": { "kind": "destroy_system", "system_id": "system-a" } }),
        ),
    ];
    for (action_id, action) in operations {
        actions.push(json!({ "action_id": action_id, "phase": "phase10", "action": action }));
    }
    let checkpoint = timeline["checkpoints"]
        .as_array_mut()
        .expect("fixture checkpoints should be an array")
        .last_mut()
        .expect("fixture should contain a checkpoint");
    checkpoint["after_action_id"] = json!("p10-destroy-system");
    checkpoint["phase"] = json!("phase10");
    value
}

fn encode_value(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).expect("fixture mutation should encode");
    bytes.push(b'\n');
    bytes
}

fn phase10_create_definition_mut(value: &mut Value) -> &mut Value {
    value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .iter_mut()
        .find(|action| action["action_id"] == "p10-create-a")
        .expect("create action should exist")
        .pointer_mut("/action/operation/definition")
        .expect("definition should exist")
}

#[test]
fn semantic_group_definition_preserves_exact_bits_ids_and_source_order() {
    // Arrange
    let operation = Phase10Operation::CreateGroup {
        definition: definition(),
    };

    // Act
    let encoded = serde_json::to_value(&operation).expect("semantic operation should encode");
    let decoded: Phase10Operation =
        serde_json::from_value(encoded.clone()).expect("semantic operation should decode");

    // Assert
    assert_eq!(decoded, operation);
    assert_eq!(encoded["kind"], "create_group");
    assert_eq!(
        encoded["definition"]["member_ids"],
        json!(["particle-a", "particle-b"])
    );
    assert_eq!(encoded["definition"]["strength_bits"], bits(0.75).bits());
}

#[test]
fn semantic_full_triad_preserves_all_pinned_coefficients() {
    // Arrange
    let triad = Phase10TriadSnapshot {
        ordinal: 0,
        particle_a_id: id("particle-a"),
        particle_b_id: id("particle-b"),
        particle_c_id: id("particle-c"),
        flags_bits: 1 << 4,
        strength_bits: bits(0.5),
        pa: vector(-1.0, 0.0),
        pb: vector(1.0, 0.0),
        pc: vector(0.0, 1.0),
        ka_bits: bits(1.0),
        kb_bits: bits(2.0),
        kc_bits: bits(3.0),
        s_bits: bits(4.0),
    };

    // Act
    let encoded = serde_json::to_value(&triad).expect("triad should encode");
    let decoded: Phase10TriadSnapshot =
        serde_json::from_value(encoded).expect("triad should decode");

    // Assert
    assert_eq!(decoded, triad);
    assert_eq!(decoded.pa, vector(-1.0, 0.0));
    assert_eq!(decoded.pb, vector(1.0, 0.0));
    assert_eq!(decoded.pc, vector(0.0, 1.0));
    assert_eq!(decoded.ka_bits, bits(1.0));
    assert_eq!(decoded.kb_bits, bits(2.0));
    assert_eq!(decoded.kc_bits, bits(3.0));
    assert_eq!(decoded.s_bits, bits(4.0));
}

#[test]
fn semantic_witness_exposes_role_and_behavior_without_private_pass_data() {
    // Arrange
    let witness = Phase10Witness {
        ordinal: 0,
        behavior_leaf: Phase10BehaviorLeaf::Spring,
        role: WitnessRole::Interaction,
        observation: Phase10WitnessObservation::Topology {
            pair_count: 1,
            triad_count: 1,
        },
    };

    // Act
    let value = serde_json::to_value(&witness).expect("witness should encode");
    let object = value.as_object().expect("witness should be an object");

    // Assert
    assert_eq!(value["behavior_leaf"], "spring");
    assert_eq!(value["role"], "interaction");
    assert!(!object.contains_key("pass_id"));
    assert!(!object.contains_key("pass_trace"));
    assert!(!object.contains_key("pass_inventory"));
}

#[test]
fn wire_phase10_request_normalizes_to_byte_identical_canonical_json() {
    // Arrange
    let value = phase10_request_value();
    let limits = HarnessLimits::phase2_default_v1();
    let request = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect("complete Phase 10 request should decode");

    // Act
    let canonical = encode_jsonl(&request, &limits, RecordLimit::Input)
        .expect("validated request should encode");
    let replay = decode_rigid_world_request_jsonl(&canonical, &limits)
        .expect("canonical request should decode");
    let replayed =
        encode_jsonl(&replay, &limits, RecordLimit::Input).expect("replay should encode");

    // Assert
    assert_eq!(replayed, canonical);
}

#[test]
fn tracked_schemas_accept_complete_phase10_request_and_result() {
    // Arrange
    let request_value = phase10_request_value();
    let limits = HarnessLimits::phase2_default_v1();
    let request = decode_rigid_world_request_jsonl(&encode_value(&request_value), &limits)
        .expect("complete Phase 10 request should decode");
    let result =
        NativeRigidWorldExecutor::execute(&request).expect("complete Phase 10 request should run");
    let scenario_schema: Value =
        serde_json::from_slice(SCENARIO_SCHEMA).expect("scenario schema should be valid JSON");
    let trace_schema: Value =
        serde_json::from_slice(TRACE_SCHEMA).expect("trace schema should be valid JSON");
    let scenario_validator =
        jsonschema::validator_for(&scenario_schema).expect("scenario schema should compile");
    let trace_validator =
        jsonschema::validator_for(&trace_schema).expect("trace schema should compile");
    let result_value = serde_json::to_value(result).expect("Phase 10 result should encode");

    // Act
    let scenario_validation = scenario_validator.validate(&request_value["scenario"]);
    let trace_validation = trace_validator.validate(&result_value);

    // Assert
    assert!(
        scenario_validation.is_ok(),
        "complete Phase 10 request should match the tracked scenario schema: {scenario_validation:?}"
    );
    assert!(
        trace_validation.is_ok(),
        "complete Phase 10 result should match the tracked trace schema: {trace_validation:?}"
    );
}

#[test]
fn wire_rejects_unknown_duplicate_private_and_unknown_tag_members() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut private = phase10_request_value();
    phase10_create_definition_mut(&mut private)["pass_id"] = json!("s19");
    let mut unknown_tag = phase10_request_value();
    unknown_tag["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .iter_mut()
        .find(|action| action["action_id"] == "p10-inspect")
        .expect("inspect action should exist")["action"]["operation"]["kind"] = json!("pass_trace");
    let canonical = encode_value(&phase10_request_value());
    let canonical = String::from_utf8(canonical).expect("fixture should be UTF-8");
    let duplicate = canonical.replacen(
        "\"extension_version\":1",
        "\"extension_version\":1,\"extension_version\":1",
        1,
    );

    // Act
    let private_error = decode_rigid_world_request_jsonl(&encode_value(&private), &limits)
        .expect_err("private pass identity must be rejected");
    let tag_error = decode_rigid_world_request_jsonl(&encode_value(&unknown_tag), &limits)
        .expect_err("unknown operation tag must be rejected");
    let duplicate_error = decode_rigid_world_request_jsonl(duplicate.as_bytes(), &limits)
        .expect_err("duplicate member must be rejected");

    // Assert
    assert_eq!(
        codec_kind(&private_error),
        Some(CodecErrorKind::UnknownField)
    );
    assert_eq!(
        codec_kind(&tag_error),
        Some(CodecErrorKind::UnknownRecordKind)
    );
    assert_eq!(
        codec_kind(&duplicate_error),
        Some(CodecErrorKind::DuplicateMember)
    );
}

#[test]
fn wire_rejects_duplicate_ids_wrong_ownership_flags_versions_and_nonfinite_values() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut duplicate = phase10_request_value();
    phase10_create_definition_mut(&mut duplicate)["member_ids"] =
        json!(["particle-a", "particle-a"]);
    let mut wrong_owner = phase10_request_value();
    let append = wrong_owner["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .iter_mut()
        .find(|action| action["action_id"] == "p10-append-a")
        .expect("append action should exist");
    append["action"]["operation"]["definition"]["group_id"] = json!("group-other");
    let mut private_flags = phase10_request_value();
    phase10_create_definition_mut(&mut private_flags)["group_flags_bits"] = json!(0x0018);
    let mut wrong_version = phase10_request_value();
    phase10_create_definition_mut(&mut wrong_version)["provenance"]["extension_version"] = json!(2);
    let mut nonfinite = phase10_request_value();
    phase10_create_definition_mut(&mut nonfinite)["transform"]["angle_bits"] =
        json!(f32::NAN.to_bits());

    // Act
    let results = [
        duplicate,
        wrong_owner,
        private_flags,
        wrong_version,
        nonfinite,
    ]
    .map(|value| decode_rigid_world_request_jsonl(&encode_value(&value), &limits));

    // Assert
    assert!(results.into_iter().all(|result| {
        result
            .expect_err("malformed Phase 10 semantic input must fail")
            .rigid_world_kind()
            == Some(liquidfun_test_protocol::RigidWorldErrorKind::InvalidParticleGroupAction)
    }));
}

#[test]
fn wire_particle_boundary_accepts_limit_and_rejects_one_over() {
    // Arrange
    let mut at_limit = definition();
    at_limit.member_ids = (0..liquidfun_test_protocol::PHASE10_MAXIMUM_PARTICLES)
        .map(|index| id(&format!("particle-{index}")))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    at_limit.source = Phase10GroupSource::Filled {
        shapes: vec![Phase10Shape::Circle {
            center: vector(0.0, 0.0),
            radius_bits: bits(1.0),
        }]
        .into_boxed_slice(),
    };
    let at_limit = Phase10Operation::CreateGroup {
        definition: at_limit.clone(),
    };
    let mut over_definition = at_limit_definition(&at_limit);
    over_definition.member_ids = (0..=liquidfun_test_protocol::PHASE10_MAXIMUM_PARTICLES)
        .map(|index| id(&format!("over-particle-{index}")))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let one_over = Phase10Operation::CreateGroup {
        definition: over_definition,
    };

    // Act
    let accepted = validate_phase10_operation(&at_limit);
    let rejected = validate_phase10_operation(&one_over);

    // Assert
    assert_eq!(accepted, Ok(()));
    assert_eq!(
        rejected.map_err(liquidfun_test_protocol::Phase10ValidationError::kind),
        Err(Phase10ValidationKind::BoundaryLimitExceeded)
    );
}

#[test]
fn wire_phase9_request_and_result_variants_round_trip_unchanged() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let request = decode_rigid_world_request_jsonl(PHASE9_REQUEST, &limits)
        .expect("reviewed Phase 9 request should decode");
    let result =
        NativeRigidWorldExecutor::execute(&request).expect("reviewed Phase 9 request should run");

    // Act
    let request_bytes =
        encode_jsonl(&request, &limits, RecordLimit::Input).expect("request should encode");
    let result_bytes =
        encode_jsonl(&result, &limits, RecordLimit::Output).expect("result should encode");
    let replay_request = decode_rigid_world_request_jsonl(&request_bytes, &limits)
        .expect("request replay should decode");
    let replay_result = decode_rigid_world_result_jsonl(&result_bytes, &limits)
        .expect("result replay should decode");

    // Assert
    assert_eq!(
        encode_jsonl(&replay_request, &limits, RecordLimit::Input),
        Ok(request_bytes)
    );
    assert_eq!(
        encode_jsonl(&replay_result, &limits, RecordLimit::Output),
        Ok(result_bytes)
    );
}

#[test]
fn wire_result_rejects_duplicate_witness_binding_order_and_nonfinite_observation() {
    // Arrange
    let witness = Phase10Witness {
        ordinal: 0,
        behavior_leaf: Phase10BehaviorLeaf::Spring,
        role: WitnessRole::Interaction,
        observation: Phase10WitnessObservation::Scalar {
            value_bits: bits(1.0),
        },
    };
    let mut duplicate = empty_observation();
    let Phase10Observation::State { state } = &mut duplicate;
    let mut repeated = witness.clone();
    repeated.ordinal = 1;
    state.witnesses = vec![witness.clone(), repeated].into_boxed_slice();
    let mut wrong_order = empty_observation();
    let Phase10Observation::State { state } = &mut wrong_order;
    let mut ordinal_one = witness.clone();
    ordinal_one.ordinal = 1;
    state.witnesses = vec![ordinal_one].into_boxed_slice();
    let mut nonfinite = empty_observation();
    let Phase10Observation::State { state } = &mut nonfinite;
    let mut nan_witness = witness;
    nan_witness.observation = Phase10WitnessObservation::Scalar {
        value_bits: FloatBits::new(f32::NAN.to_bits()),
    };
    state.witnesses = vec![nan_witness].into_boxed_slice();

    // Act
    let results =
        [duplicate, wrong_order, nonfinite].map(|observation| observation.validate_semantics());

    // Assert
    assert_eq!(
        results[0].map_err(liquidfun_test_protocol::Phase10ValidationError::kind),
        Err(Phase10ValidationKind::InvalidWitness)
    );
    assert_eq!(
        results[1].map_err(liquidfun_test_protocol::Phase10ValidationError::kind),
        Err(Phase10ValidationKind::InvalidOrdering)
    );
    assert_eq!(
        results[2].map_err(liquidfun_test_protocol::Phase10ValidationError::kind),
        Err(Phase10ValidationKind::InvalidFloat)
    );
}

fn empty_observation() -> Phase10Observation {
    Phase10Observation::State {
        state: Phase10StateObservation {
            provenance: Phase10Provenance {
                extension_version: 1,
                generator_id: id("phase10-test-generator"),
                generator_version: id("v1"),
                upstream_revision: id("upstream-revision"),
                toolchain_id: id("rust-test-toolchain"),
                seed: 42,
            },
            outcome: Phase10SemanticOutcome::Completed,
            groups: Box::new([]),
            particles: Box::new([]),
            pairs: Box::new([]),
            triads: Box::new([]),
            particle_contacts: Box::new([]),
            body_contacts: Box::new([]),
            events: Box::new([]),
            witnesses: Box::new([]),
        },
    }
}

fn codec_kind(error: &RigidWorldDecodeError) -> Option<CodecErrorKind> {
    match error {
        RigidWorldDecodeError::Codec(error) => Some(error.kind()),
        RigidWorldDecodeError::Validation(_) => None,
    }
}

fn at_limit_definition(operation: &Phase10Operation) -> Phase10GroupDefinition {
    let Phase10Operation::CreateGroup { definition } = operation else {
        panic!("test operation should create a group");
    };
    definition.clone()
}

// Keep the complete public result surface linked into this integration target.
fn _semantic_surface_compile_guard(
    _provenance: Phase10Provenance,
    _outcome: Phase10SemanticOutcome,
    _state: Phase10StateObservation,
    _pair: Phase10PairSnapshot,
    _value: Value,
) {
}
