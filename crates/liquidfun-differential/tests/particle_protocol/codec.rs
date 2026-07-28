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
        "density_bits": 1_065_353_216,
        "gravity_scale_bits": 1_065_353_216,
        "radius_bits": 1_036_831_949,
        "damping_bits": 0,
        "destruction_by_age": true,
        "lifetime_granularity_bits": 1_008_981_770,
        "maximum_count": 8
    }]);
    timeline["particles"] = json!([{
        "particle_id": "phase9-particle",
        "system_id": "phase9-system",
        "position": { "x_bits": 0, "y_bits": 0 },
        "velocity": { "x_bits": 0, "y_bits": 0 },
        "flags_bits": 0,
        "color": [255, 255, 255, 255],
        "lifetime_bits": 1_065_353_216
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
        "density_bits": 1_065_353_216, "gravity_scale_bits": 1_065_353_216,
        "radius_bits": 1_036_831_949, "damping_bits": 0, "destruction_by_age": true,
        "lifetime_granularity_bits": 1_008_981_770, "maximum_count": 8
    }]);
    timeline["particles"] = json!([{
        "particle_id": "phase9-particle", "system_id": "phase9-system",
        "position": { "x_bits": 0, "y_bits": 0 }, "velocity": { "x_bits": 0, "y_bits": 0 },
        "flags_bits": 0, "color": [255, 255, 255, 255], "lifetime_bits": 1_065_353_216
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
            json!({ "kind": "apply_force", "particle_ids": ["phase9-particle"], "force": { "x_bits": 1_065_353_216, "y_bits": 0 } }),
        ),
        (
            "phase9-impulse",
            json!({ "kind": "apply_impulse", "particle_ids": ["phase9-particle"], "impulse": { "x_bits": 0, "y_bits": 1_065_353_216 } }),
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
