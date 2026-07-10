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
    ForbiddenContent,
    ParentTraversal,
    AbsolutePath,
}

struct PackageFixture {
    root: PathBuf,
    archive: PathBuf,
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
        let archive = root.join("fixture.crate");
        write_archive(&archive, case)?;
        Ok(Self {
            cargo_marker: root.join("cargo-called"),
            root,
            archive,
        })
    }

    fn command(&self) -> io::Result<Output> {
        Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["package", "verify"])
            .env("LIQUIDFUN_XTASK_ROOT", &self.root)
            .env("LIQUIDFUN_XTASK_TEST_PACKAGE_ARCHIVE", &self.archive)
            .env("LIQUIDFUN_XTASK_CARGO", fake_cargo()?)
            .env("LIQUIDFUN_TEST_CARGO_MARKER", &self.cargo_marker)
            .output()
    }

    fn cleanup(self) -> io::Result<()> {
        fs::remove_dir_all(self.root)
    }
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
    match case {
        ArchiveCase::ForbiddenContent => append_file(
            &mut archive,
            "liquidfun-0.0.0/tools/oracle.cpp",
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

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}
