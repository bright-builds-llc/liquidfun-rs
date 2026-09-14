use super::*;
use serde::Deserialize;
use std::process::Stdio;
use std::time::{Duration, Instant};

#[path = "claims.rs"]
mod claims;

const SOURCE: &str = "reference/release/source-candidate.json";
const MANIFEST: &str = "reference/release/candidate-manifest.json";
const REPORT: &str = "reference/release/audit-report.json";

#[derive(Deserialize)]
struct RequiredRegistry {
    evidence: Vec<RequiredEvidence>,
}

#[derive(Deserialize)]
struct RequiredEvidence {
    kind: String,
    target: String,
    workflow: String,
    job: String,
    toolchain: String,
}

pub(super) struct Checkout {
    pub(super) root: PathBuf,
    pub(super) candidate: String,
}

impl Checkout {
    pub(super) fn new(name: &str) -> Self {
        let ordinal = TEST_ORDINAL.fetch_add(1, Ordering::Relaxed);
        let root = repository_root().join(format!(
            "target/xtask-attestation-tests/{name}-{}-{ordinal}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("fixture root");
        for relative in [
            "Cargo.toml",
            "deny.toml",
            "tools/xtask/Cargo.toml",
            "README.md",
            "COMPATIBILITY.md",
            "RELEASE.md",
            "BENCHMARKING.md",
            "LICENSE",
            "THIRD_PARTY_NOTICES.md",
            "CONTRIBUTING.md",
            "SAFETY.md",
            "ARCHITECTURE.md",
            "TESTING.md",
            "crates/liquidfun/src/lib.rs",
            "protocol/fixtures/accepted/rigid-world-request.jsonl",
            "reference/release/required-evidence.toml",
            "reference/platform/support.json",
            "reference/performance/manifest.toml",
            "reference/coverage/contract.json",
            "reference/regressions/manifest.toml",
            "scenarios/regressions/phase14-particle-groups.txt",
            "reference/upstream-corpus.json",
            "reference/compatibility.json",
        ] {
            let path = root.join(relative);
            fs::create_dir_all(path.parent().expect("file parent")).expect("parents");
            fs::copy(repository_root().join(relative), path).expect("source fixture");
        }
        for relative in ["README.md", "COMPATIBILITY.md", "RELEASE.md"] {
            let path = root.join(relative);
            let contents = fs::read_to_string(&path).expect("public document");
            fs::write(path, maturity::non_ready_copy(&contents))
                .expect("explicit non-ready fixture");
        }
        fs::write(root.join(".gitignore"), "target/\n").expect("ignore retained evidence");
        git_output(&root, &["init", "--quiet"]);
        git_output(&root, &["config", "user.name", "Attestation Test"]);
        git_output(
            &root,
            &["config", "user.email", "attestation@example.invalid"],
        );
        git_output(&root, &["add", "."]);
        git_output(
            &root,
            &[
                "-c",
                "core.hooksPath=/dev/null",
                "commit",
                "--quiet",
                "-m",
                "source C",
            ],
        );
        let candidate = git_output(&root, &["rev-parse", "HEAD"]);
        Self { root, candidate }
    }

    pub(super) fn materialize(&self) {
        let retained = self.root.join("target/retained");
        fs::create_dir_all(&retained).expect("retained directory");
        let archive = retained.join("liquidfun.crate");
        fs::write(&archive, b"reviewed package archive\n").expect("package fixture");
        let package_sha256 = sha256(&fs::read(&archive).expect("archive"));
        let required: RequiredRegistry = toml::from_str(
            &fs::read_to_string(self.root.join("reference/release/required-evidence.toml"))
                .expect("registry"),
        )
        .expect("registry TOML");
        let mut items = Vec::new();
        for (index, evidence) in required.evidence.iter().enumerate() {
            let claims = claims::claims_for(&self.root, evidence, &archive, &package_sha256);
            let payload_sha256 = sha256(&serde_json::to_vec(&claims).expect("claims JSON"));
            let artifact = json!({"schema_version": 1, "kind": evidence.kind, "target": evidence.target,
                "candidate_commit": self.candidate, "status": "passed", "payload_sha256": payload_sha256, "claims": claims});
            let relative = format!("target/retained/artifact-{index}.json");
            write_json(&self.root.join(&relative), &artifact);
            items.push(json!({"kind": evidence.kind, "target": evidence.target, "candidate_commit": self.candidate,
                "producer": {"workflow": evidence.workflow, "job": evidence.job, "run_id": (index + 1).to_string()},
                "artifact_path": relative, "artifact_sha256": file_sha256(&self.root, &relative),
                "payload_sha256": payload_sha256, "toolchain": evidence.toolchain,
                "review_status": "reviewed", "status": "passed"}));
        }
        write_json(
            &self.root.join(MANIFEST),
            &json!({"schema_version": 1, "candidate_commit": self.candidate, "items": items}),
        );
        let output = self.command(&[
            "release",
            "audit",
            "--manifest",
            MANIFEST,
            "--candidate",
            &self.candidate,
            "--output",
            "json",
        ]);
        assert_pass(&output);
        fs::write(self.root.join(REPORT), output.stdout).expect("ready report");
        let tree = bounded(Command::new("git").current_dir(&self.root).args([
            "ls-tree",
            "-r",
            "-z",
            "--full-tree",
            &self.candidate,
        ]));
        assert_pass(&tree);
        write_json(
            &self.root.join(SOURCE),
            &json!({"schema_version": 1, "ready": true,
            "source_candidate_commit": self.candidate, "source_tree_sha256": sha256(&tree.stdout),
            "candidate_manifest_sha256": file_sha256(&self.root, MANIFEST), "audit_report_sha256": file_sha256(&self.root, REPORT)}),
        );
    }

    pub(super) fn attest(&self) -> String {
        self.commit(&[SOURCE, MANIFEST, REPORT], "attestation A")
    }

    pub(super) fn commit(&self, paths: &[&str], message: &str) -> String {
        let mut args = vec!["add", "--"];
        args.extend_from_slice(paths);
        git_output(&self.root, &args);
        git_output(
            &self.root,
            &[
                "-c",
                "core.hooksPath=/dev/null",
                "commit",
                "--quiet",
                "-m",
                message,
            ],
        );
        git_output(&self.root, &["rev-parse", "HEAD"])
    }

    pub(super) fn validate(&self, maybe_attestation: Option<&str>) -> Output {
        let mut args = vec![
            "release",
            "attestation",
            if maybe_attestation.is_some() {
                "validate"
            } else {
                "validate-worktree"
            },
            "--source",
            SOURCE,
            "--manifest",
            MANIFEST,
            "--report",
            REPORT,
        ];
        if let Some(attestation) = maybe_attestation {
            args.extend(["--attestation-commit", attestation]);
        }
        self.command(&args)
    }

    pub(super) fn command(&self, args: &[&str]) -> Output {
        bounded(
            Command::new(env!("CARGO_BIN_EXE_xtask"))
                .current_dir(&self.root)
                .env("LIQUIDFUN_XTASK_ROOT", &self.root)
                .args(args),
        )
    }

    pub(super) fn project(&self, attestation: &str) {
        for relative in ["README.md", "COMPATIBILITY.md", "RELEASE.md"] {
            let path = self.root.join(relative);
            let contents = fs::read_to_string(&path)
                .expect("document")
                .replace("not release-ready", "release-ready")
                .lines()
                .filter(|line| {
                    !line.starts_with("Status:")
                        && !line.starts_with("Source candidate:")
                        && !line.starts_with("Attestation commit:")
                })
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(path, format!("{contents}\nStatus: **release-ready**\nSource candidate: `{}`\nAttestation commit: `{attestation}`\n", self.candidate)).expect("projection");
        }
    }
}

pub(super) fn assert_pass(output: &Output) {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn file_sha256(root: &Path, relative: &str) -> String {
    sha256(&fs::read(root.join(relative)).expect("hash input"))
}

pub(super) fn bounded(command: &mut Command) -> Output {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn fixture command");
    let deadline = Instant::now() + Duration::from_secs(30);
    while child.try_wait().expect("poll child").is_none() {
        if Instant::now() >= deadline {
            child.kill().expect("stop timed out command");
            child.wait().expect("reap child");
            panic!("fixture command exceeded 30 seconds");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output().expect("fixture output")
}
