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
const SYMS_JSON: &str = r#"{"string_table":["alloc::vec::Vec"]}"#;
const MISMATCHED_HEAD: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

struct PairProfileFixture {
    fixture: RepositoryFixture,
    pair_json: Vec<u8>,
    pair_md: Vec<u8>,
    profile_blob: Vec<u8>,
    syms: Vec<u8>,
}

fn prepare_pair_and_profile() -> std::io::Result<PairProfileFixture> {
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
    let syms_path = fixture
        .root
        .join("target/dam-break-perf")
        .join(SECOND_STAMP)
        .join(SYMS_NAME);
    fs::write(&syms_path, SYMS_JSON)?;
    Ok(PairProfileFixture {
        pair_json: fs::read(fixture.pair_json(FIRST_STAMP))?,
        pair_md: fs::read(fixture.pair_md(FIRST_STAMP))?,
        profile_blob: fs::read(fixture.profile_gz(SECOND_STAMP))?,
        syms: fs::read(&syms_path)?,
        fixture,
    })
}

fn run_bundle(
    fixture: &RepositoryFixture,
    stamp_unix: &str,
    extra_args: &[&str],
) -> std::io::Result<Output> {
    let mut command = fixture.command()?;
    command
        .env("LIQUIDFUN_XTASK_STAMP_UNIX", stamp_unix)
        .args(["playground", "dam-break-audit-bundle"])
        .args(extra_args);
    command.output()
}

fn dest_stamp(fixture: &RepositoryFixture, stamp: &str) -> PathBuf {
    fixture.root.join("target/dam-break-perf").join(stamp)
}

fn pair_profile_flags() -> [&'static str; 4] {
    ["--pair-stamp", FIRST_STAMP, "--profile-stamp", SECOND_STAMP]
}

fn audit_identity_dirs(fixture: &RepositoryFixture) -> std::io::Result<Vec<PathBuf>> {
    let evidence = fixture.root.join("target/dam-break-perf");
    if !evidence.exists() {
        return Ok(Vec::new());
    }
    let mut dirs = Vec::new();
    for entry in fs::read_dir(evidence)? {
        let path = entry?.path();
        if path.is_dir() && path.join("audit-bundle-identity.json").is_file() {
            dirs.push(path);
        }
    }
    Ok(dirs)
}

fn assert_fail_closed(output: &Output, fixture: &RepositoryFixture) -> TestResult {
    assert!(
        !output.status.success(),
        "must fail closed, stdout: {}",
        stdout(output)
    );
    let display = stderr(output);
    assert!(
        !display.to_ascii_lowercase().contains("skip"),
        "stderr `{display}` must not mention skipping"
    );
    assert!(
        display.contains("stamp") || display.contains("bundle"),
        "stderr `{display}` should mention stamp or bundle"
    );
    let identities = audit_identity_dirs(fixture)?;
    assert!(
        identities.is_empty(),
        "must not write audit-bundle-identity.json: {identities:?}"
    );
    assert!(!dest_stamp(fixture, THIRD_STAMP).join("pair.json").is_file());
    Ok(())
}

#[test]
fn dam_break_audit_bundle_copies_pair_gzip_and_syms_into_a_new_stamp() -> TestResult {
    // Arrange
    let prepared = prepare_pair_and_profile()?;

    // Act
    let output = run_bundle(&prepared.fixture, THIRD_STAMP_UNIX, &pair_profile_flags())?;

    // Assert
    assert!(
        output.status.success(),
        "stderr: {}\nstdout: {}",
        stderr(&output),
        stdout(&output)
    );
    let dest = dest_stamp(&prepared.fixture, THIRD_STAMP);
    assert!(dest.is_dir(), "bundle should mint {THIRD_STAMP}");
    assert_ne!(
        dest.file_name().and_then(|name| name.to_str()),
        Some(FIRST_STAMP)
    );
    assert_ne!(
        dest.file_name().and_then(|name| name.to_str()),
        Some(SECOND_STAMP)
    );
    let copied_pair = fs::read(dest.join("pair.json"))?;
    let copied_md = fs::read(dest.join("pair.md"))?;
    let copied_blob = fs::read(dest.join("rust.json.gz"))?;
    let copied_syms = fs::read(dest.join(SYMS_NAME))?;
    assert!(!copied_pair.is_empty());
    assert!(!copied_md.is_empty());
    assert!(!copied_blob.is_empty());
    assert!(!copied_syms.is_empty());
    assert_eq!(copied_pair, prepared.pair_json);
    assert_eq!(copied_md, prepared.pair_md);
    assert_eq!(copied_blob, prepared.profile_blob);
    assert_eq!(copied_syms, prepared.syms);
    assert_eq!(
        fs::read(prepared.fixture.pair_json(FIRST_STAMP))?,
        prepared.pair_json
    );
    assert_eq!(
        fs::read(prepared.fixture.pair_md(FIRST_STAMP))?,
        prepared.pair_md
    );
    assert_eq!(
        fs::read(prepared.fixture.profile_gz(SECOND_STAMP))?,
        prepared.profile_blob
    );
    assert_eq!(
        fs::read(
            prepared
                .fixture
                .root
                .join("target/dam-break-perf")
                .join(SECOND_STAMP)
                .join(SYMS_NAME)
        )?,
        prepared.syms
    );
    let identity: serde_json::Value =
        serde_json::from_slice(&fs::read(dest.join("audit-bundle-identity.json"))?)?;
    assert_eq!(identity["kind"], "audit_bundle");
    assert_eq!(identity["not_timing_authority"].as_bool(), Some(true));
    assert_eq!(identity["timing_authority"], "unprofiled_wall_clock");
    assert_eq!(identity["profile_blob"], "rust.json.gz");
    assert_eq!(identity["source_pair_stamp"], FIRST_STAMP);
    assert_eq!(identity["source_profile_stamp"], SECOND_STAMP);
    let copied = identity["copied"]
        .as_array()
        .expect("copied should be an array");
    let names: Vec<&str> = copied
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect();
    assert!(names.contains(&"pair.json"));
    assert!(names.contains(&"pair.md"));
    assert!(names.contains(&"rust.json.gz"));
    assert!(names.contains(&SYMS_NAME));
    prepared.fixture.cleanup()?;
    Ok(())
}

#[test]
fn justfile_keeps_the_one_line_dam_break_audit_bundle_alias() {
    // Arrange
    let justfile = include_str!("../../../../justfile");

    // Act
    let recipe_start = justfile
        .find("playground-dam-break-audit-bundle:")
        .expect("justfile should contain the playground Dam Break audit-bundle recipe");
    let recipe = justfile[recipe_start..]
        .split("\n\n")
        .next()
        .expect("recipe should end at a blank line");

    // Assert
    assert_eq!(
        recipe,
        "playground-dam-break-audit-bundle:\n    cargo xtask playground dam-break-audit-bundle"
    );
}

#[test]
fn dam_break_audit_bundle_rejects_git_head_mismatch() -> TestResult {
    // Arrange
    let prepared = prepare_pair_and_profile()?;
    let mut identity: serde_json::Value =
        serde_json::from_slice(&fs::read(prepared.fixture.profile_identity(SECOND_STAMP))?)?;
    identity["git_head"] = serde_json::Value::String(String::from(MISMATCHED_HEAD));
    fs::write(
        prepared.fixture.profile_identity(SECOND_STAMP),
        serde_json::to_vec(&identity)?,
    )?;

    // Act
    let output = run_bundle(&prepared.fixture, THIRD_STAMP_UNIX, &pair_profile_flags())?;

    // Assert
    assert_fail_closed(&output, &prepared.fixture)?;
    prepared.fixture.cleanup()?;
    Ok(())
}

#[test]
fn dam_break_audit_bundle_rejects_missing_rust_json_gz() -> TestResult {
    // Arrange
    let prepared = prepare_pair_and_profile()?;
    fs::remove_file(prepared.fixture.profile_gz(SECOND_STAMP))?;

    // Act
    let output = run_bundle(&prepared.fixture, THIRD_STAMP_UNIX, &pair_profile_flags())?;

    // Assert
    assert_fail_closed(&output, &prepared.fixture)?;
    prepared.fixture.cleanup()?;
    Ok(())
}

#[test]
fn dam_break_audit_bundle_rejects_pair_stamp_path_traversal() -> TestResult {
    // Arrange
    let fixture = RepositoryFixture::new()?;

    // Act
    let parent = run_bundle(&fixture, THIRD_STAMP_UNIX, &["--pair-stamp", "../etc"])?;
    let colon = run_bundle(
        &fixture,
        THIRD_STAMP_UNIX,
        &["--pair-stamp", "2001-09-09T01:46:40Z"],
    )?;

    // Assert
    assert_fail_closed(&parent, &fixture)?;
    assert_fail_closed(&colon, &fixture)?;
    assert!(!fixture.root.join("target/dam-break-perf/etc").exists());
    assert!(!fixture.root.join("etc").exists());
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn dam_break_audit_bundle_rejects_samply_in_pair_json() -> TestResult {
    // Arrange
    let prepared = prepare_pair_and_profile()?;
    let mut pair: serde_json::Value = serde_json::from_slice(&prepared.pair_json)?;
    pair["samply"] = serde_json::Value::Bool(true);
    fs::write(
        prepared.fixture.pair_json(FIRST_STAMP),
        serde_json::to_vec(&pair)?,
    )?;

    // Act
    let output = run_bundle(&prepared.fixture, THIRD_STAMP_UNIX, &pair_profile_flags())?;

    // Assert
    assert_fail_closed(&output, &prepared.fixture)?;
    assert!(!dest_stamp(&prepared.fixture, THIRD_STAMP).is_dir());
    prepared.fixture.cleanup()?;
    Ok(())
}
