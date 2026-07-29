fn phase9_request() -> liquidfun_test_protocol::RigidWorldRequestRecord {
    let mut value: Value =
        serde_json::from_slice(REQUEST).expect("checked-in Phase 8 request should be JSON");
    let timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .first_mut()
        .expect("fixture should contain a timeline");
    timeline["particle_systems"] = json!([{
        "system_id": "oracle-system",
        "buffer_mode": { "kind": "growable", "initial_capacity": 4 },
        "paused": false,
        "strict_contact_check": true,
        "stuck_threshold": 2,
        "density_bits": 1.0_f32.to_bits(),
        "gravity_scale_bits": 1.0_f32.to_bits(),
        "radius_bits": 0.1_f32.to_bits(),
        "damping_bits": 0.0_f32.to_bits(),
        "destruction_by_age": true,
        "lifetime_granularity_bits": (1.0_f32 / 60.0_f32).to_bits(),
        "maximum_count": 8
    }]);
    timeline["particles"] = json!([{
        "particle_id": "oracle-particle",
        "system_id": "oracle-system",
        "position": { "x_bits": 0.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
        "velocity": { "x_bits": 0.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
        "flags_bits": 0,
        "color": [255, 255, 255, 255],
        "lifetime_bits": 1.0_f32.to_bits()
    }]);
    let actions = timeline["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    actions.push(json!({
        "action_id": "oracle-create-system",
        "phase": "phase9",
        "action": {
            "kind": "particle",
            "action": { "kind": "create_system", "system_id": "oracle-system" }
        }
    }));
    actions.push(json!({
        "action_id": "oracle-create-particle",
        "phase": "phase9",
        "action": {
            "kind": "particle",
            "action": { "kind": "create_particle", "particle_id": "oracle-particle" }
        }
    }));
    actions.push(json!({
        "action_id": "oracle-destroy-system",
        "phase": "phase9",
        "action": {
            "kind": "particle",
            "action": { "kind": "destroy_system", "system_id": "oracle-system" }
        }
    }));
    preserve_retained_checkpoint_before_phase9(
        timeline,
        "nc-bodies-destroyed",
        "oracle-destroy-system",
    );

    let mut bytes = serde_json::to_vec(&value).expect("Phase 9 request should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded Phase 9 request should decode")
}

fn coupling_request() -> liquidfun_test_protocol::RigidWorldRequestRecord {
    let mut value: Value =
        serde_json::from_slice(REQUEST).expect("checked-in Phase 8 request should be JSON");
    let timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("fixture timelines should be an array")
        .first_mut()
        .expect("fixture should contain a timeline");
    timeline["particle_systems"] = json!([{
        "system_id": "coupling-system",
        "buffer_mode": { "kind": "fixed", "capacity": 4 },
        "paused": false,
        "strict_contact_check": true,
        "stuck_threshold": 1,
        "density_bits": 1.0_f32.to_bits(),
        "gravity_scale_bits": 0.0_f32.to_bits(),
        "radius_bits": 0.5_f32.to_bits(),
        "damping_bits": 1.0_f32.to_bits(),
        "destruction_by_age": true,
        "lifetime_granularity_bits": (1.0_f32 / 60.0_f32).to_bits(),
        "maximum_count": 4
    }]);
    timeline["particles"] = json!([{
        "particle_id": "coupling-particle",
        "system_id": "coupling-system",
        "position": { "x_bits": 20.0_f32.to_bits(), "y_bits": 0.25_f32.to_bits() },
        "velocity": { "x_bits": (-2.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
        "flags_bits": 0,
        "color": [64, 128, 255, 255],
        "lifetime_bits": 2.0_f32.to_bits()
    }]);
    let actions = timeline["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    let step_index = actions
        .iter()
        .position(|record| record["action_id"] == "nc-step-static-kinematic")
        .expect("first configured step should exist");
    actions.splice(
        step_index..step_index,
        [
            json!({
                "action_id": "coupling-create-system", "phase": "phase9",
                "action": { "kind": "particle", "action": {
                    "kind": "create_system", "system_id": "coupling-system"
                }}
            }),
            json!({
                "action_id": "coupling-create-particle", "phase": "phase9",
                "action": { "kind": "particle", "action": {
                    "kind": "create_particle", "particle_id": "coupling-particle"
                }}
            }),
        ],
    );
    let step_index = actions
        .iter()
        .position(|record| record["action_id"] == "nc-step-static-kinematic")
        .expect("first configured step should remain present");
    let after_step_index = step_index + 1;
    actions.splice(
        after_step_index..after_step_index,
        [
            json!({
                "action_id": "coupling-statistics", "phase": "phase9",
                "action": { "kind": "particle", "action": {
                    "kind": "request_statistics", "system_id": "coupling-system"
                }}
            }),
            json!({
                "action_id": "coupling-destroy-system", "phase": "phase9",
                "action": { "kind": "particle", "action": {
                    "kind": "destroy_system", "system_id": "coupling-system"
                }}
            }),
        ],
    );
    preserve_retained_checkpoint_before_phase9(
        timeline,
        "nc-static-kinematic-rejected",
        "coupling-destroy-system",
    );

    let mut bytes = serde_json::to_vec(&value).expect("coupling request should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("bounded coupling request should decode")
}

fn preserve_retained_checkpoint_before_phase9(
    timeline: &mut Value,
    checkpoint_id: &str,
    phase9_after_action_id: &str,
) {
    let checkpoints = timeline["checkpoints"]
        .as_array_mut()
        .expect("fixture checkpoints should be an array");
    let checkpoint_index = checkpoints
        .iter()
        .position(|checkpoint| checkpoint["checkpoint_id"] == checkpoint_id)
        .expect("retargeted checkpoint should exist");
    let mut retained = checkpoints[checkpoint_index].clone();
    retained["checkpoint_id"] = json!(format!("{checkpoint_id}-retained"));
    checkpoints.insert(checkpoint_index, retained);
    let phase9 = &mut checkpoints[checkpoint_index + 1];
    phase9["after_action_id"] = json!(phase9_after_action_id);
    phase9["phase"] = json!("phase9");
    phase9["counts"]["destructions"] = json!(0);
    phase9["transitions"] = json!([]);
}

fn full_phase9_request() -> liquidfun_test_protocol::RigidWorldRequestRecord {
    let mut value = serde_json::to_value(phase9_request()).expect("request should serialize");
    let timeline = &mut value["scenario"]["timelines"][0];
    timeline["particle_systems"]
        .as_array_mut()
        .expect("systems should be an array")
        .push(json!({
            "system_id": "oracle-system-newest",
            "buffer_mode": { "kind": "fixed", "capacity": 4 },
            "paused": false, "strict_contact_check": true, "stuck_threshold": 1,
            "density_bits": 1.0_f32.to_bits(), "gravity_scale_bits": 1.0_f32.to_bits(),
            "radius_bits": 0.1_f32.to_bits(), "damping_bits": 0.25_f32.to_bits(),
            "destruction_by_age": true,
            "lifetime_granularity_bits": (1.0_f32 / 60.0_f32).to_bits(),
            "maximum_count": 4
        }));
    timeline["particles"]
        .as_array_mut()
        .expect("particles should be an array")
        .push(json!({
            "particle_id": "oracle-particle-newest", "system_id": "oracle-system-newest",
            "position": { "x_bits": 0.25_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "velocity": { "x_bits": 0.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "flags_bits": 0, "color": [255, 128, 64, 255], "lifetime_bits": 2.0_f32.to_bits()
        }));
    let actions = timeline["actions"]
        .as_array_mut()
        .expect("actions should be an array");
    let destroy_index = actions
        .iter()
        .position(|record| record["action_id"] == "oracle-destroy-system")
        .expect("oldest system destruction should exist");
    actions.splice(
        destroy_index..destroy_index,
        [
            json!({ "action_id": "oracle-create-newest-system", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "create_system", "system_id": "oracle-system-newest" }}}),
            json!({ "action_id": "oracle-create-newest-particle", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "create_particle", "particle_id": "oracle-particle-newest" }}}),
            json!({ "action_id": "oracle-inspect-system", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "inspect_system", "system_id": "oracle-system-newest" }}}),
            json!({ "action_id": "oracle-inspect-particle", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "inspect_particle", "particle_id": "oracle-particle-newest" }}}),
            json!({ "action_id": "oracle-pause", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "set_paused", "system_id": "oracle-system-newest", "paused": true }}}),
            json!({ "action_id": "oracle-resume", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "set_paused", "system_id": "oracle-system-newest", "paused": false }}}),
            json!({ "action_id": "oracle-position", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "set_position", "particle_id": "oracle-particle-newest", "position": { "x_bits": 0.5_f32.to_bits(), "y_bits": 0.0_f32.to_bits() }}}}),
            json!({ "action_id": "oracle-velocity", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "set_velocity", "particle_id": "oracle-particle-newest", "velocity": { "x_bits": 0.0_f32.to_bits(), "y_bits": 1.0_f32.to_bits() }}}}),
            json!({ "action_id": "oracle-force", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "apply_force", "particle_ids": ["oracle-particle-newest"], "force": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() }}}}),
            json!({ "action_id": "oracle-impulse", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "apply_impulse", "particle_ids": ["oracle-particle-newest"], "impulse": { "x_bits": 0.0_f32.to_bits(), "y_bits": 1.0_f32.to_bits() }}}}),
            json!({ "action_id": "oracle-statistics", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "request_statistics", "system_id": "oracle-system-newest" }}}),
            json!({ "action_id": "oracle-query", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "query_aabb", "system_id": null, "lower": { "x_bits": (-1.0_f32).to_bits(), "y_bits": (-1.0_f32).to_bits() }, "upper": { "x_bits": 1.0_f32.to_bits(), "y_bits": 1.0_f32.to_bits() }}}}),
            json!({ "action_id": "oracle-ray", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "ray_cast", "system_id": null, "start": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() }, "end": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() }}}}),
            json!({ "action_id": "oracle-mark", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "mark_for_destruction", "particle_id": "oracle-particle-newest" }}}),
            json!({ "action_id": "oracle-compact", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "compact", "system_id": "oracle-system-newest" }}}),
            json!({ "action_id": "oracle-destroy-newest-system", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "destroy_system", "system_id": "oracle-system-newest" }}}),
        ],
    );
    let mut bytes = serde_json::to_vec(&value).expect("full request should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("full bounded Phase 9 request should decode")
}

fn static_coupling_request() -> liquidfun_test_protocol::RigidWorldRequestRecord {
    let mut value = serde_json::to_value(coupling_request()).expect("request should serialize");
    let bodies = value["scenario"]["timelines"][0]["bodies"]
        .as_array_mut()
        .expect("bodies should be an array");
    let dynamic = bodies
        .iter_mut()
        .find(|body| body["body_id"] == "nc-dynamic")
        .expect("coupling body should exist");
    dynamic["body_kind"] = json!("static");
    let mut bytes = serde_json::to_vec(&value).expect("static request should encode");
    bytes.push(b'\n');
    decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("static coupling request should decode")
}

fn phase9_action_index(value: &Value, action_id: &str) -> usize {
    value["scenario"]["timelines"][0]["actions"]
        .as_array()
        .expect("actions should be an array")
        .iter()
        .position(|record| record["action_id"] == action_id)
        .expect("requested Phase 9 action should exist")
}

fn raw_oracle_rejection(executable: &std::path::Path, value: &Value) -> String {
    let mut bytes = serde_json::to_vec(value).expect("invalid request should encode");
    bytes.push(b'\n');
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("oracle should spawn");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout should be captured"));
    let mut handshake = String::new();
    stdout
        .read_line(&mut handshake)
        .expect("handshake should be readable");
    let mut stdin = child.stdin.take().expect("stdin should be captured");
    stdin
        .write_all(&bytes)
        .and_then(|()| stdin.flush())
        .expect("invalid request should reach the decoder");
    drop(stdin);
    let mut unexpected_stdout = String::new();
    stdout
        .read_to_string(&mut unexpected_stdout)
        .expect("remaining stdout should be readable");
    let output = child.wait_with_output().expect("oracle should be reaped");
    assert!(serde_json::from_str::<Value>(&handshake).is_ok());
    assert!(
        unexpected_stdout.is_empty(),
        "stdout must remain JSONL-only"
    );
    assert!(
        output.status.success(),
        "a rejected request must not poison the reusable oracle process"
    );
    String::from_utf8(output.stderr).expect("oracle diagnostics should be UTF-8")
}
