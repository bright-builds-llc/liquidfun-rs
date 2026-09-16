use super::super::support::refresh_identity;
use super::*;

fn semantic_result(root: &Path) -> TestResult<Value> {
    let identity: Value = serde_json::from_slice(&fs::read(root.join("identity.json"))?)?;
    Ok(json!({
        "schema_version": 1,
        "evidence_kind": "canonical_differential",
        "candidate_commit": SHA,
        "complete": true,
        "parity_tier": "d1_canonical",
        "coverage_authority": false,
        "performance_authority": false,
        "gap_count": 0,
        "semantic_sha256": identity["semantic_sha256"],
    }))
}

fn install_result(root: &TestRoot, run: &mut Value, kind: &str, result: &Value) -> TestResult {
    let directory = root.path.join(kind);
    write_json(&directory.join("semantic-result.json"), result)?;
    refresh_identity(&directory)?;
    let archive = root.path.join(format!("{kind}.zip"));
    zip(&directory, &archive)?;
    let bytes = fs::read(archive)?;
    let digest = format!("sha256:{}", sha256(&bytes));
    let index = usize::from(kind == "sanitizer");
    run["artifacts"][kind]["digest"] = json!(digest);
    run["artifacts"][kind]["size_in_bytes"] = json!(bytes.len());
    run["live_artifacts"][index]["digest"] = json!(digest);
    run["live_artifacts"][index]["size_in_bytes"] = json!(bytes.len());
    root.write_run(run)
}

#[test]
fn exact_ref_accepts_final_canonical_semantic_result() -> TestResult {
    // Arrange
    let root = TestRoot::new("exact-semantic-result")?;
    let mut run = root.write_exact_pair()?;
    let result = semantic_result(&root.path.join("canonical"))?;
    install_result(&root, &mut run, "canonical", &result)?;
    // Act
    let output = root.run_exact(&[])?;
    // Assert
    assert_success(&output);
    Ok(())
}

#[test]
fn exact_ref_rejects_semantic_result_contract_mutations() -> TestResult {
    // Arrange
    let mutations = [
        ("schema_version", json!(2)),
        ("evidence_kind", json!("diagnostic")),
        ("candidate_commit", json!("b".repeat(40))),
        ("candidate_commit", json!("local")),
        ("complete", json!(false)),
        ("parity_tier", json!("d2")),
        ("coverage_authority", json!(true)),
        ("performance_authority", json!(true)),
        ("gap_count", json!(1)),
        ("gap_count", json!("0")),
        ("semantic_sha256", json!("b".repeat(64))),
        ("unrecognized", json!(false)),
    ];
    for (field, value) in mutations {
        let root = TestRoot::new("exact-semantic-mutation")?;
        let mut run = root.write_exact_pair()?;
        let mut result = semantic_result(&root.path.join("canonical"))?;
        result[field] = value;
        install_result(&root, &mut run, "canonical", &result)?;
        // Act
        let output = root.run_exact(&[])?;
        // Assert
        assert_failure(&output);
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("semantic-result"),
            "{field}"
        );
    }
    Ok(())
}

#[test]
fn semantic_result_requires_every_field() -> TestResult {
    // Arrange
    let root = TestRoot::new("semantic-missing-field")?;
    let mut run = root.write_exact_pair()?;
    let result = semantic_result(&root.path.join("canonical"))?;
    for field in result
        .as_object()
        .ok_or("semantic result must be an object")?
        .keys()
    {
        let mut incomplete = result.clone();
        incomplete
            .as_object_mut()
            .ok_or("semantic result must be an object")?
            .remove(field);
        install_result(&root, &mut run, "canonical", &incomplete)?;
        // Act
        let output = root.run_exact(&[])?;
        // Assert
        assert_failure(&output);
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("semantic-result"),
            "{field}"
        );
    }
    Ok(())
}

#[test]
fn sanitizer_rejects_canonical_semantic_result() -> TestResult {
    // Arrange
    let root = TestRoot::new("sanitizer-semantic-extra")?;
    let mut run = root.write_exact_pair()?;
    let result = semantic_result(&root.path.join("sanitizer"))?;
    install_result(&root, &mut run, "sanitizer", &result)?;
    // Act
    let output = root.run_exact(&[])?;
    // Assert
    assert_failure(&output);
    assert!(String::from_utf8_lossy(&output.stderr).contains("closed Phase 11 topology"));
    Ok(())
}

#[test]
fn preliminary_content_rejects_final_semantic_result() -> TestResult {
    // Arrange
    let root = TestRoot::new("preliminary-semantic-extra")?;
    root.write_local_pair()?;
    let canonical = root.path.join("canonical");
    let result = semantic_result(&canonical)?;
    write_json(&canonical.join("semantic-result.json"), &result)?;
    fs::remove_file(canonical.join("identity.json"))?;
    // Act
    let output = run_xtask(&[
        "phase11-evidence",
        "validate-content",
        "canonical",
        &root.relative("canonical"),
    ])?;
    // Assert
    assert_failure(&output);
    assert!(String::from_utf8_lossy(&output.stderr).contains("identity-last"));
    Ok(())
}

#[test]
fn local_semantic_result_remains_nonpromotable_without_live_source_binding() -> TestResult {
    // Arrange
    let root = TestRoot::new("local-semantic-result")?;
    root.write_local_pair()?;
    let canonical = root.path.join("canonical");
    let result = semantic_result(&canonical)?;
    write_json(&canonical.join("semantic-result.json"), &result)?;
    refresh_identity(&canonical)?;
    // Act
    let output = root.run_local()?;
    // Assert
    assert_success(&output);
    assert!(String::from_utf8_lossy(&output.stdout).contains("local D2 non-promotable"));
    Ok(())
}

#[test]
fn local_semantic_result_rejects_malformed_source_identity() -> TestResult {
    // Arrange
    let root = TestRoot::new("local-semantic-invalid-source")?;
    root.write_local_pair()?;
    let canonical = root.path.join("canonical");
    let mut result = semantic_result(&canonical)?;
    result["candidate_commit"] = json!("A".repeat(40));
    write_json(&canonical.join("semantic-result.json"), &result)?;
    refresh_identity(&canonical)?;
    // Act
    let output = root.run_local()?;
    // Assert
    assert_failure(&output);
    assert!(String::from_utf8_lossy(&output.stderr).contains("semantic-result"));
    Ok(())
}
