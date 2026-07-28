struct RigidFixtureRepository {
    root: PathBuf,
    oracle_directory: PathBuf,
}

impl RigidFixtureRepository {
    fn new(behavior: &str) -> io::Result<Self> {
        let sequence = NEXT_REPOSITORY.fetch_add(1, Ordering::Relaxed);
        let root = workspace_root().join(format!(
            "target/rigid-fixture-workflow-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("protocol/fixtures/accepted"))?;
        fs::create_dir_all(root.join("protocol/tolerances"))?;
        fs::create_dir_all(root.join("reference/artifacts"))?;
        fs::create_dir_all(root.join("scenarios/regressions"))?;
        fs::copy(
            workspace_root().join("protocol/fixtures/accepted/rigid-world-request.jsonl"),
            root.join("protocol/fixtures/accepted/rigid-world-request.jsonl"),
        )?;
        fs::copy(
            workspace_root().join("protocol/tolerances/phase6-v1.toml"),
            root.join("protocol/tolerances/phase6-v1.toml"),
        )?;
        fs::copy(
            workspace_root().join("protocol/tolerances/phase7-v1.toml"),
            root.join("protocol/tolerances/phase7-v1.toml"),
        )?;
        fs::copy(
            workspace_root().join("protocol/tolerances/phase8-v1.toml"),
            root.join("protocol/tolerances/phase8-v1.toml"),
        )?;
        fs::copy(
            workspace_root().join("reference/artifacts/manifest.toml"),
            root.join("reference/artifacts/manifest.toml"),
        )?;
        fs::write(root.join("THIRD_PARTY_NOTICES.md"), "fixture notices\n")?;
        write_adapter_inputs(&root)?;
        run_git(&root, &["init", "--quiet"])?;
        run_git(&root, &["config", "user.name", "Fixture User"])?;
        run_git(&root, &["config", "user.email", "fixture@example.invalid"])?;
        run_git(&root, &["add", "."])?;
        run_git(&root, &["commit", "--quiet", "-m", "fixture"])?;

        let oracle_directory = root.join("target/reference/oracle-debug");
        fs::create_dir_all(&oracle_directory)?;
        fs::copy(
            env!("CARGO_BIN_EXE_liquidfun-fake-oracle"),
            oracle_directory.join(oracle_name()),
        )?;
        write_compile_database(&root, "-DREVIEWED=1")?;
        fs::write(oracle_directory.join("behavior.txt"), behavior)?;
        Ok(Self {
            root,
            oracle_directory,
        })
    }

    fn stage(&self, artifact_id: &str) -> io::Result<Output> {
        self.command(&[
            "fixture",
            "stage",
            "--scenario",
            "rigid-world",
            "--preset",
            "oracle-debug",
            "--session-profile",
            "one-shot",
            "--artifact-kind",
            "reviewed-trace",
            "--artifact-id",
            artifact_id,
        ])
    }

    fn review(&self, artifact_id: &str) -> io::Result<Output> {
        self.command(&[
            "fixture",
            "review",
            "--artifact-id",
            artifact_id,
            "--reviewer",
            "fixture-reviewer",
            "--reviewed-at",
            "2026-07-12T12:00:00Z",
            "--review-status",
            "approved",
        ])
    }

    fn promote(&self, artifact_id: &str) -> io::Result<Output> {
        self.command(&["fixture", "promote", "--artifact-id", artifact_id])
    }

    fn command(&self, arguments: &[&str]) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_liquidfun-differential"))
            .current_dir(&self.root)
            .args(arguments)
            .output()
    }

    fn set_behavior(&self, behavior: &str) -> io::Result<()> {
        fs::write(self.oracle_directory.join("behavior.txt"), behavior)
    }

    fn set_request_policy_hash(&self, hash: &str) -> io::Result<()> {
        let path = self
            .root
            .join("protocol/fixtures/accepted/rigid-world-request.jsonl");
        let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
        value["tolerance_profile_sha256"] = serde_json::Value::String(hash.to_owned());
        let mut bytes = serde_json::to_vec(&value)?;
        bytes.push(b'\n');
        fs::write(path, bytes)
    }

    fn candidate(&self, artifact_id: &str) -> PathBuf {
        self.root
            .join("target/differential/staging")
            .join(artifact_id)
    }

    fn adapter_input(&self) -> PathBuf {
        self.root.join("tools/reference/src/fixture_adapter.hpp")
    }

    fn compile_database(&self) -> PathBuf {
        self.oracle_directory.join("compile_commands.json")
    }
}

#[derive(Debug, PartialEq, Eq)]
struct FixtureMutationSnapshot {
    staging: Vec<(PathBuf, Vec<u8>)>,
    traces: Vec<(PathBuf, Vec<u8>)>,
    regressions: Vec<(PathBuf, Vec<u8>)>,
    manifest: Vec<u8>,
}

impl FixtureMutationSnapshot {
    fn capture(repository: &RigidFixtureRepository) -> io::Result<Self> {
        Ok(Self {
            staging: snapshot_tree(&repository.root.join("target/differential/staging"))?,
            traces: snapshot_tree(&repository.root.join("reference/artifacts/traces"))?,
            regressions: snapshot_tree(&repository.root.join("scenarios/regressions"))?,
            manifest: fs::read(repository.root.join("reference/artifacts/manifest.toml"))?,
        })
    }
}

impl Drop for RigidFixtureRepository {
    fn drop(&mut self) {
        let _ignored = fs::remove_dir_all(&self.root);
    }
}
