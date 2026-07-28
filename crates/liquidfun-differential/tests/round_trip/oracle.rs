fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn real_oracle_path(preset: OraclePreset) -> Option<PathBuf> {
    let root = repository_root();
    let directory = match preset {
        OraclePreset::Debug => "oracle-debug",
        OraclePreset::Release => "oracle-release",
        OraclePreset::AsanUbsan => "oracle-asan-ubsan",
    };
    OracleExecutable::resolve(&root, preset).ok().map(|_| {
        root.join("target/reference")
            .join(directory)
            .join(if cfg!(windows) {
                "liquidfun-reference.exe"
            } else {
                "liquidfun-reference"
            })
    })
}

fn run_cpp_math_probe_twice(
    preset: OraclePreset,
) -> Option<(Vec<MathProbeResult>, Vec<serde_json::Value>)> {
    let maybe_executable = real_oracle_path(preset);
    let Some(executable) = maybe_executable else {
        eprintln!(
            "SKIP real oracle integration prerequisite: run cargo xtask upstream configure/build --preset oracle-debug"
        );
        return None;
    };
    let request_bytes =
        fs::read(repository_root().join("protocol/fixtures/accepted/math-probe-request.jsonl"))
            .expect("math probe request should be readable");
    let request =
        decode_math_probe_request_jsonl(&request_bytes, &HarnessLimits::phase2_default_v1())
            .expect("math probe request should decode");
    let expected = EmptyWorldAdapter::execute_math_probe(&request)
        .expect("native math probes should execute")
        .into_vec();
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("real oracle should start");
    let mut stdin = child.stdin.take().expect("stdin should be piped");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout should be piped"));
    let mut handshake = String::new();
    stdout
        .read_line(&mut handshake)
        .expect("handshake should be readable");
    let mut ends = Vec::new();
    for _ in 0..2 {
        stdin
            .write_all(&request_bytes)
            .and_then(|()| stdin.flush())
            .expect("math request should write");
        let mut actual = Vec::with_capacity(expected.len());
        for _ in 0..expected.len() {
            let mut line = String::new();
            stdout
                .read_line(&mut line)
                .expect("result should be readable");
            actual.push(
                serde_json::from_str::<MathProbeResult>(&line)
                    .expect("C++ result should decode as the Rust contract"),
            );
        }
        for (actual_result, expected_result) in actual.iter().zip(&expected) {
            assert_eq!(actual_result.case_id(), expected_result.case_id());
            assert_eq!(actual_result.operation(), expected_result.operation());
            assert_eq!(actual_result.policy_path(), expected_result.policy_path());
            assert_eq!(actual_result.horizon(), expected_result.horizon());
            assert_eq!(
                actual_result
                    .values()
                    .iter()
                    .map(|value| value.field())
                    .collect::<Vec<_>>(),
                expected_result
                    .values()
                    .iter()
                    .map(|value| value.field())
                    .collect::<Vec<_>>()
            );
            assert_eq!(actual_result.discrete(), expected_result.discrete());
        }
        for witness in [
            "cancellation",
            "halfway-rounding",
            "overflow",
            "underflow",
            "fma-witness",
        ] {
            let actual_witness = actual
                .iter()
                .find(|result| result.case_id() == witness)
                .expect("C++ witness should exist");
            let expected_witness = expected
                .iter()
                .find(|result| result.case_id() == witness)
                .expect("Rust witness should exist");
            assert_eq!(actual_witness.values(), expected_witness.values());
        }
        let mut end = String::new();
        stdout
            .read_line(&mut end)
            .expect("end record should be readable");
        ends.push(serde_json::from_str(&end).expect("end record should be JSON"));
    }
    drop(stdin);
    let output = child.wait_with_output().expect("oracle should be reaped");
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    Some((expected, ends))
}

#[test]
fn cpp_protocol_self_test_executes_phase8_after_plan_08_21() {
    // Arrange
    let root = repository_root();
    if real_oracle_path(OraclePreset::Debug).is_none() {
        eprintln!(
            "SKIP real oracle integration prerequisite: run cargo xtask upstream configure/build --preset oracle-debug"
        );
        return;
    }

    // Act
    let output = Command::new("ctest")
        .args([
            "--test-dir",
            root.join("target/reference/oracle-debug")
                .to_str()
                .expect("repository path should be UTF-8"),
            "--output-on-failure",
            "-R",
            "liquidfun-reference-protocol",
        ])
        .output()
        .expect("C++ protocol self-test should run");

    // Assert
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn cpp_math_probe_matches_operation_contract() {
    // Arrange / Act / Assert
    for preset in [OraclePreset::Debug, OraclePreset::Release] {
        let Some((results, ends)) = run_cpp_math_probe_twice(preset) else {
            assert!(
                std::env::var_os("LIQUIDFUN_DIFFERENTIAL_LEAF_DIRECTORY").is_none(),
                "differential coverage requires both exact C++ math-probe oracle presets"
            );
            return;
        };
        assert_eq!(results.len(), 39);
        assert_eq!(ends[0]["result_count"], 39);
        assert_eq!(ends[0]["reset_epoch"], 1);
        assert_eq!(ends[1]["reset_epoch"], 2);
        assert_eq!(ends[0]["reset_verified"], true);
        assert_eq!(ends[1]["reset_verified"], true);
    }
    coverage_observation::observe(&[
        "public-api.liquidfun-box2d-box2d-common-b2math-h",
        "subsystem.common-math-and-settings",
    ])
    .expect("successful math comparison should emit its covered leaves");
}
