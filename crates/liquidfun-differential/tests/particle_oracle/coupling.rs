#[test]
fn coupling_trace_records_body_contact_and_rigid_reaction() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: build oracle-debug to exercise pinned coupling");
        return;
    };
    let request = coupling_request();

    // Act
    let captured = execute_rigid_world_process(&executable, &request, REVISION)
        .expect("bounded coupling request should execute");
    let value = serde_json::to_value(captured.result()).expect("result should serialize");
    let checkpoint = value["timelines"][0]["checkpoints"]
        .as_array()
        .expect("checkpoints should be an array")
        .iter()
        .find(|checkpoint| checkpoint["checkpoint_id"] == "nc-static-kinematic-rejected")
        .expect("coupling checkpoint should exist");
    let statistics = checkpoint["observations"]
        .as_array()
        .expect("observations should be an array")
        .iter()
        .find(|observation| observation["observation"]["kind"] == "statistics")
        .expect("source statistics should be recorded");
    let dynamic = checkpoint["bodies"]
        .as_array()
        .expect("bodies should be an array")
        .iter()
        .find(|body| body["body_id"] == "nc-dynamic")
        .expect("dynamic coupling body should remain live");

    // Assert
    assert!(statistics["observation"]["statistics"]["body_contact_count"] != 0);
    assert!(
        dynamic["linear_velocity"]["x_bits"] != 0
            || dynamic["linear_velocity"]["y_bits"] != 0
            || dynamic["angular_velocity_bits"] != 0,
        "off-center particle contact should produce a rigid reaction"
    );
}

#[test]
fn coupling_static_body_contact_remains_stationary() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: build oracle-debug to exercise static coupling");
        return;
    };
    let request = static_coupling_request();

    // Act
    let captured = execute_rigid_world_process(&executable, &request, REVISION)
        .expect("static coupling request should execute");
    let value = serde_json::to_value(captured.result()).expect("result should serialize");
    let checkpoint = value["timelines"][0]["checkpoints"]
        .as_array()
        .expect("checkpoints should be an array")
        .iter()
        .find(|checkpoint| checkpoint["checkpoint_id"] == "nc-static-kinematic-rejected")
        .expect("coupling checkpoint should exist");
    let dynamic = checkpoint["bodies"]
        .as_array()
        .expect("bodies should be an array")
        .iter()
        .find(|body| body["body_id"] == "nc-dynamic")
        .expect("static coupling body should remain live");

    // Assert
    assert_eq!(dynamic["body_kind"], "static");
    assert_eq!(dynamic["linear_velocity"]["x_bits"], 0);
    assert_eq!(dynamic["linear_velocity"]["y_bits"], 0);
    assert_eq!(dynamic["angular_velocity_bits"], 0);
}

#[test]
fn every_phase9_action_family_round_trips_with_semantic_observations() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: build oracle-debug to exercise the full particle surface");
        return;
    };
    let request = full_phase9_request();
    let action_count = request.scenario().timelines()[0]
        .actions()
        .iter()
        .filter(|record| record.phase() == "phase9")
        .count();

    // Act
    let captured = execute_rigid_world_process(&executable, &request, REVISION)
        .expect("every Phase 9 action family should execute");
    let value = serde_json::to_value(captured.result()).expect("result should serialize");
    let observations = value["timelines"][0]["checkpoints"]
        .as_array()
        .expect("checkpoints should be an array")
        .iter()
        .flat_map(|checkpoint| checkpoint["observations"].as_array().into_iter().flatten())
        .filter(|observation| observation["kind"] == "particle")
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(observations.len(), action_count);
    for expected in ["mixed_state", "statistics", "query", "ray_cast"] {
        assert!(
            observations
                .iter()
                .any(|observation| observation["observation"]["kind"] == expected),
            "missing semantic observation family {expected}"
        );
    }
}

#[test]
fn long_lived_phase9_process_resets_between_requests() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let executable = root.join("target/reference/oracle-debug/liquidfun-reference");
    if !executable.is_file() {
        eprintln!("SKIP: build oracle-debug to exercise process reuse");
        return;
    }
    let mut request = serde_json::to_vec(&full_phase9_request()).expect("request should encode");
    request.push(b'\n');
    let mut child = Command::new(executable)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("oracle should spawn");
    let mut stdout = BufReader::new(child.stdout.take().expect("stdout should be captured"));
    let mut handshake = String::new();
    stdout
        .read_line(&mut handshake)
        .expect("handshake should be readable");

    // Act
    let mut stdin = child.stdin.take().expect("stdin should be captured");
    stdin
        .write_all(&request)
        .expect("first request should write");
    stdin.flush().expect("first request should flush");
    let mut records = Vec::new();
    for _ in 0..2 {
        let mut record = String::new();
        stdout
            .read_line(&mut record)
            .expect("record should be readable");
        records.push(serde_json::from_str::<Value>(&record).expect("stdout must be JSONL"));
    }
    stdin
        .write_all(&request)
        .expect("second request should write");
    stdin.flush().expect("second request should flush");
    for _ in 0..2 {
        let mut record = String::new();
        stdout
            .read_line(&mut record)
            .expect("record should be readable");
        records.push(serde_json::from_str::<Value>(&record).expect("stdout must be JSONL"));
    }
    drop(stdin);
    let output = child.wait_with_output().expect("oracle should be reaped");

    // Assert
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(records[0]["record_kind"], "rigid_world_result");
    assert_eq!(records[1]["reset_epoch"], 1);
    assert_eq!(records[2]["record_kind"], "rigid_world_result");
    assert_eq!(records[3]["reset_epoch"], 2);
}
