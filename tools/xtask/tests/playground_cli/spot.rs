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
    command.args(["playground", "scene-spot", "--warmup", "0", "--steps", "1"]);

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
    let names: std::collections::BTreeSet<&str> = report["scenes"]
        .as_array()
        .expect("scenes array")
        .iter()
        .map(|scene| scene["scene"].as_str().expect("scene name"))
        .collect();
    assert_eq!(
        names,
        [
            "color-mixer",
            "float-or-sink",
            "fountain",
            "jelly-drop",
            "water-wheel",
        ]
        .into_iter()
        .collect()
    );
    assert!(!fixture.pair_json(FIRST_STAMP).is_file());
    fixture.cleanup()?;
    Ok(())
}
