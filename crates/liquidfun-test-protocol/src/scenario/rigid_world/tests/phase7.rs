#[test]
fn rigid_world_accepts_closed_phase7_actions() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let vector = json!({ "x_bits": 1.0_f32.to_bits(), "y_bits": 2.0_f32.to_bits() });
    let body_id = "nc-dynamic";
    let fixture_id = "nc-dynamic-fixture";
    let actions = vec![
        json!({ "kind": "set_linear_velocity", "body_id": body_id, "velocity": vector }),
        json!({ "kind": "set_angular_velocity", "body_id": body_id, "angular_velocity_bits": 1.0_f32.to_bits() }),
        json!({ "kind": "apply_force", "body_id": body_id, "force": vector, "point": vector, "wake_policy": "wake" }),
        json!({ "kind": "apply_torque", "body_id": body_id, "torque_bits": 1.0_f32.to_bits(), "wake_policy": "preserve_sleep" }),
        json!({ "kind": "apply_linear_impulse", "body_id": body_id, "impulse": vector, "point": vector, "wake_policy": "wake" }),
        json!({ "kind": "apply_angular_impulse", "body_id": body_id, "impulse_bits": 1.0_f32.to_bits(), "wake_policy": "preserve_sleep" }),
        json!({ "kind": "set_body_damping", "body_id": body_id, "linear_damping_bits": 0.1_f32.to_bits(), "angular_damping_bits": 0.2_f32.to_bits() }),
        json!({ "kind": "set_gravity_scale", "body_id": body_id, "gravity_scale_bits": 1.0_f32.to_bits() }),
        json!({ "kind": "set_fixed_rotation", "body_id": body_id, "fixed_rotation": true }),
        json!({ "kind": "set_sleeping_allowed", "body_id": body_id, "sleeping_allowed": true }),
        json!({ "kind": "set_awake", "body_id": body_id, "awake": true }),
        json!({ "kind": "set_bullet", "body_id": body_id, "bullet": true }),
        json!({ "kind": "set_world_gravity", "gravity": vector }),
        json!({ "kind": "set_automatic_force_clearing", "enabled": true }),
        json!({ "kind": "set_warm_starting", "enabled": true }),
        json!({ "kind": "set_continuous_physics", "enabled": true }),
        json!({ "kind": "set_sub_stepping", "enabled": true }),
        json!({ "kind": "clear_forces" }),
        json!({ "kind": "configured_step", "timestep_bits": (1.0_f32 / 60.0).to_bits(), "velocity_iterations": 8, "position_iterations": 3, "continuous_work_budget": 4096 }),
        json!({ "kind": "query_aabb", "aabb": { "lower": { "x_bits": (-1.0_f32).to_bits(), "y_bits": (-1.0_f32).to_bits() }, "upper": vector }, "directive_rules": [{ "target": { "fixture_id": fixture_id, "child_index": 0 }, "directive": "terminate" }] }),
        json!({ "kind": "ray_cast", "start": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() }, "end": vector, "directive_rules": [{ "target": { "fixture_id": fixture_id, "child_index": 0 }, "directive": { "kind": "clip", "fraction_bits": 0.5_f32.to_bits() } }] }),
        json!({ "kind": "shift_origin", "shift": vector }),
    ];
    for (index, action) in actions.into_iter().enumerate() {
        insert_non_colliding_action(&mut value, &format!("phase7-action-{index}"), action);
    }

    // Act
    let result = decode_rigid_world_request_jsonl(&encode_value(&value), &limits);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn rigid_world_rejects_invalid_phase7_step_and_directive_bounds() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut invalid_step = fixture_value();
    insert_non_colliding_action(
        &mut invalid_step,
        "invalid-configured-step",
        json!({ "kind": "configured_step", "timestep_bits": 0.0_f32.to_bits(), "velocity_iterations": 0, "position_iterations": 3, "continuous_work_budget": 1 }),
    );
    let mut invalid_ray = fixture_value();
    insert_non_colliding_action(
        &mut invalid_ray,
        "invalid-ray-clip",
        json!({
            "kind": "ray_cast",
            "start": { "x_bits": 0.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "end": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "directive_rules": [{
                "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 0 },
                "directive": { "kind": "clip", "fraction_bits": 2.0_f32.to_bits() }
            }]
        }),
    );
    let mut invalid_query_child = fixture_value();
    insert_non_colliding_action(
        &mut invalid_query_child,
        "invalid-query-child",
        json!({
            "kind": "query_aabb",
            "aabb": {
                "lower": { "x_bits": (-1.0_f32).to_bits(), "y_bits": (-1.0_f32).to_bits() },
                "upper": { "x_bits": 1.0_f32.to_bits(), "y_bits": 1.0_f32.to_bits() }
            },
            "directive_rules": [{
                "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 1 },
                "directive": "terminate"
            }]
        }),
    );
    let mut invalid_ray_child = fixture_value();
    insert_non_colliding_action(
        &mut invalid_ray_child,
        "invalid-ray-child",
        json!({
            "kind": "ray_cast",
            "start": { "x_bits": (-1.0_f32).to_bits(), "y_bits": 0.0_f32.to_bits() },
            "end": { "x_bits": 1.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() },
            "directive_rules": [{
                "target": { "fixture_id": "nc-dynamic-fixture", "child_index": 1 },
                "directive": { "kind": "terminate" }
            }]
        }),
    );

    // Act
    let step_error = decode_rigid_world_request_jsonl(&encode_value(&invalid_step), &limits)
        .expect_err("zero velocity iterations must fail");
    let ray_error = decode_rigid_world_request_jsonl(&encode_value(&invalid_ray), &limits)
        .expect_err("clip fractions above one must fail");
    let query_child_error =
        decode_rigid_world_request_jsonl(&encode_value(&invalid_query_child), &limits)
            .expect_err("a query selector outside fixture topology must fail");
    let ray_child_error =
        decode_rigid_world_request_jsonl(&encode_value(&invalid_ray_child), &limits)
            .expect_err("a ray selector outside fixture topology must fail");

    // Assert
    assert_eq!(
        step_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidStepConfiguration)
    );
    assert_eq!(
        ray_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidRayDirective)
    );
    assert_eq!(
        query_child_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidQueryDirective)
    );
    assert_eq!(
        ray_child_error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidRayDirective)
    );
}

#[test]
fn rigid_world_rejects_both_zero_ray_clip_bit_patterns_before_execution() {
    // Arrange and Act
    let limits = HarnessLimits::phase2_default_v1();
    let errors = [0.0_f32.to_bits(), (-0.0_f32).to_bits()].map(|fraction_bits| {
        let mut value = fixture_value();
        let timeline = timeline_mut(&mut value, "world_query_and_ray_cast");
        for body_id in ["query-left", "query-center"] {
            let body = timeline["bodies"]
                .as_array_mut()
                .expect("query bodies should be an array")
                .iter_mut()
                .find(|body| body["body_id"] == body_id)
                .expect("fraction-zero witness body should exist");
            body["transform"]["position"]["x_bits"] = json!((-3.0_f32).to_bits());
        }
        let action = timeline["actions"]
            .as_array_mut()
            .expect("query actions should be an array")
            .iter_mut()
            .find(|action| action["action_id"] == "query-10")
            .expect("clip action should exist");
        action["action"]["directive_rules"][0]["directive"]["fraction_bits"] = json!(fraction_bits);
        decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
            .expect_err("zero clips must fail before multiple fraction-zero hits execute")
    });

    // Assert
    for error in errors {
        assert_eq!(
            error.rigid_world_kind(),
            Some(RigidWorldErrorKind::InvalidRayDirective)
        );
    }
}

#[test]
fn rigid_world_rejects_derived_degenerate_and_overflowing_rays_before_execution() {
    // Arrange and Act
    let limits = HarnessLimits::phase2_default_v1();
    let errors = [
        (0.0_f32.to_bits(), 0_u32, (-0.0_f32).to_bits(), 0_u32),
        (0.0_f32.to_bits(), 0_u32, 1_u32, 0_u32),
        ((-f32::MAX).to_bits(), 0_u32, f32::MAX.to_bits(), 0_u32),
        (0.0_f32.to_bits(), 0_u32, f32::MAX.to_bits(), 0_u32),
    ]
    .map(|(start_x_bits, start_y_bits, end_x_bits, end_y_bits)| {
        let mut value = fixture_value();
        let timeline = timeline_mut(&mut value, "world_query_and_ray_cast");
        let action = timeline["actions"]
            .as_array_mut()
            .expect("query actions should be an array")
            .iter_mut()
            .find(|action| action["action_id"] == "query-08")
            .expect("ray action should exist");
        action["action"]["start"] = json!({
            "x_bits": start_x_bits,
            "y_bits": start_y_bits
        });
        action["action"]["end"] = json!({
            "x_bits": end_x_bits,
            "y_bits": end_y_bits
        });
        decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
            .expect_err("invalid derived ray geometry must fail before execution")
    });

    // Assert
    for error in errors {
        assert_eq!(
            error.rigid_world_kind(),
            Some(RigidWorldErrorKind::InvalidRayDirective)
        );
    }
}

#[test]
fn rigid_world_phase7_results_expose_semantics_without_ccd_storage() {
    // Arrange
    let observations = vec![
        RigidWorldObservation::Step {
            outcome: RigidStepOutcome::Completed {
                completion: RigidStepCompletion::ContinuousPending,
            },
        },
        RigidWorldObservation::Step {
            outcome: RigidStepOutcome::Partial {
                classification: RigidPartialProgressClassification::ContinuousWorkBudgetExhausted,
            },
        },
    ]
    .into_boxed_slice();
    let timeline = RigidWorldTimelineResult {
        witness_family: RigidWorldWitnessFamily::NonCollidingBodyFixtureLifecycle,
        checkpoints: vec![RigidWorldCheckpointResult {
            checkpoint_id: ScenarioId::new("phase7-result").expect("ID should validate"),
            phase: "phase7".into(),
            counts: RigidExpectedCounts {
                bodies: 0,
                fixtures: 0,
                contacts: 0,
                manifold_points: 0,
                events: 0,
                destructions: 0,
            },
            bodies: Box::new([]),
            fixtures: Box::new([]),
            contacts: Box::new([]),
            events: Box::new([]),
            destructions: Box::new([]),
            observations,
        }]
        .into_boxed_slice(),
    };
    let record = RigidWorldResultRecord::new(
        RequestId::new("phase7-result-request").expect("ID should validate"),
        ScenarioId::new("phase7-result-scenario").expect("ID should validate"),
        vec![timeline],
    )
    .expect("bounded result should construct");
    let limits = HarnessLimits::phase2_default_v1();

    // Act
    let bytes =
        encode_jsonl(&record, &limits, RecordLimit::Output).expect("semantic result should encode");
    let decoded =
        decode_rigid_world_result_jsonl(&bytes, &limits).expect("semantic result should decode");
    let text = std::str::from_utf8(&bytes).expect("result should be UTF-8");

    // Assert
    assert_eq!(decoded.timelines()[0].checkpoints[0].observations.len(), 2);
    assert!(!text.contains("toi_count"));
    assert!(!text.contains("cache"));
    assert!(!text.contains("candidate"));
}

#[test]
fn rigid_world_result_round_trip_contains_only_semantic_identity() {
    // Arrange
    let checkpoint = |id: &str, family: RigidWorldWitnessFamily| RigidWorldTimelineResult {
        witness_family: family,
        checkpoints: vec![RigidWorldCheckpointResult {
            checkpoint_id: ScenarioId::new(id).expect("test checkpoint ID should validate"),
            phase: "empty".into(),
            counts: RigidExpectedCounts {
                bodies: 0,
                fixtures: 0,
                contacts: 0,
                manifold_points: 0,
                events: 0,
                destructions: 0,
            },
            bodies: Box::new([]),
            fixtures: Box::new([]),
            contacts: Box::new([]),
            events: Box::new([]),
            destructions: Box::new([]),
            observations: Box::new([]),
        }]
        .into_boxed_slice(),
    };
    let record = RigidWorldResultRecord::new(
        RequestId::new("result-request").expect("test request ID should validate"),
        ScenarioId::new("result-scenario").expect("test scenario ID should validate"),
        vec![
            checkpoint(
                "result-non-contact",
                RigidWorldWitnessFamily::NonCollidingBodyFixtureLifecycle,
            ),
            checkpoint(
                "result-contact",
                RigidWorldWitnessFamily::SingleContactLifecycle,
            ),
        ],
    )
    .expect("bounded result should construct");
    let limits = HarnessLimits::phase2_default_v1();

    // Act
    let bytes =
        encode_jsonl(&record, &limits, RecordLimit::Output).expect("bounded result should encode");
    let decoded =
        decode_rigid_world_result_jsonl(&bytes, &limits).expect("encoded result should decode");
    let text = std::str::from_utf8(&bytes).expect("result should be UTF-8");

    // Assert
    assert_eq!(decoded.timelines().len(), 2);
    assert!(!text.contains("handle"));
    assert!(!text.contains("pointer"));
}
