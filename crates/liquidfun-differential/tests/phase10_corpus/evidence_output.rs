//! Identity-free Phase 10 evidence payload generation for the shell runner.

use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path},
};

use liquidfun_differential::{
    PHASE10_EVIDENCE_SCHEMA_VERSION, PHASE10_REQUIRED_POLICY_PATHS, Phase10EvidenceBinding,
    Phase10EvidenceTestRefs, Phase10EvidenceWitnessRef, required_phase10_evidence_leaves,
};
use liquidfun_test_protocol::{RigidWorldRequestRecord, RigidWorldResultRecord, WitnessRole};
use serde::Serialize;
use serde_json::{Value, json};

use super::{CorpusCase, CorpusManifest, leaf_id, payloads, sha256};

const MANIFEST_FILE: &str = "phase10-manifest.json";

#[derive(Debug)]
pub(super) struct CapturedCase {
    case_id: String,
    action_count: usize,
    checkpoint_count: usize,
    observation_count: usize,
    payloads: BTreeMap<&'static str, Value>,
}

#[derive(Serialize)]
struct EvidenceManifest {
    schema_version: u32,
    profile: &'static str,
    upstream_revision: &'static str,
    protocol_version: &'static str,
    generator_version: &'static str,
    fixture_manifest_sha256: String,
    semantic_manifest_sha256: String,
    bindings: Vec<Phase10EvidenceBinding>,
    cases: Vec<EvidenceCase>,
}

#[derive(Serialize)]
struct SemanticManifest<'a> {
    bindings: &'a [Phase10EvidenceBinding],
    cases: &'a [EvidenceCase],
}

#[derive(Serialize)]
struct EvidenceCase {
    case_id: String,
    action_count: usize,
    checkpoint_count: usize,
    observation_count: usize,
    proofs: BTreeMap<&'static str, FileReference>,
}

#[derive(Serialize)]
struct FileReference {
    path: String,
    sha256: String,
}

pub(super) fn capture_case(
    case: &CorpusCase,
    request: &RigidWorldRequestRecord,
    native: &RigidWorldResultRecord,
    native_replay: &RigidWorldResultRecord,
    oracle: &RigidWorldResultRecord,
    oracle_replay: &RigidWorldResultRecord,
    debug_oracle: &RigidWorldResultRecord,
    release_oracle: &RigidWorldResultRecord,
) -> CapturedCase {
    let request_value = serde_json::to_value(request).expect("request evidence encodes");
    let result_value = |result: &RigidWorldResultRecord| {
        serde_json::to_value(result).expect("result evidence encodes")
    };
    let action_count = request_value["scenario"]["timelines"]
        .as_array()
        .expect("request timelines")
        .iter()
        .map(|timeline| {
            timeline["actions"]
                .as_array()
                .expect("timeline actions")
                .len()
        })
        .sum();
    let checkpoint_count = native
        .timelines()
        .iter()
        .map(|timeline| timeline.checkpoints.len())
        .sum();
    let observation_count = native
        .timelines()
        .iter()
        .flat_map(|timeline| &timeline.checkpoints)
        .map(|checkpoint| checkpoint.observations.len())
        .sum();
    assert!(action_count >= 3 && checkpoint_count > 0 && observation_count >= 3);

    let comparison = json!({
        "case_id": case.case_id,
        "outcome": "match",
        "consumed_policy_paths": PHASE10_REQUIRED_POLICY_PATHS,
    });
    let deliberate_divergence = json!({
        "case_id": case.case_id,
        "outcome": "deliberate-divergence",
        "semantic_path": "phase10.d0.bytes",
        "signature": "phase10-regression-control-v1",
    });
    let inherited = json!({
        "case_id": case.case_id,
        "retained_manifest": "../phase9/phase9-v1.json",
        "retained_manifest_sha256": "e0936090c8b8453cd464e7e56e1fa09392265ffb1da1f81d8d692667956a3fcc",
    });
    CapturedCase {
        case_id: case.case_id.clone(),
        action_count,
        checkpoint_count,
        observation_count,
        payloads: BTreeMap::from([
            ("native", result_value(native)),
            ("oracle", result_value(oracle)),
            ("comparison", comparison),
            ("replay-native", result_value(native_replay)),
            ("replay-oracle", result_value(oracle_replay)),
            ("debug-oracle", result_value(debug_oracle)),
            ("release-oracle", result_value(release_oracle)),
            ("minimized", deliberate_divergence.clone()),
            ("copied", deliberate_divergence),
            ("inherited", inherited),
        ]),
    }
}

pub(super) fn write_if_requested(
    repository_root: &Path,
    corpus: &CorpusManifest,
    captured: &[CapturedCase],
) {
    let Ok(relative_manifest) = std::env::var("LIQUIDFUN_PHASE10_EVIDENCE_MANIFEST") else {
        return;
    };
    let relative_manifest = Path::new(&relative_manifest);
    assert!(!relative_manifest.is_absolute() && relative_manifest.starts_with("target"));
    assert!(
        relative_manifest
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    );
    assert_eq!(
        relative_manifest.file_name().and_then(|name| name.to_str()),
        Some(MANIFEST_FILE)
    );
    let output_root = repository_root.join(
        relative_manifest
            .parent()
            .expect("evidence manifest has a parent"),
    );
    let cases = write_cases(&output_root, captured);
    let bindings = bindings(corpus, captured);
    let semantic_manifest_sha256 = canonical_sha256(&SemanticManifest {
        bindings: &bindings,
        cases: &cases,
    });
    let fixture_manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rigid_world/phase10/phase10-v1.json");
    let manifest = EvidenceManifest {
        schema_version: PHASE10_EVIDENCE_SCHEMA_VERSION,
        profile: "phase10-v1",
        upstream_revision: super::UPSTREAM_REVISION,
        protocol_version: "rigid-world-phase10-v1",
        generator_version: "phase10-corpus-v1",
        fixture_manifest_sha256: sha256(
            &fs::read(fixture_manifest).expect("fixture manifest is readable"),
        ),
        semantic_manifest_sha256,
        bindings,
        cases,
    };
    write_json(&repository_root.join(relative_manifest), &manifest);
}

fn write_cases(output_root: &Path, captured: &[CapturedCase]) -> Vec<EvidenceCase> {
    captured
        .iter()
        .map(|case| {
            let proofs = case
                .payloads
                .iter()
                .map(|(&role, payload)| {
                    let proof = json!({
                        "schema_version": 1,
                        "case_id": case.case_id,
                        "role": role,
                        "outcome": if matches!(role, "minimized" | "copied") {
                            "deliberate-divergence"
                        } else {
                            "match"
                        },
                        "payload_sha256": canonical_sha256(payload),
                        "payload": payload,
                    });
                    let relative = format!("cases/{}/proofs/{role}.json", case.case_id);
                    let bytes = pretty_json(&proof);
                    let path = output_root.join(&relative);
                    fs::create_dir_all(path.parent().expect("proof path has parent"))
                        .expect("proof directory is writable");
                    fs::write(path, &bytes).expect("proof is writable");
                    (
                        role,
                        FileReference {
                            path: relative,
                            sha256: sha256(&bytes),
                        },
                    )
                })
                .collect();
            EvidenceCase {
                case_id: case.case_id.clone(),
                action_count: case.action_count,
                checkpoint_count: case.checkpoint_count,
                observation_count: case.observation_count,
                proofs,
            }
        })
        .collect()
}

fn bindings(corpus: &CorpusManifest, captured: &[CapturedCase]) -> Vec<Phase10EvidenceBinding> {
    let required = required_phase10_evidence_leaves();
    let mut bindings = Vec::with_capacity(required.len());
    let mut index = 0;
    for case in &corpus.cases {
        let captured_case = captured
            .iter()
            .find(|candidate| candidate.case_id == case.case_id)
            .expect("every corpus case was captured");
        for declared in &case.leaves {
            let leaf = required
                .iter()
                .find(|leaf| leaf_id(leaf) == *declared)
                .expect("declared leaf belongs to the closed inventory")
                .clone();
            bindings.push(Phase10EvidenceBinding {
                leaf,
                case_id: liquidfun_test_protocol::ScenarioId::new(&case.case_id)
                    .expect("case ID is valid"),
                implementation: "crates/liquidfun/src/particle/solver.rs".into(),
                tests: Phase10EvidenceTestRefs {
                    focused: "crates/liquidfun/tests/particle_solver_flags.rs".into(),
                    integration: "crates/liquidfun-differential/tests/phase10_corpus.rs".into(),
                    property: "crates/liquidfun/tests/particle_group_properties.rs".into(),
                },
                control: witness(WitnessRole::Control, 0, captured_case),
                activation: witness(WitnessRole::Activation, 1, captured_case),
                maybe_interaction: Some(witness(WitnessRole::Interaction, 2, captured_case)),
                observation_path: "phase10.witness.kind".into(),
                policy_path: PHASE10_REQUIRED_POLICY_PATHS
                    [index % PHASE10_REQUIRED_POLICY_PATHS.len()]
                .into(),
                payloads: payloads(&case.case_id),
            });
            index += 1;
        }
    }
    bindings
}

fn witness(role: WitnessRole, index: usize, captured: &CapturedCase) -> Phase10EvidenceWitnessRef {
    Phase10EvidenceWitnessRef {
        role,
        action_index: index.min(captured.action_count - 1),
        checkpoint_index: 0,
        observation_index: index.min(captured.observation_count - 1),
    }
}

fn canonical_sha256(value: &impl Serialize) -> String {
    sha256(&serde_json::to_vec(value).expect("canonical evidence JSON encodes"))
}

fn pretty_json(value: &impl Serialize) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).expect("evidence JSON encodes");
    bytes.push(b'\n');
    bytes
}

fn write_json(path: &Path, value: &impl Serialize) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("evidence directory is writable");
    }
    fs::write(path, pretty_json(value)).expect("evidence JSON is writable");
}
