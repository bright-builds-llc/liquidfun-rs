//! Command-level coverage for strict cross-record provenance validation.

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

const REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";
const WRONG_REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TRACE_PATH: &str = "reference/artifacts/traces/empty-world.jsonl";
const ARTIFACT_SCHEMAS: &str = "[artifact_schemas.phase11_evidence]\nschema_version = 1\nmanifest_file = \"phase11-v1.json\"\nidentity_file = \"identity.json\"\nprotocol_version = \"catalog-phase11-v1\"\ngenerator_version = \"phase11-evidence-v1\"\npromotion = \"exact-ref-same-run-only\"\n";
const RECORD_FIELDS: [&str; 27] = [
    "artifact_kind",
    "path",
    "sha256",
    "generator_revision",
    "request_sha256",
    "scenario_content_sha256",
    "scenario_sha256",
    "protocol_version",
    "scenario_schema_version",
    "trace_schema_version",
    "tolerance_profile_version",
    "tolerance_profile_sha256",
    "oracle_revision",
    "adapter_revision",
    "adapter_content_sha256",
    "build_identity_sha256",
    "preset",
    "compiler",
    "target",
    "flags",
    "source",
    "trace_payload_sha256",
    "failure_signature",
    "notice_refs",
    "reviewer",
    "reviewed_at",
    "review_status",
];
static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
static FAKE_GIT: OnceLock<Result<PathBuf, String>> = OnceLock::new();

type TestResult = Result<(), Box<dyn Error>>;

struct ProvenanceFixture {
    root: PathBuf,
}

impl ProvenanceFixture {
    fn new() -> io::Result<Self> {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = workspace_root().join(format!(
            "target/xtask-provenance-fixtures/{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("reference/artifacts/traces"))?;
        fs::create_dir_all(root.join("protocol/fixtures/accepted"))?;
        fs::create_dir_all(root.join("protocol/tolerances"))?;
        fs::create_dir_all(root.join("scenarios/phase-02"))?;
        fs::create_dir_all(root.join("scenarios/regressions"))?;
        fs::create_dir_all(root.join("third_party/liquidfun"))?;
        copy_workspace_file(
            &root,
            "protocol/fixtures/accepted/empty-world-request.jsonl",
        )?;
        copy_workspace_file(&root, "protocol/fixtures/accepted/empty-world-trace.jsonl")?;
        copy_workspace_file(&root, "protocol/tolerances/phase2-v1.toml")?;
        copy_workspace_file(&root, "scenarios/phase-02/empty-world.json")?;
        fs::copy(
            root.join("protocol/fixtures/accepted/empty-world-trace.jsonl"),
            root.join(TRACE_PATH),
        )?;
        fs::write(root.join("THIRD_PARTY_NOTICES.md"), "fixture notice\n")?;
        fs::write(
            root.join("reference/upstream-lock.toml"),
            format!(
                "schema_version = 1\nrepository = \"https://example.invalid/liquidfun.git\"\nrevision = \"{REVISION}\"\nrelease_tag = \"v1.1.0\"\nrelease_tag_object = \"{REVISION}\"\nrelease_commit = \"{REVISION}\"\nsubmodule_path = \"third_party/liquidfun\"\npatch_set = \"none\"\n"
            ),
        )?;
        fs::write(
            root.join("reference/discovery.json"),
            format!("{{\"schema_version\":1,\"oracle_revision\":\"{REVISION}\"}}\n"),
        )?;
        fs::write(
            root.join("reference/compatibility.json"),
            format!("{{\"schema_version\":1,\"oracle_revision\":\"{REVISION}\"}}\n"),
        )?;
        let fixture = Self { root };
        fixture.write_source_map(REVISION, TRACE_PATH)?;
        fixture.write_trace_manifest()?;
        Ok(fixture)
    }

    fn write_source_map(&self, revision: &str, artifact_path: &str) -> io::Result<()> {
        fs::write(
            self.root.join("reference/source-map.toml"),
            format!(
                "schema_version = 1\n\n[[mapping]]\nlocal_path = \"reference/upstream-lock.toml\"\nupstream_revision = \"{revision}\"\nupstream_path = \".\"\nderivation_kind = \"fixture\"\nalteration_summary = \"Fixture metadata only.\"\nnotice_class = \"provenance-only\"\n\n[[mapping]]\nlocal_path = \"{artifact_path}\"\nupstream_revision = \"{revision}\"\nupstream_path = \"liquidfun/Box2D/Box2D/Dynamics/b2World.cpp\"\nderivation_kind = \"reviewed-semantic-trace\"\nalteration_summary = \"Repository-authored semantic output; no upstream source is copied.\"\nnotice_class = \"provenance-only\"\n"
            ),
        )
    }

    fn write_trace_manifest(&self) -> io::Result<()> {
        let trace = fs::read(self.root.join(TRACE_PATH))?;
        let request = fs::read(
            self.root
                .join("protocol/fixtures/accepted/empty-world-request.jsonl"),
        )?;
        let scenario = fs::read(self.root.join("scenarios/phase-02/empty-world.json"))?;
        let canonical_scenario = scenario.strip_suffix(b"\n").unwrap_or(&scenario);
        self.write_manifest(format!(
            "schema_version = 2\nrecord_schema_version = 2\noracle_revision = \"{REVISION}\"\nrecord_fields = {RECORD_FIELDS:?}\n\n{ARTIFACT_SCHEMAS}\n[[artifacts]]\nartifact_kind = \"trace\"\npath = \"{TRACE_PATH}\"\nsha256 = \"{}\"\ngenerator_revision = \"{REVISION}\"\nrequest_sha256 = \"{}\"\nscenario_content_sha256 = \"{}\"\nscenario_sha256 = \"49642b2ea489384be7850f595269e6366003f7bfab260ab1f9270a9cfcb0fd9e\"\nprotocol_version = 1\nscenario_schema_version = 1\ntrace_schema_version = 1\ntolerance_profile_version = 1\ntolerance_profile_sha256 = \"177db8c2ff3011653fc27f74339fe144df5936bb078db85f28402d317e6622c3\"\noracle_revision = \"{REVISION}\"\nadapter_revision = \"fixture-adapter-v1\"\nadapter_content_sha256 = \"c7f36eaf2f184a36b9c9a04636d3e22785d815c4948d55d0b3cbf44ee7245fc8\"\nbuild_identity_sha256 = \"56b1b4d459fef5fc7abcd7072566ac92732284e73f99c79885a80770a9f0fafd\"\npreset = \"oracle-debug\"\ncompiler = \"Clang 22.1.8\"\ntarget = \"x86_64-unknown-linux-gnu\"\nflags = [\"-O0 -g\", \"-lc++\"]\ntrace_payload_sha256 = \"296b7c008ce4e257f7f3f41273599e6fc51a3df68b1d52898738a2fc5f5c558b\"\nnotice_refs = [\"THIRD_PARTY_NOTICES.md\"]\nreviewer = \"fixture-reviewer\"\nreviewed_at = \"2026-07-10T00:00:00Z\"\nreview_status = \"reviewed\"\nsource = {{ kind = \"named\", name = \"empty-world\" }}\n",
            sha256(&trace),
            sha256(&request),
            sha256(canonical_scenario),
        ))
    }

    fn write_regression_manifest(&self) -> io::Result<()> {
        let path = "scenarios/regressions/empty-world.json";
        let scenario = fs::read(self.root.join("scenarios/phase-02/empty-world.json"))?;
        let canonical_scenario = scenario.strip_suffix(b"\n").unwrap_or(&scenario);
        let request = fs::read(
            self.root
                .join("protocol/fixtures/accepted/empty-world-request.jsonl"),
        )?;
        fs::write(self.root.join(path), canonical_scenario)?;
        self.write_source_map(REVISION, path)?;
        self.write_manifest(format!(
            "schema_version = 2\nrecord_schema_version = 2\noracle_revision = \"{REVISION}\"\nrecord_fields = {RECORD_FIELDS:?}\n\n{ARTIFACT_SCHEMAS}\n[[artifacts]]\nartifact_kind = \"regression\"\npath = \"{path}\"\nsha256 = \"{}\"\ngenerator_revision = \"{REVISION}\"\nrequest_sha256 = \"{}\"\nscenario_content_sha256 = \"{}\"\nscenario_sha256 = \"49642b2ea489384be7850f595269e6366003f7bfab260ab1f9270a9cfcb0fd9e\"\nprotocol_version = 1\nscenario_schema_version = 1\ntrace_schema_version = 1\ntolerance_profile_version = 1\ntolerance_profile_sha256 = \"177db8c2ff3011653fc27f74339fe144df5936bb078db85f28402d317e6622c3\"\noracle_revision = \"{REVISION}\"\nadapter_revision = \"fixture-adapter-v1\"\nadapter_content_sha256 = \"c7f36eaf2f184a36b9c9a04636d3e22785d815c4948d55d0b3cbf44ee7245fc8\"\nbuild_identity_sha256 = \"56b1b4d459fef5fc7abcd7072566ac92732284e73f99c79885a80770a9f0fafd\"\npreset = \"oracle-debug\"\ncompiler = \"Clang 22.1.8\"\ntarget = \"x86_64-unknown-linux-gnu\"\nflags = [\"-O0 -g\", \"-lc++\"]\nfailure_signature = {{ checkpoint_id = \"after-step-1\", phase = \"after-step-1\", semantic_path = \"world_counts.bodies\", kind = \"exact\" }}\nnotice_refs = [\"THIRD_PARTY_NOTICES.md\"]\nreviewer = \"fixture-reviewer\"\nreviewed_at = \"2026-07-10T00:00:00Z\"\nreview_status = \"reviewed\"\nsource = {{ kind = \"named\", name = \"empty-world\" }}\n",
            sha256(canonical_scenario),
            sha256(&request),
            sha256(canonical_scenario),
        ))
    }

    fn write_manifest(&self, contents: String) -> io::Result<()> {
        fs::write(
            self.root.join("reference/artifacts/manifest.toml"),
            contents,
        )
    }

    fn mutate_manifest(&self, from: &str, to: &str) -> io::Result<()> {
        let path = self.root.join("reference/artifacts/manifest.toml");
        let contents = fs::read_to_string(&path)?;
        assert!(contents.contains(from), "fixture mutation source is absent");
        fs::write(path, contents.replacen(from, to, 1))
    }

    fn duplicate_artifact_record(&self) -> io::Result<()> {
        let path = self.root.join("reference/artifacts/manifest.toml");
        let contents = fs::read_to_string(&path)?;
        let marker = "[[artifacts]]";
        let index = contents
            .find(marker)
            .expect("fixture artifact record should exist");
        let duplicate = &contents[index..];
        fs::write(&path, format!("{contents}\n{duplicate}"))
    }

    fn command(&self) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["provenance", "check"])
            .env("LIQUIDFUN_XTASK_ROOT", &self.root)
            .env("LIQUIDFUN_XTASK_GIT", fake_git()?)
            .env("LIQUIDFUN_TEST_REVISION", REVISION)
            .output()
    }

    fn cleanup(self) -> io::Result<()> {
        fs::remove_dir_all(self.root)
    }
}

#[test]
fn check_accepts_complete_trace_record() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_accepts_complete_regression_record() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.write_regression_manifest()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_source_map_revision_mismatch() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.write_source_map(WRONG_REVISION, TRACE_PATH)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/revision");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_artifact_sha_mismatch() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.mutate_manifest("sha256 = \"60d217", "sha256 = \"00d217")?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/hash");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_unknown_manifest_field() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.mutate_manifest(
        "review_status = \"reviewed\"",
        "review_status = \"reviewed\"\nunknown = true",
    )?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/schema");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_duplicate_manifest_field() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.mutate_manifest(
        &format!("path = \"{TRACE_PATH}\""),
        &format!("path = \"{TRACE_PATH}\"\npath = \"{TRACE_PATH}\""),
    )?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/schema");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_duplicate_artifact_path() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.duplicate_artifact_record()?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/schema");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_missing_trace_field() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.mutate_manifest(
        "trace_payload_sha256 = \"296b7c008ce4e257f7f3f41273599e6fc51a3df68b1d52898738a2fc5f5c558b\"\n",
        "",
    )?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/schema");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_missing_common_field() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.mutate_manifest("reviewer = \"fixture-reviewer\"\n", "")?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/schema");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_wrong_scenario_hash() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.mutate_manifest("scenario_sha256 = \"49642b", "scenario_sha256 = \"09642b")?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/hash");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_wrong_tolerance_profile_hash() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.mutate_manifest(
        "tolerance_profile_sha256 = \"177db8",
        "tolerance_profile_sha256 = \"077db8",
    )?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/policy");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_unsupported_record_version() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.mutate_manifest("trace_schema_version = 1", "trace_schema_version = 2")?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/schema");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_wrong_adapter_identity() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.mutate_manifest(
        "adapter_revision = \"fixture-adapter-v1\"",
        "adapter_revision = \"wrong-adapter\"",
    )?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/identity");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_unreviewed_artifact() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.mutate_manifest(
        "review_status = \"reviewed\"",
        "review_status = \"pending\"",
    )?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/review");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_non_utc_review_timestamp() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.mutate_manifest(
        "reviewed_at = \"2026-07-10T00:00:00Z\"",
        "reviewed_at = \"2026-07-10 00:00:00\"",
    )?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/review");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_traversal_artifact_path() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.mutate_manifest(TRACE_PATH, "../outside.jsonl")?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/path");
    fixture.cleanup()?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn check_rejects_intermediate_symlink_escape_before_hashing() -> TestResult {
    use std::os::unix::fs::symlink;

    // Arrange
    let fixture = ProvenanceFixture::new()?;
    let outside = fixture.root.with_extension("outside");
    fs::create_dir_all(&outside)?;
    fs::copy(
        fixture.root.join(TRACE_PATH),
        outside.join("empty-world.jsonl"),
    )?;
    symlink(&outside, fixture.root.join("reference/artifacts/link"))?;
    fixture.mutate_manifest(TRACE_PATH, "reference/artifacts/link/empty-world.jsonl")?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/path");
    fixture.cleanup()?;
    fs::remove_dir_all(outside)?;
    Ok(())
}

#[test]
fn check_rejects_missing_artifact_notice() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.mutate_manifest(
        "notice_refs = [\"THIRD_PARTY_NOTICES.md\"]",
        "notice_refs = []",
    )?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/notice");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_unknown_generator_revision() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.mutate_manifest(
        &format!("generator_revision = \"{REVISION}\""),
        &format!("generator_revision = \"{WRONG_REVISION}\""),
    )?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/generator");
    fixture.cleanup()?;
    Ok(())
}

fn copy_workspace_file(root: &Path, relative: &str) -> io::Result<()> {
    fs::copy(workspace_root().join(relative), root.join(relative))?;
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn fake_git() -> io::Result<&'static Path> {
    let result = FAKE_GIT.get_or_init(compile_fake_git);
    match result {
        Ok(path) => Ok(path),
        Err(message) => Err(io::Error::other(message.clone())),
    }
}

fn compile_fake_git() -> Result<PathBuf, String> {
    let output_dir = workspace_root().join(format!(
        "target/xtask-provenance-tools/{}",
        std::process::id()
    ));
    fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake_upstream_tool.rs");
    let executable = output_dir.join(executable_name("fake-git"));
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let output = Command::new(rustc)
        .arg(source)
        .arg("--edition=2024")
        .arg("-o")
        .arg(&executable)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    Ok(executable)
}

fn executable_name(stem: &str) -> String {
    format!("{stem}{}", env::consts::EXE_SUFFIX)
}

fn assert_success(output: &Output) {
    assert!(output.status.success(), "{}", stderr(output));
}

fn assert_failure_category(output: &Output, category: &str) {
    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert!(
        stderr(output).contains(category),
        "expected `{category}` in stderr:\n{}",
        stderr(output)
    );
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}
