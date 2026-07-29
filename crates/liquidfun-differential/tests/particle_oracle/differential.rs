#[test]
fn differential_runner_hashes_one_request_for_native_and_cpp_roles() {
    // Arrange
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: build oracle-debug to exercise the Phase 9 differential runner");
        return;
    };
    let request = coupling_request();

    // Act
    let run = run_phase9_differential(&executable, &request, REVISION)
        .expect("the native and pinned C++ Phase 9 results should compare");

    // Assert
    assert_eq!(run.native_request_sha256(), run.request_sha256());
    assert_eq!(run.oracle_request_sha256(), run.request_sha256());
    assert_eq!(run.consumed_paths(), PHASE9_REQUIRED_POLICY_PATHS);
    assert!(
        matches!(run.outcome(), Phase9ComparisonOutcome::Match { .. }),
        "unexpected Phase 9 differential outcome: {:?}",
        run.outcome()
    );
}

#[test]
fn differential_comparison_rejects_a_deterministic_semantic_mutation() {
    // Arrange
    let request = full_phase9_request();
    let native =
        NativeRigidWorldExecutor::execute(&request).expect("native Phase 9 request should execute");
    let mut mutated_value = serde_json::to_value(&native).expect("result should serialize");
    let statistics = mutated_value["timelines"][0]["checkpoints"]
        .as_array_mut()
        .expect("checkpoints should be an array")
        .iter_mut()
        .filter_map(|checkpoint| {
            checkpoint
                .get_mut("observations")
                .and_then(Value::as_array_mut)
        })
        .flatten()
        .find(|observation| {
            observation["kind"] == "particle" && observation["observation"]["kind"] == "statistics"
        })
        .expect("statistics observation should exist");
    statistics["observation"]["statistics"]["particle_contact_count"] = json!(1);
    let mut bytes = serde_json::to_vec(&mutated_value).expect("mutation should encode");
    bytes.push(b'\n');
    let mutated = liquidfun_test_protocol::decode_rigid_world_result_jsonl(
        &bytes,
        &HarnessLimits::phase2_default_v1(),
    )
    .expect("mutation should remain bounded");

    // Act
    let first = compare_phase9_rigid_world_results(&request, &native, &mutated)
        .expect("semantic disagreement should not be a harness failure");
    let second = compare_phase9_rigid_world_results(&request, &native, &mutated)
        .expect("replay should preserve mismatch classification");

    // Assert
    let (
        Phase9ComparisonOutcome::PhysicsMismatch(first),
        Phase9ComparisonOutcome::PhysicsMismatch(second),
    ) = (first, second)
    else {
        panic!("the deterministic mutation must be a physics mismatch");
    };
    assert_eq!(first.semantic_path(), "particle.statistics.counts");
    assert_eq!(first.signature_sha256(), second.signature_sha256());
}

#[test]
fn differential_runner_keeps_malformed_child_output_as_harness_failure() {
    // Arrange
    let fake = FakeOracleRoot::new("malformed");
    let executable = OracleExecutable::resolve(fake.path(), OraclePreset::Debug)
        .expect("fake oracle should occupy the reviewed preset path");
    let request = full_phase9_request();

    // Act
    let result = run_phase9_differential(&executable, &request, REVISION);

    // Assert
    let error = result.expect_err("malformed child output must fail");
    assert!(matches!(error, Phase9DifferentialError::Oracle(_)));
}

struct FakeOracleRoot {
    root: PathBuf,
}

impl FakeOracleRoot {
    fn new(behavior: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should follow the Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "liquidfun-phase9-oracle-{}-{nonce}",
            std::process::id()
        ));
        let preset = root.join("target/reference/oracle-debug");
        fs::create_dir_all(&preset).expect("fake preset directory should be created");
        let destination = preset.join(if cfg!(windows) {
            "liquidfun-reference.exe"
        } else {
            "liquidfun-reference"
        });
        fs::copy(env!("CARGO_BIN_EXE_liquidfun-fake-oracle"), &destination)
            .expect("fake oracle should copy into the reviewed path");
        fs::write(preset.join("behavior.txt"), behavior)
            .expect("fake oracle behavior should be written");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for FakeOracleRoot {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).expect("fake oracle root should be removable");
    }
}
