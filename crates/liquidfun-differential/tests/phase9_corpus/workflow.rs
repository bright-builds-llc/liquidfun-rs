#[test]
fn corpus_rejects_missing_declarations_and_phase10_members() {
    // Arrange
    let manifest = manifest();
    let mut missing = serde_json::from_str::<Value>(MANIFEST).expect("manifest should be JSON");
    missing["cases"][0]["witnesses"]
        .as_array_mut()
        .expect("branches should be an array")
        .remove(0);

    // Act / Assert
    let decoded: CorpusManifest = serde_json::from_value(missing).expect("shape remains valid");
    let decoded_branches = decoded
        .cases
        .iter()
        .flat_map(|case| &case.witnesses)
        .map(|witness| witness.branch_id.as_str())
        .collect::<BTreeSet<_>>();
    assert_ne!(
        decoded_branches,
        REQUIRED_BRANCHES.iter().copied().collect()
    );
    for member in &manifest.forbidden_phase10_members {
        let mut value = request_value();
        value["scenario"]["timelines"][0][member] = json!([]);
        assert!(decode_value(&value).is_err(), "{member} must fail closed");
    }
    assert!(
        PHASE9_REQUIRED_POLICY_PATHS
            .iter()
            .all(|path| phase9_policy_for_path(path).is_some())
    );
    assert_eq!(phase9_policy_for_path("particle.*"), None);
    assert_eq!(phase9_policy_for_path("particle.group.topology"), None);
    assert_eq!(phase9_policy_for_path("particle.solver.baseline"), None);
}

#[test]
fn required_oracle_mode_proves_replay_and_profile_agreement() {
    // Arrange
    let Ok(mode) = std::env::var("LIQUIDFUN_PHASE9_ORACLE_MODE") else {
        return;
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let request = bounded_phase9_request("closed-evidence-contract");
    let revision = manifest().pinned_upstream_revision;
    let primary_preset = match mode.as_str() {
        "canonical" => OraclePreset::Debug,
        "sanitizer" => OraclePreset::AsanUbsan,
        _ => panic!("LIQUIDFUN_PHASE9_ORACLE_MODE must be canonical or sanitizer"),
    };
    let primary = OracleExecutable::resolve(&root, primary_preset)
        .expect("the required primary Phase 9 oracle must exist");

    // Act
    let first = execute_rigid_world_process(&primary, &request, &revision)
        .expect("the primary Phase 9 oracle run should pass");
    let replay = execute_rigid_world_process(&primary, &request, &revision)
        .expect("the Phase 9 oracle replay should pass");
    let differential = run_phase9_differential(&primary, &request, &revision)
        .expect("the bounded Phase 9 corpus should compare");

    // Assert
    assert_eq!(first.response_bytes(), replay.response_bytes());
    assert_eq!(first.result(), replay.result());
    assert!(
        matches!(
            differential.outcome(),
            liquidfun_differential::Phase9ComparisonOutcome::Match { .. }
        ),
        "unexpected outcome: {:?}",
        differential.outcome()
    );
    if mode == "canonical" {
        let release = OracleExecutable::resolve(&root, OraclePreset::Release)
            .expect("the required release Phase 9 oracle must exist");
        let optimized = execute_rigid_world_process(&release, &request, &revision)
            .expect("the release Phase 9 oracle run should pass");
        assert_eq!(first.result(), optimized.result());
    }
}

#[test]
#[cfg(unix)]
fn workflow_contract_blocks_failed_evidence_identity() {
    use std::os::unix::fs::PermissionsExt;

    // Arrange
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let workflow = std::fs::read_to_string(root.join(".github/workflows/oracle.yml"))
        .expect("Oracle workflow");
    assert_eq!(
        workflow.matches("bash scripts/phase9-evidence.sh").count(),
        2
    );
    let script = root.join("scripts/phase9-evidence.sh");
    let script_text = std::fs::read_to_string(&script).expect("evidence script");
    assert!(script_text.contains("set -euo pipefail"));
    let validation = script_text
        .find("cargo xtask phase9-evidence validate-content")
        .expect("shared content validator");
    let identity = script_text
        .find("> \"$output_dir/identity.json\"")
        .expect("identity emission");
    assert!(
        validation < identity,
        "content validation must precede identity"
    );

    for (name, cargo_body) in [
        ("command-failure", "exit 7\n"),
        (
            "failed-log",
            "printf '%s\\n' 'test result: ok. 4 passed' 'test result: FAILED. 4 passed; 1 failed'\nexit 0\n",
        ),
    ] {
        let contract_root = root.join("target").join(format!(
            "phase9-workflow-contract-{}-{name}",
            std::process::id()
        ));
        let fake_bin = contract_root.join("bin");
        let output = contract_root.join("canonical");
        std::fs::create_dir_all(&fake_bin).expect("fake command directory");
        let fake_cargo = fake_bin.join("cargo");
        std::fs::write(
            &fake_cargo,
            format!("#!/usr/bin/env bash\nset -euo pipefail\n{cargo_body}"),
        )
        .expect("fake cargo");
        let mut permissions = std::fs::metadata(&fake_cargo)
            .expect("fake cargo metadata")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&fake_cargo, permissions).expect("fake cargo executable");
        let path = format!(
            "{}:{}",
            fake_bin.display(),
            std::env::var("PATH").expect("PATH")
        );

        // Act
        let contract = Command::new("bash")
            .arg(&script)
            .arg("canonical")
            .arg(
                output
                    .strip_prefix(&root)
                    .expect("repository-relative output"),
            )
            .current_dir(&root)
            .env("PATH", path)
            .output()
            .expect("evidence script should execute");

        // Assert
        assert!(!contract.status.success(), "{name} must fail closed");
        assert!(!output.join("identity.json").exists());
        std::fs::remove_dir_all(&contract_root).expect("contract cleanup");
    }
}

#[test]
#[cfg(unix)]
fn workflow_contract_rejects_symlinked_output_before_cleanup() {
    use std::os::unix::fs::symlink;

    // Arrange
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let script = root.join("scripts/phase9-evidence.sh");
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow the Unix epoch")
        .as_nanos();
    let contract_root = root
        .join("target")
        .join(format!("phase9-symlink-contract-{nonce}"));
    let external_root = std::env::temp_dir().join(format!("liquidfun-phase9-external-{nonce}"));
    fs::create_dir_all(&contract_root).expect("contract root");

    for shape in ["final", "ancestor"] {
        let external_output = external_root.join(shape).join("canonical");
        let marker = external_output.join("cases/external-marker");
        fs::create_dir_all(marker.parent().expect("marker parent")).expect("external cases");
        fs::write(&marker, b"must survive").expect("external marker");

        let relative_output = if shape == "final" {
            let link = contract_root.join("canonical");
            symlink(&external_output, &link).expect("final output symlink");
            link.strip_prefix(&root)
                .expect("contract output remains repository-relative")
                .to_path_buf()
        } else {
            let link = contract_root.join("linked-ancestor");
            symlink(
                external_output.parent().expect("external output parent"),
                &link,
            )
            .expect("output ancestor symlink");
            link.join("canonical")
                .strip_prefix(&root)
                .expect("contract output remains repository-relative")
                .to_path_buf()
        };

        // Act
        let output = Command::new("bash")
            .arg(&script)
            .arg("canonical")
            .arg(&relative_output)
            .current_dir(&root)
            .output()
            .expect("evidence script should execute");

        // Assert
        assert!(!output.status.success(), "{shape} symlink must fail closed");
        assert_eq!(
            fs::read(&marker).expect("external marker must remain readable"),
            b"must survive"
        );

        let link = if shape == "final" {
            contract_root.join("canonical")
        } else {
            contract_root.join("linked-ancestor")
        };
        fs::remove_file(link).expect("contract symlink cleanup");
    }

    fs::remove_dir_all(&contract_root).expect("contract root cleanup");
    fs::remove_dir_all(&external_root).expect("external root cleanup");
}
