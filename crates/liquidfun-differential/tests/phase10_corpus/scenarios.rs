#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusManifest {
    schema_version: u32,
    profile: String,
    upstream_revision: String,
    policy_count: usize,
    policy_sha256: String,
    leaf_sha256: String,
    manifest_payload_sha256: String,
    retained_phase9_manifest: String,
    retained_phase9_manifest_sha256: String,
    cases: Vec<CorpusCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusCase {
    case_id: String,
    fixture: String,
    fixture_sha256: String,
    request_sha256: String,
    leaves: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseRecipe {
    case_id: String,
    seed: u64,
    particle_flags_bits: u32,
    group_flags_bits: u32,
    mutation_family: bool,
}

fn corpus_manifest() -> CorpusManifest {
    serde_json::from_str(MANIFEST).expect("Phase 10 corpus manifest is strict JSON")
}

fn fixture_path(case: &CorpusCase) -> PathBuf {
    let relative = Path::new(&case.fixture);
    assert!(!relative.is_absolute());
    assert!(
        relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    );
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rigid_world/phase10")
        .join(relative)
}

fn recipe(case: &CorpusCase) -> CaseRecipe {
    let bytes = std::fs::read(fixture_path(case)).expect("case recipe bytes are readable");
    let recipe: CaseRecipe =
        serde_json::from_slice(&bytes).expect("case recipe is one strict JSON record");
    assert_eq!(recipe.case_id, case.case_id);
    recipe
}

fn bits(value: f32) -> u32 {
    FloatBits::from_f32(value).bits()
}

fn vector(x: f32, y: f32) -> Value {
    json!({ "x_bits": bits(x), "y_bits": bits(y) })
}

fn system_declaration() -> Value {
    json!({
        "system_id": "phase10-system",
        "buffer_mode": { "kind": "growable", "initial_capacity": 32 },
        "paused": false,
        "strict_contact_check": true,
        "stuck_threshold": 2,
        "density_bits": bits(1.0),
        "gravity_scale_bits": bits(0.0),
        "radius_bits": bits(0.25),
        "damping_bits": bits(0.0),
        "destruction_by_age": true,
        "lifetime_granularity_bits": bits(1.0 / 60.0),
        "maximum_count": null
    })
}

fn definition(
    recipe: &CaseRecipe,
    group_id: &str,
    member_ids: &[&str],
    positions: &[(f32, f32)],
) -> Value {
    json!({
        "provenance": {
            "extension_version": 1,
            "generator_id": recipe.case_id,
            "generator_version": "phase10-v1",
            "upstream_revision": UPSTREAM_REVISION,
            "toolchain_id": "phase10-corpus",
            "seed": recipe.seed
        },
        "system_id": "phase10-system",
        "group_id": group_id,
        "member_ids": member_ids,
        "source": {
            "kind": "explicit",
            "positions": positions.iter().map(|(x, y)| vector(*x, *y)).collect::<Vec<_>>()
        },
        "destination": { "kind": "new" },
        "particle_flags_bits": recipe.particle_flags_bits,
        "group_flags_bits": recipe.group_flags_bits & 4,
        "transform": { "position": vector(0.0, 0.0), "angle_bits": bits(0.0) },
        "linear_velocity": vector(0.0, 0.0),
        "angular_velocity_bits": bits(0.0),
        "color": [64, 128, 192, 255],
        "strength_bits": bits(1.0),
        "maybe_stride_bits": null,
        "lifetime_bits": bits(0.0)
    })
}

fn particle_action(kind: &str) -> Value {
    json!({ "kind": "particle", "action": { "kind": kind, "system_id": "phase10-system" } })
}

fn group_action(operation: Value) -> Value {
    let value = json!({ "kind": "particle_group", "operation": operation });
    drop(operation);
    value
}

fn action(action_id: &str, action: Value) -> Value {
    let value = json!({ "action_id": action_id, "phase": "phase10", "action": action });
    drop(action);
    value
}

fn group_construction_actions(recipe: &CaseRecipe) -> Vec<Value> {
    let mut appended = definition(recipe, "group-b", &["particle-d"], &[(20.25, 0.0)]);
    appended["destination"] = json!({ "kind": "append_to", "target_group_id": "group-b" });
    vec![
        action("p10-create-system", particle_action("create_system")),
        action(
            "p10-create-a",
            group_action(json!({
                "kind": "create_group",
                "definition": definition(
                    recipe,
                    "group-a",
                    &["particle-a", "particle-b", "particle-e", "particle-g"],
                    &[(-0.2, 0.0), (0.2, 0.0), (-10.0, 0.0), (10.0, 0.0)]
                )
            })),
        ),
        action(
            "p10-pre-split-step",
            group_action(json!({
                "kind": "step", "timestep_bits": bits(1.0 / 60.0),
                "velocity_iterations": 8, "position_iterations": 3, "particle_iterations": 1
            })),
        ),
        action(
            "p10-inspect-control",
            group_action(json!({ "kind": "inspect_state" })),
        ),
        action(
            "p10-split-a",
            group_action(json!({
                "kind": "split_group",
                "group_id": "group-a",
                "created_group_ids": ["group-c", "group-e"]
            })),
        ),
        action(
            "p10-create-b",
            group_action(json!({
                "kind": "create_group",
                "definition": definition(recipe, "group-b", &["particle-c"], &[(20.0, 0.0)])
            })),
        ),
        action(
            "p10-append-b",
            group_action(json!({ "kind": "create_group", "definition": appended })),
        ),
        action(
            "p10-join",
            group_action(json!({
                "kind": "join_groups",
                "target_group_id": "group-e",
                "source_group_id": "group-b"
            })),
        ),
        action(
            "p10-step",
            group_action(json!({
                "kind": "step", "timestep_bits": bits(1.0 / 60.0),
                "velocity_iterations": 8, "position_iterations": 3, "particle_iterations": 1
            })),
        ),
        action(
            "p10-inspect",
            group_action(json!({ "kind": "inspect_state" })),
        ),
        action(
            "p10-destroy-a",
            group_action(json!({ "kind": "destroy_group", "group_id": "group-a" })),
        ),
        action(
            "p10-destroy-c",
            group_action(json!({ "kind": "destroy_group", "group_id": "group-c" })),
        ),
        action(
            "p10-destroy-e",
            group_action(json!({ "kind": "destroy_group", "group_id": "group-e" })),
        ),
        action(
            "p10-compact",
            group_action(json!({
                "kind": "step", "timestep_bits": bits(1.0 / 60.0),
                "velocity_iterations": 8, "position_iterations": 3, "particle_iterations": 1
            })),
        ),
        action("p10-destroy-system", particle_action("destroy_system")),
    ]
}

fn standard_actions(recipe: &CaseRecipe) -> Vec<Value> {
    let initial_definition = if recipe.case_id == "pressure-constraints-and-rigid" {
        definition(recipe, "group-a", &["particle-a"], &[(-0.9, 0.0)])
    } else {
        definition(
            recipe,
            "group-a",
            &["particle-a", "particle-b", "particle-c"],
            &[(0.0, 0.0), (0.5, 0.0), (0.25, 0.5)],
        )
    };
    let mut actions = vec![
        action("p10-create-system", particle_action("create_system")),
        action(
            "p10-create-a",
            group_action(json!({
                "kind": "create_group",
                "definition": initial_definition
            })),
        ),
    ];
    if recipe.case_id == "boundary-order-and-inherited" {
        actions.push(action(
            "p10-inspect-boundary-flags",
            group_action(json!({ "kind": "inspect_state" })),
        ));
    }
    if recipe.mutation_family {
        let mut appended = definition(recipe, "group-a", &["particle-d"], &[(1.0, 0.0)]);
        appended["destination"] = json!({ "kind": "append_to", "target_group_id": "group-a" });
        actions.push(action(
            "p10-append-a",
            group_action(json!({ "kind": "create_group", "definition": appended })),
        ));
    }
    actions.push(action(
        "p10-step",
        group_action(json!({
            "kind": "step", "timestep_bits": bits(1.0 / 60.0),
            "velocity_iterations": 8, "position_iterations": 3, "particle_iterations": 1
        })),
    ));
    if recipe.group_flags_bits & 3 != 0 {
        actions.push(action(
            "p10-set-group-flags",
            group_action(json!({
                "kind": "set_group_flags", "group_id": "group-a",
                "group_flags_bits": recipe.group_flags_bits
            })),
        ));
    }
    actions.push(action(
        "p10-inspect",
        group_action(json!({ "kind": "inspect_state" })),
    ));
    actions.push(action(
        "p10-destroy",
        group_action(json!({ "kind": "destroy_group", "group_id": "group-a" })),
    ));
    actions.extend([
        action(
            "p10-compact",
            group_action(json!({
                "kind": "step", "timestep_bits": bits(1.0 / 60.0),
                "velocity_iterations": 8, "position_iterations": 3, "particle_iterations": 1
            })),
        ),
        action(
            "p10-inspect-destroy",
            group_action(json!({ "kind": "inspect_state" })),
        ),
        action("p10-destroy-system", particle_action("destroy_system")),
    ]);
    actions
}

fn case_request_value(recipe: &CaseRecipe) -> Value {
    let mut value: Value = serde_json::from_slice(BASE_REQUEST).expect("base request is JSON");
    value["request_id"] = json!(format!("phase10-{}", recipe.case_id));
    let timeline = value["scenario"]["timelines"]
        .as_array_mut()
        .expect("timelines is an array")
        .first_mut()
        .expect("base request has a timeline");
    timeline["particle_systems"] = json!([system_declaration()]);
    timeline["particles"] = json!([]);
    let actions = if recipe.case_id == "group-construction-and-mutation" {
        group_construction_actions(recipe)
    } else {
        standard_actions(recipe)
    };
    let timeline_actions = timeline["actions"]
        .as_array_mut()
        .expect("actions is an array");
    if recipe.case_id == "pressure-constraints-and-rigid" {
        let teardown_index = timeline_actions
            .iter()
            .position(|record| record["action_id"] == "nc-destroy-fixture-static")
            .expect("base request has a teardown boundary");
        timeline_actions.splice(teardown_index..teardown_index, actions);
    } else {
        timeline_actions.extend(actions);
        let checkpoint = timeline["checkpoints"]
            .as_array_mut()
            .expect("checkpoints is an array")
            .last_mut()
            .expect("base request has a final checkpoint");
        checkpoint["after_action_id"] = json!("p10-destroy-system");
        checkpoint["phase"] = json!("phase10");
    }
    value
}

fn case_request(
    recipe: &CaseRecipe,
) -> (Vec<u8>, liquidfun_test_protocol::RigidWorldRequestRecord) {
    let limits = HarnessLimits::phase2_default_v1();
    let mut candidate =
        serde_json::to_vec(&case_request_value(recipe)).expect("request candidate encodes");
    candidate.push(b'\n');
    let request = decode_rigid_world_request_jsonl(&candidate, &limits)
        .expect("bounded case request validates");
    let canonical =
        encode_jsonl(&request, &limits, RecordLimit::Input).expect("request canonicalizes");
    (canonical, request)
}

fn phase10_observations(
    result: &liquidfun_test_protocol::RigidWorldResultRecord,
) -> Vec<&Phase10Observation> {
    result
        .timelines()
        .iter()
        .flat_map(|timeline| &timeline.checkpoints)
        .flat_map(|checkpoint| &checkpoint.observations)
        .filter_map(|observation| {
            let RigidWorldObservation::ParticleGroup { observation } = observation else {
                return None;
            };
            Some(observation)
        })
        .collect()
}

fn phase10_observation(
    result: &liquidfun_test_protocol::RigidWorldResultRecord,
) -> &Phase10Observation {
    phase10_observations(result)
        .into_iter()
        .next()
        .expect("case result contains Phase 10 state")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn policy_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    for calibration in phase10_policy_calibrations() {
        bytes.extend_from_slice(calibration.path.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(format!("{:?}", calibration.policy).as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(calibration.justification.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(calibration.boundary_test.as_bytes());
        bytes.push(b'\n');
    }
    bytes
}

fn leaf_bytes() -> Vec<u8> {
    let mut bytes = Vec::new();
    for leaf in required_phase10_evidence_leaves() {
        bytes.extend_from_slice(leaf_id(&leaf).as_bytes());
        bytes.push(b'\n');
    }
    bytes
}

fn manifest_payload_bytes(manifest: &CorpusManifest) -> Vec<u8> {
    let mut bytes = format!(
        "{}\0{}\0{}\0{}\0{}\0{}\n",
        manifest.schema_version,
        manifest.profile,
        manifest.upstream_revision,
        manifest.policy_count,
        manifest.policy_sha256,
        manifest.leaf_sha256,
    )
    .into_bytes();
    bytes.extend_from_slice(manifest.retained_phase9_manifest.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(manifest.retained_phase9_manifest_sha256.as_bytes());
    bytes.push(b'\n');
    for case in &manifest.cases {
        bytes.extend_from_slice(case.case_id.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(case.fixture.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(case.fixture_sha256.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(case.request_sha256.as_bytes());
        bytes.push(b'\n');
        for leaf in &case.leaves {
            bytes.extend_from_slice(leaf.as_bytes());
            bytes.push(b'\n');
        }
    }
    bytes
}

fn leaf_id(leaf: &Phase10EvidenceLeaf) -> String {
    match leaf {
        Phase10EvidenceLeaf::Phase10 { behavior } => {
            let value = serde_json::to_value(behavior).expect("behavior serializes");
            format!("phase10/{}", value.as_str().expect("behavior is a string"))
        }
        Phase10EvidenceLeaf::Inherited { branch_id } => {
            format!("inherited/{}", branch_id.as_str())
        }
    }
}
