//! Command-level coverage for cross-record provenance validation.

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
        fs::create_dir_all(root.join("reference/artifacts"))?;
        fs::create_dir_all(root.join("third_party/liquidfun"))?;
        fs::write(root.join("reference/artifacts/sample.bin"), b"artifact")?;
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
        fixture.write_source_map(REVISION)?;
        fixture.write_manifest(&artifact_hash(), true)?;
        Ok(fixture)
    }

    fn write_source_map(&self, revision: &str) -> io::Result<()> {
        fs::write(
            self.root.join("reference/source-map.toml"),
            format!(
                "schema_version = 1\n\n[[mapping]]\nlocal_path = \"reference/upstream-lock.toml\"\nupstream_revision = \"{revision}\"\nupstream_path = \".\"\nderivation_kind = \"fixture\"\nalteration_summary = \"Fixture metadata only.\"\nnotice_class = \"provenance-only\"\n"
            ),
        )
    }

    fn write_manifest(&self, hash: &str, include_notice: bool) -> io::Result<()> {
        let notices = if include_notice {
            "[\"THIRD_PARTY_NOTICES.md\"]"
        } else {
            "[]"
        };
        fs::write(
            self.root.join("reference/artifacts/manifest.toml"),
            format!(
                "schema_version = 1\nrecord_schema_version = 1\noracle_revision = \"{REVISION}\"\nrecord_fields = [\"path\", \"sha256\", \"generator_revision\", \"oracle_revision\", \"preset\", \"compiler\", \"target\", \"flags\", \"notice_refs\", \"review_status\"]\n\n[[artifacts]]\npath = \"reference/artifacts/sample.bin\"\nsha256 = \"{hash}\"\ngenerator_revision = \"{REVISION}\"\noracle_revision = \"{REVISION}\"\npreset = \"oracle-debug\"\ncompiler = \"fixture-clang\"\ntarget = \"fixture-target\"\nflags = []\nnotice_refs = {notices}\nreview_status = \"reviewed\"\n"
            ),
        )
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
fn check_accepts_matching_provenance_records() -> TestResult {
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
fn check_rejects_source_map_revision_mismatch() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.write_source_map(WRONG_REVISION)?;

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
    fixture.write_manifest(&"0".repeat(64), true)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/hash");
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn check_rejects_missing_artifact_notice() -> TestResult {
    // Arrange
    let fixture = ProvenanceFixture::new()?;
    fixture.write_manifest(&artifact_hash(), false)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "provenance/notice");
    fixture.cleanup()?;
    Ok(())
}

fn artifact_hash() -> String {
    format!("{:x}", Sha256::digest(b"artifact"))
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
