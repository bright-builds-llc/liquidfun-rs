use std::{
    env, fs,
    io::Write,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

use super::{TestResult, workspace_root};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[path = "canonical.rs"]
pub(crate) mod canonical;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(1);

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn new() -> TestResult<Self> {
        let path = env::temp_dir().join(format!(
            "liquidfun-phase13-1-gap-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        if path.exists() {
            return Err("test fixture path already exists".into());
        }
        fs::create_dir(&path)?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        if self.0.exists() {
            fs::remove_dir_all(&self.0).expect("owned test fixture should be removable");
        }
    }
}

struct Fixture {
    _temporary: TemporaryDirectory,
    repository: PathBuf,
    retained: PathBuf,
    manifest: PathBuf,
    evidence: PathBuf,
}

fn hash_file(path: &Path) -> TestResult<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}

fn hash_canonical_value(value: &Value) -> TestResult<String> {
    let mut canonical = Command::new("jq")
        .args(["-cS", "."])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    canonical
        .stdin
        .take()
        .ok_or("jq stdin must be piped")?
        .write_all(&serde_json::to_vec(value)?)?;
    let output = canonical.wait_with_output()?;
    if !output.status.success() {
        return Err("jq failed to canonicalize fixture JSON".into());
    }
    Ok(format!("{:x}", Sha256::digest(output.stdout)))
}

fn run_git(repository: &Path, arguments: &[&str]) -> TestResult<String> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(repository)
        .output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned().into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn write_json(path: &Path, value: &Value) -> TestResult {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

impl Fixture {
    // Keeping the complete trust-chain fixture in one constructor makes each
    // mutation test start from the same auditable evidence graph.
    #[allow(clippy::too_many_lines)]
    fn new() -> TestResult<Self> {
        let temporary = TemporaryDirectory::new()?;
        let repository = temporary.path().join("repository");
        let retained = temporary.path().join("retained");
        fs::create_dir_all(&repository)?;
        fs::create_dir_all(retained.join("logs"))?;
        fs::create_dir_all(retained.join("canonical/logs"))?;
        let retained = fs::canonicalize(retained)?;
        run_git(&repository, &["init", "-q", "-b", "main"])?;
        run_git(
            &repository,
            &["config", "user.email", "fixture@example.invalid"],
        )?;
        run_git(&repository, &["config", "user.name", "Fixture"])?;
        fs::write(repository.join("source.txt"), "base\n")?;
        run_git(&repository, &["add", "source.txt"])?;
        run_git(&repository, &["commit", "-q", "-m", "base"])?;
        let structural_parent = run_git(&repository, &["rev-parse", "HEAD"])?;
        fs::write(repository.join("source.txt"), "candidate\n")?;
        run_git(&repository, &["add", "source.txt"])?;
        run_git(&repository, &["commit", "-q", "-m", "candidate"])?;
        let candidate = run_git(&repository, &["rev-parse", "HEAD"])?;
        let tree = run_git(&repository, &["rev-parse", "HEAD^{tree}"])?;

        for name in ["one.stdout", "one.stderr", "two.stdout", "two.stderr"] {
            fs::write(retained.join("logs").join(name), format!("{name}\n"))?;
        }
        fs::write(
            retained.join("logs/dispatch.stdout"),
            "https://github.com/fixture/repository/actions/runs/7\n",
        )?;
        fs::write(retained.join("logs/dispatch.stderr"), "")?;
        write_json(
            &retained.join("logs/initial.stdout"),
            &json!({"databaseId":7,"headSha":candidate,"event":"workflow_dispatch","status":"queued","conclusion":null,"url":"https://github.com/fixture/repository/actions/runs/7"}),
        )?;
        fs::write(retained.join("logs/initial.stderr"), "")?;
        write_json(
            &retained.join("logs/terminal.stdout"),
            &json!({"databaseId":7,"headSha":candidate,"event":"workflow_dispatch","status":"completed","conclusion":"success","url":"https://github.com/fixture/repository/actions/runs/7"}),
        )?;
        fs::write(retained.join("logs/terminal.stderr"), "")?;
        canonical::write_bundle(&retained, &candidate, &tree)?;

        let commands = json!([
            {"id":"one","argv":["true"],"environment":{},"stdout_log":"logs/one.stdout","stderr_log":"logs/one.stderr","evidence_class":"fixture"},
            {"id":"canonical-dispatch","argv":["gh","workflow","run","phase13-1-canonical-native.yml","--ref","main","-f",format!("candidate_sha={candidate}")],"environment":{},"stdout_log":"logs/dispatch.stdout","stderr_log":"logs/dispatch.stderr","evidence_class":"canonical-d1"},
            {"id":"canonical-initial-view","argv":["gh","run","view","7","--json","databaseId,headSha,event,status,conclusion,url"],"environment":{},"stdout_log":"logs/initial.stdout","stderr_log":"logs/initial.stderr","evidence_class":"canonical-d1"},
            {"id":"canonical-inspect","argv":["gh","run","view","7","--json","databaseId,headSha,event,status,conclusion,url"],"environment":{},"stdout_log":"logs/terminal.stdout","stderr_log":"logs/terminal.stderr","evidence_class":"canonical-d1"},
            {"id":"two","argv":["true","two"],"environment":{"FIXTURE":"yes"},"stdout_log":"logs/two.stdout","stderr_log":"logs/two.stderr","evidence_class":"fixture"}
        ]);
        let manifest_json = json!({
            "schema": "phase13-1-gap-verification-manifest-v1",
            "test_fixture": true,
            "allowed_placeholders": ["CANDIDATE", "CANDIDATE_TREE", "OUTPUT_ROOT", "REMOTE_REF", "CANONICAL_RUN_ID"],
            "structural_source": {"commit": candidate, "parent": structural_parent},
            "deferred_xtask_targets": ["phase13_acceptance_contract"],
            "artifacts": [{"id":"canonical-native","identity":"canonical/identity.json","logs":"canonical/logs","evidence_tier":"D1"}],
            "commands": commands
        });
        let manifest = temporary.path().join("manifest.json");
        write_json(&manifest, &manifest_json)?;
        let evidence_commands = manifest_json["commands"]
            .as_array()
            .ok_or("fixture commands must be an array")?
            .iter()
            .map(|command| {
                let mut record = command.clone();
                let object = record
                    .as_object_mut()
                    .ok_or("fixture command must be an object")?;
                object.insert("exit_code".into(), json!(0));
                for stream in ["stdout", "stderr"] {
                    let relative = command[format!("{stream}_log")]
                        .as_str()
                        .ok_or("fixture log path must be a string")?;
                    object.insert(
                        format!("{stream}_sha256"),
                        json!(hash_file(&retained.join(relative))?),
                    );
                }
                Ok(record)
            })
            .collect::<TestResult<Vec<_>>>()?;
        let dispatch_record = evidence_commands
            .iter()
            .find(|command| command["id"] == "canonical-dispatch")
            .ok_or("dispatch record must exist")?;
        let intent_json = json!({
            "schema":"phase13-1-dispatch-intent-v1",
            "candidate_sha":candidate,
            "candidate_tree":tree,
            "repository_slug":"fixture/repository",
            "workflow_file":"phase13-1-canonical-native.yml",
            "remote_ref":"main",
            "dispatch_argv":dispatch_record["argv"],
            "manifest_sha256":hash_file(&manifest)?
        });
        write_json(&retained.join("dispatch-intent.json"), &intent_json)?;
        let result_json = json!({
            "schema":"phase13-1-dispatch-result-v1",
            "candidate_sha":candidate,
            "candidate_tree":tree,
            "repository_slug":"fixture/repository",
            "workflow_file":"phase13-1-canonical-native.yml",
            "remote_ref":"main",
            "intent_sha256":hash_file(&retained.join("dispatch-intent.json"))?,
            "dispatch_url":"https://github.com/fixture/repository/actions/runs/7",
            "canonical_run_id":"7",
            "command_id":"canonical-dispatch",
            "command_record_sha256":hash_canonical_value(dispatch_record)?,
            "command_stdout_sha256":hash_file(&retained.join("logs/dispatch.stdout"))?
        });
        write_json(&retained.join("dispatch-result.json"), &result_json)?;
        let evidence_json = json!({
            "schema": "phase13-1-gap-verification-evidence-v1",
            "candidate_sha": candidate,
            "candidate_tree": tree,
            "output_root": retained.to_string_lossy(),
            "remote_ref": "main",
            "repository_slug": "fixture/repository",
            "canonical_run_id": "7",
            "manifest_sha256": hash_file(&manifest)?,
            "dispatch_intent": {"path":"dispatch-intent.json","sha256":hash_file(&retained.join("dispatch-intent.json"))?},
            "dispatch_result": {"path":"dispatch-result.json","sha256":hash_file(&retained.join("dispatch-result.json"))?},
            "commands": evidence_commands,
            "checker": {"findings": 0, "exceptions": 0},
            "canonical_identity": {"path":"canonical/identity.json","sha256":hash_file(&retained.join("canonical/identity.json"))?},
            "complete": true
        });
        let evidence = retained.join("final-verification.json.pending");
        write_json(&evidence, &evidence_json)?;
        Ok(Self {
            _temporary: temporary,
            repository,
            retained,
            manifest,
            evidence,
        })
    }

    fn run(&self) -> TestResult<Output> {
        Ok(
            Command::new(workspace_root().join("scripts/phase13-1-validate-gap-evidence.sh"))
                .arg(&self.manifest)
                .arg(&self.evidence)
                .arg(&self.repository)
                .arg(&self.retained)
                .output()?,
        )
    }

    fn mutate_evidence(&self, mutation: impl FnOnce(&mut Value)) -> TestResult {
        let mut evidence: Value = serde_json::from_slice(&fs::read(&self.evidence)?)?;
        mutation(&mut evidence);
        write_json(&self.evidence, &evidence)
    }

    fn assert_rejected(&self) -> TestResult {
        let output = self.run()?;
        assert!(
            !output.status.success(),
            "validator unexpectedly accepted mutated evidence"
        );
        Ok(())
    }
}

#[test]
fn evidence_validator_accepts_complete_candidate_bound_evidence() -> TestResult {
    let fixture = Fixture::new()?;
    let output = fixture.run()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn evidence_validator_rejects_failed_command_publication() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.mutate_evidence(|evidence| evidence["commands"][0]["exit_code"] = json!(1))?;
    fixture.assert_rejected()
}

#[test]
fn evidence_validator_rejects_missing_log() -> TestResult {
    let fixture = Fixture::new()?;
    fs::remove_file(fixture.retained.join("logs/one.stdout"))?;
    fixture.assert_rejected()
}

#[test]
fn evidence_validator_rejects_digest_mismatch() -> TestResult {
    let fixture = Fixture::new()?;
    fs::write(fixture.retained.join("logs/one.stdout"), "mutated\n")?;
    fixture.assert_rejected()
}

#[test]
fn evidence_validator_rejects_command_order_drift() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.mutate_evidence(|evidence| {
        if let Some(commands) = evidence["commands"].as_array_mut() {
            commands.swap(0, 1);
        }
    })?;
    fixture.assert_rejected()
}

#[test]
fn evidence_validator_rejects_environment_drift() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.mutate_evidence(|evidence| {
        evidence["commands"][0]["environment"] = json!({"DRIFT":"yes"});
    })?;
    fixture.assert_rejected()
}

#[test]
fn evidence_validator_rejects_candidate_tree_drift() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.mutate_evidence(|evidence| {
        evidence["candidate_tree"] = json!("0000000000000000000000000000000000000000");
    })?;
    fixture.assert_rejected()
}

#[test]
fn evidence_validator_rejects_candidate_commit_drift() -> TestResult {
    let fixture = Fixture::new()?;
    let parent = run_git(&fixture.repository, &["rev-parse", "HEAD^"])?;
    fixture.mutate_evidence(|evidence| evidence["candidate_sha"] = json!(parent))?;
    fixture.assert_rejected()
}

#[test]
fn evidence_validator_rejects_structural_parent_failure() -> TestResult {
    let fixture = Fixture::new()?;
    let mut manifest: Value = serde_json::from_slice(&fs::read(&fixture.manifest)?)?;
    manifest["structural_source"]["parent"] = manifest["structural_source"]["commit"].clone();
    write_json(&fixture.manifest, &manifest)?;
    let manifest_digest = hash_file(&fixture.manifest)?;
    fixture.mutate_evidence(|evidence| evidence["manifest_sha256"] = json!(manifest_digest))?;
    fixture.assert_rejected()
}

#[test]
fn evidence_validator_rejects_symlink_substitution() -> TestResult {
    let fixture = Fixture::new()?;
    let log = fixture.retained.join("logs/one.stdout");
    fs::remove_file(&log)?;
    symlink("../canonical/identity.json", &log)?;
    fixture.assert_rejected()
}

#[test]
fn evidence_validator_rejects_path_escape() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.mutate_evidence(|evidence| {
        evidence["commands"][0]["stdout_log"] = json!("../escape.log");
    })?;
    fixture.assert_rejected()
}

#[test]
fn evidence_validator_rejects_dirty_post_run_tree() -> TestResult {
    let fixture = Fixture::new()?;
    fs::write(fixture.repository.join("dirty.txt"), "dirty\n")?;
    fixture.assert_rejected()
}

#[test]
fn evidence_validator_rejects_false_canonical_d1_identity() -> TestResult {
    let fixture = Fixture::new()?;
    let identity_path = fixture.retained.join("canonical/identity.json");
    let mut identity: Value = serde_json::from_slice(&fs::read(&identity_path)?)?;
    identity["evidence_tier"] = json!("D2");
    write_json(&identity_path, &identity)?;
    let identity_digest = hash_file(&identity_path)?;
    fixture.mutate_evidence(|evidence| {
        evidence["canonical_identity"]["sha256"] = json!(identity_digest);
    })?;
    fixture.assert_rejected()
}

#[test]
fn evidence_validator_rejects_missing_canonical_identity() -> TestResult {
    let fixture = Fixture::new()?;
    fs::remove_file(fixture.retained.join("canonical/identity.json"))?;
    fixture.assert_rejected()
}
