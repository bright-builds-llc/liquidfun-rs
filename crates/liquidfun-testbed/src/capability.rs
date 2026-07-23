//! Executable replacement-renderer capability matrix.

mod fixture;
mod input;
mod passive;
mod render;
mod report;

use std::fs;
use std::path::{Component, Path, PathBuf};

use fixture::load_fixture_snapshot;
use passive::{build_passive_inputs, observe_passive_inputs};
use render::render_capability_frames;
pub use report::{CapabilityArtifact, CapabilityReport, REQUIRED_CAPABILITY_NAMES};
use sha2::{Digest, Sha256};

const MAXIMUM_REPORT_BYTES: usize = 128 * 1024;

/// Inputs accepted by the bounded capability command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityOptions {
    fixture: PathBuf,
    output: PathBuf,
}

impl CapabilityOptions {
    /// Creates one immutable capability invocation.
    #[must_use]
    pub fn new(fixture: PathBuf, output: PathBuf) -> Self {
        Self { fixture, output }
    }

    /// Returns the requested fixture path.
    #[must_use]
    pub fn fixture(&self) -> &Path {
        &self.fixture
    }

    /// Returns the requested output path.
    #[must_use]
    pub fn output(&self) -> &Path {
        &self.output
    }
}

/// Bounded fail-closed capability error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CapabilityError {
    /// Fixture path or content is untrusted or inconsistent.
    #[error("capability fixture is invalid")]
    InvalidFixture,
    /// Output path escapes the reviewed workspace target or crosses a link.
    #[error("capability output path is invalid")]
    InvalidOutputPath,
    /// A bounded filesystem operation failed.
    #[error("capability filesystem operation failed")]
    Filesystem,
    /// The immutable semantic comparison fixture could not be constructed.
    #[error("capability comparison input is invalid")]
    InvalidComparison,
    /// The selected adapter failed a required renderer capability.
    #[error("selected renderer failed the capability matrix")]
    CapabilityFailed,
    /// The bounded machine report could not be encoded.
    #[error("capability report encoding failed")]
    ReportEncoding,
}

impl CapabilityError {
    /// Returns a stable non-sensitive diagnostic category.
    #[must_use]
    pub const fn category(self) -> &'static str {
        match self {
            Self::InvalidFixture => "invalid_fixture",
            Self::InvalidOutputPath => "invalid_output_path",
            Self::Filesystem => "filesystem",
            Self::InvalidComparison => "invalid_comparison",
            Self::CapabilityFailed => "capability_failed",
            Self::ReportEncoding => "report_encoding",
        }
    }
}

/// Runs the deterministic offscreen replacement-renderer capability matrix.
///
/// The adapter consumes only shared controller and comparison references. Rendering cannot submit
/// commands, advance a logical step, or create a semantic checkpoint.
///
/// # Errors
///
/// Returns a bounded error for invalid inputs, unsafe output paths, filesystem failures, semantic
/// fixture failures, or any failed required capability.
pub fn run_capability_check(
    options: &CapabilityOptions,
) -> Result<CapabilityReport, CapabilityError> {
    let repository = repository_root()?;
    let output = prepare_output_directory(&repository, options.output())?;
    let fixture = load_fixture_snapshot(&repository, options.fixture())?;
    let (controller, comparison) = build_passive_inputs(&fixture)?;
    let before = observe_passive_inputs(&controller, &comparison);
    let rendered = render_capability_frames(
        &fixture,
        &controller,
        &comparison,
        input::verified_keyboard_binding_count(),
        &output,
    )?;
    let after = observe_passive_inputs(&controller, &comparison);
    let mut report = CapabilityReport::from_evidence(&fixture, before, after, rendered);
    report.validate_required_capabilities();
    if !report.all_passed() {
        return Err(CapabilityError::CapabilityFailed);
    }
    write_machine_report(&output, &mut report)?;
    Ok(report)
}

fn repository_root() -> Result<PathBuf, CapabilityError> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let Some(repository) = manifest.parent().and_then(Path::parent) else {
        return Err(CapabilityError::Filesystem);
    };
    repository
        .canonicalize()
        .map_err(|_| CapabilityError::Filesystem)
}

fn prepare_output_directory(
    repository: &Path,
    relative: &Path,
) -> Result<PathBuf, CapabilityError> {
    if relative.is_absolute() {
        return Err(CapabilityError::InvalidOutputPath);
    }
    let mut components = relative.components();
    if components.next() != Some(Component::Normal("target".as_ref())) {
        return Err(CapabilityError::InvalidOutputPath);
    }
    if components.any(|component| !matches!(component, Component::Normal(_))) {
        return Err(CapabilityError::InvalidOutputPath);
    }
    let output = repository.join(relative);
    reject_existing_links(repository, &output)?;
    fs::create_dir_all(&output).map_err(|_| CapabilityError::Filesystem)?;
    let metadata = fs::symlink_metadata(&output).map_err(|_| CapabilityError::Filesystem)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(CapabilityError::InvalidOutputPath);
    }
    Ok(output)
}

fn reject_existing_links(repository: &Path, output: &Path) -> Result<(), CapabilityError> {
    let Ok(relative) = output.strip_prefix(repository) else {
        return Err(CapabilityError::InvalidOutputPath);
    };
    let mut cursor = repository.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            return Err(CapabilityError::InvalidOutputPath);
        };
        cursor.push(part);
        match fs::symlink_metadata(&cursor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(CapabilityError::InvalidOutputPath);
            }
            Ok(metadata) if !metadata.is_dir() && cursor != output => {
                return Err(CapabilityError::InvalidOutputPath);
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(CapabilityError::Filesystem),
        }
    }
    Ok(())
}

fn write_machine_report(
    output: &Path,
    report: &mut CapabilityReport,
) -> Result<(), CapabilityError> {
    let path = output.join("capability-report.json");
    reject_link_file(&path)?;
    let encoded = serde_json::to_vec_pretty(report).map_err(|_| CapabilityError::ReportEncoding)?;
    if encoded.len() > MAXIMUM_REPORT_BYTES {
        return Err(CapabilityError::ReportEncoding);
    }
    fs::write(&path, &encoded).map_err(|_| CapabilityError::Filesystem)?;
    let metadata = fs::symlink_metadata(&path).map_err(|_| CapabilityError::Filesystem)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(CapabilityError::InvalidOutputPath);
    }
    report.set_report_sha256(hex_sha256(&encoded));
    Ok(())
}

pub(super) fn reject_link_file(path: &Path) -> Result<(), CapabilityError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            Err(CapabilityError::InvalidOutputPath)
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(CapabilityError::Filesystem),
    }
}

pub(super) fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
