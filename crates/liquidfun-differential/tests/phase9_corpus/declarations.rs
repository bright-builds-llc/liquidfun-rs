#[allow(
    clippy::similar_names,
    reason = "semantic fixture coordinates deliberately preserve fixed_c/fixed_d labels"
)]
fn configure_phase9_declarations(timeline: &mut Value, case_id: &str) {
    let contact_case = case_id == "contacts-listeners-filters-and-coupling";
    let fixed_offset = if contact_case { 18.515 } else { 0.0 };
    let fixed_d_x = if contact_case { fixed_offset } else { 0.4 };
    let fixed_c_y = if contact_case { -0.2 } else { 0.0 };
    let fixed_d_y = if contact_case { 0.2 } else { 0.0 };
    let (coupling_x, coupling_y) = if contact_case {
        (0.75, 0.0)
    } else {
        (20.0, 0.25)
    };
    timeline["particle_systems"] = json!([
        {
            "system_id": "phase9-growable", "buffer_mode": { "kind": "growable", "initial_capacity": 4 },
            "paused": false, "strict_contact_check": false, "stuck_threshold": 1,
            "density_bits": 1.0_f32.to_bits(), "gravity_scale_bits": 0.0_f32.to_bits(),
            "radius_bits": 0.25_f32.to_bits(), "damping_bits": 0.0_f32.to_bits(),
            "destruction_by_age": true, "lifetime_granularity_bits": (1.0_f32 / 60.0_f32).to_bits(),
            "maximum_count": 4
        },
        {
            "system_id": "phase9-fixed-paused", "buffer_mode": { "kind": "fixed", "capacity": 2 },
            "paused": true, "strict_contact_check": true, "stuck_threshold": 1,
            "density_bits": 1.0_f32.to_bits(), "gravity_scale_bits": 1.0_f32.to_bits(),
            "radius_bits": 0.25_f32.to_bits(), "damping_bits": 0.25_f32.to_bits(),
            "destruction_by_age": true, "lifetime_granularity_bits": (1.0_f32 / 60.0_f32).to_bits(),
            "maximum_count": 2
        }
    ]);
    timeline["particles"] = json!([
        particle_with_flags(
            "phase9-a",
            "phase9-growable",
            0.0,
            0.05,
            0,
            (1 << 9) | (1 << 14) | (1 << 16)
        ),
        particle_with_flags(
            "phase9-b",
            "phase9-growable",
            0.4,
            -1.0,
            1,
            (1 << 9) | (1 << 15) | (1 << 17)
        ),
        particle_with_velocity(
            "phase9-coupling",
            "phase9-growable",
            coupling_x,
            coupling_y,
            -2.0,
            0.0,
            2.0,
            4
        ),
        particle("phase9-capacity", "phase9-growable", 3.0, 2.0, 5),
        particle("phase9-evicting", "phase9-growable", 4.0, 4.0, 6),
        particle_with_velocity(
            "phase9-c",
            "phase9-fixed-paused",
            fixed_offset,
            fixed_c_y,
            0.0,
            0.0,
            2.0,
            2
        ),
        particle_with_flags_and_velocity(
            "phase9-d",
            "phase9-fixed-paused",
            fixed_d_x,
            fixed_d_y,
            0.0,
            0.0,
            2.0,
            3,
            (1 << 9) | (1 << 15)
        ),
        particle("phase9-e", "phase9-fixed-paused", 0.8, 2.0, 9)
    ]);
}

fn phase9_actions() -> Vec<Value> {
    let mut phase9_actions = vec![
        action(
            "create-growable",
            json!({ "kind": "create_system", "system_id": "phase9-growable" }),
        ),
        action(
            "create-fixed",
            json!({ "kind": "create_system", "system_id": "phase9-fixed-paused" }),
        ),
    ];
    for id in [
        "phase9-a",
        "phase9-b",
        "phase9-coupling",
        "phase9-capacity",
        "phase9-c",
        "phase9-d",
    ] {
        phase9_actions.push(action(
            &format!("create-{id}"),
            json!({ "kind": "create_particle", "particle_id": id }),
        ));
    }
    phase9_actions.extend([
        action("inspect-system", json!({ "kind": "inspect_system", "system_id": "phase9-growable" })),
        action("inspect-particle", json!({ "kind": "inspect_particle", "particle_id": "phase9-a" })),
        action("resume", json!({ "kind": "set_paused", "system_id": "phase9-fixed-paused", "paused": false })),
        json!({
            "action_id": "phase9-step",
            "phase": "phase9",
            "action": {
                "kind": "step",
                "timestep_bits": (1.0_f32 / 60.0_f32).to_bits(),
                "velocity_iterations": 8,
                "position_iterations": 3
            }
        }),
        action("energy-velocity-c", json!({ "kind": "set_velocity", "particle_id": "phase9-c", "velocity": bits(1.0, 0.0) })),
        action("energy-velocity-d", json!({ "kind": "set_velocity", "particle_id": "phase9-d", "velocity": bits(-1.0, 0.0) })),
        action("statistics", json!({ "kind": "request_statistics", "system_id": "phase9-growable" })),
        action("statistics-fixed", json!({ "kind": "request_statistics", "system_id": "phase9-fixed-paused" })),
        action("inspect-occurrence-zero", json!({ "kind": "inspect_occurrence", "occurrence_index": 0 })),
        action("inspect-occurrence-one", json!({ "kind": "inspect_occurrence", "occurrence_index": 1 })),
        action("inspect-system-after-step", json!({ "kind": "inspect_system", "system_id": "phase9-growable" })),
        action("system-query", json!({ "kind": "query_aabb", "system_id": "phase9-growable", "lower": bits(-1.0, -1.0), "upper": bits(2.0, 2.0), "control": "continue" })),
        action("world-query", json!({ "kind": "query_aabb", "system_id": null, "lower": bits(-1.0, -1.0), "upper": bits(2.0, 2.0), "control": "continue" })),
        action("query-terminate", json!({ "kind": "query_aabb", "system_id": "phase9-growable", "lower": bits(-1.0, -1.0), "upper": bits(2.0, 2.0), "control": "terminate" })),
        action("system-ray", json!({ "kind": "ray_cast", "system_id": "phase9-growable", "start": bits(-1.0, 0.0), "end": bits(2.0, 0.0), "control": "continue" })),
        action("world-ray", json!({ "kind": "ray_cast", "system_id": null, "start": bits(-1.0, 0.0), "end": bits(2.0, 0.0), "control": "continue" })),
        action("ray-ignore", json!({ "kind": "ray_cast", "system_id": "phase9-growable", "start": bits(-1.0, 0.0), "end": bits(0.1, 0.0), "control": "ignore" })),
        action("ray-clip", json!({ "kind": "ray_cast", "system_id": "phase9-growable", "start": bits(-1.0, 0.0), "end": bits(2.0, 0.0), "control": "clip" })),
        action("ray-terminate", json!({ "kind": "ray_cast", "system_id": "phase9-growable", "start": bits(-1.0, 0.0), "end": bits(2.0, 0.0), "control": "terminate" })),
        action("position", json!({ "kind": "set_position", "particle_id": "phase9-a", "position": bits(0.25, 0.0) })),
        action("velocity", json!({ "kind": "set_velocity", "particle_id": "phase9-a", "velocity": bits(0.0, 1.0) })),
        action("force", json!({ "kind": "apply_force", "particle_ids": ["phase9-a", "phase9-b"], "force": bits(1.0, 0.0) })),
        action("inspect-after-force", json!({ "kind": "inspect_particle", "particle_id": "phase9-a" })),
        action("impulse", json!({ "kind": "apply_impulse", "particle_ids": ["phase9-a", "phase9-b"], "impulse": bits(0.0, 1.0) })),
        action("inspect-after-impulse", json!({ "kind": "inspect_particle", "particle_id": "phase9-a" })),
        action("create-evicting", json!({ "kind": "create_particle", "particle_id": "phase9-evicting" })),
        action("create-phase9-e", json!({ "kind": "create_particle", "particle_id": "phase9-e" })),
        action("mark", json!({ "kind": "mark_for_destruction", "particle_id": "phase9-b" })),
        action("compact", json!({ "kind": "compact", "system_id": "phase9-growable" })),
        action("mark-unrequested", json!({ "kind": "mark_for_destruction", "particle_id": "phase9-capacity" })),
        action("compact-unrequested", json!({ "kind": "compact", "system_id": "phase9-growable" })),
        action("destroy-fixed", json!({ "kind": "destroy_system", "system_id": "phase9-fixed-paused" })),
        action("destroy-growable", json!({ "kind": "destroy_system", "system_id": "phase9-growable" })),
    ]);
    phase9_actions
}

fn order_phase9_actions(phase9_actions: &mut Vec<Value>, case_id: &str) {
    let statistics = ["statistics", "statistics-fixed"]
        .into_iter()
        .map(|action_id| {
            let index = phase9_actions
                .iter()
                .position(|record| record["action_id"] == action_id)
                .expect("the bounded corpus retains its statistics actions");
            phase9_actions.remove(index)
        })
        .collect::<Vec<_>>();
    let step_index = phase9_actions
        .iter()
        .position(|record| record["action_id"] == "phase9-step")
        .expect("the bounded corpus retains its particle step");
    let statistics_index = if case_id == "forces-impulses-and-statistics" {
        phase9_actions
            .iter()
            .position(|record| record["action_id"] == "energy-velocity-d")
            .expect("the force case retains its collision-energy velocity setup")
            + 1
    } else {
        step_index + 1
    };
    phase9_actions.splice(statistics_index..statistics_index, statistics);
    if matches!(
        case_id,
        "lifetime-zombie-and-eviction" | "contacts-listeners-filters-and-coupling"
    ) {
        let first_step = phase9_actions
            .iter()
            .position(|record| record["action_id"] == "phase9-step")
            .expect("the lifetime case retains its first particle step");
        let step_template = phase9_actions[first_step].clone();
        let final_step = if case_id == "lifetime-zombie-and-eviction" {
            4
        } else {
            3
        };
        let insertion_index = first_step + 1;
        phase9_actions.splice(
            insertion_index..insertion_index,
            (2..=final_step).map(|ordinal| {
                let mut step = step_template.clone();
                step["action_id"] = json!(format!("phase9-step-{ordinal}"));
                step
            }),
        );
    }
    if case_id == "contacts-listeners-filters-and-coupling" {
        let step_index = phase9_actions
            .iter()
            .position(|record| record["action_id"] == "phase9-step")
            .expect("the contact case retains its particle step")
            + 1;
        phase9_actions.splice(
            step_index..step_index,
            [
                action(
                    "inspect-particle-contact",
                    json!({ "kind": "inspect_particle_contact", "system_id": "phase9-fixed-paused", "contact_index": 0 }),
                ),
                action(
                    "inspect-body-contact",
                    json!({ "kind": "inspect_body_contact", "system_id": "phase9-growable", "contact_index": 1 }),
                ),
                action(
                    "contact-statistics-growable",
                    json!({ "kind": "request_statistics", "system_id": "phase9-growable" }),
                ),
                action(
                    "contact-statistics-fixed",
                    json!({ "kind": "request_statistics", "system_id": "phase9-fixed-paused" }),
                ),
            ],
        );
    }
    if case_id == "forces-impulses-and-statistics" {
        let mut moved = Vec::new();
        for action_id in FORCE_ACTIONS {
            let index = phase9_actions
                .iter()
                .position(|record| record["action_id"] == *action_id)
                .expect("the force case retains its pre-step action");
            moved.push(phase9_actions.remove(index));
        }
        let step_index = phase9_actions
            .iter()
            .position(|record| record["action_id"] == "phase9-step")
            .expect("the force case retains its particle step");
        phase9_actions.splice(step_index..step_index, moved);
    }
}

fn retain_relevant_actions(phase9_actions: &mut Vec<Value>, case_id: &str) {
    let relevant: &[&str] = match case_id {
        "storage-systems-and-permutations" | "lifetime-zombie-and-eviction" => &[
            "create-evicting",
            "create-phase9-e",
            "mark",
            "compact",
            "mark-unrequested",
            "compact-unrequested",
            "inspect-system-after-step",
        ],
        "contacts-listeners-filters-and-coupling" => &[
            "inspect-particle-contact",
            "inspect-body-contact",
            "inspect-occurrence-zero",
            "contact-statistics-growable",
            "contact-statistics-fixed",
        ],
        "forces-impulses-and-statistics" => &[
            "position",
            "velocity",
            "force",
            "inspect-after-force",
            "impulse",
            "inspect-after-impulse",
            "energy-velocity-c",
            "energy-velocity-d",
        ],
        "aabb-query-control-and-culling" => &["system-query", "world-query", "query-terminate"],
        "ray-control-and-culling" => &[
            "system-ray",
            "world-ray",
            "ray-ignore",
            "ray-clip",
            "ray-terminate",
        ],
        "closed-evidence-contract" => &[],
        other => panic!("unreviewed Phase 9 corpus case {other}"),
    };
    phase9_actions.retain(|record| {
        let action_id = record["action_id"].as_str().expect("action ID");
        COMMON_ACTIONS.contains(&action_id)
            || relevant.contains(&action_id)
            || action_id.starts_with("phase9-step-")
    });
}

fn insert_phase9_checkpoints(timeline: &mut Value, case_id: &str, final_action: &Value) {
    let checkpoints = timeline["checkpoints"]
        .as_array_mut()
        .expect("retained checkpoints should be an array");
    let checkpoint_index = usize::from(case_id == "contacts-listeners-filters-and-coupling");
    let first_rigid_counts = if case_id == "contacts-listeners-filters-and-coupling" {
        json!({
            "bodies": 3, "fixtures": 3, "contacts": 0,
            "manifold_points": 0, "events": 0, "destructions": 0
        })
    } else {
        json!({
            "bodies": 0, "fixtures": 0, "contacts": 0,
            "manifold_points": 0, "events": 0, "destructions": 0
        })
    };
    checkpoints.insert(
        checkpoint_index,
        json!({
            "checkpoint_id": "phase9-only-checkpoint",
            "after_action_id": "inspect-system",
            "phase": "phase9",
            "counts": first_rigid_counts,
            "transitions": []
        }),
    );
    checkpoints.insert(
        checkpoint_index + 1,
        json!({
            "checkpoint_id": "phase9-corpus",
            "after_action_id": final_action,
            "phase": "phase9",
            "counts": {
                "bodies": if case_id == "contacts-listeners-filters-and-coupling" { 3 } else { 0 },
                "fixtures": if case_id == "contacts-listeners-filters-and-coupling" { 3 } else { 0 },
                "contacts": 0,
                "manifold_points": 0,
                "events": 0,
                "destructions": 0
            },
            "transitions": []
        }),
    );
}
