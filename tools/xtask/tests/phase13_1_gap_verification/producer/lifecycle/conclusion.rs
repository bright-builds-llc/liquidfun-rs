use std::fs;

use serde_json::{Value, json};

use super::{
    ProducerFixture, TestResult, assert_success, mutate_command_stdout, run_validator,
    workspace_root,
};

const NARROW_CONCLUSION_PREDICATE: &str = r#"if .status == "completed" then .conclusion == "success" else (.conclusion == null or .conclusion == "") end"#;

fn assert_script_has_narrow_conclusion_contract(script: &str) -> TestResult {
    let source = fs::read_to_string(workspace_root().join(script))?;
    assert!(
        source.contains(NARROW_CONCLUSION_PREDICATE),
        "{script} does not contain the exact narrow initial conclusion predicate"
    );
    Ok(())
}

fn run_producer_initial_view(
    status: &str,
    conclusion_json: &str,
) -> TestResult<(ProducerFixture, std::process::Output)> {
    let fixture = ProducerFixture::new(true, "success")?;
    let output = fixture.run_with_settings(
        &fixture.candidate,
        "main",
        &[
            ("PHASE13_1_GAP_FAKE_INITIAL_STATUS", status),
            (
                "PHASE13_1_GAP_FAKE_INITIAL_CONCLUSION_JSON",
                conclusion_json,
            ),
        ],
    )?;
    Ok((fixture, output))
}

fn validator_output_for_initial_view(
    status: &str,
    conclusion: Value,
) -> TestResult<std::process::Output> {
    let fixture = ProducerFixture::new(true, "success")?;
    let producer_output = fixture.run()?;
    assert_success(&producer_output);
    mutate_command_stdout(
        &fixture,
        "canonical-initial-view",
        &json!({
            "databaseId": 7,
            "headSha": fixture.candidate,
            "event": "workflow_dispatch",
            "status": status,
            "conclusion": conclusion,
            "url": "https://github.com/fixture/repository/actions/runs/7"
        }),
    )?;
    run_validator(&fixture)
}

#[test]
fn producer_accepts_null_or_empty_conclusion_for_noncompleted_initial_view() -> TestResult {
    // Arrange
    let cases = [
        ("queued", "null"),
        ("queued", r#""""#),
        ("in_progress", "null"),
        ("in_progress", r#""""#),
    ];

    // Act / Assert
    for (status, conclusion_json) in cases {
        let (_fixture, output) = run_producer_initial_view(status, conclusion_json)?;
        assert_success(&output);
    }
    assert_script_has_narrow_conclusion_contract("scripts/phase13-1-gap-verification.sh")
}

#[test]
fn producer_rejects_nonempty_conclusion_for_noncompleted_initial_view() -> TestResult {
    // Arrange
    let cases = [
        ("queued", r#""failure""#),
        ("queued", r#""success""#),
        ("in_progress", r#""unexpected""#),
    ];

    // Act / Assert
    for (status, conclusion_json) in cases {
        let (fixture, output) = run_producer_initial_view(status, conclusion_json)?;
        fixture.assert_rejected_without_terminal(&output);
    }
    assert_script_has_narrow_conclusion_contract("scripts/phase13-1-gap-verification.sh")
}

#[test]
fn producer_rejects_null_or_empty_conclusion_for_completed_initial_view() -> TestResult {
    // Arrange
    let rejected = ["null", r#""""#];

    // Act / Assert
    for conclusion_json in rejected {
        let (fixture, output) = run_producer_initial_view("completed", conclusion_json)?;
        fixture.assert_rejected_without_terminal(&output);
    }
    let (_fixture, success) = run_producer_initial_view("completed", r#""success""#)?;
    assert_success(&success);
    assert_script_has_narrow_conclusion_contract("scripts/phase13-1-gap-verification.sh")
}

#[test]
fn validator_accepts_null_or_empty_conclusion_for_noncompleted_initial_view() -> TestResult {
    // Arrange
    let cases = [
        ("queued", Value::Null),
        ("queued", json!("")),
        ("in_progress", Value::Null),
        ("in_progress", json!("")),
    ];

    // Act / Assert
    for (status, conclusion) in cases {
        let output = validator_output_for_initial_view(status, conclusion)?;
        assert!(
            output.status.success(),
            "validator rejected {status}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert_script_has_narrow_conclusion_contract("scripts/phase13-1-validate-gap-evidence.sh")
}

#[test]
fn validator_rejects_nonempty_conclusion_for_noncompleted_initial_view() -> TestResult {
    // Arrange
    let cases = [
        ("queued", json!("failure")),
        ("queued", json!("success")),
        ("in_progress", json!("unexpected")),
    ];

    // Act / Assert
    for (status, conclusion) in cases {
        let output = validator_output_for_initial_view(status, conclusion)?;
        assert!(!output.status.success());
    }
    assert_script_has_narrow_conclusion_contract("scripts/phase13-1-validate-gap-evidence.sh")
}

#[test]
fn validator_rejects_null_or_empty_conclusion_for_completed_initial_view() -> TestResult {
    // Arrange
    let rejected = [Value::Null, json!("")];

    // Act / Assert
    for conclusion in rejected {
        let output = validator_output_for_initial_view("completed", conclusion)?;
        assert!(!output.status.success());
    }
    let success = validator_output_for_initial_view("completed", json!("success"))?;
    assert!(success.status.success());
    assert_script_has_narrow_conclusion_contract("scripts/phase13-1-validate-gap-evidence.sh")
}
