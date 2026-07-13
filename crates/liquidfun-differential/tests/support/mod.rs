use liquidfun_test_protocol::{
    HarnessLimits, RigidWorldRequestRecord, Sha256Hex, decode_rigid_world_request_jsonl,
};
use serde_json::{Value, json};

const REQUEST: &[u8] =
    include_bytes!("../../../../protocol/fixtures/accepted/rigid-world-request.jsonl");

pub fn phase7_request() -> RigidWorldRequestRecord {
    phase7_request_with_profile(None)
}

pub fn phase7_request_with_profile(
    maybe_profile_sha256: Option<&Sha256Hex>,
) -> RigidWorldRequestRecord {
    let mut value = serde_json::from_slice::<Value>(REQUEST).expect("fixture should be JSON");
    if let Some(profile_sha256) = maybe_profile_sha256 {
        value["tolerance_profile_sha256"] = json!(profile_sha256.as_str());
    }
    let actions = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    let insert_at = actions
        .iter()
        .position(|record| record["action"]["kind"] == "destroy_fixture")
        .expect("fixture should contain destruction actions");
    let vector = json!({ "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() });
    let phase7_actions = [
        json!({ "kind": "set_linear_velocity", "body_id": "nc-dynamic", "velocity": vector }),
        json!({ "kind": "set_angular_velocity", "body_id": "nc-dynamic", "angular_velocity_bits": 0.5_f32.to_bits() }),
        json!({ "kind": "apply_force", "body_id": "nc-dynamic", "force": vector, "point": vector, "wake_policy": "wake" }),
        json!({ "kind": "apply_torque", "body_id": "nc-dynamic", "torque_bits": 0.25_f32.to_bits(), "wake_policy": "preserve_sleep" }),
        json!({ "kind": "apply_linear_impulse", "body_id": "nc-dynamic", "impulse": vector, "point": vector, "wake_policy": "wake" }),
        json!({ "kind": "apply_angular_impulse", "body_id": "nc-dynamic", "impulse_bits": 0.25_f32.to_bits(), "wake_policy": "preserve_sleep" }),
        json!({ "kind": "set_body_damping", "body_id": "nc-dynamic", "linear_damping_bits": 0.1_f32.to_bits(), "angular_damping_bits": 0.2_f32.to_bits() }),
        json!({ "kind": "set_gravity_scale", "body_id": "nc-dynamic", "gravity_scale_bits": 0.75_f32.to_bits() }),
        json!({ "kind": "set_fixed_rotation", "body_id": "nc-dynamic", "fixed_rotation": true }),
        json!({ "kind": "set_sleeping_allowed", "body_id": "nc-dynamic", "sleeping_allowed": true }),
        json!({ "kind": "set_awake", "body_id": "nc-dynamic", "awake": true }),
        json!({ "kind": "set_bullet", "body_id": "nc-dynamic", "bullet": true }),
        json!({ "kind": "set_world_gravity", "gravity": { "x_bits": 0.0_f32.to_bits(), "y_bits": (-10.0_f32).to_bits() } }),
        json!({ "kind": "set_automatic_force_clearing", "enabled": false }),
        json!({ "kind": "set_warm_starting", "enabled": false }),
        json!({ "kind": "set_continuous_physics", "enabled": true }),
        json!({ "kind": "set_sub_stepping", "enabled": false }),
        json!({ "kind": "clear_forces" }),
        json!({ "kind": "configured_step", "timestep_bits": (1.0_f32 / 60.0).to_bits(), "velocity_iterations": 8, "position_iterations": 3, "continuous_work_budget": 64 }),
        json!({ "kind": "query_aabb", "aabb": { "lower": { "x_bits": (-100.0_f32).to_bits(), "y_bits": (-100.0_f32).to_bits() }, "upper": { "x_bits": 100.0_f32.to_bits(), "y_bits": 100.0_f32.to_bits() } }, "directive_rules": [{ "target": { "fixture_id": "nc-static-fixture", "child_index": 0 }, "directive": "terminate" }] }),
        json!({ "kind": "ray_cast", "start": { "x_bits": (-100.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() }, "end": { "x_bits": 100.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() }, "directive_rules": [{ "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 0 }, "directive": { "kind": "clip", "fraction_bits": 0.5_f32.to_bits() } }] }),
        json!({ "kind": "shift_origin", "shift": vector }),
    ];
    for (index, action) in phase7_actions.into_iter().enumerate().rev() {
        actions.insert(
            insert_at,
            json!({
                "action_id": format!("phase7-action-{index}"),
                "phase": "phase7-adapter",
                "action": action,
            }),
        );
    }
    let mut bytes = serde_json::to_vec(&value).expect("fixture mutation should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("closed Phase 7 adapter request should decode")
}
