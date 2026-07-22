use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::TestResult;
pub(super) use super::workspace_root;

const ROLES: [&str; 4] = ["debug", "release", "replay", "sanitizer"];
const SOURCE: &str = "crates/liquidfun-differential/tests/fixtures/catalog";

pub(super) struct TestRoot {
    pub(super) path: PathBuf,
}

impl TestRoot {
    pub(super) fn new(label: &str) -> TestResult<Self> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = workspace_root()
            .join("target")
            .join(format!("phase11-evidence-cli-{label}-{nonce}"));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    pub(super) fn relative(&self, child: &str) -> String {
        self.path
            .join(child)
            .strip_prefix(workspace_root())
            .expect("test paths remain beneath the workspace")
            .to_string_lossy()
            .into_owned()
    }

    pub(super) fn write_local_pair(&self) -> TestResult {
        write_directory(&self.path.join("canonical"), "phase11-canonical-local")?;
        write_directory(&self.path.join("sanitizer"), "phase11-sanitizer-local")
    }

    pub(super) fn run_local(&self) -> std::io::Result<Output> {
        run_xtask(&[
            "phase11-evidence",
            "validate",
            "--mode",
            "local",
            "--canonical-dir",
            &self.relative("canonical"),
            "--sanitizer-dir",
            &self.relative("sanitizer"),
        ])
    }

    pub(super) fn mutate_record(&self, role: &str, mutate: impl FnOnce(&mut Value)) -> TestResult {
        let root = self.path.join("canonical");
        let path = root.join(format!("{role}.jsonl"));
        let text = fs::read_to_string(&path)?;
        let mut lines = text.lines().map(str::to_owned).collect::<Vec<_>>();
        let mut record: Value = serde_json::from_str(&lines[0])?;
        mutate(&mut record);
        lines[0] = serde_json::to_string(&record)?;
        fs::write(&path, format!("{}\n", lines.join("\n")))?;
        refresh_identity(&root)
    }

    pub(super) fn mutate_payload(&self, mutate: impl FnOnce(&mut Value)) -> TestResult {
        let root = self.path.join("canonical");
        let manifest_path = root.join("phase11-v1.json");
        let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;
        let relative = manifest["payloads"][0]["path"]
            .as_str()
            .expect("payload path")
            .to_owned();
        let filename = Path::new(&relative).file_name().expect("payload filename");
        let payload_path = root.join("cases").join(filename);
        let mut payload: Value = serde_json::from_slice(&fs::read(&payload_path)?)?;
        mutate(&mut payload);
        let bytes = serde_json::to_vec(&payload)?;
        fs::write(&payload_path, &bytes)?;
        let digest = sha256(&bytes);
        manifest["payloads"][0]["sha256"] = json!(digest);
        manifest["cases"][0]["payload_sha256"] = json!(digest);
        write_json(&manifest_path, &manifest)?;
        refresh_identity(&root)
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(super) fn write_directory(root: &Path, local_name: &str) -> TestResult {
    fs::create_dir_all(root.join("cases"))?;
    let source = workspace_root().join(SOURCE);
    fs::copy(source.join("phase11-v1.json"), root.join("phase11-v1.json"))?;
    for entry in fs::read_dir(source.join("cases"))? {
        let entry = entry?;
        fs::copy(entry.path(), root.join("cases").join(entry.file_name()))?;
    }
    for role in ROLES {
        let output = run_xtask(&["phase11-evidence", "render-records", role])?;
        assert_success(&output);
        fs::write(root.join(format!("{role}.jsonl")), output.stdout)?;
    }
    let relative = root
        .strip_prefix(workspace_root())?
        .to_string_lossy()
        .into_owned();
    let output = run_xtask(&[
        "phase11-evidence",
        "validate-content",
        if local_name.contains("sanitizer") {
            "sanitizer"
        } else {
            "canonical"
        },
        &relative,
    ])?;
    assert_success(&output);
    let stdout = String::from_utf8(output.stdout)?;
    let semantic_sha256 = stdout
        .split("semantic-sha256=")
        .nth(1)
        .map(str::trim)
        .ok_or("validator did not report semantic digest")?;
    write_identity(root, local_name, semantic_sha256)
}

fn write_identity(root: &Path, name: &str, semantic_sha256: &str) -> TestResult {
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
            "protocol_version": "catalog-phase11-v1",
            "generator_version": "phase11-evidence-v1",
            "semantic_sha256": semantic_sha256,
            "files": inventory(root)?,
        }),
    )
}

pub(super) fn refresh_identity(root: &Path) -> TestResult {
    let path = root.join("identity.json");
    let mut identity: Value = serde_json::from_slice(&fs::read(&path)?)?;
    identity["files"] = json!(inventory(root)?);
    write_json(&path, &identity)
}

pub(super) fn inventory(root: &Path) -> TestResult<Vec<Value>> {
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

pub(super) fn write_json(path: &Path, value: &impl Serialize) -> TestResult {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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
