use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};

use super::TestResult;
use super::support::{
    TestRoot, assert_failure, assert_success, inventory, run_xtask, sha256, workspace_root,
    write_directory, write_json,
};

const RUN_ID: u64 = 32_000_000_001;
const CANONICAL_ARTIFACT: u64 = 3201;
const SANITIZER_ARTIFACT: u64 = 3202;
const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

impl TestRoot {
    fn write_exact_pair(&self) -> TestResult<Value> {
        let canonical = self.path.join("canonical");
        let sanitizer = self.path.join("sanitizer");
        write_directory(&canonical, "phase11-canonical-local")?;
        write_directory(&sanitizer, "phase11-sanitizer-local")?;
        exact_identity(
            &canonical,
            "Phase 11 canonical Linux oracle",
            &artifact_name("canonical"),
        )?;
        exact_identity(
            &sanitizer,
            "Phase 11 fail-fast sanitizer",
            &artifact_name("sanitizer"),
        )?;
        let canonical_archive = self.path.join("canonical.zip");
        let sanitizer_archive = self.path.join("sanitizer.zip");
        zip(&canonical, &canonical_archive)?;
        zip(&sanitizer, &sanitizer_archive)?;
        let canonical_bytes = fs::read(&canonical_archive)?;
        let sanitizer_bytes = fs::read(&sanitizer_archive)?;
        Ok(json!({
            "repository": "bright-builds-llc/liquidfun-rs",
            "branch": "main",
            "approved_sha": SHA,
            "head_sha": SHA,
            "run_id": RUN_ID,
            "workflow_name": "Oracle CI",
            "event": "workflow_dispatch",
            "conclusion": "success",
            "run_url": "https://example.invalid/runs/32000000001",
            "dispatched_at": "2026-07-21T10:59:00Z",
            "created_at": "2026-07-21T11:00:00Z",
            "updated_at": "2026-07-21T11:59:00Z",
            "captured_at": "2026-07-21T12:00:00Z",
            "metadata_source": "github-api-live",
            "platform": "linux-x86_64",
            "rust_version": "1.97.0",
            "clang_version": "22.1.8",
            "upstream_revision": "7f20402173fd143a3988c921bc384459c6a858f2",
            "protocol_version": "catalog-phase11-v1",
            "generator_version": "phase11-evidence-v1",
            "jobs": {
                "canonical": job(2201, "Phase 11 canonical Linux oracle"),
                "sanitizer": job(2202, "Phase 11 fail-fast sanitizer")
            },
            "artifacts": {
                "canonical": artifact(CANONICAL_ARTIFACT, "canonical", &canonical_archive, &canonical_bytes),
                "sanitizer": artifact(SANITIZER_ARTIFACT, "sanitizer", &sanitizer_archive, &sanitizer_bytes)
            },
            "live_run": {
                "id": RUN_ID, "head_sha": SHA, "name": "Oracle CI",
                "event": "workflow_dispatch", "conclusion": "success",
                "updated_at": "2026-07-21T11:59:00Z"
            },
            "live_jobs": [
                live_job(2201, "Phase 11 canonical Linux oracle"),
                live_job(2202, "Phase 11 fail-fast sanitizer")
            ],
            "live_artifacts": [
                live_artifact(CANONICAL_ARTIFACT, "canonical", &canonical_bytes),
                live_artifact(SANITIZER_ARTIFACT, "sanitizer", &sanitizer_bytes)
            ]
        }))
    }

    fn write_run(&self, run: &Value) -> TestResult {
        write_json(&self.path.join("run.json"), run)
    }

    fn run_exact(&self, denied: &[(&str, u64)]) -> std::io::Result<std::process::Output> {
        let mut args = vec![
            "phase11-evidence".to_owned(),
            "validate".to_owned(),
            "--mode".to_owned(),
            "exact-ref".to_owned(),
            "--canonical-dir".to_owned(),
            self.relative("canonical"),
            "--sanitizer-dir".to_owned(),
            self.relative("sanitizer"),
            "--run-json".to_owned(),
            self.relative("run.json"),
        ];
        for (option, id) in denied {
            args.push((*option).to_owned());
            args.push(id.to_string());
        }
        let borrowed = args.iter().map(String::as_str).collect::<Vec<_>>();
        run_xtask(&borrowed)
    }

    fn add_extra_archive_entry(&self, run: &mut Value) -> TestResult {
        fs::write(self.path.join("unexpected.txt"), b"unexpected")?;
        let status = Command::new("zip")
            .args(["-q", "canonical.zip", "unexpected.txt"])
            .current_dir(&self.path)
            .status()?;
        if !status.success() {
            return Err("zip archive mutation failed".into());
        }
        let bytes = fs::read(self.path.join("canonical.zip"))?;
        let digest = format!("sha256:{}", sha256(&bytes));
        run["artifacts"]["canonical"]["digest"] = json!(digest);
        run["artifacts"]["canonical"]["size_in_bytes"] = json!(bytes.len());
        run["live_artifacts"][0]["digest"] = json!(digest);
        run["live_artifacts"][0]["size_in_bytes"] = json!(bytes.len());
        Ok(())
    }
}

#[test]
fn exact_ref_accepts_one_live_same_sha_pair() -> TestResult {
    // Arrange
    let root = TestRoot::new("exact-valid")?;
    let run = root.write_exact_pair()?;
    root.write_run(&run)?;

    // Act / Assert
    assert_success(&root.run_exact(&[])?);
    Ok(())
}

#[test]
fn mixed_stale_zero_or_historical_authority_is_rejected() -> TestResult {
    // Arrange / Act / Assert: mixed SHA
    let mixed = TestRoot::new("exact-mixed")?;
    let mut run = mixed.write_exact_pair()?;
    run["jobs"]["canonical"]["head_sha"] = json!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    mixed.write_run(&run)?;
    assert_failure(&mixed.run_exact(&[])?);

    // Arrange / Act / Assert: zero live artifact ID
    let zero = TestRoot::new("exact-zero")?;
    let mut run = zero.write_exact_pair()?;
    run["artifacts"]["canonical"]["id"] = json!(0);
    run["live_artifacts"][0]["id"] = json!(0);
    zero.write_run(&run)?;
    assert_failure(&zero.run_exact(&[])?);

    // Arrange / Act / Assert: explicit historical denysets
    let denied = TestRoot::new("exact-denied")?;
    let run = denied.write_exact_pair()?;
    denied.write_run(&run)?;
    assert_failure(&denied.run_exact(&[("--deny-run-id", RUN_ID)])?);
    assert_failure(&denied.run_exact(&[("--deny-artifact-id", CANONICAL_ARTIFACT)])?);
    Ok(())
}

#[test]
fn archive_topology_is_inspected_and_closed_before_use() -> TestResult {
    // Arrange
    let root = TestRoot::new("exact-archive")?;
    let mut run = root.write_exact_pair()?;
    root.add_extra_archive_entry(&mut run)?;
    root.write_run(&run)?;

    // Act / Assert
    assert_failure(&root.run_exact(&[])?);
    Ok(())
}

fn exact_identity(root: &Path, job_name: &str, artifact_name: &str) -> TestResult {
    let path = root.join("identity.json");
    let mut identity: Value = serde_json::from_slice(&fs::read(&path)?)?;
    identity["mode"] = json!("exact-ref");
    identity["run_id"] = json!(RUN_ID);
    identity["head_sha"] = json!(SHA);
    identity["job_name"] = json!(job_name);
    identity["artifact_id"] = json!(0);
    identity["artifact_name"] = json!(artifact_name);
    identity["platform"] = json!("linux-x86_64");
    identity["rust_version"] = json!("1.97.0");
    identity["clang_version"] = json!("22.1.8");
    identity["files"] = json!(inventory(root)?);
    write_json(&path, &identity)
}

fn zip(root: &Path, archive: &Path) -> TestResult {
    let status = Command::new("zip")
        .args(["-q", "-r"])
        .arg(archive)
        .arg(".")
        .current_dir(root)
        .status()?;
    if !status.success() {
        return Err("zip fixture creation failed".into());
    }
    Ok(())
}

fn job(id: u64, name: &str) -> Value {
    json!({
        "id": id, "name": name, "url": format!("https://example.invalid/jobs/{id}"),
        "conclusion": "success", "head_sha": SHA
    })
}

fn live_job(id: u64, name: &str) -> Value {
    json!({"id": id, "name": name, "conclusion": "success", "head_sha": SHA})
}

fn artifact_name(kind: &str) -> String {
    format!("phase11-{kind}-{RUN_ID}-{SHA}")
}

fn artifact(id: u64, kind: &str, path: &Path, bytes: &[u8]) -> Value {
    json!({
        "id": id,
        "name": artifact_name(kind),
        "api_url": format!("https://example.invalid/artifacts/{id}"),
        "archive_download_url": format!("https://example.invalid/artifacts/{id}/zip"),
        "digest": format!("sha256:{}", sha256(bytes)),
        "size_in_bytes": bytes.len(),
        "expired": false,
        "created_at": "2026-07-21T11:00:00Z",
        "expires_at": "2026-08-21T11:00:00Z",
        "archive_path": path.strip_prefix(workspace_root()).expect("archive under workspace").to_string_lossy()
    })
}

fn live_artifact(id: u64, kind: &str, bytes: &[u8]) -> Value {
    json!({
        "id": id, "name": artifact_name(kind),
        "digest": format!("sha256:{}", sha256(bytes)),
        "size_in_bytes": bytes.len(), "expired": false,
        "created_at": "2026-07-21T11:00:00Z"
    })
}
