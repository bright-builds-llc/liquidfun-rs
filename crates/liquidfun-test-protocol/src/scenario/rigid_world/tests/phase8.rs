#[test]
fn rigid_world_phase8_fixture_covers_all_new_families_and_joint_kinds() {
    // Arrange
    let value = fixture_value();
    let timelines = value["scenario"]["timelines"]
        .as_array()
        .expect("fixture timelines should be an array");
    let expected_families = RigidWorldWitnessFamily::PHASE8_REQUIRED
        .map(|family| serde_json::to_value(family).expect("family should serialize"));
    let joint_timeline = timelines
        .iter()
        .find(|timeline| timeline["witness_family"] == "joint_definitions_and_mutations")
        .expect("joint definition timeline should exist");

    // Act
    let actual_families = timelines
        .iter()
        .skip(RigidWorldWitnessFamily::ALL.len() - RigidWorldWitnessFamily::PHASE8_REQUIRED.len())
        .map(|timeline| timeline["witness_family"].clone())
        .collect::<Vec<_>>();
    let actual_kinds = joint_timeline["joints"]
        .as_array()
        .expect("joint declarations should be an array")
        .iter()
        .map(|joint| joint["definition"]["kind"].clone())
        .collect::<Vec<_>>();
    let expected_kinds = [
        RigidJointKind::Revolute,
        RigidJointKind::Prismatic,
        RigidJointKind::Distance,
        RigidJointKind::Pulley,
        RigidJointKind::Mouse,
        RigidJointKind::Wheel,
        RigidJointKind::Weld,
        RigidJointKind::Friction,
        RigidJointKind::Rope,
        RigidJointKind::Motor,
        RigidJointKind::Gear,
    ]
    .map(|kind| serde_json::to_value(kind).expect("joint kind should serialize"));

    // Assert
    assert_eq!(actual_families, expected_families);
    assert_eq!(actual_kinds, expected_kinds);
}

#[test]
fn rigid_world_phase8_step_dependent_families_step_before_observation() {
    // Arrange
    let value = fixture_value();
    let step_dependent_families = [
        "joint_definitions_and_mutations",
        "revolute_prismatic_limits_and_motors",
        "distance_pulley_mouse_constraints",
        "wheel_weld_friction_rope_motor_constraints",
        "gear_dependencies_and_four_body_solver",
        "mixed_joint_island_order_and_collision_suppression",
        "contact_filter_listener_and_pre_solve_timing",
        "destruction_listener_and_dependency_cascades",
    ];

    // Act
    let invalid_families = step_dependent_families
        .into_iter()
        .filter(|family| {
            let timeline = value["scenario"]["timelines"]
                .as_array()
                .expect("fixture timelines should be an array")
                .iter()
                .find(|timeline| timeline["witness_family"] == *family)
                .expect("fixture should contain requested witness family");
            let actions = timeline["actions"]
                .as_array()
                .expect("fixture actions should be an array");
            let maybe_step = actions.iter().position(|record| {
                matches!(record["action"]["kind"].as_str(), Some("step" | "configured_step"))
                    && record["action"]["timestep_bits"].as_u64().is_some_and(|bits| bits != 0)
            });
            let maybe_observation = actions.iter().position(|record| {
                matches!(
                    record["action"]["kind"].as_str(),
                    Some("inspect_joint" | "inspect_body" | "destroy_fixture" | "destroy_body")
                )
            });
            !matches!((maybe_step, maybe_observation), (Some(step), Some(observation)) if step < observation)
        })
        .collect::<Vec<_>>();

    // Assert
    assert!(
        invalid_families.is_empty(),
        "every step-dependent family must step before observation: {invalid_families:?}"
    );
}

#[test]
fn rigid_world_phase8_mixed_joint_restores_collision_after_suppressor_destruction() {
    // Arrange
    let mut value = fixture_value();
    let timeline = timeline_mut(
        &mut value,
        "mixed_joint_island_order_and_collision_suppression",
    );
    let joints = timeline["joints"]
        .as_array()
        .expect("mixed-joint declarations should be an array");
    let actions = timeline["actions"]
        .as_array()
        .expect("mixed-joint actions should be an array");

    // Act
    let suppresses_collision = joints
        .iter()
        .find(|joint| joint["joint_id"] == "joint-mixed-suppress")
        .and_then(|joint| joint["collide_connected"].as_bool());
    let permits_collision = joints
        .iter()
        .find(|joint| joint["joint_id"] == "joint-mixed-connected")
        .and_then(|joint| joint["collide_connected"].as_bool());
    let destroy_suppressor = actions
        .iter()
        .position(|action| action["action_id"] == "joint-mixed-destroy-suppress");
    let restored_step = actions
        .iter()
        .position(|action| action["action_id"] == "joint-mixed-step-restored");
    let destroy_connected = actions
        .iter()
        .position(|action| action["action_id"] == "joint-mixed-destroy-connected");

    // Assert
    assert_eq!(suppresses_collision, Some(false));
    assert_eq!(permits_collision, Some(true));
    assert!(
        matches!(
            (destroy_suppressor, restored_step, destroy_connected),
            (Some(suppressor), Some(step), Some(connected))
                if suppressor < step && step < connected
        ),
        "the permitting joint must remain live when collision is restored"
    );
}

#[test]
fn rigid_world_phase8_uses_explicit_behavior_witnesses() {
    // Arrange
    let value = fixture_value();

    // Act
    let placeholder_witnesses = value["scenario"]["timelines"]
        .as_array()
        .expect("fixture timelines should be an array")
        .iter()
        .skip(RigidWorldWitnessFamily::ALL.len() - RigidWorldWitnessFamily::PHASE8_REQUIRED.len())
        .flat_map(|timeline| {
            timeline["checkpoints"]
                .as_array()
                .expect("fixture checkpoints should be an array")
        })
        .flat_map(|checkpoint| {
            checkpoint["transitions"]
                .as_array()
                .expect("fixture transitions should be an array")
        })
        .filter_map(|transition| transition["witness"].as_str())
        .filter(|witness| witness.ends_with("_covered"))
        .collect::<Vec<_>>();

    // Assert
    assert!(
        placeholder_witnesses.is_empty(),
        "coverage labels are not behavior evidence: {placeholder_witnesses:?}"
    );
}

#[test]
fn rigid_world_phase8_rejects_zero_step_early_observation_and_ineligible_callbacks() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut zero_step = fixture_value();
    action_mut(&mut zero_step, "joint-rp-step-cold")["action"]["timestep_bits"] = json!(0);

    let mut early_observation = fixture_value();
    let actions = timeline_mut(
        &mut early_observation,
        "revolute_prismatic_limits_and_motors",
    )["actions"]
        .as_array_mut()
        .expect("fixture actions should be an array");
    let step = actions
        .iter()
        .position(|record| record["action_id"] == "joint-rp-step-cold")
        .expect("step should exist");
    let observation = actions
        .iter()
        .position(|record| record["action_id"] == "joint-rp-inspect-joint-rp-revolute-cold")
        .expect("post-step inspection should exist");
    actions.swap(step, observation);

    let mut ineligible_callback = fixture_value();
    timeline_mut(
        &mut ineligible_callback,
        "contact_filter_listener_and_pre_solve_timing",
    )["bodies"][1]["transform"]["position"]["x_bits"] = json!(10.0_f32.to_bits());

    // Act
    let errors = [zero_step, early_observation, ineligible_callback].map(|value| {
        decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
            .expect_err("non-behavioral Phase 8 timelines must fail closed")
    });

    // Assert
    assert_eq!(
        errors[0].rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidActionOrder)
    );
    assert_eq!(
        errors[1].rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidActionOrder)
    );
    assert_eq!(
        errors[2].rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidGeometry)
    );
}

#[test]
fn rigid_world_phase8_rejects_missing_gear_combination_and_unordered_destruction() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut missing_combination = fixture_value();
    let gear_joints = timeline_mut(
        &mut missing_combination,
        "gear_dependencies_and_four_body_solver",
    )["joints"]
        .as_array_mut()
        .expect("gear declarations should be an array");
    gear_joints[9]["definition"] = gear_joints[0]["definition"].clone();

    let mut unordered_destruction = fixture_value();
    let actions = timeline_mut(
        &mut unordered_destruction,
        "destruction_listener_and_dependency_cascades",
    )["actions"]
        .as_array_mut()
        .expect("destruction actions should be an array");
    let fixture_destruction = actions
        .iter()
        .position(|record| record["action_id"] == "destruction-destroy-explicit-fixture")
        .expect("explicit fixture destruction should exist");
    let dependent_cascade = actions
        .iter()
        .position(|record| record["action_id"] == "destruction-destroy-source-revolute")
        .expect("source destruction should trigger the dependent gear cascade");
    let fixture_action = actions.remove(fixture_destruction);
    actions.insert(dependent_cascade, fixture_action);

    // Act
    let errors = [missing_combination, unordered_destruction].map(|value| {
        decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
            .expect_err("incomplete topology or destruction ordering must fail")
    });

    // Assert
    assert_eq!(
        errors[0].rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidJointDefinition)
    );
    assert_eq!(
        errors[1].rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidActionOrder)
    );
}

#[test]
fn rigid_world_phase8_retains_phase6_and_phase7_family_order_and_actions() {
    // Arrange
    let value = fixture_value();
    let timelines = value["scenario"]["timelines"]
        .as_array()
        .expect("fixture timelines should be an array");
    let expected = [
        ("non_colliding_body_fixture_lifecycle", 28),
        ("single_contact_lifecycle", 19),
        ("body_control_and_force_policy", 18),
        ("multi_contact_island_and_warm_start", 14),
        ("sleeping_and_waking", 13),
        ("continuous_collision_and_sub_stepping", 13),
        ("continuous_budget_resume", 10),
        ("world_query_and_ray_cast", 15),
        ("origin_shift_covariance", 11),
    ];

    // Act
    let retained = timelines[..expected.len()]
        .iter()
        .map(|timeline| {
            (
                timeline["witness_family"]
                    .as_str()
                    .expect("family should be a string"),
                timeline["actions"]
                    .as_array()
                    .expect("actions should be an array")
                    .len(),
            )
        })
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(retained, expected);
}

#[test]
fn rigid_world_phase8_family_deletion_fails_closed() {
    // Arrange and Act
    let limits = HarnessLimits::phase2_default_v1();
    let errors = RigidWorldWitnessFamily::PHASE8_REQUIRED.map(|family| {
        let mut value = fixture_value();
        let family = serde_json::to_value(family).expect("family should serialize");
        value["scenario"]["timelines"]
            .as_array_mut()
            .expect("fixture timelines should be an array")
            .retain(|timeline| timeline["witness_family"] != family);
        decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
            .expect_err("missing Phase 8 family must fail")
    });

    // Assert
    assert!(errors.iter().all(|error| {
        error.rigid_world_kind() == Some(RigidWorldErrorKind::MissingWitnessFamily)
    }));
}

#[test]
fn rigid_world_phase8_accepts_every_closed_joint_mutation() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let vector = json!({ "x_bits": 2.0_f32.to_bits(), "y_bits": 0.0_f32.to_bits() });
    let mutations = [
        (
            "joint-def-revolute",
            json!({ "kind": "limit_enabled", "enabled": true }),
        ),
        (
            "joint-def-prismatic",
            json!({ "kind": "limits", "lower_bits": 0.0_f32.to_bits(), "upper_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-wheel",
            json!({ "kind": "motor_enabled", "enabled": true }),
        ),
        (
            "joint-def-revolute",
            json!({ "kind": "motor_speed", "speed_bits": 2.0_f32.to_bits() }),
        ),
        (
            "joint-def-prismatic",
            json!({ "kind": "max_motor_force", "force_bits": 2.0_f32.to_bits() }),
        ),
        (
            "joint-def-wheel",
            json!({ "kind": "max_motor_torque", "torque_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-distance",
            json!({ "kind": "length", "length_bits": 2.0_f32.to_bits() }),
        ),
        (
            "joint-def-weld",
            json!({ "kind": "frequency", "frequency_bits": 2.0_f32.to_bits() }),
        ),
        (
            "joint-def-mouse",
            json!({ "kind": "damping_ratio", "damping_ratio_bits": 0.25_f32.to_bits() }),
        ),
        (
            "joint-def-mouse",
            json!({ "kind": "mouse_target", "target": vector }),
        ),
        (
            "joint-def-friction",
            json!({ "kind": "max_force", "force_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-motor",
            json!({ "kind": "max_torque", "torque_bits": 2.0_f32.to_bits() }),
        ),
        (
            "joint-def-gear",
            json!({ "kind": "gear_ratio", "ratio_bits": (-1.0_f32).to_bits() }),
        ),
        (
            "joint-def-rope-joint",
            json!({ "kind": "rope_max_length", "max_length_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-motor",
            json!({ "kind": "linear_offset", "offset": vector }),
        ),
        (
            "joint-def-motor",
            json!({ "kind": "angular_offset", "offset_bits": 1.0_f32.to_bits() }),
        ),
        (
            "joint-def-motor",
            json!({ "kind": "correction_factor", "factor_bits": 0.75_f32.to_bits() }),
        ),
    ];

    // Act
    let results = mutations.map(|(joint_id, mutation)| {
        let mut value = fixture_value();
        if matches!(
            mutation["kind"].as_str(),
            Some("limit_enabled" | "motor_enabled")
        ) {
            let declaration = timeline_mut(&mut value, "joint_definitions_and_mutations")["joints"]
                .as_array_mut()
                .expect("fixture joints should be an array")
                .iter_mut()
                .find(|joint| joint["joint_id"] == joint_id)
                .expect("mutation target declaration should exist");
            let field = if mutation["kind"] == "limit_enabled" {
                "limit_enabled"
            } else {
                "motor_enabled"
            };
            declaration["definition"][field] = json!(false);
        }
        let action = action_mut(&mut value, "joint-def-mutate");
        action["action"]["joint_id"] = json!(joint_id);
        action["action"]["mutation"] = mutation;
        decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
    });

    // Assert
    assert!(
        results.iter().all(Result::is_ok),
        "each mutation should differ from its declaration: {results:?}"
    );
}

#[test]
fn rigid_world_phase8_rejects_mutation_for_wrong_joint_kind() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let action = action_mut(&mut value, "joint-def-mutate");
    action["action"]["joint_id"] = json!("joint-def-pulley");
    action["action"]["mutation"] =
        json!({ "kind": "motor_speed", "speed_bits": 1.0_f32.to_bits() });

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("unsupported joint mutations must fail closed");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidJointDefinition)
    );
}

#[test]
fn rigid_world_phase8_rejects_noop_mouse_target_mutation() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let action = action_mut(&mut value, "joint-dpm-mutate");
    action["action"]["mutation"]["target"] =
        json!({ "x_bits": 2.0_f32.to_bits(), "y_bits": 1.0_f32.to_bits() });

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("a mouse-target mutation must differ from its declaration");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidJointDefinition)
    );
}

#[test]
fn rigid_world_phase8_rejects_noop_motor_correction_mutation() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let action = action_mut(&mut value, "joint-coupled-mutate");
    action["action"]["mutation"]["factor_bits"] = json!(0.5_f32.to_bits());

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("a motor-correction mutation must differ from its declaration");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidJointDefinition)
    );
}

#[test]
fn rigid_world_phase8_rejects_noop_gear_ratio_mutation() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let action = action_mut(&mut value, "gear-mutate");
    action["action"]["mutation"]["ratio_bits"] = json!((-1.0_f32).to_bits());

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("a gear-ratio mutation must differ from its declaration");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidJointDefinition)
    );
}

#[test]
fn rigid_world_phase8_rejects_rope_with_fewer_than_three_vertices() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let declaration = &mut timeline_mut(&mut value, "standalone_rope_evolution")["ropes"][0];
    declaration["vertices"]
        .as_array_mut()
        .expect("vertices should be an array")
        .truncate(2);
    declaration["masses_bits"]
        .as_array_mut()
        .expect("masses should be an array")
        .truncate(2);

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("a two-vertex rope cannot map to the native RopeDef contract");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidRopeDefinition)
    );
}

#[test]
fn rigid_world_phase8_rejects_gear_sources_with_same_moving_body() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut value = fixture_value();
    let timeline = timeline_mut(&mut value, "gear_dependencies_and_four_body_solver");
    timeline["joints"][1]["body_a_id"] = json!("gear-0-base-a");
    timeline["joints"][1]["body_b_id"] = json!("gear-0-moving-a");

    // Act
    let error = decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
        .expect_err("gear sources must expose distinct moving endpoints");

    // Assert
    assert_eq!(
        error.rigid_world_kind(),
        Some(RigidWorldErrorKind::InvalidJointDependency)
    );
}

#[test]
fn rigid_world_phase8_rejects_unknown_kind_and_n_plus_one_bounds() {
    // Arrange
    let limits = HarnessLimits::phase2_default_v1();
    let mut unknown = fixture_value();
    timeline_mut(&mut unknown, "joint_definitions_and_mutations")["joints"][0]["definition"]["kind"] =
        json!("unknown_joint");
    let mut joints = fixture_value();
    let declarations = timeline_mut(&mut joints, "joint_definitions_and_mutations")["joints"]
        .as_array_mut()
        .expect("joint declarations should be an array");
    while declarations.len() <= RIGID_WORLD_MAXIMUM_JOINTS {
        let mut declaration = declarations[0].clone();
        declaration["joint_id"] = json!(format!("extra-joint-{}", declarations.len()));
        declarations.push(declaration);
    }
    let mut rope = fixture_value();
    let declaration = &mut timeline_mut(&mut rope, "standalone_rope_evolution")["ropes"][0];
    while declaration["vertices"]
        .as_array()
        .expect("vertices should be an array")
        .len()
        <= RIGID_WORLD_MAXIMUM_ROPE_VERTICES
    {
        declaration["vertices"]
            .as_array_mut()
            .expect("vertices should be an array")
            .push(json!({ "x_bits": 0, "y_bits": 0 }));
        declaration["masses_bits"]
            .as_array_mut()
            .expect("masses should be an array")
            .push(json!(0));
    }

    // Act
    let errors = [unknown, joints, rope].map(|value| {
        decode_rigid_world_request_jsonl(&encode_value(&value), &limits)
            .expect_err("unknown or N+1 Phase 8 input must fail")
    });

    // Assert
    assert!(
        errors
            .iter()
            .all(|error| matches!(error, RigidWorldDecodeError::Codec(_)))
    );
}
