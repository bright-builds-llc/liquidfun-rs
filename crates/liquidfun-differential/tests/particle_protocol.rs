//! Closed Phase 9 particle protocol contracts.

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
