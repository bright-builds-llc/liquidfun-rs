fn fake_repository(behavior: &str) -> PathBuf {
    let id = TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
    let root = repository_root()
        .join("target/round-trip-tests")
        .join(format!("{}-{id}", std::process::id()));
    for preset in ["oracle-debug", "oracle-asan-ubsan"] {
        let oracle_output = root.join("target/reference").join(preset);
        fs::create_dir_all(&oracle_output).expect("fake output should be creatable");
        let executable = oracle_output.join(if cfg!(windows) {
            "liquidfun-reference.exe"
        } else {
            "liquidfun-reference"
        });
        fs::copy(env!("CARGO_BIN_EXE_liquidfun-fake-oracle"), executable)
            .expect("fake oracle should copy");
        fs::write(oracle_output.join("behavior.txt"), behavior).expect("behavior should write");
    }

    let fixture_output = root.join("protocol/fixtures/accepted");
    fs::create_dir_all(&fixture_output).expect("fixture output should be creatable");
    fs::copy(
        repository_root().join("protocol/fixtures/accepted/empty-world-request.jsonl"),
        fixture_output.join("empty-world-request.jsonl"),
    )
    .expect("request fixture should copy");
    root
}

fn run_cli(root: &Path, behavior: &str, arguments: &[&str]) -> std::process::Output {
    run_cli_with_root(root, behavior, arguments).0
}

fn run_cli_with_root(
    root: &Path,
    behavior: &str,
    arguments: &[&str],
) -> (std::process::Output, PathBuf) {
    let fake_root = fake_repository(behavior);
    assert!(root == repository_root() || root == fake_root);
    let output = Command::new(env!("CARGO_BIN_EXE_liquidfun-differential"))
        .current_dir(&fake_root)
        .args(arguments)
        .output()
        .expect("differential CLI should run");
    (output, fake_root)
}

#[test]
fn real_oracle_one_shot_and_two_request_reuse_match_or_skip_explicitly() {
    // Arrange
    let root = repository_root();
    if OracleExecutable::resolve(&root, OraclePreset::Debug).is_err() {
        eprintln!(
            "SKIP real oracle integration prerequisite: run cargo xtask upstream configure/build --preset oracle-debug"
        );
        return;
    }

    // Act
    let one_shot = run_named(
        &root,
        "empty-world",
        OraclePreset::Debug,
        SessionProfile::OneShot,
        REVISION,
    )
    .expect("real one-shot orchestration should run");
    let reuse = run_named(
        &root,
        "empty-world",
        OraclePreset::Debug,
        SessionProfile::Reuse,
        REVISION,
    )
    .expect("real reuse orchestration should run");

    // Assert
    assert!(matches!(one_shot, DifferentialRunOutcome::Match(_)));
    let DifferentialRunOutcome::Match(reused) = reuse else {
        panic!("real reuse should match");
    };
    assert_eq!(reused.requests().len(), 2);
    assert_eq!(reused.requests()[0].cpp_reset_epoch(), 1);
    assert_eq!(reused.requests()[1].cpp_reset_epoch(), 2);
}

#[test]
fn real_oracle_rejects_oversized_stdin_before_waiting_for_a_newline() {
    // Arrange
    let root = repository_root();
    if OracleExecutable::resolve(&root, OraclePreset::Debug).is_err() {
        eprintln!(
            "SKIP real oracle integration prerequisite: run cargo xtask upstream configure/build --preset oracle-debug"
        );
        return;
    }
    let executable = root
        .join("target/reference/oracle-debug")
        .join(if cfg!(windows) {
            "liquidfun-reference.exe"
        } else {
            "liquidfun-reference"
        });
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("real oracle should start");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout should be piped"));
    let mut handshake = String::new();
    stdout
        .read_line(&mut handshake)
        .expect("oracle handshake should be readable");
    let mut stdin = child.stdin.take().expect("stdin should be piped");
    let oversized = vec![b' '; HarnessLimits::phase2_default_v1().input_record_bytes() + 1];

    // Act
    let write_result = stdin.write_all(&oversized).and_then(|()| stdin.flush());
    let deadline = Instant::now() + Duration::from_secs(2);
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .expect("oracle status should be observable")
        {
            break status;
        }
        if Instant::now() >= deadline {
            drop(stdin);
            child.kill().expect("stalled oracle should be killed");
            child.wait().expect("killed oracle should be reaped");
            panic!("oracle waited for the oversized record remainder");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    drop(stdin);
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr should be piped")
        .read_to_string(&mut stderr)
        .expect("oracle stderr should be readable");

    // Assert
    assert!(write_result.is_ok());
    assert!(!status.success());
    assert!(stderr.contains("input record exceeds reviewed byte limit"));
}

#[test]
fn real_oracle_rejects_invalid_query_child_without_poisoning_process() {
    // Arrange
    let Some(executable) = real_oracle_path(OraclePreset::Debug) else {
        eprintln!(
            "SKIP real oracle integration prerequisite: run cargo xtask upstream configure/build --preset oracle-debug"
        );
        return;
    };
    let request_bytes =
        fs::read(repository_root().join("protocol/fixtures/accepted/rigid-world-request.jsonl"))
            .expect("rigid-world request should be readable");
    let mut request: serde_json::Value =
        serde_json::from_slice(&request_bytes).expect("rigid-world request should be JSON");
    let query_timeline = request["scenario"]["timelines"]
        .as_array_mut()
        .expect("timelines should be an array")
        .iter_mut()
        .find(|timeline| timeline["witness_family"] == "world_query_and_ray_cast")
        .expect("query timeline should exist");
    let terminate_query = query_timeline["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .iter_mut()
        .find(|action| action["action_id"] == "query-07")
        .expect("terminating query should exist");
    terminate_query["action"]["directive_rules"][0]["target"]["child_index"] = serde_json::json!(1);
    let request = encode_jsonl(
        &request,
        &HarnessLimits::phase2_default_v1(),
        RecordLimit::Input,
    )
    .expect("mutated request should encode");
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("real oracle should start");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout should be piped"));
    let mut handshake = String::new();
    stdout
        .read_line(&mut handshake)
        .expect("oracle handshake should be readable");
    let mut stdin = child.stdin.take().expect("stdin should be piped");

    // Act
    stdin
        .write_all(&request)
        .and_then(|()| stdin.flush())
        .expect("invalid query request should write");
    drop(stdin);
    let mut result_records = String::new();
    stdout
        .read_to_string(&mut result_records)
        .expect("oracle result stream should be readable");
    let output = child.wait_with_output().expect("oracle should be reaped");

    // Assert
    assert!(
        output.status.success(),
        "a rejected request must not poison the reusable oracle process"
    );
    assert!(result_records.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("query directive references invalid fixture child")
    );
}
