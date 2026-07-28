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

fn insert_phase10_actions_before_destroy(value: &mut Value, inserted: Vec<Value>) {
    let actions = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array");
    let insertion_index = actions
        .iter()
        .position(|action| action["action_id"] == "p10-destroy-a")
        .expect("destroy action should exist");
    actions.splice(insertion_index..insertion_index, inserted);
}

fn add_transient_created_groups(value: &mut Value, count: usize) {
    let mut actions = Vec::with_capacity(count * 2);
    for index in 0..count {
        let index_u16 = u16::try_from(index).expect("test group count fits in u16");
        let group_id = format!("extra-group-{index}");
        let particle_id = format!("extra-particle-{index}");
        let mut group = serde_json::to_value(definition()).expect("definition should encode");
        group["group_id"] = json!(group_id);
        group["member_ids"] = json!([particle_id]);
        group["source"] = json!({
            "kind": "explicit",
            "positions": [{ "x_bits": bits(f32::from(index_u16)).bits(), "y_bits": bits(0.0).bits() }]
        });
        actions.push(json!({
            "action_id": format!("p10-extra-create-{index}"),
            "phase": "phase10",
            "action": { "kind": "particle_group", "operation": { "kind": "create_group", "definition": group } }
        }));
        actions.push(json!({
            "action_id": format!("p10-extra-destroy-{index}"),
            "phase": "phase10",
            "action": { "kind": "particle_group", "operation": { "kind": "destroy_group", "group_id": group_id } }
        }));
    }
    insert_phase10_actions_before_destroy(value, actions);
}

fn insert_phase10_actions_before_flags(value: &mut Value, inserted: Vec<Value>) {
    let actions = value["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array");
    let insertion_index = actions
        .iter()
        .position(|action| action["action_id"] == "p10-flags")
        .expect("flags action should exist");
    actions.splice(insertion_index..insertion_index, inserted);
}

fn add_split_group_identities(value: &mut Value, count: usize) {
    let created_group_ids = (0..count)
        .map(|index| format!("split-group-{index}"))
        .collect::<Vec<_>>();
    let mut actions = created_group_ids
        .chunks(30)
        .enumerate()
        .map(|(index, ids)| {
            json!({
                "action_id": format!("p10-extra-split-{index}"),
                "phase": "phase10",
                "action": { "kind": "particle_group", "operation": {
                    "kind": "split_group",
                    "group_id": "group-a",
                    "created_group_ids": ids
                } }
            })
        })
        .collect::<Vec<_>>();
    actions.extend(created_group_ids.into_iter().enumerate().map(|(index, group_id)| {
        json!({
            "action_id": format!("p10-extra-split-destroy-{index}"),
            "phase": "phase10",
            "action": { "kind": "particle_group", "operation": { "kind": "destroy_group", "group_id": group_id } }
        })
    }));
    insert_phase10_actions_before_flags(value, actions);
}
