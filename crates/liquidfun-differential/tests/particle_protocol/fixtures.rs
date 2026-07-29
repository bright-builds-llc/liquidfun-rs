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
    set_phase9_declarations(timeline);
    let actions = timeline["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    for (action_id, action) in phase9_lifecycle_actions() {
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

fn set_phase9_declarations(timeline: &mut Value) {
    timeline["particle_systems"] = json!([
        {
            "system_id": "phase9-system-a",
            "buffer_mode": { "kind": "growable", "initial_capacity": 4 },
            "paused": false, "strict_contact_check": true, "stuck_threshold": 2,
            "density_bits": 1_065_353_216, "gravity_scale_bits": 1_065_353_216,
            "radius_bits": 1_036_831_949, "damping_bits": 0, "destruction_by_age": true,
            "lifetime_granularity_bits": 1_008_981_770, "maximum_count": 8
        },
        {
            "system_id": "phase9-system-b",
            "buffer_mode": { "kind": "growable", "initial_capacity": 4 },
            "paused": false, "strict_contact_check": false, "stuck_threshold": 0,
            "density_bits": 1_065_353_216, "gravity_scale_bits": 1_065_353_216,
            "radius_bits": 1_036_831_949, "damping_bits": 0, "destruction_by_age": false,
            "lifetime_granularity_bits": 1_008_981_770, "maximum_count": 8
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
}

fn phase9_lifecycle_actions() -> Vec<(&'static str, Value)> {
    vec![
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
            json!({ "kind": "query_aabb", "system_id": "phase9-system-b", "lower": { "x_bits": 0, "y_bits": 0 }, "upper": { "x_bits": 1_065_353_216, "y_bits": 1_065_353_216 } }),
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
    ]
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
    let mut record = json!({
        "action_id": action_id,
        "phase": "phase9",
        "action": { "kind": "particle" }
    });
    record["action"]["action"] = action;
    actions.insert(index + 1, record);
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
        json!({ "x_bits": 1_048_576_000, "y_bits": 1_056_964_608 });
    value["scenario"]["timelines"][0]["particles"][1]["position"] =
        json!({ "x_bits": 1_056_964_608, "y_bits": 1_056_964_608 });
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
            "start": { "x_bits": 3_212_836_864_u32, "y_bits": 1_056_964_608 },
            "end": { "x_bits": 1_065_353_216, "y_bits": 1_056_964_608 }
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
