fn assert_observed_semantic(request: &Value, result: &Value, branch: &str) {
    let asserted = assert_system_witness(request, result, branch)
        || assert_lifecycle_witness(request, result, branch)
        || assert_contact_witness(request, result, branch)
        || assert_query_witness(request, result, branch)
        || assert_contract_witness(request, result, branch);
    assert!(
        asserted,
        "missing semantic assertion for Phase 9 branch {branch}"
    );
}

fn assert_system_witness(request: &Value, result: &Value, branch: &str) -> bool {
    let observe = |action_id| phase9_observation(request, result, action_id);
    let timeline = &request["scenario"]["timelines"][0];
    let growable = system_declaration(request, "phase9-growable");
    let fixed = system_declaration(request, "phase9-fixed-paused");
    let statistics = observe("statistics");
    match branch {
        "multiple_systems" | "newest_first" => {
            assert_eq!(statistics["statistics"]["system_count"], 2);
            assert_eq!(
                timeline["particle_systems"][0]["system_id"],
                "phase9-growable"
            );
            assert_eq!(
                timeline["particle_systems"][1]["system_id"],
                "phase9-fixed-paused"
            );
        }
        "paused_system" => {
            assert_eq!(fixed["paused"], true);
            assert_eq!(
                observe("statistics-fixed")["statistics"]["particle_count"],
                2
            );
        }
        "stable_ids_sort" => assert_eq!(
            observe("inspect-system")["particle_ids"],
            json!(["phase9-a", "phase9-b", "phase9-coupling", "phase9-capacity"])
        ),
        "stable_ids_compact" => assert_eq!(
            observe("compact-unrequested")["particle_ids"],
            json!(["phase9-coupling", "phase9-evicting", "phase9-c", "phase9-e"])
        ),
        "optional_lanes" => {
            let snapshot = &observe("inspect-particle")["snapshot"];
            assert_eq!(snapshot["color"], json!([0, 0, 255, 255]));
            assert_eq!(snapshot["force"], bits(0.0, 0.0));
            assert_eq!(snapshot["weight_bits"], 0);
        }
        "fixed_buffer" => assert_eq!(
            fixed["buffer_mode"],
            json!({ "kind": "fixed", "capacity": 2 })
        ),
        "growable_buffer" => assert_eq!(
            growable["buffer_mode"],
            json!({ "kind": "growable", "initial_capacity": 4 })
        ),
        "fixed_full" => {
            let fixed_statistics = &observe("statistics-fixed")["statistics"];
            assert_eq!(fixed_statistics["particle_count"], 2);
            assert_eq!(fixed_statistics["effective_capacity"], 2);
        }
        "teardown" => {
            assert_eq!(
                observe("destroy-fixed")["occurrence"]["kind"],
                "system_destroyed"
            );
            assert_eq!(
                observe("destroy-fixed")["occurrence"]["system_id"],
                "phase9-fixed-paused"
            );
            assert_eq!(
                observe("destroy-growable")["occurrence"]["system_id"],
                "phase9-growable"
            );
        }
        "retained_phase6_through_phase8" => assert_eq!(
            request["scenario"]["timelines"]
                .as_array()
                .expect("timelines")
                .len(),
            RigidWorldWitnessFamily::ALL.len()
        ),
        "phase10_rejection" => {
            for member in [
                "particle_groups",
                "particle_pairs",
                "particle_triads",
                "particle_solver",
            ] {
                assert!(
                    timeline.get(member).is_none(),
                    "{member} must remain absent"
                );
            }
        }
        _ => return false,
    }
    true
}

fn assert_lifecycle_witness(request: &Value, result: &Value, branch: &str) -> bool {
    let observe = |action_id| phase9_observation(request, result, action_id);
    let growable = system_declaration(request, "phase9-growable");
    let a = particle_declaration(request, "phase9-a");
    let b = particle_declaration(request, "phase9-b");
    let statistics = observe("statistics");
    match branch {
        "finite_lifetime" => assert_eq!(a["lifetime_bits"], 0.05_f32.to_bits()),
        "infinite_lifetime" => assert_eq!(b["lifetime_bits"], (-1.0_f32).to_bits()),
        "equal_lifetime" => assert_eq!(
            particle_declaration(request, "phase9-c")["lifetime_bits"],
            particle_declaration(request, "phase9-d")["lifetime_bits"]
        ),
        "oldest_lifetime" | "capacity_eviction" => {
            let occurrence = &observe("create-evicting")["occurrence"];
            assert_eq!(occurrence["kind"], "particle_destroyed");
            assert_eq!(occurrence["maybe_particle_id"], "phase9-a");
        }
        "maximum_lifetime" => {
            assert_eq!(growable["maximum_count"], 4);
            assert_eq!(statistics["statistics"]["effective_capacity"], 4);
        }
        "requested_destruction_callback" => {
            let occurrence = &observe("compact")["occurrence"];
            assert_eq!(occurrence["kind"], "particle_destroyed");
            assert_eq!(occurrence["maybe_particle_id"], "phase9-b");
        }
        "unrequested_destruction_callback" => {
            assert_eq!(observe("compact-unrequested")["kind"], "mixed_state");
            assert_no_particle_lifecycle(result, "phase9-capacity");
        }
        "zombie_pending" => {
            assert_eq!(observe("mark")["kind"], "mixed_state");
            assert!(
                observe("mark")["particle_ids"]
                    .as_array()
                    .expect("pending particle IDs")
                    .iter()
                    .any(|particle_id| particle_id == "phase9-b")
            );
            assert_eq!(
                observe("compact")["occurrence"]["maybe_particle_id"],
                "phase9-b"
            );
        }
        _ => return false,
    }
    true
}

fn assert_contact_witness(request: &Value, result: &Value, branch: &str) -> bool {
    let observe = |action_id| phase9_observation(request, result, action_id);
    let growable = system_declaration(request, "phase9-growable");
    let fixed = system_declaration(request, "phase9-fixed-paused");
    let a = particle_declaration(request, "phase9-a");
    let b = particle_declaration(request, "phase9-b");
    match branch {
        "particle_contact" => assert_eq!(
            observe("inspect-particle-contact")["contact"]["system_id"],
            "phase9-fixed-paused"
        ),
        "body_contact" => assert_eq!(
            observe("inspect-body-contact")["contact"]["fixture_id"],
            "nc-kinematic-fixture"
        ),
        "strict_contact_enabled" => assert_eq!(fixed["strict_contact_check"], true),
        "strict_contact_disabled" => assert_eq!(growable["strict_contact_check"], false),
        "listener_flag_enabled" => {
            assert_ne!(b["flags_bits"].as_u64().expect("flags") & (1 << 15), 0);
        }
        "listener_flag_disabled" => assert_eq!(
            particle_declaration(request, "phase9-capacity")["flags_bits"],
            0
        ),
        "filter_flag_enabled" => {
            assert_ne!(a["flags_bits"].as_u64().expect("flags") & (1 << 16), 0);
        }
        "filter_flag_disabled" => assert_eq!(
            particle_declaration(request, "phase9-coupling")["flags_bits"],
            0
        ),
        "contact_order" => {
            let particle_contact = observe("inspect-particle-contact");
            assert_eq!(particle_contact["contact"]["particle_a_id"], "phase9-c");
            assert_eq!(particle_contact["contact"]["particle_b_id"], "phase9-d");
        }
        "contact_multiplicity" => {
            let particle_contact = observe("inspect-particle-contact");
            assert_ne!(
                particle_contact["contact"]["particle_a_id"],
                particle_contact["contact"]["particle_b_id"]
            );
            assert!(
                particle_contact["contact"]["weight_bits"]
                    .as_u64()
                    .expect("weight")
                    > 0
            );
        }
        "coupling_fields" => {
            let body_contact = observe("inspect-body-contact");
            assert_eq!(body_contact["contact"]["particle_id"], "phase9-coupling");
            assert!(body_contact["contact"]["mass_bits"].as_u64().expect("mass") > 0);
            assert!(
                body_contact["contact"]["weight_bits"]
                    .as_u64()
                    .expect("weight")
                    > 0
            );
        }
        "dynamic_body_reaction" => {
            let body = phase9_checkpoint(result, "phase9-corpus")["bodies"]
                .as_array()
                .expect("bodies")
                .iter()
                .find(|body| body["body_id"] == "nc-dynamic")
                .expect("dynamic body");
            assert!(
                body["linear_velocity"]["x_bits"] != 0 || body["linear_velocity"]["y_bits"] != 0
            );
        }
        "static_body_no_reaction" => {
            let body = phase9_checkpoint(result, "phase9-corpus")["bodies"]
                .as_array()
                .expect("bodies")
                .iter()
                .find(|body| body["body_id"] == "nc-static")
                .expect("static body");
            assert_eq!(body["linear_velocity"], bits(0.0, 0.0));
        }
        _ => return false,
    }
    true
}

fn assert_query_witness(request: &Value, result: &Value, branch: &str) -> bool {
    let observe = |action_id| phase9_observation(request, result, action_id);
    let statistics = observe("statistics");
    match branch {
        "force_range" => assert_ne!(
            observe("inspect-after-force")["snapshot"]["force"]["x_bits"],
            0
        ),
        "impulse_range" => assert_ne!(
            observe("inspect-after-impulse")["snapshot"]["velocity"]["y_bits"],
            0
        ),
        "statistics_counts" => {
            assert_eq!(statistics["statistics"]["particle_count"], 4);
            assert_eq!(statistics["statistics"]["system_count"], 2);
        }
        "collision_energy" => assert_eq!(statistics["statistics"]["collision_energy_bits"], 0),
        "stuck_candidates" => assert_eq!(statistics["statistics"]["stuck_particle_ids"], json!([])),
        "system_aabb" | "system_culling" | "query_continue" => {
            let system_query = observe("system-query");
            assert_eq!(
                system_query["particle_ids"],
                json!(["phase9-a", "phase9-b"])
            );
            assert_eq!(system_query["terminated"], false);
        }
        "world_aabb" => assert_eq!(
            observe("world-query")["particle_ids"],
            json!(["phase9-c", "phase9-d", "phase9-a", "phase9-b"])
        ),
        "query_terminate" => {
            assert_eq!(
                observe("query-terminate")["particle_ids"],
                json!(["phase9-a"])
            );
            assert_eq!(observe("query-terminate")["terminated"], true);
        }
        "system_ray" | "ray_culling" | "ray_start_inside_exclusion" | "ray_continue" => {
            let system_ray = observe("system-ray");
            assert_eq!(system_ray["particle_ids"], json!(["phase9-a", "phase9-b"]));
            assert_eq!(system_ray["terminated"], false);
            assert!(
                system_ray["fractions_bits"]
                    .as_array()
                    .expect("fractions")
                    .iter()
                    .all(|bits| bits.as_u64().expect("fraction bits") != 0)
            );
        }
        "world_ray" => assert_eq!(
            observe("world-ray")["particle_ids"],
            json!(["phase9-c", "phase9-d", "phase9-a", "phase9-b"])
        ),
        "ray_ignore" => assert_eq!(observe("ray-ignore")["terminated"], false),
        "ray_clip" => assert_eq!(observe("ray-clip")["particle_ids"], json!(["phase9-a"])),
        "ray_terminate" => assert_eq!(observe("ray-terminate")["terminated"], true),
        _ => return false,
    }
    true
}

fn assert_contract_witness(request: &Value, result: &Value, branch: &str) -> bool {
    match branch {
        "closed_policy_registry" => assert_eq!(PHASE9_REQUIRED_POLICY_PATHS.len(), 22),
        "replay_identity"
        | "minimization_identity"
        | "first_divergence_stability"
        | "d0_byte_identity"
        | "debug_release_agreement" => {
            assert_eq!(result["request_id"], request["request_id"]);
            assert_eq!(result["scenario_id"], request["scenario"]["scenario_id"]);
        }
        _ => return false,
    }
    true
}
