//! Closed corpus and corruption coverage for Phase 10 evidence.

#[path = "phase10_corpus/evidence_output.rs"]
mod evidence_output;

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    path::{Component, Path, PathBuf},
};

use liquidfun_differential::{
    NativeRigidWorldExecutor, OracleExecutable, OraclePreset, PHASE10_REQUIRED_POLICY_PATHS,
    Phase10ComparisonMode, Phase10ComparisonOutcome, Phase10EvidenceBinding, Phase10EvidenceLeaf,
    Phase10EvidencePayloads, Phase10EvidenceTestRefs, Phase10EvidenceWitnessRef,
    compare_phase10_observations, execute_rigid_world_process, phase10_policy_calibrations,
    required_phase10_evidence_leaves, validate_phase10_evidence_contract,
};
use liquidfun_test_protocol::{
    FloatBits, HarnessLimits, Phase10Observation, RecordLimit, RigidWorldObservation, ScenarioId,
    WitnessRole, decode_rigid_world_request_jsonl, encode_jsonl,
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const BASE_REQUEST: &[u8] =
    include_bytes!("../../../protocol/fixtures/accepted/rigid-world-request.jsonl");
const MANIFEST: &str = include_str!("fixtures/rigid_world/phase10/phase10-v1.json");
const UPSTREAM_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";

fn scenario_id(value: &str) -> ScenarioId {
    ScenarioId::new(value).expect("test identity is valid")
}

fn payloads(case_id: &str) -> Phase10EvidencePayloads {
    let path = |role: &str| format!("cases/{case_id}/proofs/{role}.json").into_boxed_str();
    Phase10EvidencePayloads {
        native: path("native"),
        oracle: path("oracle"),
        comparison: path("comparison"),
        replay_native: path("replay-native"),
        replay_oracle: path("replay-oracle"),
        debug_oracle: path("debug-oracle"),
        release_oracle: path("release-oracle"),
        minimized: path("minimized"),
        copied: path("copied"),
        inherited: path("inherited"),
    }
}

fn witness(role: WitnessRole, index: usize) -> Phase10EvidenceWitnessRef {
    Phase10EvidenceWitnessRef {
        role,
        action_index: index,
        checkpoint_index: 0,
        observation_index: index,
    }
}

fn complete_contract() -> (
    Vec<Phase10EvidenceBinding>,
    HashMap<ScenarioId, (usize, usize, usize)>,
) {
    let case_id = scenario_id("group-construction-and-mutation");
    let bindings = required_phase10_evidence_leaves()
        .into_iter()
        .enumerate()
        .map(|(index, leaf)| Phase10EvidenceBinding {
            leaf,
            case_id: case_id.clone(),
            implementation: "crates/liquidfun/src/particle/solver.rs:phase10_semantics".into(),
            tests: Phase10EvidenceTestRefs {
                focused: format!("crates/liquidfun/tests/particle_solver_flags.rs:focused_{index}")
                    .into_boxed_str(),
                integration: format!(
                    "crates/liquidfun-differential/tests/phase10_native.rs:integration_{index}"
                )
                .into_boxed_str(),
                property: format!(
                    "crates/liquidfun/tests/particle_group_properties.rs:property_{index}"
                )
                .into_boxed_str(),
            },
            control: witness(WitnessRole::Control, index * 3),
            activation: witness(WitnessRole::Activation, index * 3 + 1),
            maybe_interaction: Some(witness(WitnessRole::Interaction, index * 3 + 2)),
            observation_path: "phase10.witness.kind".into(),
            policy_path: "phase10.witness.kind".into(),
            payloads: payloads(case_id.as_str()),
        })
        .collect::<Vec<_>>();
    let bounds = HashMap::from([(case_id, (bindings.len() * 3, 1, bindings.len() * 3))]);
    (bindings, bounds)
}

#[test]
fn evidence_contract_accepts_exact_closed_leaf_inventory() {
    // Arrange
    let (bindings, bounds) = complete_contract();

    // Act
    let result = validate_phase10_evidence_contract(&bindings, &bounds);

    // Assert
    assert!(result.is_ok());
    assert_eq!(bindings.len(), 80);
}

#[test]
fn evidence_contract_rejects_missing_duplicate_and_unknown_leaves() {
    // Arrange
    let (bindings, bounds) = complete_contract();
    let mut missing = bindings.clone();
    missing.pop();
    let mut duplicate = bindings.clone();
    duplicate[1].leaf = duplicate[0].leaf.clone();
    let mut unknown = bindings.clone();
    unknown[0].leaf = Phase10EvidenceLeaf::Inherited {
        branch_id: scenario_id("unreviewed-branch"),
    };

    // Act / Assert
    assert!(validate_phase10_evidence_contract(&missing, &bounds).is_err());
    assert!(validate_phase10_evidence_contract(&duplicate, &bounds).is_err());
    assert!(validate_phase10_evidence_contract(&unknown, &bounds).is_err());
}

#[test]
fn evidence_contract_rejects_aliases_ranges_private_passes_and_open_paths() {
    // Arrange
    let (bindings, bounds) = complete_contract();
    let mut aliased = bindings.clone();
    aliased[0].payloads.oracle = aliased[0].payloads.native.clone();
    let mut out_of_range = bindings.clone();
    out_of_range[0].activation.action_index = usize::MAX;
    let mut private_pass = bindings.clone();
    private_pass[0].implementation = "crates/liquidfun/src/particle/PassId.rs".into();
    let mut wildcard = bindings.clone();
    wildcard[0].observation_path = "phase10.*".into();

    // Act / Assert
    assert!(validate_phase10_evidence_contract(&aliased, &bounds).is_err());
    assert!(validate_phase10_evidence_contract(&out_of_range, &bounds).is_err());
    assert!(validate_phase10_evidence_contract(&private_pass, &bounds).is_err());
    assert!(validate_phase10_evidence_contract(&wildcard, &bounds).is_err());
}

#[test]
fn evidence_contract_rejects_metadata_repair_after_semantic_substitution() {
    // Arrange
    let (mut bindings, bounds) = complete_contract();
    bindings[0].activation.role = WitnessRole::Control;
    bindings[0].payloads = payloads(bindings[0].case_id.as_str());

    // Act
    let result = validate_phase10_evidence_contract(&bindings, &bounds);

    // Assert
    assert!(result.is_err());
}

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
    json!({ "kind": "particle_group", "operation": operation })
}

fn action(action_id: &str, action: Value) -> Value {
    json!({ "action_id": action_id, "phase": "phase10", "action": action })
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

#[test]
fn manifest_closes_five_cases_all_leaves_and_named_policy_calibration() {
    // Arrange
    let manifest = corpus_manifest();
    let expected = required_phase10_evidence_leaves()
        .iter()
        .map(leaf_id)
        .collect::<BTreeSet<_>>();

    // Act
    let actual = manifest
        .cases
        .iter()
        .flat_map(|case| case.leaves.iter().cloned())
        .collect::<BTreeSet<_>>();
    let calibrations = phase10_policy_calibrations().collect::<Vec<_>>();

    // Assert
    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.profile, "phase10-v1");
    assert_eq!(manifest.upstream_revision, UPSTREAM_REVISION);
    assert_eq!(manifest.cases.len(), 5);
    assert_eq!(manifest.policy_count, PHASE10_REQUIRED_POLICY_PATHS.len());
    assert_eq!(calibrations.len(), PHASE10_REQUIRED_POLICY_PATHS.len());
    assert!(calibrations.iter().all(|calibration| {
        !calibration.justification.is_empty()
            && calibration
                .boundary_test
                .starts_with("crates/liquidfun-differential/tests/")
    }));
    assert_eq!(manifest.policy_sha256, sha256(&policy_bytes()));
    assert_eq!(manifest.leaf_sha256, sha256(&leaf_bytes()));
    let retained_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rigid_world/phase10")
        .join(&manifest.retained_phase9_manifest);
    let retained = std::fs::read(retained_path).expect("retained Phase 9 manifest is readable");
    assert_eq!(manifest.retained_phase9_manifest_sha256, sha256(&retained));
    assert_eq!(
        manifest.manifest_payload_sha256,
        sha256(&manifest_payload_bytes(&manifest))
    );
    assert_eq!(actual, expected);
    assert_eq!(actual.len(), 80);
    assert_eq!(
        manifest
            .cases
            .iter()
            .map(|case| case.leaves.len())
            .sum::<usize>(),
        actual.len(),
        "a leaf may appear in exactly one case"
    );
}

#[test]
fn corpus_request_digests_are_sealed() {
    // Arrange
    let manifest = corpus_manifest();

    // Act
    let actual = manifest
        .cases
        .iter()
        .map(|case| {
            let recipe = recipe(case);
            let (bytes, _) = case_request(&recipe);
            let fixture = std::fs::read(fixture_path(case)).expect("fixture bytes are readable");
            (case.case_id.clone(), sha256(&fixture), sha256(&bytes))
        })
        .collect::<Vec<_>>();
    let expected = manifest
        .cases
        .iter()
        .map(|case| {
            (
                case.case_id.clone(),
                case.fixture_sha256.clone(),
                case.request_sha256.clone(),
            )
        })
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(expected, actual);
}

#[test]
fn normalized_optional_depth_still_rejects_required_lane_disappearance() {
    // Arrange
    let manifest = corpus_manifest();
    let pressure_case = manifest
        .cases
        .iter()
        .find(|case| case.case_id == "pressure-constraints-and-rigid")
        .expect("pressure case is sealed");
    let (_, request) = case_request(&recipe(pressure_case));
    let result = NativeRigidWorldExecutor::execute(&request).expect("native case executes");
    let normalized = phase10_observation(&result).clone();
    let mut required = normalized.clone();
    let Phase10Observation::State { state } = &mut required;
    let group = state.groups.first_mut().expect("case has one group");
    group.maybe_depths_bits =
        Some(vec![FloatBits::from_f32(0.0); group.member_ids.len()].into_boxed_slice());

    // Act
    let outcome =
        compare_phase10_observations(Phase10ComparisonMode::D1Semantic, &required, &normalized)
            .expect("both observations remain semantically valid");

    // Assert
    let Phase10ComparisonOutcome::PhysicsMismatch(mismatch) = outcome else {
        panic!("required optional lane disappearance must fail closed");
    };
    assert_eq!(mismatch.semantic_path(), "phase10.group.depth");
}

#[test]
fn corpus_executes_d0_replay_and_two_engine_debug_release_comparison() {
    // Arrange
    let manifest = corpus_manifest();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let maybe_mode = std::env::var("LIQUIDFUN_PHASE10_ORACLE_MODE").ok();
    let primary_preset = match maybe_mode.as_deref() {
        None | Some("canonical") => OraclePreset::Debug,
        Some("sanitizer") => OraclePreset::AsanUbsan,
        Some(mode) => panic!("unsupported Phase 10 oracle mode {mode}"),
    };
    let resolve = |preset| match OracleExecutable::resolve(&root, preset) {
        Ok(executable) => Some(executable),
        Err(error) if maybe_mode.is_none() => {
            eprintln!("SKIP: build the Phase 10 {preset:?} oracle: {error}");
            None
        }
        Err(error) => panic!("required Phase 10 {preset:?} oracle is unavailable: {error}"),
    };
    let Some(primary) = resolve(primary_preset) else {
        return;
    };
    let Some(debug) = resolve(OraclePreset::Debug) else {
        return;
    };
    let Some(release) = resolve(OraclePreset::Release) else {
        return;
    };

    let expected_phase10_leaves = required_phase10_evidence_leaves()
        .into_iter()
        .filter_map(|leaf| {
            let Phase10EvidenceLeaf::Phase10 { behavior } = leaf else {
                return None;
            };
            Some(behavior)
        })
        .collect::<HashSet<_>>();
    let mut witnessed_phase10_leaves = HashSet::new();
    let mut evidence_cases = Vec::new();

    // Act / Assert
    for case in &manifest.cases {
        let (request_bytes, request) = case_request(&recipe(case));
        let canonical = encode_jsonl(
            &request,
            &HarnessLimits::phase2_default_v1(),
            RecordLimit::Input,
        )
        .expect("validated request re-encodes");
        assert_eq!(
            request_bytes, canonical,
            "{} request authority",
            case.case_id
        );
        let native = NativeRigidWorldExecutor::execute(&request).expect("native case executes");
        let native_replay =
            NativeRigidWorldExecutor::execute(&request).expect("native replay executes");
        let oracle = execute_rigid_world_process(&primary, &request, UPSTREAM_REVISION)
            .expect("selected oracle executes");
        let oracle_replay = execute_rigid_world_process(&primary, &request, UPSTREAM_REVISION)
            .expect("selected oracle replay executes");
        let debug_oracle = execute_rigid_world_process(&debug, &request, UPSTREAM_REVISION)
            .expect("debug oracle executes");
        let optimized = execute_rigid_world_process(&release, &request, UPSTREAM_REVISION)
            .expect("release oracle executes");
        assert_eq!(
            native, native_replay,
            "native D0 differs for {}",
            case.case_id
        );
        assert_eq!(
            oracle.response_bytes(),
            oracle_replay.response_bytes(),
            "oracle D0 differs for {}",
            case.case_id
        );
        assert_eq!(
            oracle.result(),
            debug_oracle.result(),
            "selected and debug modes differ for {}",
            case.case_id
        );
        assert_eq!(
            oracle.result(),
            optimized.result(),
            "build modes differ for {}",
            case.case_id
        );
        let native_observations = phase10_observations(&native);
        let oracle_observations = phase10_observations(oracle.result());
        assert_eq!(native_observations.len(), oracle_observations.len());
        for (native_observation, oracle_observation) in
            native_observations.iter().zip(&oracle_observations)
        {
            let outcome = compare_phase10_observations(
                Phase10ComparisonMode::D1Semantic,
                native_observation,
                oracle_observation,
            )
            .expect("strict observations compare");
            assert!(
                matches!(outcome, Phase10ComparisonOutcome::Match { .. }),
                "{} differs: {outcome:?}",
                case.case_id
            );
            let Phase10Observation::State { state } = native_observation;
            witnessed_phase10_leaves.extend(
                state
                    .witnesses
                    .iter()
                    .filter(|witness| witness.role != WitnessRole::Control)
                    .map(|witness| witness.behavior_leaf),
            );
        }
        evidence_cases.push(evidence_output::capture_case(
            case,
            &request,
            &native,
            &native_replay,
            oracle.result(),
            oracle_replay.result(),
            debug_oracle.result(),
            optimized.result(),
        ));
    }
    assert_eq!(witnessed_phase10_leaves, expected_phase10_leaves);
    evidence_output::write_if_requested(&root, &manifest, &evidence_cases);
}
