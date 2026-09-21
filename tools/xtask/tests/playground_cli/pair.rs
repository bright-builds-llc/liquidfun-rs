use std::fs;

use super::support::{
    FIRST_STAMP, RepositoryFixture, SECOND_STAMP, TestResult, run_pair, stderr, stdout,
};

#[test]
fn dam_break_bench_persists_unprofiled_pair_without_profile_cmake_flags() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;

    // Act
    let output = run_pair(&fixture)?;

    // Assert
    assert!(
        output.status.success(),
        "stderr: {}\nstdout: {}",
        stderr(&output),
        stdout(&output)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(fixture.pair_json(FIRST_STAMP))?)?;
    assert_eq!(report["kind"], "unprofiled_pair");
    assert_eq!(report["timing_authority"], "unprofiled_wall_clock");
    assert_eq!(report["rust_over_cpp_ratio"].as_f64(), Some(300.0));
    assert_eq!(report["rust"]["wall_ms"].as_f64(), Some(300.0));
    assert_eq!(report["cpp"]["wall_ms"].as_f64(), Some(1.0));
    let markdown = fs::read_to_string(fixture.pair_md(FIRST_STAMP))?;
    assert!(markdown.contains("Rust/C++"));
    assert!(markdown.contains("Unreviewed local playground Dam Break sample"));
    assert!(stdout(&output).contains("Unreviewed local playground Dam Break sample"));
    assert!(
        stdout(&output).contains(&markdown),
        "stdout should include the pair.md Markdown table"
    );
    let cmake_args = fixture.cmake_arguments()?;
    assert!(cmake_args.iter().any(|argument| argument == "--target"));
    assert!(
        cmake_args
            .iter()
            .any(|argument| argument == "playground-dam-break-bench")
    );
    assert!(!cmake_args.iter().any(|argument| argument == "-g"));
    assert!(
        !cmake_args
            .iter()
            .any(|argument| argument.contains("REFERENCE_PROFILE_DEBUG_INFO"))
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn second_pair_in_the_same_unix_second_mints_a_new_stamp() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;
    let first = run_pair(&fixture)?;
    assert!(first.status.success(), "{}", stderr(&first));
    let first_json = fs::read(fixture.pair_json(FIRST_STAMP))?;

    // Act
    let second = run_pair(&fixture)?;

    // Assert
    assert!(second.status.success(), "{}", stderr(&second));
    let first_json_after = fs::read(fixture.pair_json(FIRST_STAMP))?;
    assert_eq!(first_json, first_json_after);
    assert!(fixture.pair_json(SECOND_STAMP).is_file());
    assert_ne!(fs::read(fixture.pair_json(SECOND_STAMP))?, first_json_after);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn justfile_keeps_the_one_line_dam_break_bench_alias() {
    // Arrange
    let justfile = include_str!("../../../../justfile");

    // Act
    let recipe_start = justfile
        .find("playground-dam-break-bench:")
        .expect("justfile should contain the playground Dam Break recipe");
    let recipe = justfile[recipe_start..]
        .split("\n\n")
        .next()
        .expect("recipe should end at a blank line");

    // Assert
    assert_eq!(
        recipe,
        "playground-dam-break-bench:\n    cargo xtask playground dam-break-bench"
    );
    assert!(!justfile.contains("samply"));
    assert!(!justfile.to_ascii_lowercase().contains("cmake"));
}
