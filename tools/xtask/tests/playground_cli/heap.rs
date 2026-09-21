use std::fs;
use std::path::PathBuf;
use std::process::Output;

use super::support::{
    FIRST_STAMP, RepositoryFixture, SECOND_STAMP, TestResult, run_pair, stderr, stdout,
};

const SECOND_STAMP_UNIX: &str = "1000000001";
const THIRD_STAMP: &str = "2001-09-09T01-46-42Z";
const THIRD_STAMP_UNIX: &str = "1000000002";
const SYMS_NAME: &str = "rust.json.syms.json";
const VEC_SYMS_JSON: &str = r#"{"string_table":["alloc::vec::Vec"]}"#;

fn prepare_pair_and_profile(maybe_sidecar: Option<&str>) -> std::io::Result<RepositoryFixture> {
    let fixture = RepositoryFixture::new()?;
    let pair = run_pair(&fixture)?;
    assert!(pair.status.success(), "{}", stderr(&pair));
    let mut profile_cmd = fixture.command()?;
    profile_cmd
        .env("LIQUIDFUN_XTASK_STAMP_UNIX", SECOND_STAMP_UNIX)
        .args([
            "playground",
            "dam-break-profile",
            "--warmup",
            "0",
            "--steps",
            "1",
        ]);
    let profile = profile_cmd.output()?;
    assert!(profile.status.success(), "{}", stderr(&profile));
    if let Some(sidecar) = maybe_sidecar {
        fs::write(
            fixture
                .root
                .join("target/dam-break-perf")
                .join(SECOND_STAMP)
                .join(SYMS_NAME),
            sidecar,
        )?;
    }
    Ok(fixture)
}

fn run_heap(fixture: &RepositoryFixture, stamp_unix: &str) -> std::io::Result<Output> {
    let mut command = fixture.command()?;
    command.env("LIQUIDFUN_XTASK_STAMP_UNIX", stamp_unix).args([
        "playground",
        "dam-break-heap",
        "--warmup",
        "0",
        "--steps",
        "1",
    ]);
    command.output()
}

fn stamp_dir(fixture: &RepositoryFixture, stamp: &str) -> PathBuf {
    fixture.root.join("target/dam-break-perf").join(stamp)
}

fn heap_dump_paths(fixture: &RepositoryFixture) -> std::io::Result<Vec<PathBuf>> {
    let evidence = fixture.root.join("target/dam-break-perf");
    if !evidence.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(evidence)? {
        let path = entry?.path().join("dhat-heap.json");
        if path.is_file() {
            files.push(path);
        }
    }
    Ok(files)
}

#[test]
fn justfile_keeps_the_one_line_dam_break_heap_alias() {
    // Arrange
    let justfile = include_str!("../../../../justfile");

    // Act
    let recipe_start = justfile
        .find("playground-dam-break-heap:")
        .expect("justfile should contain the playground Dam Break heap recipe");
    let recipe = justfile[recipe_start..]
        .split("\n\n")
        .next()
        .expect("recipe should end at a blank line");

    // Assert
    assert_eq!(
        recipe,
        "playground-dam-break-heap:\n    cargo xtask playground dam-break-heap"
    );
}

#[test]
fn dam_break_heap_persists_sibling_dump_marked_not_timing_authority() -> TestResult {
    // Arrange
    let fixture = prepare_pair_and_profile(Some(VEC_SYMS_JSON))?;

    // Act
    let output = run_heap(&fixture, THIRD_STAMP_UNIX)?;

    // Assert
    assert!(
        output.status.success(),
        "stderr: {}\nstdout: {}",
        stderr(&output),
        stdout(&output)
    );
    let dump = stamp_dir(&fixture, THIRD_STAMP).join("dhat-heap.json");
    let dump_bytes = fs::read(&dump)?;
    assert!(
        dump_bytes.len() >= 16,
        "dhat-heap.json should be nonempty, got {} bytes",
        dump_bytes.len()
    );
    let identity: serde_json::Value = serde_json::from_slice(&fs::read(
        stamp_dir(&fixture, THIRD_STAMP).join("heap-identity.json"),
    )?)?;
    assert_eq!(identity["kind"], "dhat_heap");
    assert_eq!(identity["not_timing_authority"].as_bool(), Some(true));
    assert_eq!(identity["cargo_profile"], "profiling");
    let features = identity["features"]
        .as_array()
        .expect("features should be an array");
    assert!(
        features
            .iter()
            .any(|value| value.as_str() == Some("dhat-heap")),
        "features should include dhat-heap: {features:?}"
    );
    let stamp_name = dump
        .parent()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str());
    assert_ne!(stamp_name, Some(FIRST_STAMP));
    assert_ne!(stamp_name, Some(SECOND_STAMP));
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn dam_break_heap_cargo_uses_profiling_dhat_heap_and_not_release() -> TestResult {
    // Arrange
    let fixture = prepare_pair_and_profile(Some(VEC_SYMS_JSON))?;

    // Act
    let output = run_heap(&fixture, THIRD_STAMP_UNIX)?;

    // Assert
    assert!(
        output.status.success(),
        "stderr: {}\nstdout: {}",
        stderr(&output),
        stdout(&output)
    );
    let cargo_args = fs::read_to_string(fixture.cargo_marker())?;
    assert!(
        cargo_args.contains("--profile") && cargo_args.contains("profiling"),
        "cargo should run with --profile profiling, got `{cargo_args}`"
    );
    assert!(
        cargo_args.contains("--features") && cargo_args.contains("dhat-heap"),
        "cargo should enable --features dhat-heap, got `{cargo_args}`"
    );
    assert!(
        cargo_args.contains("--bin") && cargo_args.contains("dam-break-bench"),
        "cargo should target --bin dam-break-bench, got `{cargo_args}`"
    );
    assert!(
        !cargo_args.contains("--release"),
        "heap cargo invocation must not pass --release, got `{cargo_args}`"
    );
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn dam_break_heap_leaves_source_profile_stamp_without_dump() -> TestResult {
    // Arrange
    let fixture = prepare_pair_and_profile(Some(VEC_SYMS_JSON))?;

    // Act
    let output = run_heap(&fixture, THIRD_STAMP_UNIX)?;

    // Assert
    assert!(
        output.status.success(),
        "stderr: {}\nstdout: {}",
        stderr(&output),
        stdout(&output)
    );
    assert!(
        !stamp_dir(&fixture, SECOND_STAMP)
            .join("dhat-heap.json")
            .is_file()
    );
    let dumps = heap_dump_paths(&fixture)?;
    assert_eq!(dumps.len(), 1, "expected one sibling dump, got {dumps:?}");
    assert_eq!(
        dumps[0].parent().and_then(|path| path.file_name()),
        Some(std::ffi::OsStr::new(THIRD_STAMP))
    );
    fixture.cleanup()?;
    Ok(())
}
