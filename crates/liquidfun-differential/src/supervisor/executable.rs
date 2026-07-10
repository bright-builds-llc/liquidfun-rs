//! Allowlisted canonical oracle executable resolution.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Reviewed `CMake` preset whose output may contain an oracle executable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OraclePreset {
    /// Assertion-enabled development oracle.
    Debug,
    /// Optimized benchmark/reference oracle.
    Release,
    /// Fail-fast `AddressSanitizer` and `UndefinedBehaviorSanitizer` oracle.
    AsanUbsan,
}

impl OraclePreset {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "oracle-debug",
            Self::Release => "oracle-release",
            Self::AsanUbsan => "oracle-asan-ubsan",
        }
    }
}

/// Failure while resolving a reviewed oracle executable.
#[derive(Debug, thiserror::Error)]
pub enum OracleExecutableError {
    /// Repository root or expected output does not exist.
    #[error("oracle executable path does not exist: {0}")]
    Missing(PathBuf),
    /// A reviewed path component is a symbolic link.
    #[error("oracle executable path contains a symbolic link: {0}")]
    Symlink(PathBuf),
    /// Canonical output escaped its reviewed preset directory.
    #[error("oracle executable escaped the reviewed preset output")]
    OutsidePreset,
    /// Resolved output is not a regular executable file.
    #[error("oracle output is not a regular executable file")]
    NotExecutable,
    /// Filesystem inspection failed.
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Canonical regular executable confined to one reviewed preset output.
#[derive(Debug, Clone)]
pub struct OracleExecutable {
    pub(super) resolved: PathBuf,
    pub(super) preset: OraclePreset,
}

impl OracleExecutable {
    /// Resolves only `target/reference/<reviewed-preset>/liquidfun-reference` below a repository.
    ///
    /// # Errors
    ///
    /// Rejects missing, symlinked, non-regular, non-executable, or out-of-preset candidates.
    pub fn resolve(
        repository_root: &Path,
        preset: OraclePreset,
    ) -> Result<Self, OracleExecutableError> {
        if !repository_root.exists() {
            return Err(OracleExecutableError::Missing(
                repository_root.to_path_buf(),
            ));
        }
        let canonical_root = repository_root.canonicalize()?;
        let target = repository_root.join("target");
        let reference = target.join("reference");
        let preset_directory = reference.join(preset.as_str());
        let candidate = preset_directory.join(oracle_file_name());
        for component in [&target, &reference, &preset_directory, &candidate] {
            reject_symlink(component)?;
        }
        let canonical_preset = preset_directory.canonicalize()?;
        let canonical_candidate = candidate.canonicalize()?;
        if !canonical_preset.starts_with(canonical_root.join("target/reference"))
            || !canonical_candidate.starts_with(&canonical_preset)
        {
            return Err(OracleExecutableError::OutsidePreset);
        }
        let metadata = fs::metadata(&canonical_candidate)?;
        if !metadata.is_file() || !is_executable(&metadata) {
            return Err(OracleExecutableError::NotExecutable);
        }
        Ok(Self {
            resolved: canonical_candidate,
            preset,
        })
    }
}

fn reject_symlink(path: &Path) -> Result<(), OracleExecutableError> {
    if !path.exists() {
        return Err(OracleExecutableError::Missing(path.to_path_buf()));
    }
    if fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err(OracleExecutableError::Symlink(path.to_path_buf()));
    }
    Ok(())
}

fn oracle_file_name() -> &'static str {
    if cfg!(windows) {
        "liquidfun-reference.exe"
    } else {
        "liquidfun-reference"
    }
}

#[cfg(unix)]
fn is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_metadata: &fs::Metadata) -> bool {
    true
}
