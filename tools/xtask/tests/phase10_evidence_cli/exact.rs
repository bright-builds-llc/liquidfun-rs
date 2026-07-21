use std::{fs, path::Path, process::Command};

use serde_json::{Value, json};

use super::support::{
    TestResult, TestRoot, file_inventory, run_xtask, sha256, workspace_root, write_directory,
    write_json,
};

pub(super) const EXACT_RUN: u64 = 31_000_000_001;
pub(super) const CANONICAL_ARTIFACT: u64 = 3101;
pub(super) const SANITIZER_ARTIFACT: u64 = 3102;
pub(super) const APPROVED_SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

impl TestRoot {
    pub(super) fn write_exact_pair(&self) -> TestResult<Value> {
        let canonical = self.path.join("canonical");
        let sanitizer = self.path.join("sanitizer");
        write_directory(&canonical, "phase10-canonical-local")?;
        write_directory(&sanitizer, "phase10-sanitizer-local")?;
        write_exact_identity(
            &canonical,
            "Phase 10 canonical Linux oracle",
            CANONICAL_ARTIFACT,
            &artifact_name("canonical"),
        )?;
        write_exact_identity(
            &sanitizer,
            "Phase 10 fail-fast sanitizer",
            SANITIZER_ARTIFACT,
            &artifact_name("sanitizer"),
        )?;
        let canonical_archive = self.path.join("canonical.zip");
        let sanitizer_archive = self.path.join("sanitizer.zip");
        write_zip(&canonical, &canonical_archive)?;
        write_zip(&sanitizer, &sanitizer_archive)?;
        let canonical_bytes = fs::read(&canonical_archive)?;
        let sanitizer_bytes = fs::read(&sanitizer_archive)?;
        Ok(json!({
            "repository": "bright-builds-llc/liquidfun-rs",
            "branch": "main",
            "approved_sha": APPROVED_SHA,
            "head_sha": APPROVED_SHA,
            "run_id": EXACT_RUN,
            "workflow_name": "Oracle CI",
            "event": "workflow_dispatch",
            "conclusion": "success",
            "run_url": "https://example.invalid/runs/31000000001",
            "dispatched_at": "2026-07-21T10:59:00Z",
            "created_at": "2026-07-21T11:00:00Z",
            "updated_at": "2026-07-21T11:59:00Z",
            "captured_at": "2026-07-21T12:00:00Z",
            "platform": "linux-x86_64",
            "rust_version": "1.97.0",
            "clang_version": "22.1.8",
            "upstream_revision": "7f20402173fd143a3988c921bc384459c6a858f2",
            "protocol_version": "rigid-world-phase10-v1",
            "generator_version": "phase10-corpus-v1",
            "jobs": {
                "canonical": {"id": 2101, "name": "Phase 10 canonical Linux oracle", "url": "https://example.invalid/jobs/2101", "conclusion": "success"},
                "sanitizer": {"id": 2102, "name": "Phase 10 fail-fast sanitizer", "url": "https://example.invalid/jobs/2102", "conclusion": "success"}
            },
            "artifacts": {
                "canonical": artifact(CANONICAL_ARTIFACT, "canonical", &canonical_archive, &canonical_bytes),
                "sanitizer": artifact(SANITIZER_ARTIFACT, "sanitizer", &sanitizer_archive, &sanitizer_bytes)
            },
            "live_run": {"id": EXACT_RUN, "head_sha": APPROVED_SHA, "name": "Oracle CI", "event": "workflow_dispatch", "conclusion": "success"},
            "live_jobs": [
                {"id": 2101, "name": "Phase 10 canonical Linux oracle", "conclusion": "success"},
                {"id": 2102, "name": "Phase 10 fail-fast sanitizer", "conclusion": "success"}
            ],
            "live_artifacts": [
                {"id": CANONICAL_ARTIFACT, "name": artifact_name("canonical"), "digest": format!("sha256:{}", sha256(&canonical_bytes)), "expired": false},
                {"id": SANITIZER_ARTIFACT, "name": artifact_name("sanitizer"), "digest": format!("sha256:{}", sha256(&sanitizer_bytes)), "expired": false}
            ]
        }))
    }

    pub(super) fn write_run(&self, run: &Value) -> TestResult {
        write_json(&self.path.join("run.json"), run)
    }

    pub(super) fn run_exact_ref(
        &self,
        denied: &[(&str, u64)],
    ) -> std::io::Result<std::process::Output> {
        let mut arguments = vec![
            "phase10-evidence".to_owned(),
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
            arguments.push((*option).to_owned());
            arguments.push(id.to_string());
        }
        let borrowed = arguments.iter().map(String::as_str).collect::<Vec<_>>();
        run_xtask(&borrowed)
    }

    pub(super) fn add_extra_archive_entry(&self, run: &mut Value) -> TestResult {
        fs::write(self.path.join("unexpected.txt"), b"unexpected")?;
        let archive = self.path.join("canonical.zip");
        let status = Command::new("zip")
            .args(["-q", "canonical.zip", "unexpected.txt"])
            .current_dir(&self.path)
            .status()?;
        if !status.success() {
            return Err("zip archive mutation failed".into());
        }
        let bytes = fs::read(archive)?;
        let digest = format!("sha256:{}", sha256(&bytes));
        run["artifacts"]["canonical"]["digest"] = json!(digest);
        run["artifacts"]["canonical"]["size_in_bytes"] = json!(bytes.len());
        run["live_artifacts"][0]["digest"] = json!(digest);
        Ok(())
    }
}

fn write_exact_identity(
    root: &Path,
    job_name: &str,
    artifact_id: u64,
    artifact_name: &str,
) -> TestResult {
    let path = root.join("identity.json");
    let mut identity: Value = serde_json::from_slice(&fs::read(&path)?)?;
    identity["mode"] = json!("exact-ref");
    identity["run_id"] = json!(EXACT_RUN);
    identity["head_sha"] = json!(APPROVED_SHA);
    identity["job_name"] = json!(job_name);
    identity["artifact_id"] = json!(artifact_id);
    identity["artifact_name"] = json!(artifact_name);
    identity["platform"] = json!("linux-x86_64");
    identity["rust_version"] = json!("1.97.0");
    identity["clang_version"] = json!("22.1.8");
    identity["files"] = json!(file_inventory(root)?);
    write_json(&path, &identity)
}

fn write_zip(root: &Path, archive: &Path) -> TestResult {
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

fn artifact_name(kind: &str) -> String {
    format!("phase10-{kind}-{EXACT_RUN}-{APPROVED_SHA}")
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
