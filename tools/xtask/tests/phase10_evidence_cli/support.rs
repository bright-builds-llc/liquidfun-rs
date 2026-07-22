use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use liquidfun_differential::{
    PHASE10_EVIDENCE_SCHEMA_VERSION, PHASE10_REQUIRED_POLICY_PATHS, Phase10EvidenceBinding,
    Phase10EvidencePayloads, Phase10EvidenceTestRefs, Phase10EvidenceWitnessRef,
    required_phase10_evidence_leaves,
};
use liquidfun_test_protocol::{ScenarioId, WitnessRole};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub(super) type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const CASES: [&str; 5] = [
    "group-construction-and-mutation",
    "topology-join-split-reactive",
    "solver-material-flags",
    "pressure-constraints-and-rigid",
    "boundary-order-and-inherited",
];
const ROLES: [&str; 10] = [
    "native",
    "oracle",
    "comparison",
    "replay-native",
    "replay-oracle",
    "debug-oracle",
    "release-oracle",
    "minimized",
    "copied",
    "inherited",
];

pub(super) struct TestRoot {
    pub(super) path: PathBuf,
}

impl TestRoot {
    pub(super) fn new(label: &str) -> TestResult<Self> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = workspace_root()
            .join("target")
            .join(format!("phase10-evidence-cli-{label}-{nonce}"));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    pub(super) fn relative(&self, child: &str) -> String {
        self.path
            .join(child)
            .strip_prefix(workspace_root())
            .expect("test data remains beneath workspace")
            .to_string_lossy()
            .into_owned()
    }

    pub(super) fn write_local_pair(&self) -> TestResult {
        write_directory(&self.path.join("canonical"), "phase10-canonical-local")?;
        write_directory(&self.path.join("sanitizer"), "phase10-sanitizer-local")?;
        Ok(())
    }

    pub(super) fn run_local(&self) -> std::io::Result<Output> {
        run_xtask(&[
            "phase10-evidence",
            "validate",
            "--mode",
            "local",
            "--canonical-dir",
            &self.relative("canonical"),
            "--sanitizer-dir",
            &self.relative("sanitizer"),
        ])
    }

    pub(super) fn mutate_manifest(&self, mutate: impl FnOnce(&mut Value)) -> TestResult {
        let root = self.path.join("canonical");
        let path = root.join("phase10-manifest.json");
        let mut manifest: Value = serde_json::from_slice(&fs::read(&path)?)?;
        mutate(&mut manifest);
        refresh_semantic_digest(&mut manifest)?;
        write_json(&path, &manifest)?;
        refresh_identity(&root)
    }

    pub(super) fn mutate_proof(&self, role: &str, mutate: impl FnOnce(&mut Value)) -> TestResult {
        let root = self.path.join("canonical");
        let manifest_path = root.join("phase10-manifest.json");
        let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;
        let reference = &mut manifest["cases"][0]["proofs"][role];
        let relative = reference["path"].as_str().expect("proof path").to_owned();
        let path = root.join(relative);
        let mut proof: Value = serde_json::from_slice(&fs::read(&path)?)?;
        mutate(&mut proof);
        proof["payload_sha256"] = json!(canonical_sha256(&proof["payload"])?);
        let bytes = serde_json::to_vec_pretty(&proof)?;
        fs::write(&path, &bytes)?;
        reference["sha256"] = json!(sha256(&bytes));
        refresh_semantic_digest(&mut manifest)?;
        write_json(&manifest_path, &manifest)?;
        refresh_identity(&root)
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[derive(Serialize)]
struct Manifest {
    schema_version: u32,
    profile: String,
    upstream_revision: String,
    protocol_version: String,
    generator_version: String,
    fixture_manifest_sha256: String,
    semantic_manifest_sha256: String,
    bindings: Vec<Phase10EvidenceBinding>,
    cases: Vec<EvidenceCase>,
}

#[derive(Serialize)]
struct Semantic<'a> {
    bindings: &'a [Phase10EvidenceBinding],
    cases: &'a [EvidenceCase],
}

#[derive(Serialize)]
struct EvidenceCase {
    case_id: String,
    action_count: usize,
    checkpoint_count: usize,
    observation_count: usize,
    proofs: BTreeMap<String, FileReference>,
}

#[derive(Serialize)]
struct FileReference {
    path: String,
    sha256: String,
}

pub(super) fn write_directory(root: &Path, local_name: &str) -> TestResult {
    fs::create_dir_all(root)?;
    let cases = CASES
        .iter()
        .map(|case_id| write_case(root, case_id))
        .collect::<TestResult<Vec<_>>>()?;
    let bindings = build_bindings()?;
    let semantic_manifest_sha256 = canonical_sha256(&Semantic {
        bindings: &bindings,
        cases: &cases,
    })?;
    let fixture = workspace_root()
        .join("crates/liquidfun-differential/tests/fixtures/rigid_world/phase10/phase10-v1.json");
    let manifest = Manifest {
        schema_version: PHASE10_EVIDENCE_SCHEMA_VERSION,
        profile: "phase10-v1".to_owned(),
        upstream_revision: "7f20402173fd143a3988c921bc384459c6a858f2".to_owned(),
        protocol_version: "rigid-world-phase10-v1".to_owned(),
        generator_version: "phase10-corpus-v1".to_owned(),
        fixture_manifest_sha256: sha256(&fs::read(fixture)?),
        semantic_manifest_sha256,
        bindings,
        cases,
    };
    write_json(&root.join("phase10-manifest.json"), &manifest)?;
    for log in [
        "phase10-trace.log",
        "provenance.log",
        "inventory.log",
        "read-only.log",
    ] {
        fs::write(root.join(log), b"status: ok\n")?;
    }
    write_local_identity(root, local_name)
}

fn write_case(root: &Path, case_id: &str) -> TestResult<EvidenceCase> {
    let mut proofs = BTreeMap::new();
    for role in ROLES {
        let payload_role = match role {
            "replay-native" => "native",
            "replay-oracle" | "debug-oracle" | "release-oracle" => "oracle",
            "copied" => "minimized",
            value => value,
        };
        let payload = json!({"case_id": case_id, "semantic": payload_role});
        let proof = json!({
            "schema_version": 1,
            "case_id": case_id,
            "role": role,
            "outcome": if matches!(role, "minimized" | "copied") { "deliberate-divergence" } else { "match" },
            "payload_sha256": canonical_sha256(&payload)?,
            "payload": payload,
        });
        let relative = format!("cases/{case_id}/proofs/{role}.json");
        let path = root.join(&relative);
        fs::create_dir_all(path.parent().expect("proof parent"))?;
        let bytes = serde_json::to_vec_pretty(&proof)?;
        fs::write(path, &bytes)?;
        proofs.insert(
            role.to_owned(),
            FileReference {
                path: relative,
                sha256: sha256(&bytes),
            },
        );
    }
    Ok(EvidenceCase {
        case_id: case_id.to_owned(),
        action_count: 8,
        checkpoint_count: 8,
        observation_count: 8,
        proofs,
    })
}

fn build_bindings() -> TestResult<Vec<Phase10EvidenceBinding>> {
    required_phase10_evidence_leaves()
        .into_iter()
        .enumerate()
        .map(|(index, leaf)| {
            let case_id = CASES[index % CASES.len()];
            Ok(Phase10EvidenceBinding {
                leaf,
                case_id: ScenarioId::new(case_id)?,
                implementation: "crates/liquidfun/src/particle/solver.rs".into(),
                tests: Phase10EvidenceTestRefs {
                    focused: format!("crates/liquidfun/tests/focused_{index}.rs").into(),
                    integration: format!("crates/liquidfun/tests/integration_{index}.rs").into(),
                    property: format!("crates/liquidfun/tests/property_{index}.rs").into(),
                },
                control: witness(WitnessRole::Control, 0),
                activation: witness(WitnessRole::Activation, 1),
                maybe_interaction: Some(witness(WitnessRole::Interaction, 2)),
                observation_path: format!("phase10.leaf.{index}").into(),
                policy_path: PHASE10_REQUIRED_POLICY_PATHS
                    [index % PHASE10_REQUIRED_POLICY_PATHS.len()]
                .into(),
                payloads: payloads(case_id),
            })
        })
        .collect()
}

fn witness(role: WitnessRole, index: usize) -> Phase10EvidenceWitnessRef {
    Phase10EvidenceWitnessRef {
        role,
        action_index: index,
        checkpoint_index: index,
        observation_index: index,
    }
}

fn payloads(case_id: &str) -> Phase10EvidencePayloads {
    Phase10EvidencePayloads {
        native: format!("cases/{case_id}/proofs/native.json").into(),
        oracle: format!("cases/{case_id}/proofs/oracle.json").into(),
        comparison: format!("cases/{case_id}/proofs/comparison.json").into(),
        replay_native: format!("cases/{case_id}/proofs/replay-native.json").into(),
        replay_oracle: format!("cases/{case_id}/proofs/replay-oracle.json").into(),
        debug_oracle: format!("cases/{case_id}/proofs/debug-oracle.json").into(),
        release_oracle: format!("cases/{case_id}/proofs/release-oracle.json").into(),
        minimized: format!("cases/{case_id}/proofs/minimized.json").into(),
        copied: format!("cases/{case_id}/proofs/copied.json").into(),
        inherited: format!("cases/{case_id}/proofs/inherited.json").into(),
    }
}

fn write_local_identity(root: &Path, name: &str) -> TestResult {
    write_json(
        &root.join("identity.json"),
        &json!({
            "schema_version": 1,
            "mode": "local",
            "run_id": 0,
            "head_sha": "local",
            "job_name": name,
            "artifact_id": 0,
            "artifact_name": name,
            "platform": "local",
            "rust_version": "local",
            "clang_version": "local",
            "upstream_revision": "7f20402173fd143a3988c921bc384459c6a858f2",
            "protocol_version": "rigid-world-phase10-v1",
            "generator_version": "phase10-corpus-v1",
            "semantic_manifest_sha256": serde_json::from_slice::<Value>(&fs::read(root.join("phase10-manifest.json"))?)?["semantic_manifest_sha256"],
            "files": file_inventory(root)?,
        }),
    )
}

pub(super) fn refresh_identity(root: &Path) -> TestResult {
    let path = root.join("identity.json");
    let mut identity: Value = serde_json::from_slice(&fs::read(&path)?)?;
    identity["files"] = json!(file_inventory(root)?);
    identity["semantic_manifest_sha256"] = serde_json::from_slice::<Value>(&fs::read(
        root.join("phase10-manifest.json"),
    )?)?["semantic_manifest_sha256"]
        .clone();
    write_json(&path, &identity)
}

pub(super) fn file_inventory(root: &Path) -> TestResult<Vec<Value>> {
    let mut files = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            if entry.path().is_dir() {
                pending.push(entry.path());
            } else if entry.file_name() != "identity.json" {
                let relative = entry
                    .path()
                    .strip_prefix(root)?
                    .to_string_lossy()
                    .into_owned();
                files.push(json!({"path": relative, "sha256": sha256(&fs::read(entry.path())?)}));
            }
        }
    }
    files.sort_by(|left, right| left["path"].as_str().cmp(&right["path"].as_str()));
    Ok(files)
}

fn refresh_semantic_digest(manifest: &mut Value) -> TestResult {
    let semantic = json!({
        "bindings": manifest["bindings"].clone(),
        "cases": manifest["cases"].clone(),
    });
    manifest["semantic_manifest_sha256"] = json!(canonical_sha256(&semantic)?);
    Ok(())
}

pub(super) fn write_json(path: &Path, value: &impl Serialize) -> TestResult {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

fn canonical_sha256(value: &impl Serialize) -> TestResult<String> {
    Ok(sha256(&serde_json::to_vec(value)?))
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("xtask is two levels below workspace")
        .to_path_buf()
}

pub(super) fn run_xtask(args: &[&str]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(workspace_root())
        .output()
}

pub(super) fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub(super) fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
}
