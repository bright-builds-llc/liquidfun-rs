use std::fs;

use super::support::{FIRST_STAMP, RepositoryFixture, TestResult, stderr, stdout};

#[test]
fn dam_break_timers_persists_timers_json_without_pair() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "playground",
        "dam-break-timers",
        "--warmup",
        "0",
        "--steps",
        "1",
    ]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(
        output.status.success(),
        "stderr: {}\nstdout: {}",
        stderr(&output),
        stdout(&output)
    );
    let path = fixture
        .root
        .join("target/dam-break-perf")
        .join(FIRST_STAMP)
        .join("timers.json");
    let report: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    assert_eq!(report["kind"], "step_profiled_parents");
    assert_eq!(report["not_timing_authority"].as_bool(), Some(true));
    for token in ["particle_prepare", "particle_solve", "rigid_solve"] {
        assert!(
            report["parents"][token]["wall_ms"].as_f64().is_some(),
            "parents must contain `{token}`"
        );
    }
    assert!(!fixture.pair_json(FIRST_STAMP).is_file());
    assert!(!fixture.profile_gz(FIRST_STAMP).is_file());
    assert!(!stdout(&output).contains("Rust/C++"));
    fixture.cleanup()?;
    Ok(())
}
