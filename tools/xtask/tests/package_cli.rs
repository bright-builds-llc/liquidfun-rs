//! Command-level coverage for packaged-crate isolation and archive confinement.

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{self, Cursor};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

use flate2::Compression;
use flate2::write::GzEncoder;

static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);
static FAKE_CARGO: OnceLock<Result<PathBuf, String>> = OnceLock::new();

type TestResult = Result<(), Box<dyn Error>>;

#[derive(Clone, Copy)]
enum ArchiveCase {
    Valid,
    ForbiddenContent,
    ForbiddenGraphics,
    ForbiddenNativeSource,
    ForbiddenTestbed,
    ParentTraversal,
    AbsolutePath,
}

#[test]
fn verify_rejects_native_source_extensions() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::ForbiddenNativeSource)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/forbidden-content");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn verify_rejects_graphics_assets() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::ForbiddenGraphics)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/forbidden-content");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn verify_rejects_testbed_content() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::ForbiddenTestbed)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/forbidden-content");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

struct PackageFixture {
    root: PathBuf,
    archive: PathBuf,
    metadata: PathBuf,
    cargo_marker: PathBuf,
}

impl PackageFixture {
    fn new(case: ArchiveCase) -> io::Result<Self> {
        let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        let root = workspace_root().join(format!(
            "target/xtask-package-fixtures/{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("crates/liquidfun"))?;
        fs::write(
            root.join("crates/liquidfun/Cargo.toml"),
            "[package]\nname = \"liquidfun\"\nversion = \"0.0.0\"\n",
        )?;
        fs::write(root.join("LICENSE"), "fixture license\n")?;
        let metadata = root.join("metadata.json");
        fs::write(&metadata, valid_metadata(&root))?;
        let archive = root.join("fixture.crate");
        write_archive(&archive, case)?;
        Ok(Self {
            cargo_marker: root.join("cargo-called"),
            root,
            archive,
            metadata,
        })
    }

    fn command(&self) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["package", "verify"])
            .env("LIQUIDFUN_XTASK_ROOT", &self.root)
            .env("LIQUIDFUN_XTASK_TEST_PACKAGE_ARCHIVE", &self.archive)
            .env("LIQUIDFUN_XTASK_TEST_METADATA", &self.metadata)
            .env("LIQUIDFUN_XTASK_CARGO", fake_cargo()?)
            .env("LIQUIDFUN_TEST_CARGO_MARKER", &self.cargo_marker)
            .env("LIQUIDFUN_TEST_ASSERT_PACKAGE_ISOLATION", "1")
            .env("DISPLAY", ":99")
            .env("WAYLAND_DISPLAY", "wayland-test")
            .output()
    }

    fn cleanup(self) -> io::Result<()> {
        fs::remove_dir_all(self.root)
    }
}

#[test]
fn ci_keeps_the_focused_headless_gate_submodule_free_and_before_visual_work() {
    // Arrange
    let workflow = include_str!("../../../.github/workflows/ci.yml");

    // Act
    let gate = workflow
        .find("Prove headless workflows and consumer package isolation")
        .expect("CI should name the focused gate");
    let package = workflow[gate..]
        .find("cargo xtask package verify")
        .expect("focused gate should verify the packaged crate");
    let maybe_visual = workflow.find("visual");

    // Assert
    assert!(workflow.matches("submodules: false").count() >= 3);
    assert!(workflow[gate..].contains("--test phase11_public_observability"));
    assert!(workflow[gate..].contains("--test headless_catalog"));
    assert!(workflow[gate..].contains("--test package_cli"));
    assert!(workflow.contains("cargo xtask inventory corpus check-snapshot"));
    assert!(workflow.contains("cargo xtask inventory corpus check-closure"));
    assert!(workflow.contains("cargo xtask inventory corpus generate-report"));
    assert!(workflow.contains("git diff --exit-code -- UPSTREAM-CORPUS.md"));
    assert!(workflow.contains("cargo build -p liquidfun-testbed --all-targets --all-features"));
    assert!(workflow.contains("cargo test -p liquidfun-testbed --all-features"));
    assert!(workflow.contains("DISPLAY: \"\""));
    assert!(workflow.contains("WAYLAND_DISPLAY: \"\""));
    assert!(maybe_visual.is_none_or(|visual| gate + package < visual));
}

#[test]
fn phase11_decisions_and_requirements_have_audited_evidence() {
    // Arrange
    let testing = include_str!("../../../TESTING.md");

    // Act
    let maybe_audit = testing
        .split("## Phase 11 decision and requirement audit")
        .nth(1);

    // Assert
    let audit = maybe_audit.expect("TESTING.md should contain the final Phase 11 audit");
    for decision in 1..=26 {
        let id = format!("`D-{decision:02}`");
        assert!(audit.contains(&id), "missing {id} from the Phase 11 audit");
    }
    for requirement in [
        "RIGD-10", "TEST-03", "EXMP-01", "EXMP-02", "EXMP-03", "EXMP-04", "EXMP-05", "EXMP-06",
    ] {
        let id = format!("`{requirement}`");
        assert!(audit.contains(&id), "missing {id} from the Phase 11 audit");
    }
}

#[test]
fn advisory_waiver_is_bounded_to_the_private_testbed() -> TestResult {
    // Arrange
    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let deny: toml::Value =
        toml::from_str(&fs::read_to_string(repository_root.join("deny.toml"))?)?;
    let liquidfun: toml::Value = toml::from_str(&fs::read_to_string(
        repository_root.join("crates/liquidfun/Cargo.toml"),
    )?)?;
    let testbed: toml::Value = toml::from_str(&fs::read_to_string(
        repository_root.join("crates/liquidfun-testbed/Cargo.toml"),
    )?)?;

    // Act
    let ignored = deny["advisories"]["ignore"]
        .as_array()
        .expect("advisory ignores should be an array")
        .iter()
        .map(|value| value.as_str().expect("advisory IDs should be strings"))
        .collect::<Vec<_>>();
    let liquidfun_dependencies = liquidfun["dependencies"]
        .as_table()
        .expect("liquidfun dependencies should be a table");

    // Assert
    assert_eq!(ignored, ["RUSTSEC-2025-0035", "RUSTSEC-2026-0192"]);
    assert!(!liquidfun_dependencies.contains_key("macroquad"));
    assert_eq!(testbed["package"]["publish"].as_bool(), Some(false));
    assert!(testbed["dependencies"]["macroquad"].is_str());
    Ok(())
}

#[test]
fn verify_rejects_private_or_graphical_dependencies_from_consumer_metadata() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    let mut metadata: serde_json::Value = serde_json::from_slice(&fs::read(&fixture.metadata)?)?;
    metadata["packages"][0]["dependencies"] = serde_json::json!([{
        "name": "liquidfun-differential",
        "kind": null
    }]);
    fs::write(&fixture.metadata, serde_json::to_vec(&metadata)?)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/dependency-graph");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn verify_rejects_more_than_one_default_publishable_package() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    let mut metadata: serde_json::Value = serde_json::from_slice(&fs::read(&fixture.metadata)?)?;
    metadata["workspace_default_members"] = serde_json::json!([
        "liquidfun 0.0.0 (path+file:///fixture/liquidfun)",
        "private 0.0.0"
    ]);
    fs::write(&fixture.metadata, serde_json::to_vec(&metadata)?)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/default-members");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn verify_rejects_a_second_publishable_workspace_package() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;
    let mut metadata: serde_json::Value = serde_json::from_slice(&fs::read(&fixture.metadata)?)?;
    metadata["packages"]
        .as_array_mut()
        .expect("fixture packages should be an array")
        .push(serde_json::json!({
            "id": "private 0.0.0 (path+file:///fixture/private)",
            "name": "private",
            "publish": null,
            "manifest_path": fixture.root.join("crates/private/Cargo.toml"),
            "dependencies": [],
            "features": {"default": []}
        }));
    metadata["workspace_members"] = serde_json::json!([
        "liquidfun 0.0.0 (path+file:///fixture/liquidfun)",
        "private 0.0.0 (path+file:///fixture/private)"
    ]);
    fs::write(&fixture.metadata, serde_json::to_vec(&metadata)?)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/publish-policy");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn verify_accepts_archive_with_matching_license() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::Valid)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_success(&output);
    assert!(fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn verify_rejects_forbidden_package_files() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::ForbiddenContent)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/forbidden-content");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn verify_rejects_parent_traversal_before_building() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::ParentTraversal)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/archive-path");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

#[test]
fn verify_rejects_absolute_paths_before_building() -> TestResult {
    // Arrange
    let fixture = PackageFixture::new(ArchiveCase::AbsolutePath)?;

    // Act
    let output = fixture.command()?;

    // Assert
    assert_failure_category(&output, "package/archive-path");
    assert!(!fixture.cargo_marker.exists());
    fixture.cleanup()?;
    Ok(())
}

fn write_archive(path: &Path, case: ArchiveCase) -> io::Result<()> {
    let file = File::create(path)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut archive = tar::Builder::new(encoder);
    append_file(
        &mut archive,
        "liquidfun-0.0.0/Cargo.toml",
        b"[package]\nname = \"liquidfun\"\nversion = \"0.0.0\"\n",
    )?;
    append_file(
        &mut archive,
        "liquidfun-0.0.0/LICENSE",
        b"fixture license\n",
    )?;
    match case {
        ArchiveCase::Valid => {}
        ArchiveCase::ForbiddenContent => append_file(
            &mut archive,
            "liquidfun-0.0.0/tools/oracle.cpp",
            b"forbidden",
        )?,
        ArchiveCase::ForbiddenGraphics => {
            append_file(
                &mut archive,
                "liquidfun-0.0.0/assets/frame.png",
                b"forbidden",
            )?;
        }
        ArchiveCase::ForbiddenNativeSource => {
            append_file(&mut archive, "liquidfun-0.0.0/src/oracle.cpp", b"forbidden")?;
        }
        ArchiveCase::ForbiddenTestbed => append_file(
            &mut archive,
            "liquidfun-0.0.0/crates/liquidfun-testbed/src/lib.rs",
            b"forbidden",
        )?,
        ArchiveCase::ParentTraversal => {
            append_raw_file(&mut archive, "../escape", b"traversal")?;
        }
        ArchiveCase::AbsolutePath => {
            append_raw_file(&mut archive, "/escape", b"absolute")?;
        }
    }
    archive.finish()?;
    let encoder = archive.into_inner()?;
    let _file = encoder.finish()?;
    Ok(())
}

fn valid_metadata(root: &Path) -> Vec<u8> {
    let manifest_path = root.join("crates/liquidfun/Cargo.toml");
    serde_json::to_vec(&serde_json::json!({
        "packages": [{
            "id": "liquidfun 0.0.0 (path+file:///fixture/liquidfun)",
            "name": "liquidfun",
            "publish": null,
            "manifest_path": manifest_path,
            "dependencies": [{"name": "bitflags", "kind": null}],
            "features": {"default": [], "differential-internals": []}
        }],
        "workspace_members": ["liquidfun 0.0.0 (path+file:///fixture/liquidfun)"],
        "workspace_default_members": ["liquidfun 0.0.0 (path+file:///fixture/liquidfun)"]
    }))
    .expect("fixture metadata should serialize")
}

fn append_file<W: io::Write>(
    archive: &mut tar::Builder<W>,
    path: &str,
    contents: &[u8],
) -> io::Result<()> {
    let mut header = regular_header(contents.len())?;
    archive.append_data(&mut header, path, Cursor::new(contents))
}

fn append_raw_file<W: io::Write>(
    archive: &mut tar::Builder<W>,
    path: &str,
    contents: &[u8],
) -> io::Result<()> {
    let mut header = regular_header(contents.len())?;
    let name = path.as_bytes();
    if name.len() > 99 {
        return Err(io::Error::other("raw fixture path is too long"));
    }
    let header_bytes = header.as_mut_bytes();
    header_bytes[..100].fill(0);
    header_bytes[..name.len()].copy_from_slice(name);
    header.set_cksum();
    archive.append(&header, Cursor::new(contents))
}

fn regular_header(size: usize) -> io::Result<tar::Header> {
    let size = u64::try_from(size).map_err(io::Error::other)?;
    let mut header = tar::Header::new_gnu();
    header.set_size(size);
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_entry_type(tar::EntryType::Regular);
    header.set_cksum();
    Ok(header)
}

fn fake_cargo() -> io::Result<&'static Path> {
    let result = FAKE_CARGO.get_or_init(compile_fake_cargo);
    match result {
        Ok(path) => Ok(path),
        Err(message) => Err(io::Error::other(message.clone())),
    }
}

fn compile_fake_cargo() -> Result<PathBuf, String> {
    let output_dir =
        workspace_root().join(format!("target/xtask-package-tools/{}", std::process::id()));
    fs::create_dir_all(&output_dir).map_err(|error| error.to_string())?;
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake_upstream_tool.rs");
    let executable = output_dir.join(executable_name("fake-cargo"));
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

fn assert_failure_category(output: &Output, category: &str) {
    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert!(
        stderr(output).contains(category),
        "expected `{category}` in stderr:\n{}",
        stderr(output)
    );
}

fn assert_success(output: &Output) {
    assert!(output.status.success(), "{}", stderr(output));
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
