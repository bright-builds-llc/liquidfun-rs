#[test]
fn decode_accepts_bounded_phase9_request_in_existing_cpp_process() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: build oracle-debug to exercise the pinned C++ decoder");
        return;
    };
    let request = phase9_request();

    // Act
    let result = execute_rigid_world_process(&executable, &request, REVISION);

    // Assert
    assert!(result.is_ok(), "bounded Phase 9 request failed: {result:?}");
}

#[test]
fn decode_accepts_negative_finite_lifetime_as_infinite_in_cpp() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: build oracle-debug to exercise the pinned C++ decoder");
        return;
    };
    let mut value = serde_json::to_value(phase9_request()).expect("request should serialize");
    value["scenario"]["timelines"][0]["particles"][0]["lifetime_bits"] =
        json!((-1.0_f32).to_bits());
    let mut bytes = serde_json::to_vec(&value).expect("negative lifetime request should encode");
    bytes.push(b'\n');
    let request = decode_rigid_world_request_jsonl(&bytes, &HarnessLimits::phase2_default_v1())
        .expect("finite negative lifetime should cross the Rust boundary");

    // Act
    let result = execute_rigid_world_process(&executable, &request, REVISION);

    // Assert
    assert!(
        result.is_ok(),
        "negative lifetime must remain infinite: {result:?}"
    );
    assert_eq!(
        request.scenario().timelines()[0].particles()[0]
            .lifetime_bits
            .bits(),
        (-1.0_f32).to_bits()
    );
}

#[test]
fn mixed_identity_matches_native_declaration_order() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let Ok(executable) = OracleExecutable::resolve(&root, OraclePreset::Debug) else {
        eprintln!("SKIP: build oracle-debug to exercise mixed identity");
        return;
    };
    let request = coupling_request();

    // Act
    let native = NativeRigidWorldExecutor::execute(&request)
        .expect("native mixed identity request should execute");
    let oracle = execute_rigid_world_process(&executable, &request, REVISION)
        .expect("oracle mixed identity request should execute");
    let native_value = serde_json::to_value(native).expect("native result should serialize");
    let oracle_value =
        serde_json::to_value(oracle.result()).expect("oracle result should serialize");
    let find_live_mixed = |value: &Value| {
        value["timelines"][0]["checkpoints"]
            .as_array()
            .expect("checkpoints should be an array")
            .iter()
            .filter_map(|checkpoint| checkpoint["observations"].as_array())
            .flatten()
            .find(|observation| {
                observation["observation"]["kind"] == "mixed_state"
                    && observation["observation"]["particle_ids"]
                        .as_array()
                        .is_some_and(|ids| !ids.is_empty())
            })
            .expect("a live mixed-state observation should exist")
            .clone()
    };

    // Assert
    let native_mixed = find_live_mixed(&native_value);
    let oracle_mixed = find_live_mixed(&oracle_value);
    assert_eq!(
        oracle_mixed["observation"]["body_ids"],
        native_mixed["observation"]["body_ids"]
    );
    assert_eq!(
        oracle_mixed["observation"]["particle_ids"],
        native_mixed["observation"]["particle_ids"]
    );
}

#[test]
fn pinned_witness_is_consumed_before_generalized_oracle_execution() {
    // Arrange
    let witness: Value = serde_json::from_slice(WITNESS).expect("witness should be JSON");
    let provenance: Value =
        serde_json::from_str(WITNESS_PROVENANCE).expect("provenance should be JSON");

    // Act
    let digest = format!("{:x}", Sha256::digest(WITNESS));
    let oldest = witness["witnesses"][0]["oldest_selection_order"]
        .as_array()
        .expect("oldest order should be an array");
    let strict = witness["witnesses"][1]["strict_order"]
        .as_array()
        .expect("strict order should be an array");

    // Assert
    assert_eq!(witness["oracle_revision"], REVISION);
    assert_eq!(provenance["oracle_revision"], REVISION);
    assert_eq!(provenance["witness_sha256"], digest);
    assert_eq!(
        oldest,
        json!([
            "particle-7",
            "particle-6",
            "particle-5",
            "particle-4",
            "particle-3",
            "particle-2",
            "particle-1",
            "particle-0"
        ])
        .as_array()
        .expect("expected oldest order should be an array")
    );
    assert_eq!(
        strict,
        json!([
            "fixture-5",
            "fixture-4",
            "fixture-3",
            "fixture-2",
            "fixture-0"
        ])
            .as_array()
            .expect("expected strict order should be an array")
    );
}

#[test]
fn decode_rejects_phase10_group_topology_without_poisoning_cpp_process() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let executable = root.join("target/reference/oracle-debug/liquidfun-reference");
    if !executable.is_file() {
        eprintln!("SKIP: build oracle-debug to exercise the pinned C++ decoder");
        return;
    }
    let mut value = serde_json::to_value(phase9_request()).expect("request should serialize");
    value["scenario"]["timelines"][0]["particle_groups"] = json!([{ "group_id": "phase10-group" }]);
    let mut bytes = serde_json::to_vec(&value).expect("invalid request should encode");
    bytes.push(b'\n');
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
        .write_all(&bytes)
        .and_then(|()| stdin.flush())
        .expect("invalid request should reach the decoder");
    drop(stdin);
    let mut unexpected_stdout = String::new();
    stdout
        .read_to_string(&mut unexpected_stdout)
        .expect("remaining stdout should be readable");
    let output = child.wait_with_output().expect("oracle should be reaped");

    // Assert
    assert!(serde_json::from_str::<Value>(&handshake).is_ok());
    assert!(
        unexpected_stdout.is_empty(),
        "stdout must remain JSONL-only"
    );
    assert!(
        output.status.success(),
        "a rejected Phase 10 request must not poison the reusable oracle process"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unknown member particle_groups"),
        "stderr should classify the undeclared Phase 10 family: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn decode_rejects_invalid_phase9_lifecycle_matrix_without_poisoning_process() {
    // Arrange
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let executable = root.join("target/reference/oracle-debug/liquidfun-reference");
    if !executable.is_file() {
        eprintln!("SKIP: build oracle-debug to exercise lifecycle rejection");
        return;
    }
    let base = serde_json::to_value(full_phase9_request()).expect("request should serialize");
    let mut duplicate_system = base.clone();
    let index = phase9_action_index(&duplicate_system, "oracle-create-newest-system");
    let duplicate = duplicate_system["scenario"]["timelines"][0]["actions"][index].clone();
    duplicate_system["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .insert(index + 1, duplicate);

    let mut use_before_create = base.clone();
    let system_index = phase9_action_index(&use_before_create, "oracle-create-newest-system");
    let particle_index = phase9_action_index(&use_before_create, "oracle-create-newest-particle");
    use_before_create["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .swap(system_index, particle_index);

    let mut duplicate_particle = base.clone();
    let index = phase9_action_index(&duplicate_particle, "oracle-create-newest-particle");
    let duplicate = duplicate_particle["scenario"]["timelines"][0]["actions"][index].clone();
    duplicate_particle["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .insert(index + 1, duplicate);

    let mut unknown_particle = base.clone();
    let index = phase9_action_index(&unknown_particle, "oracle-inspect-particle");
    unknown_particle["scenario"]["timelines"][0]["actions"][index]["action"]["action"]["particle_id"] =
        json!("unknown-particle");

    let mut pending_use = base.clone();
    let index = phase9_action_index(&pending_use, "oracle-mark");
    pending_use["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .insert(
            index + 1,
            json!({ "action_id": "oracle-inspect-pending", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "inspect_particle", "particle_id": "oracle-particle-newest" }
            }}),
        );

    let mut repeated_mark = base.clone();
    let index = phase9_action_index(&repeated_mark, "oracle-mark");
    repeated_mark["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .insert(
            index + 1,
            json!({ "action_id": "oracle-mark-again", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "mark_for_destruction", "particle_id": "oracle-particle-newest" }
            }}),
        );

    let mut cross_system_range = base.clone();
    let index = phase9_action_index(&cross_system_range, "oracle-force");
    cross_system_range["scenario"]["timelines"][0]["actions"][index]["action"]["action"]["particle_ids"] =
        json!(["oracle-particle-newest", "oracle-particle"]);

    let mut destroyed_owner = base.clone();
    let index = phase9_action_index(&destroyed_owner, "oracle-destroy-newest-system");
    destroyed_owner["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .insert(
            index + 1,
            json!({ "action_id": "oracle-query-destroyed", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "query_aabb", "system_id": "oracle-system-newest",
                    "lower": { "x_bits": 0, "y_bits": 0 }, "upper": { "x_bits": 1_065_353_216, "y_bits": 1_065_353_216 } }
            }}),
        );

    let mut recreate_after_compaction = base.clone();
    let index = phase9_action_index(&recreate_after_compaction, "oracle-compact");
    recreate_after_compaction["scenario"]["timelines"][0]["actions"]
        .as_array_mut()
        .expect("actions should be an array")
        .insert(
            index + 1,
            json!({ "action_id": "oracle-recreate", "phase": "phase9", "action": {
                "kind": "particle", "action": { "kind": "create_particle", "particle_id": "oracle-particle-newest" }
            }}),
        );
    let mutations = [
        duplicate_system,
        use_before_create,
        duplicate_particle,
        unknown_particle,
        pending_use,
        repeated_mark,
        cross_system_range,
        destroyed_owner,
        recreate_after_compaction,
    ];

    // Act
    let diagnostics = mutations.map(|value| raw_oracle_rejection(&executable, &value));

    // Assert
    assert!(
        diagnostics
            .iter()
            .all(|message| message.contains("Phase 9"))
    );
}
