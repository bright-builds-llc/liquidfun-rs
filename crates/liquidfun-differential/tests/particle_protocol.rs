//! Closed Phase 9 particle protocol contracts.

use liquidfun_differential::{
    NativeRigidWorldExecutor, PHASE9_REGISTRY_ID, PHASE9_REQUIRED_POLICY_PATHS, Phase9PolicyKind,
    phase9_policy_for_path,
};
use liquidfun_test_protocol::{
    HarnessLimits, RigidWorldErrorKind, RigidWorldRequestRecord, RigidWorldResultRecord,
    decode_rigid_world_request_jsonl, decode_rigid_world_result_jsonl,
    validate_rigid_world_result_against_request,
};
use serde_json::{Value, json};

const PHASE8_REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");

fn decode_value(value: &Value) -> Result<liquidfun_test_protocol::RigidWorldRequestRecord, String> {
    let mut bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .map_err(|error| error.to_string())
}

fn phase9_lifecycle_value() -> Value {
    let mut value: Value =
        serde_json::from_slice(PHASE8_REQUEST).expect("checked-in Phase 8 request should be JSON");
    let timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .first_mut()
        .expect("fixture should contain a timeline");
    timeline["particle_systems"] = json!([
        {
            "system_id": "phase9-system-a",
            "buffer_mode": { "kind": "growable", "initial_capacity": 4 },
            "paused": false, "strict_contact_check": true, "stuck_threshold": 2,
            "density_bits": 1065353216, "gravity_scale_bits": 1065353216,
            "radius_bits": 1036831949, "damping_bits": 0, "destruction_by_age": true,
            "lifetime_granularity_bits": 1008981770, "maximum_count": 8
        },
        {
            "system_id": "phase9-system-b",
            "buffer_mode": { "kind": "growable", "initial_capacity": 4 },
            "paused": false, "strict_contact_check": false, "stuck_threshold": 0,
            "density_bits": 1065353216, "gravity_scale_bits": 1065353216,
            "radius_bits": 1036831949, "damping_bits": 0, "destruction_by_age": false,
            "lifetime_granularity_bits": 1008981770, "maximum_count": 8
        }
    ]);
    timeline["particles"] = json!([
        {
            "particle_id": "phase9-particle-a", "system_id": "phase9-system-a",
            "position": { "x_bits": 0, "y_bits": 0 },
            "velocity": { "x_bits": 0, "y_bits": 0 },
            "flags_bits": 0, "color": [255, 255, 255, 255],
            "lifetime_bits": (-1.0_f32).to_bits()
        },
        {
            "particle_id": "phase9-particle-b", "system_id": "phase9-system-b",
            "position": { "x_bits": 0, "y_bits": 0 },
            "velocity": { "x_bits": 0, "y_bits": 0 },
            "flags_bits": 0, "color": [255, 255, 255, 255],
            "lifetime_bits": 0
        }
    ]);
    let actions = timeline["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    for (action_id, action) in [
        (
            "phase9-create-system-a",
            json!({ "kind": "create_system", "system_id": "phase9-system-a" }),
        ),
        (
            "phase9-create-system-b",
            json!({ "kind": "create_system", "system_id": "phase9-system-b" }),
        ),
        (
            "phase9-create-particle-a",
            json!({ "kind": "create_particle", "particle_id": "phase9-particle-a" }),
        ),
        (
            "phase9-create-particle-b",
            json!({ "kind": "create_particle", "particle_id": "phase9-particle-b" }),
        ),
        (
            "phase9-inspect-particle-a",
            json!({ "kind": "inspect_particle", "particle_id": "phase9-particle-a" }),
        ),
        (
            "phase9-force-a",
            json!({ "kind": "apply_force", "particle_ids": ["phase9-particle-a"], "force": { "x_bits": 0, "y_bits": 0 } }),
        ),
        (
            "phase9-mark-a",
            json!({ "kind": "mark_for_destruction", "particle_id": "phase9-particle-a" }),
        ),
        (
            "phase9-compact-a",
            json!({ "kind": "compact", "system_id": "phase9-system-a" }),
        ),
        (
            "phase9-destroy-system-a",
            json!({ "kind": "destroy_system", "system_id": "phase9-system-a" }),
        ),
        (
            "phase9-query-b",
            json!({ "kind": "query_aabb", "system_id": "phase9-system-b", "lower": { "x_bits": 0, "y_bits": 0 }, "upper": { "x_bits": 1065353216, "y_bits": 1065353216 } }),
        ),
        (
            "phase9-mark-b",
            json!({ "kind": "mark_for_destruction", "particle_id": "phase9-particle-b" }),
        ),
        (
            "phase9-compact-b",
            json!({ "kind": "compact", "system_id": "phase9-system-b" }),
        ),
        (
            "phase9-destroy-system-b",
            json!({ "kind": "destroy_system", "system_id": "phase9-system-b" }),
        ),
    ] {
        actions.push(json!({
            "action_id": action_id,
            "phase": "phase9",
            "action": { "kind": "particle", "action": action }
        }));
    }
    let checkpoint = timeline["checkpoints"]
        .as_array_mut()
        .expect("fixture checkpoints should be an array")
        .last_mut()
        .expect("fixture should contain a checkpoint");
    checkpoint["after_action_id"] = json!("phase9-destroy-system-b");
    checkpoint["phase"] = json!("phase9");
    value
}

fn phase9_action_mut<'a>(value: &'a mut Value, action_id: &str) -> &'a mut Value {
    value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .first_mut()
        .expect("fixture should contain a timeline")["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array")
        .iter_mut()
        .find(|record| record["action_id"] == action_id)
        .expect("fixture should contain requested Phase 9 action")
}

fn insert_phase9_action_after(value: &mut Value, after_id: &str, action_id: &str, action: Value) {
    let actions = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .first_mut()
        .expect("fixture should contain a timeline")["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    let index = actions
        .iter()
        .position(|record| record["action_id"] == after_id)
        .expect("fixture should contain insertion anchor");
    actions.insert(
        index + 1,
        json!({
            "action_id": action_id,
            "phase": "phase9",
            "action": { "kind": "particle", "action": action }
        }),
    );
}

fn assert_invalid_particle_action(value: &Value) {
    let error =
        decode_value(value).expect_err("invalid Phase 9 lifecycle must fail before execution");
    assert!(
        error.contains("InvalidParticleAction"),
        "unexpected decode error: {error}"
    );
}

fn phase9_result_request() -> RigidWorldRequestRecord {
    let mut value = phase9_lifecycle_value();
    value["scenario"]["timelines"][0]["particles"][0]["position"] =
        json!({ "x_bits": 1048576000, "y_bits": 1056964608 });
    value["scenario"]["timelines"][0]["particles"][1]["position"] =
        json!({ "x_bits": 1056964608, "y_bits": 1056964608 });
    insert_phase9_action_after(
        &mut value,
        "phase9-query-b",
        "phase9-statistics-b",
        json!({ "kind": "request_statistics", "system_id": "phase9-system-b" }),
    );
    insert_phase9_action_after(
        &mut value,
        "phase9-statistics-b",
        "phase9-ray-all",
        json!({
            "kind": "ray_cast",
            "system_id": null,
            "start": { "x_bits": 3212836864_u32, "y_bits": 1056964608 },
            "end": { "x_bits": 1065353216, "y_bits": 1056964608 }
        }),
    );
    decode_value(&value).expect("bounded Phase 9 result request should decode")
}

fn result_value(result: &RigidWorldResultRecord) -> Value {
    serde_json::to_value(result).expect("Phase 9 result should serialize")
}

fn decode_result_value(value: &Value) -> Result<RigidWorldResultRecord, String> {
    let mut bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    decode_rigid_world_result_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .map_err(|error| error.to_string())
}

fn phase9_observations_mut(value: &mut Value) -> &mut Vec<Value> {
    value["timelines"][0]["checkpoints"]
        .as_array_mut()
        .expect("timeline checkpoints should be an array")
        .last_mut()
        .expect("timeline should contain a final checkpoint")["observations"]
        .as_array_mut()
        .expect("checkpoint observations should be an array")
}

fn particle_observation_mut<'a>(observations: &'a mut [Value], kind: &str) -> &'a mut Value {
    observations
        .iter_mut()
        .find(|observation| {
            observation["kind"] == "particle" && observation["observation"]["kind"] == kind
        })
        .expect("requested Phase 9 observation should exist")
}

fn mixed_observation_with_particles_mut<'a>(
    observations: &'a mut [Value],
    particle_ids: &[&str],
) -> &'a mut Value {
    observations
        .iter_mut()
        .find(|observation| {
            observation["kind"] == "particle"
                && observation["observation"]["kind"] == "mixed_state"
                && observation["observation"]["particle_ids"] == json!(particle_ids)
        })
        .expect("requested mixed-state observation should exist")
}

fn assert_result_observation_mismatch(request: &RigidWorldRequestRecord, value: &Value) {
    let result = decode_result_value(value).expect("mutation should remain a bounded result");
    let error = validate_rigid_world_result_against_request(request, &result)
        .expect_err("fabricated Phase 9 observation must fail request-bound validation");
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::ResultObservationMismatch)
    );
}

#[test]
fn result_accepts_unmodified_native_phase9_action_contracts() {
    // Arrange
    let request = phase9_result_request();

    // Act
    let result = NativeRigidWorldExecutor::execute(&request);

    // Assert
    let result = result.expect("native Phase 9 result should satisfy its exact action contract");
    validate_rigid_world_result_against_request(&request, &result)
        .expect("unmodified native result should remain request-bound valid");
}

#[test]
fn result_rejects_wrong_phase9_nested_variant_and_statistics_owner() {
    // Arrange
    let request = phase9_result_request();
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("baseline Phase 9 result should execute");
    let mut wrong_variant = result_value(&result);
    let statistics =
        particle_observation_mut(phase9_observations_mut(&mut wrong_variant), "statistics");
    statistics["observation"] = json!({ "kind": "query", "terminated": false, "particle_ids": [] });
    let mut wrong_owner = result_value(&result);
    particle_observation_mut(phase9_observations_mut(&mut wrong_owner), "statistics")["observation"]
        ["statistics"]["maybe_system_id"] = json!("phase9-system-a");

    // Act / Assert
    assert_result_observation_mismatch(&request, &wrong_variant);
    assert_result_observation_mismatch(&request, &wrong_owner);
}

#[test]
fn result_rejects_unknown_wrong_owner_and_duplicate_query_particles() {
    // Arrange
    let request = phase9_result_request();
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("baseline Phase 9 result should execute");
    let mut unknown = result_value(&result);
    particle_observation_mut(phase9_observations_mut(&mut unknown), "query")["observation"]["particle_ids"] =
        json!(["unknown-particle"]);
    let mut wrong_owner = result_value(&result);
    particle_observation_mut(phase9_observations_mut(&mut wrong_owner), "query")["observation"]["particle_ids"] =
        json!(["phase9-particle-a"]);
    let mut duplicate = result_value(&result);
    particle_observation_mut(phase9_observations_mut(&mut duplicate), "query")["observation"]["particle_ids"] =
        json!(["phase9-particle-b", "phase9-particle-b"]);

    // Act / Assert
    assert_result_observation_mismatch(&request, &unknown);
    assert_result_observation_mismatch(&request, &wrong_owner);
    assert_result_observation_mismatch(&request, &duplicate);
}

#[test]
fn result_rejects_reordered_future_and_stale_mixed_particle_identities() {
    // Arrange
    let request = phase9_result_request();
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("baseline Phase 9 result should execute");
    let mut reordered = result_value(&result);
    mixed_observation_with_particles_mut(
        phase9_observations_mut(&mut reordered),
        &["phase9-particle-a", "phase9-particle-b"],
    )["observation"]["particle_ids"] = json!(["phase9-particle-b", "phase9-particle-a"]);
    let mut future = result_value(&result);
    mixed_observation_with_particles_mut(phase9_observations_mut(&mut future), &[])["observation"]
        ["particle_ids"] = json!(["phase9-particle-b"]);
    let mut stale = result_value(&result);
    mixed_observation_with_particles_mut(
        phase9_observations_mut(&mut stale),
        &["phase9-particle-b"],
    )["observation"]["particle_ids"] = json!(["phase9-particle-a", "phase9-particle-b"]);

    // Act / Assert
    assert_result_observation_mismatch(&request, &reordered);
    assert_result_observation_mismatch(&request, &future);
    assert_result_observation_mismatch(&request, &stale);
}

#[test]
fn result_rejects_ray_parallel_length_and_extra_particle_observation() {
    // Arrange
    let request = phase9_result_request();
    let result = NativeRigidWorldExecutor::execute(&request)
        .expect("baseline Phase 9 result should execute");
    let mut wrong_length = result_value(&result);
    particle_observation_mut(phase9_observations_mut(&mut wrong_length), "ray_cast")["observation"]
        ["fractions_bits"] = json!([]);
    let mut extra = result_value(&result);
    let observations = phase9_observations_mut(&mut extra);
    let duplicate = observations
        .iter()
        .find(|observation| observation["kind"] == "particle")
        .cloned()
        .expect("baseline should contain a particle observation");
    observations.push(duplicate);

    // Act / Assert
    assert!(
        decode_result_value(&wrong_length).is_err(),
        "parallel ray arrays must fail bounded result decoding"
    );
    assert_result_observation_mismatch(&request, &extra);
}

#[test]
fn request_accepts_every_finite_infinite_lifetime_bit_pattern() {
    // Arrange
    let finite_infinite_bits = [
        (-1.0_f32).to_bits(),
        (-0.0_f32).to_bits(),
        0.0_f32.to_bits(),
    ];

    // Act
    let decoded = finite_infinite_bits.map(|lifetime_bits| {
        let mut value = phase9_lifecycle_value();
        value["scenario"]["timelines"][0]["particles"][0]["lifetime_bits"] = json!(lifetime_bits);
        decode_value(&value).map(|request| {
            request.scenario().timelines()[0].particles()[0]
                .lifetime_bits
                .bits()
        })
    });

    // Assert
    assert_eq!(decoded, finite_infinite_bits.map(Ok));
}

#[test]
fn request_rejects_nonfinite_particle_lifetimes() {
    // Arrange
    let nonfinite_bits = [
        f32::NAN.to_bits(),
        f32::INFINITY.to_bits(),
        f32::NEG_INFINITY.to_bits(),
    ];

    // Act
    let results = nonfinite_bits.map(|lifetime_bits| {
        let mut value = phase9_lifecycle_value();
        value["scenario"]["timelines"][0]["particles"][0]["lifetime_bits"] = json!(lifetime_bits);
        decode_value(&value)
    });

    // Assert
    assert!(results.iter().all(Result::is_err));
}

#[test]
fn request_rejects_duplicate_particle_system_creation() {
    // Arrange
    let mut value = phase9_lifecycle_value();
    insert_phase9_action_after(
        &mut value,
        "phase9-create-system-a",
        "phase9-create-system-a-again",
        json!({ "kind": "create_system", "system_id": "phase9-system-a" }),
    );

    // Act / Assert
    assert_invalid_particle_action(&value);
}

#[test]
fn request_rejects_particle_creation_before_owner_system() {
    // Arrange
    let mut value = phase9_lifecycle_value();
    let actions = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    let system_index = actions
        .iter()
        .position(|record| record["action_id"] == "phase9-create-system-a")
        .expect("system creation should exist");
    let particle_index = actions
        .iter()
        .position(|record| record["action_id"] == "phase9-create-particle-a")
        .expect("particle creation should exist");
    actions.swap(system_index, particle_index);

    // Act / Assert
    assert_invalid_particle_action(&value);
}

#[test]
fn request_rejects_duplicate_particle_creation() {
    // Arrange
    let mut value = phase9_lifecycle_value();
    insert_phase9_action_after(
        &mut value,
        "phase9-create-particle-a",
        "phase9-create-particle-a-again",
        json!({ "kind": "create_particle", "particle_id": "phase9-particle-a" }),
    );

    // Act / Assert
    assert_invalid_particle_action(&value);
}

#[test]
fn request_rejects_unknown_particle_use() {
    // Arrange
    let mut value = phase9_lifecycle_value();
    phase9_action_mut(&mut value, "phase9-inspect-particle-a")["action"]["action"]["particle_id"] =
        json!("unknown-particle");

    // Act / Assert
    assert_invalid_particle_action(&value);
}

#[test]
fn request_rejects_pending_particle_use_and_repeated_mark() {
    // Arrange
    let mut pending_use = phase9_lifecycle_value();
    insert_phase9_action_after(
        &mut pending_use,
        "phase9-mark-a",
        "phase9-inspect-pending-a",
        json!({ "kind": "inspect_particle", "particle_id": "phase9-particle-a" }),
    );
    let mut repeated_mark = phase9_lifecycle_value();
    insert_phase9_action_after(
        &mut repeated_mark,
        "phase9-mark-a",
        "phase9-mark-a-again",
        json!({ "kind": "mark_for_destruction", "particle_id": "phase9-particle-a" }),
    );

    // Act / Assert
    assert_invalid_particle_action(&pending_use);
    assert_invalid_particle_action(&repeated_mark);
}

#[test]
fn request_rejects_particle_recreation_after_compaction() {
    // Arrange
    let mut value = phase9_lifecycle_value();
    insert_phase9_action_after(
        &mut value,
        "phase9-compact-a",
        "phase9-recreate-particle-a",
        json!({ "kind": "create_particle", "particle_id": "phase9-particle-a" }),
    );

    // Act / Assert
    assert_invalid_particle_action(&value);
}

#[test]
fn request_rejects_particle_use_after_owner_system_destruction() {
    // Arrange
    let mut value = phase9_lifecycle_value();
    insert_phase9_action_after(
        &mut value,
        "phase9-destroy-system-a",
        "phase9-inspect-destroyed-a",
        json!({ "kind": "inspect_particle", "particle_id": "phase9-particle-a" }),
    );

    // Act / Assert
    assert_invalid_particle_action(&value);
}

#[test]
fn request_rejects_cross_system_particle_range() {
    // Arrange
    let mut value = phase9_lifecycle_value();
    phase9_action_mut(&mut value, "phase9-force-a")["action"]["action"]["particle_ids"] =
        json!(["phase9-particle-a", "phase9-particle-b"]);

    // Act / Assert
    assert_invalid_particle_action(&value);
}

#[test]
fn request_rejects_destroyed_or_unknown_query_owner() {
    // Arrange
    let mut destroyed = phase9_lifecycle_value();
    phase9_action_mut(&mut destroyed, "phase9-query-b")["action"]["action"]["system_id"] =
        json!("phase9-system-a");
    let mut unknown = phase9_lifecycle_value();
    phase9_action_mut(&mut unknown, "phase9-query-b")["action"]["action"]["system_id"] =
        json!("unknown-system");

    // Act / Assert
    assert_invalid_particle_action(&destroyed);
    assert_invalid_particle_action(&unknown);
}

#[test]
fn codec_accepts_bounded_additive_phase9_declarations() {
    // Arrange
    let mut value: Value =
        serde_json::from_slice(PHASE8_REQUEST).expect("checked-in Phase 8 request should be JSON");
    let timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .first_mut()
        .expect("fixture should contain a timeline");
    timeline["particle_systems"] = json!([{
        "system_id": "phase9-system",
        "buffer_mode": { "kind": "growable", "initial_capacity": 4 },
        "paused": false,
        "strict_contact_check": true,
        "stuck_threshold": 2,
        "density_bits": 1065353216,
        "gravity_scale_bits": 1065353216,
        "radius_bits": 1036831949,
        "damping_bits": 0,
        "destruction_by_age": true,
        "lifetime_granularity_bits": 1008981770,
        "maximum_count": 8
    }]);
    timeline["particles"] = json!([{
        "particle_id": "phase9-particle",
        "system_id": "phase9-system",
        "position": { "x_bits": 0, "y_bits": 0 },
        "velocity": { "x_bits": 0, "y_bits": 0 },
        "flags_bits": 0,
        "color": [255, 255, 255, 255],
        "lifetime_bits": 1065353216
    }]);

    // Act
    let result = decode_value(&value);

    // Assert
    assert!(
        result.is_ok(),
        "bounded Phase 9 declarations should decode: {result:?}"
    );
}

#[test]
fn codec_rejects_phase10_group_topology() {
    // Arrange
    let mut value: Value =
        serde_json::from_slice(PHASE8_REQUEST).expect("checked-in Phase 8 request should be JSON");
    let timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .first_mut()
        .expect("fixture should contain a timeline");
    timeline["particle_groups"] = json!([{ "group_id": "phase10-group" }]);

    // Act
    let result = decode_value(&value);

    // Assert
    assert!(result.is_err(), "Phase 10 topology must remain undeclared");
}

#[test]
fn codec_preserves_phase8_request_bytes() {
    // Arrange
    let request =
        decode_rigid_world_request_jsonl(PHASE8_REQUEST, &HarnessLimits::phase2_default_v1())
            .expect("checked-in Phase 8 request should decode");

    // Act
    let mut encoded = serde_json::to_vec(&request).expect("request should serialize");
    encoded.push(b'\n');

    // Assert
    assert_eq!(encoded, PHASE8_REQUEST);
}

#[test]
fn native_dispatch_executes_phase9_particle_actions() {
    // Arrange
    let mut value: Value =
        serde_json::from_slice(PHASE8_REQUEST).expect("checked-in Phase 8 request should be JSON");
    let timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .first_mut()
        .expect("fixture should contain a timeline");
    timeline["particle_systems"] = json!([{
        "system_id": "phase9-system", "buffer_mode": { "kind": "growable", "initial_capacity": 4 },
        "paused": false, "strict_contact_check": true, "stuck_threshold": 2,
        "density_bits": 1065353216, "gravity_scale_bits": 1065353216,
        "radius_bits": 1036831949, "damping_bits": 0, "destruction_by_age": true,
        "lifetime_granularity_bits": 1008981770, "maximum_count": 8
    }]);
    timeline["particles"] = json!([{
        "particle_id": "phase9-particle", "system_id": "phase9-system",
        "position": { "x_bits": 0, "y_bits": 0 }, "velocity": { "x_bits": 0, "y_bits": 0 },
        "flags_bits": 0, "color": [255, 255, 255, 255], "lifetime_bits": 1065353216
    }]);
    let actions = timeline["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    for (id, action) in [
        (
            "phase9-create-system",
            json!({ "kind": "create_system", "system_id": "phase9-system" }),
        ),
        (
            "phase9-create-particle",
            json!({ "kind": "create_particle", "particle_id": "phase9-particle" }),
        ),
        (
            "phase9-inspect-particle",
            json!({ "kind": "inspect_particle", "particle_id": "phase9-particle" }),
        ),
        (
            "phase9-force",
            json!({ "kind": "apply_force", "particle_ids": ["phase9-particle"], "force": { "x_bits": 1065353216, "y_bits": 0 } }),
        ),
        (
            "phase9-impulse",
            json!({ "kind": "apply_impulse", "particle_ids": ["phase9-particle"], "impulse": { "x_bits": 0, "y_bits": 1065353216 } }),
        ),
        (
            "phase9-statistics",
            json!({ "kind": "request_statistics", "system_id": "phase9-system" }),
        ),
        (
            "phase9-mark",
            json!({ "kind": "mark_for_destruction", "particle_id": "phase9-particle" }),
        ),
        (
            "phase9-compact",
            json!({ "kind": "compact", "system_id": "phase9-system" }),
        ),
        (
            "phase9-destroy-system",
            json!({ "kind": "destroy_system", "system_id": "phase9-system" }),
        ),
    ] {
        actions.push(json!({
            "action_id": id,
            "phase": "phase9",
            "action": { "kind": "particle", "action": action }
        }));
    }
    let checkpoint = timeline["checkpoints"]
        .as_array_mut()
        .expect("fixture checkpoints should be an array")
        .last_mut()
        .expect("fixture should contain a checkpoint");
    checkpoint["after_action_id"] = json!("phase9-destroy-system");
    checkpoint["phase"] = json!("phase9");
    let request = decode_value(&value).expect("bounded Phase 9 request should decode");

    // Act
    let result = NativeRigidWorldExecutor::execute(&request);

    // Assert
    let result = result.expect("native Phase 9 actions should execute");
    let particle_observation_count = result.timelines()[0]
        .checkpoints
        .last()
        .expect("timeline should retain its final checkpoint")
        .observations
        .iter()
        .filter(|observation| {
            matches!(
                observation,
                liquidfun_test_protocol::RigidWorldObservation::Particle { .. }
            )
        })
        .count();
    assert_eq!(particle_observation_count, 9);
}

#[test]
fn phase9_policy_registry_fails_closed_without_wildcards() {
    // Arrange
    let reviewed = [
        ("particle.storage.identity", Phase9PolicyKind::ExactDiscrete),
        ("particle.configuration.bits", Phase9PolicyKind::ExactBits),
        ("particle.position", Phase9PolicyKind::Ulps),
        (
            "particle.contact.weight",
            Phase9PolicyKind::AbsoluteRelative,
        ),
        (
            "particle.ray.fraction",
            Phase9PolicyKind::DimensionedAbsolute,
        ),
    ];

    // Act
    let actual = reviewed.map(|(path, _)| phase9_policy_for_path(path));

    // Assert
    assert_eq!(PHASE9_REGISTRY_ID, "phase9-v1");
    assert_eq!(actual, reviewed.map(|(_, policy)| Some(policy)));
    assert!(
        PHASE9_REQUIRED_POLICY_PATHS
            .iter()
            .all(|path| phase9_policy_for_path(path).is_some())
    );
    assert_eq!(phase9_policy_for_path("particle.*"), None);
    assert_eq!(phase9_policy_for_path("particle.group.topology"), None);
    assert_eq!(phase9_policy_for_path("particle.pair.generation"), None);
    assert_eq!(phase9_policy_for_path("particle.solver.baseline"), None);
}
