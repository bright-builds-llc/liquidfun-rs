use std::fs;

use super::support::{FIRST_STAMP, RepositoryFixture, TestResult, stderr, stdout};

#[test]
fn justfile_keeps_the_one_line_playground_scene_spot_alias() {
    // Arrange
    let justfile = include_str!("../../../../justfile");

    // Act
    let recipe_start = justfile
        .find("playground-scene-spot:")
        .expect("justfile should contain the playground scene-spot recipe");
    let recipe = justfile[recipe_start..]
        .split("\n\n")
        .next()
        .expect("recipe should end at a blank line");

    // Assert
    assert_eq!(
        recipe,
        "playground-scene-spot:\n    cargo xtask playground scene-spot"
    );
    assert!(!justfile.contains("samply"));
    assert!(!justfile.to_ascii_lowercase().contains("cmake"));
    assert!(!justfile.contains("dhat"));
}

#[test]
fn scene_spot_persists_native_scene_spot_without_pair() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "playground",
        "scene-spot",
        "--warmup",
        "0",
        "--steps",
        "1",
        "--runs",
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
        .join("scene-spot.json");
    let report: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    assert_eq!(report["kind"], "native_scene_spot");
    assert_eq!(report["not_timing_authority"].as_bool(), Some(true));
    assert!(report.get("rust_over_cpp_ratio").is_none());
    assert_eq!(report["runs"], 1);
    let scenes = report["scenes"]
        .as_array()
        .expect("report should list scenes");
    assert!(
        scenes.iter().all(|scene| scene["fingerprint"]
            .as_str()
            .is_some_and(|hex| hex.len() == 16)),
        "every scene should carry a 16-digit fingerprint"
    );
    // The fake tool makes later catalog scenes slower, so the last one ranks first.
    let slowest_scene = report["scenes"]
        .as_array()
        .and_then(|scenes| scenes.last())
        .and_then(|scene| scene["scene"].as_str())
        .expect("report should list catalog scenes");
    let summary = stdout(&output);
    assert!(summary.contains("| Rank | Scene | Median ms/step |"));
    let maybe_first_row = summary.lines().find(|line| line.starts_with("| 1 |"));
    assert!(
        maybe_first_row.is_some_and(|row| row.contains(slowest_scene)),
        "stdout: {summary}"
    );
    assert!(!fixture.pair_json(FIRST_STAMP).is_file());
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn scene_spot_filters_requested_scenes() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args([
        "playground",
        "scene-spot",
        "--warmup",
        "0",
        "--steps",
        "1",
        "--runs",
        "1",
        "--scene",
        "tesla-valve",
        "--scene",
        "liquid-tumbler",
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
        .join("scene-spot.json");
    let report: serde_json::Value = serde_json::from_slice(&fs::read(path)?)?;
    let scenes = report["scenes"]
        .as_array()
        .expect("report should list scenes");
    let ids: Vec<&str> = scenes
        .iter()
        .filter_map(|scene| scene["scene"].as_str())
        .collect();
    assert_eq!(ids, ["liquid-tumbler", "tesla-valve"]);
    assert!(
        scenes.iter().all(|scene| scene["fingerprint"]
            .as_str()
            .is_some_and(|hex| hex.len() == 16)),
        "every scene should carry a 16-digit fingerprint"
    );
    assert!(stdout(&output).contains("| Rank | Scene | Median ms/step |"));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn scene_spot_rejects_unknown_scene() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let mut command = fixture.command()?;
    command.args(["playground", "scene-spot", "--scene", "not-a-scene"]);

    // Act
    let output = command.output()?;

    // Assert
    assert!(!output.status.success(), "stdout: {}", stdout(&output));
    assert!(
        stderr(&output).contains("not-a-scene"),
        "stderr: {}",
        stderr(&output)
    );
    let path = fixture
        .root
        .join("target/dam-break-perf")
        .join(FIRST_STAMP)
        .join("scene-spot.json");
    assert!(!path.exists());
    fixture.cleanup()?;
    Ok(())
}
