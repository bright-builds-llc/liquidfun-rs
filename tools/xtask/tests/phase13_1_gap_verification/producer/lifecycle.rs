use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use super::{ProducerFixture, TestResult, workspace_root};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn manifest() -> TestResult<Value> {
    Ok(serde_json::from_slice(&fs::read(workspace_root().join(
        "scripts/phase13-1-gap-verification-manifest.json",
    ))?)?)
}

fn output_directory(fixture: &ProducerFixture) -> PathBuf {
    fixture
        .repository
        .join("target/phase13-1-gap-verification")
        .join(&fixture.candidate)
}

fn read_json(path: &Path) -> TestResult<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn write_json(path: &Path, value: &Value) -> TestResult {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn hash_file(path: &Path) -> TestResult<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}

fn hash_canonical_value(value: &Value) -> TestResult<String> {
    let mut child = Command::new("jq")
        .args(["-cS", "."])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("jq stdin must be piped")?
        .write_all(&serde_json::to_vec(value)?)?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err("jq failed to canonicalize JSON".into());
    }
    Ok(format!("{:x}", Sha256::digest(output.stdout)))
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "producer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn journal_count(path: &Path, needle: &str) -> TestResult<usize> {
    if !path.exists() {
        return Ok(0);
    }
    Ok(fs::read_to_string(path)?
        .lines()
        .filter(|line| line.starts_with(needle))
        .count())
}

fn successful_fixture() -> TestResult<ProducerFixture> {
    let fixture = ProducerFixture::new(true, "success")?;
    let output = fixture.run()?;
    assert_success(&output);
    Ok(fixture)
}

fn remove_file_if_present(path: &Path) -> TestResult {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn retain_prefix(fixture: &ProducerFixture, count: usize) -> TestResult {
    let directory = output_directory(fixture);
    let terminal = read_json(&fixture.terminal_path())?;
    let commands = terminal["commands"]
        .as_array()
        .ok_or("terminal commands must be an array")?;
    let retained_logs = commands[..count]
        .iter()
        .flat_map(|command| ["stdout_log", "stderr_log"].map(move |key| &command[key]))
        .map(|relative| {
            let relative = relative.as_str().ok_or("log path must be a string")?;
            Ok((relative.to_owned(), fs::read(directory.join(relative))?))
        })
        .collect::<TestResult<Vec<_>>>()?;
    let records = commands[..count]
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n")
        + "\n";

    fs::remove_dir_all(directory.join("logs"))?;
    fs::create_dir(directory.join("logs"))?;
    for (relative, bytes) in retained_logs {
        fs::write(directory.join(relative), bytes)?;
    }
    if directory.join("canonical").exists() {
        fs::remove_dir_all(directory.join("canonical"))?;
    }
    fs::write(directory.join("command-records.jsonl"), records)?;
    remove_file_if_present(&fixture.terminal_path())?;
    remove_file_if_present(&directory.join("final-verification.json.pending"))?;
    remove_file_if_present(&directory.join("dispatch-intent.json"))?;
    remove_file_if_present(&directory.join("dispatch-result.json"))?;
    remove_file_if_present(&fixture.gh_journal)?;
    remove_file_if_present(&fixture.gh_state)?;
    Ok(())
}

fn run_validator(fixture: &ProducerFixture) -> TestResult<Output> {
    let directory = output_directory(fixture);
    Ok(
        Command::new(workspace_root().join("scripts/phase13-1-validate-gap-evidence.sh"))
            .args([
                &fixture.manifest,
                &fixture.terminal_path(),
                &fixture.repository,
                &directory,
            ])
            .output()?,
    )
}

fn mutate_command_stdout(
    fixture: &ProducerFixture,
    command_id: &str,
    value: &Value,
) -> TestResult<Value> {
    let directory = output_directory(fixture);
    let terminal_path = fixture.terminal_path();
    let mut terminal = read_json(&terminal_path)?;
    let command = terminal["commands"]
        .as_array_mut()
        .and_then(|commands| {
            commands
                .iter_mut()
                .find(|command| command["id"] == command_id)
        })
        .ok_or("command must exist")?;
    let relative = command["stdout_log"]
        .as_str()
        .ok_or("stdout path must be a string")?;
    write_json(&directory.join(relative), value)?;
    command["stdout_sha256"] = json!(hash_file(&directory.join(relative))?);
    write_json(&terminal_path, &terminal)?;
    Ok(terminal)
}

fn repair_result_digest(fixture: &ProducerFixture, terminal: &mut Value) -> TestResult {
    let result_path = output_directory(fixture).join("dispatch-result.json");
    terminal["dispatch_result"]["sha256"] = json!(hash_file(&result_path)?);
    write_json(&fixture.terminal_path(), terminal)
}

#[test]
fn manifest_preserves_exact_auxiliary_argv_and_corrected_capability() -> TestResult {
    // Arrange
    let manifest = manifest()?;
    let commands = manifest["commands"]
        .as_array()
        .ok_or("commands must be an array")?;

    // Act
    let auxiliary = &commands[59..72];

    // Assert
    assert_eq!(auxiliary.len(), 13);
    assert_eq!(
        commands[65]["argv"],
        json!([
            "cargo",
            "run",
            "-p",
            "liquidfun-testbed",
            "--",
            "--capability-check",
            "--fixture",
            "crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json",
            "--output",
            "target/phase13-1-gap-verification/${CANDIDATE}/testbed-capability"
        ])
    );
    Ok(())
}

#[test]
fn exact_dispatch_url_establishes_run_id() -> TestResult {
    // Arrange / Act
    let fixture = successful_fixture()?;
    let terminal = read_json(&fixture.terminal_path())?;

    // Assert
    assert_eq!(terminal["canonical_run_id"], "7");
    assert_eq!(journal_count(&fixture.gh_journal, "workflow run")?, 1);
    Ok(())
}

#[test]
fn dispatch_url_rejects_wrong_repository() -> TestResult {
    // Arrange
    let fixture = ProducerFixture::new(true, "success")?;

    // Act
    let output = fixture.run_with_settings(
        &fixture.candidate,
        "main",
        &[(
            "PHASE13_1_GAP_FAKE_DISPATCH_URL",
            "https://github.com/wrong/repository/actions/runs/7",
        )],
    )?;

    // Assert
    fixture.assert_rejected_without_terminal(&output);
    assert_eq!(journal_count(&fixture.gh_journal, "workflow run")?, 1);
    Ok(())
}

#[test]
fn dispatch_url_rejects_malformed_or_missing_run_id() -> TestResult {
    // Arrange
    let urls = [
        "not-a-url",
        "https://github.com/fixture/repository/actions/runs/",
        "https://github.com/fixture/repository/actions/runs/0",
    ];

    // Act / Assert
    for url in urls {
        let fixture = ProducerFixture::new(true, "success")?;
        let output = fixture.run_with_settings(
            &fixture.candidate,
            "main",
            &[("PHASE13_1_GAP_FAKE_DISPATCH_URL", url)],
        )?;
        fixture.assert_rejected_without_terminal(&output);
    }
    Ok(())
}

#[test]
fn exact_dispatch_url_survives_empty_immediate_listing() -> TestResult {
    // Arrange
    let fixture = ProducerFixture::new(true, "success")?;

    // Act
    let output = fixture.run_with_settings(
        &fixture.candidate,
        "main",
        &[("PHASE13_1_GAP_FAKE_LISTING", "[]")],
    )?;

    // Assert
    assert_success(&output);
    assert_eq!(journal_count(&fixture.gh_journal, "run list")?, 0);
    Ok(())
}

#[test]
fn exact_dispatch_url_ignores_stale_same_candidate_listing() -> TestResult {
    // Arrange
    let fixture = ProducerFixture::new(true, "success")?;
    let stale = format!(
        r#"[{{"databaseId":6,"headSha":"{}","status":"completed","conclusion":"success"}}]"#,
        fixture.candidate
    );

    // Act
    let output = fixture.run_with_settings(
        &fixture.candidate,
        "main",
        &[("PHASE13_1_GAP_FAKE_LISTING", &stale)],
    )?;

    // Assert
    assert_success(&output);
    assert_eq!(journal_count(&fixture.gh_journal, "run list")?, 0);
    assert_eq!(
        read_json(&fixture.terminal_path())?["canonical_run_id"],
        "7"
    );
    Ok(())
}

#[test]
fn exact_zero_exit_prefix_resumes_without_rerunning_successes() -> TestResult {
    // Arrange
    let fixture = successful_fixture()?;
    retain_prefix(&fixture, 1)?;

    // Act
    let output = fixture.run()?;

    // Assert
    assert_success(&output);
    assert_eq!(
        journal_count(&fixture.command_journal, "fixture-prefix")?,
        1
    );
    assert_eq!(journal_count(&fixture.gh_journal, "workflow run")?, 1);
    Ok(())
}

#[test]
fn failed_prefix_refuses_resume() -> TestResult {
    // Arrange
    let fixture = successful_fixture()?;
    retain_prefix(&fixture, 1)?;
    let records_path = output_directory(&fixture).join("command-records.jsonl");
    let mut record: Value = serde_json::from_slice(&fs::read(&records_path)?)?;
    record["exit_code"] = json!(1);
    fs::write(
        &records_path,
        format!("{}\n", serde_json::to_string(&record)?),
    )?;

    // Act
    let output = fixture.run()?;

    // Assert
    fixture.assert_rejected_without_terminal(&output);
    assert_eq!(journal_count(&fixture.gh_journal, "workflow run")?, 0);
    Ok(())
}

#[test]
fn terminal_evidence_validation_is_idempotent() -> TestResult {
    // Arrange
    let fixture = successful_fixture()?;
    let before = fs::read(fixture.terminal_path())?;
    let dispatches_before = journal_count(&fixture.gh_journal, "workflow run")?;

    // Act
    let output = fixture.run()?;

    // Assert
    assert_success(&output);
    assert_eq!(fs::read(fixture.terminal_path())?, before);
    assert_eq!(
        journal_count(&fixture.gh_journal, "workflow run")?,
        dispatches_before
    );
    Ok(())
}

#[test]
fn dispatch_intent_without_result_never_redispatches() -> TestResult {
    // Arrange
    let fixture = ProducerFixture::new(true, "success")?;
    let environment = &[("PHASE13_1_GAP_FAKE_DISPATCH_URL", "not-a-url")];
    let first = fixture.run_with_settings(&fixture.candidate, "main", environment)?;
    assert!(!first.status.success());

    // Act
    let second = fixture.run_with_settings(&fixture.candidate, "main", environment)?;

    // Assert
    assert!(!second.status.success());
    assert_eq!(journal_count(&fixture.gh_journal, "workflow run")?, 1);
    assert!(
        !output_directory(&fixture)
            .join("dispatch-result.json")
            .exists()
    );
    Ok(())
}

#[test]
fn validated_dispatch_stdout_recovers_result_without_redispatch() -> TestResult {
    // Arrange
    let fixture = successful_fixture()?;
    let directory = output_directory(&fixture);
    remove_file_if_present(&fixture.terminal_path())?;
    remove_file_if_present(&directory.join("dispatch-result.json"))?;
    let dispatches_before = journal_count(&fixture.gh_journal, "workflow run")?;

    // Act
    let output = fixture.run()?;

    // Assert
    assert_success(&output);
    assert!(directory.join("dispatch-result.json").is_file());
    assert_eq!(
        journal_count(&fixture.gh_journal, "workflow run")?,
        dispatches_before
    );
    Ok(())
}

#[test]
fn fuzz_build_keeps_tracked_lock_and_clean_tree() -> TestResult {
    // Arrange
    let root = workspace_root();
    let before = Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=all"])
        .current_dir(root)
        .output()?
        .stdout;

    // Act
    let build = Command::new("cargo")
        .args(["+nightly-2026-07-15", "fuzz", "build"])
        .current_dir(root)
        .status()?;
    let after = Command::new("git")
        .args(["status", "--porcelain=v1", "--untracked-files=all"])
        .current_dir(root)
        .output()?
        .stdout;

    // Assert
    assert!(build.success());
    assert_eq!(after, before);
    assert!(root.join("fuzz/Cargo.lock").is_file());
    assert!(
        Command::new("git")
            .args(["ls-files", "--error-unmatch", "fuzz/Cargo.lock"])
            .current_dir(root)
            .status()?
            .success()
    );
    Ok(())
}

#[test]
fn validator_rejects_dispatch_url_substitution() -> TestResult {
    // Arrange
    let fixture = successful_fixture()?;
    let directory = output_directory(&fixture);
    fs::write(
        directory.join("logs/canonical-dispatch.stdout"),
        "https://github.com/fixture/repository/actions/runs/8\n",
    )?;
    let mut terminal = read_json(&fixture.terminal_path())?;
    let dispatch = terminal["commands"]
        .as_array_mut()
        .and_then(|commands| {
            commands
                .iter_mut()
                .find(|command| command["id"] == "canonical-dispatch")
        })
        .ok_or("dispatch command must exist")?;
    dispatch["stdout_sha256"] = json!(hash_file(
        &directory.join("logs/canonical-dispatch.stdout")
    )?);
    let mut result = read_json(&directory.join("dispatch-result.json"))?;
    result["dispatch_url"] = json!("https://github.com/fixture/repository/actions/runs/8");
    result["canonical_run_id"] = json!("8");
    result["command_stdout_sha256"] = dispatch["stdout_sha256"].clone();
    result["command_record_sha256"] = json!(hash_canonical_value(dispatch)?);
    write_json(&directory.join("dispatch-result.json"), &result)?;
    repair_result_digest(&fixture, &mut terminal)?;

    // Act
    let output = run_validator(&fixture)?;

    // Assert
    assert!(!output.status.success());
    Ok(())
}

#[test]
fn validator_rejects_terminal_run_id_substitution() -> TestResult {
    // Arrange
    let fixture = successful_fixture()?;
    let mut terminal = read_json(&fixture.terminal_path())?;
    terminal["canonical_run_id"] = json!("8");
    for command in terminal["commands"]
        .as_array_mut()
        .ok_or("commands must be an array")?
    {
        if let Some(arguments) = command["argv"].as_array_mut() {
            for argument in arguments {
                if argument == "7" {
                    *argument = json!("8");
                } else if let Some(value) = argument.as_str() {
                    *argument = json!(value.replace("success-7-", "success-8-"));
                }
            }
        }
    }
    write_json(&fixture.terminal_path(), &terminal)?;

    // Act
    let output = run_validator(&fixture)?;

    // Assert
    assert!(!output.status.success());
    Ok(())
}

#[test]
fn validator_rejects_initial_run_view_substitution() -> TestResult {
    // Arrange
    let fixture = successful_fixture()?;
    mutate_command_stdout(
        &fixture,
        "canonical-initial-view",
        &json!({"databaseId":8,"headSha":fixture.candidate,"event":"workflow_dispatch","status":"queued","conclusion":null,"url":"https://github.com/fixture/repository/actions/runs/8"}),
    )?;

    // Act
    let output = run_validator(&fixture)?;

    // Assert
    assert!(!output.status.success());
    Ok(())
}

#[test]
fn validator_rejects_terminal_run_view_substitution() -> TestResult {
    // Arrange
    let fixture = successful_fixture()?;
    mutate_command_stdout(
        &fixture,
        "canonical-inspect",
        &json!({"databaseId":8,"headSha":fixture.candidate,"event":"workflow_dispatch","status":"completed","conclusion":"success","url":"https://github.com/fixture/repository/actions/runs/8"}),
    )?;

    // Act
    let output = run_validator(&fixture)?;

    // Assert
    assert!(!output.status.success());
    Ok(())
}

#[test]
fn validator_rejects_dispatch_intent_journal_substitution() -> TestResult {
    // Arrange
    let fixture = successful_fixture()?;
    let directory = output_directory(&fixture);
    let intent_path = directory.join("dispatch-intent.json");
    let mut intent = read_json(&intent_path)?;
    intent["candidate_tree"] = json!("0000000000000000000000000000000000000000");
    write_json(&intent_path, &intent)?;
    let mut result = read_json(&directory.join("dispatch-result.json"))?;
    result["intent_sha256"] = json!(hash_file(&intent_path)?);
    write_json(&directory.join("dispatch-result.json"), &result)?;
    let mut terminal = read_json(&fixture.terminal_path())?;
    terminal["dispatch_intent"]["sha256"] = json!(hash_file(&intent_path)?);
    repair_result_digest(&fixture, &mut terminal)?;

    // Act
    let output = run_validator(&fixture)?;

    // Assert
    assert!(!output.status.success());
    Ok(())
}

#[test]
fn validator_rejects_dispatch_result_journal_substitution() -> TestResult {
    // Arrange
    let fixture = successful_fixture()?;
    let directory = output_directory(&fixture);
    let result_path = directory.join("dispatch-result.json");
    let mut result = read_json(&result_path)?;
    result["dispatch_url"] = json!("https://github.com/fixture/repository/actions/runs/8");
    write_json(&result_path, &result)?;
    let mut terminal = read_json(&fixture.terminal_path())?;
    repair_result_digest(&fixture, &mut terminal)?;

    // Act
    let output = run_validator(&fixture)?;

    // Assert
    assert!(!output.status.success());
    Ok(())
}
