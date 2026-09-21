use std::fs;

use super::support::{
    FIRST_STAMP, RepositoryFixture, SECOND_STAMP, TestResult, command_strings, run_profile,
    rust_profiles, stderr, stdout,
};

#[test]
fn justfile_adds_one_line_dam_break_profile_alias() {
    // Arrange
    let justfile = include_str!("../../../../justfile");

    // Act
    let recipe_start = justfile
        .find("playground-dam-break-profile:")
        .expect("justfile should contain the playground Dam Break profile recipe");
    let recipe = justfile[recipe_start..]
        .split("\n\n")
        .next()
        .expect("recipe should end at a blank line");

    // Assert
    assert_eq!(
        recipe,
        "playground-dam-break-profile:\n    cargo xtask playground dam-break-profile"
    );
    let timers_start = justfile
        .find("playground-dam-break-timers:")
        .expect("justfile should contain the playground Dam Break timers recipe");
    let timers = justfile[timers_start..]
        .split("\n\n")
        .next()
        .expect("recipe should end at a blank line");
    assert_eq!(
        timers,
        "playground-dam-break-timers:\n    cargo xtask playground dam-break-timers"
    );
}

#[test]
fn dam_break_profile_persists_rust_profile_without_pair_json() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;

    // Act
    let output = run_profile(&fixture)?;

    // Assert
    assert!(
        output.status.success(),
        "stderr: {}\nstdout: {}",
        stderr(&output),
        stdout(&output)
    );
    let gzip = fs::read(fixture.profile_gz(FIRST_STAMP))?;
    assert!(
        gzip.len() >= 16,
        "rust.json.gz should be nonempty, got {} bytes",
        gzip.len()
    );
    let identity: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.profile_identity(FIRST_STAMP))?)?;
    assert_eq!(identity["kind"], "samply_cpu");
    assert_eq!(identity["not_timing_authority"].as_bool(), Some(true));
    assert_eq!(identity["cargo_profile"], "profiling");
    assert_eq!(identity["samply_version"], "0.13.1");
    assert_eq!(identity["output"], "rust.json.gz");
    assert_eq!(identity["warmup_included_in_samples"].as_bool(), Some(true));
    let command = command_strings(&identity);
    assert!(
        command.iter().any(|argument| argument == "--save-only"),
        "command should include --save-only: {command:?}"
    );
    assert!(
        command
            .iter()
            .any(|argument| argument == "--unstable-presymbolicate"),
        "command should include --unstable-presymbolicate: {command:?}"
    );
    assert!(
        command
            .iter()
            .any(|argument| argument.contains("dam-break-bench")),
        "command should wrap dam-break-bench: {command:?}"
    );
    assert!(
        command.first().is_some_and(|program| program != "cargo"),
        "samply argv must not start with cargo: {command:?}"
    );
    assert!(
        !command.iter().any(|argument| argument == "cargo"),
        "samply argv must not record cargo: {command:?}"
    );
    assert!(!fixture.pair_json(FIRST_STAMP).is_file());
    let cargo_args = fs::read_to_string(fixture.cargo_marker())?;
    assert!(
        cargo_args.contains("--profile") && cargo_args.contains("profiling"),
        "cargo should rebuild with --profile profiling, got `{cargo_args}`"
    );
    assert!(
        cargo_args.contains("--bin") && cargo_args.contains("dam-break-bench"),
        "cargo should target --bin dam-break-bench, got `{cargo_args}`"
    );
    assert!(
        !cargo_args.contains("--release"),
        "profile cargo invocation must not pass --release, got `{cargo_args}`"
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn missing_samply_fails_closed_without_placeholder_gzip() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command
        .env(
            "LIQUIDFUN_XTASK_SAMPLY",
            "/nonexistent/liquidfun-samply-missing",
        )
        .args([
            "playground",
            "dam-break-profile",
            "--warmup",
            "0",
            "--steps",
            "1",
        ]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(
        !output.status.success(),
        "missing samply must fail closed, stdout: {}",
        stdout(&output)
    );
    let display = stderr(&output);
    assert!(
        display.contains("0.13.1"),
        "stderr `{display}` should pin samply 0.13.1"
    );
    assert!(
        display.contains("cargo install --locked samply --version 0.13.1"),
        "stderr `{display}` should include cargo install --locked"
    );
    assert!(
        !display.to_ascii_lowercase().contains("skip"),
        "stderr `{display}` must not mention skipping"
    );
    assert!(
        rust_profiles(&fixture)?.is_empty(),
        "missing samply must not write rust.json.gz"
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn second_profile_in_the_same_unix_second_mints_a_new_stamp() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let first = run_profile(&fixture)?;
    assert!(first.status.success(), "{}", stderr(&first));
    let first_gz = fs::read(fixture.profile_gz(FIRST_STAMP))?;
    let first_identity = fs::read(fixture.profile_identity(FIRST_STAMP))?;

    // Act
    let second = run_profile(&fixture)?;

    // Assert
    assert!(second.status.success(), "{}", stderr(&second));
    assert_eq!(fs::read(fixture.profile_gz(FIRST_STAMP))?, first_gz);
    assert_eq!(
        fs::read(fixture.profile_identity(FIRST_STAMP))?,
        first_identity
    );
    assert!(fixture.profile_gz(SECOND_STAMP).is_file());
    assert!(fixture.profile_identity(SECOND_STAMP).is_file());
    assert!(!fixture.pair_json(FIRST_STAMP).is_file());
    assert!(!fixture.pair_json(SECOND_STAMP).is_file());
    fixture.cleanup()?;
    Ok(())
}
