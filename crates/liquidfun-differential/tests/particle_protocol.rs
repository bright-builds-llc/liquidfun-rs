//! Closed Phase 9 particle protocol contracts.

use liquidfun_differential::{
    NativeRigidWorldExecutor, PHASE9_REGISTRY_ID, PHASE9_REQUIRED_POLICY_PATHS, Phase9PolicyKind,
    phase9_policy_for_path,
};
use liquidfun_test_protocol::{HarnessLimits, decode_rigid_world_request_jsonl};
use serde_json::{Value, json};

const PHASE8_REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");

fn decode_value(value: &Value) -> Result<liquidfun_test_protocol::RigidWorldRequestRecord, String> {
    let mut bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .map_err(|error| error.to_string())
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
