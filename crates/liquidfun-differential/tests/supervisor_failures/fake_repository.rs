use std::{
    ffi::OsString,
    fs, io,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use liquidfun_differential::{OracleExecutable, OraclePreset, OracleSupervisor, SessionProfile};

static REPOSITORY_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy)]
enum CleanupMode {
    RemoveOwnedRoot,
    InjectFailure,
}

pub(super) struct FakeRepository {
    root: PathBuf,
    cleanup_mode: CleanupMode,
}

impl FakeRepository {
    pub(super) fn create(behavior: &str) -> io::Result<Self> {
        Self::create_with_mode(behavior, CleanupMode::RemoveOwnedRoot)
    }

    fn create_with_mode(behavior: &str, cleanup_mode: CleanupMode) -> io::Result<Self> {
        let root = candidate_root()?;
        Self::claim_and_populate(root, behavior, cleanup_mode)
    }

    fn claim_and_populate(
        root: PathBuf,
        behavior: &str,
        cleanup_mode: CleanupMode,
    ) -> io::Result<Self> {
        fs::create_dir(&root)?;
        let repository = Self { root, cleanup_mode };
        repository.populate(behavior)?;
        Ok(repository)
    }

    fn populate(&self, behavior: &str) -> io::Result<()> {
        let output = self.root.join("target/reference/oracle-debug");
        fs::create_dir_all(&output)?;
        fs::copy(
            env!("CARGO_BIN_EXE_liquidfun-fake-oracle"),
            output.join(oracle_file_name()),
        )?;
        fs::write(output.join("behavior.txt"), behavior)
    }

    pub(super) fn path(&self) -> &Path {
        &self.root
    }
}

impl Deref for FakeRepository {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.path()
    }
}

impl Drop for FakeRepository {
    fn drop(&mut self) {
        let result = cleanup_owned_root(&self.root, self.cleanup_mode);
        let Err(error) = result else {
            return;
        };
        if std::thread::panicking() {
            eprintln!(
                "cleanup failed for owned fake repository root {} during unwind: {error}",
                self.root.display()
            );
            return;
        }
        panic!(
            "cleanup failed for owned fake repository root {}: {error}",
            self.root.display()
        );
    }
}

pub(super) struct TestSupervisor {
    maybe_supervisor: Option<OracleSupervisor>,
    maybe_repository: Option<FakeRepository>,
}

impl TestSupervisor {
    pub(super) fn create(behavior: &str, profile: SessionProfile, revision: &str) -> Self {
        let repository =
            FakeRepository::create(behavior).expect("owned fake repository should be creatable");
        let executable = OracleExecutable::resolve(repository.path(), OraclePreset::Debug)
            .expect("confined fake oracle should resolve");
        let supervisor = OracleSupervisor::new(executable, profile, revision);
        Self {
            maybe_supervisor: Some(supervisor),
            maybe_repository: Some(repository),
        }
    }

    pub(super) fn repository_path(&self) -> &Path {
        self.maybe_repository
            .as_ref()
            .expect("repository remains present until TestSupervisor drop")
            .path()
    }
}

impl Deref for TestSupervisor {
    type Target = OracleSupervisor;

    fn deref(&self) -> &Self::Target {
        self.maybe_supervisor
            .as_ref()
            .expect("supervisor remains present until TestSupervisor drop")
    }
}

impl DerefMut for TestSupervisor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.maybe_supervisor
            .as_mut()
            .expect("supervisor remains present until TestSupervisor drop")
    }
}

impl Drop for TestSupervisor {
    fn drop(&mut self) {
        drop(self.maybe_supervisor.take());
        drop(self.maybe_repository.take());
    }
}

fn candidate_root() -> io::Result<PathBuf> {
    let canonical_temp = std::env::temp_dir().canonicalize()?;
    let canonical_workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    if canonical_temp == canonical_workspace || canonical_temp.starts_with(&canonical_workspace) {
        return Err(io::Error::other(format!(
            "system temporary directory {} must be outside workspace {}",
            canonical_temp.display(),
            canonical_workspace.display()
        )));
    }
    let id = REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
    Ok(canonical_temp.join(format!("liquidfun-supervisor-{}-{id}", std::process::id())))
}

fn oracle_file_name() -> &'static str {
    if cfg!(windows) {
        "liquidfun-reference.exe"
    } else {
        "liquidfun-reference"
    }
}

fn cleanup_owned_root(root: &Path, cleanup_mode: CleanupMode) -> io::Result<()> {
    match cleanup_mode {
        CleanupMode::RemoveOwnedRoot => {
            if !root.try_exists()? {
                return Ok(());
            }
            fs::remove_dir_all(root)
        }
        CleanupMode::InjectFailure => Err(io::Error::other("injected cleanup failure")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_names(directory: &Path) -> io::Result<Vec<OsString>> {
        let mut names = fs::read_dir(directory)?
            .map(|entry| entry.map(|entry| entry.file_name()))
            .collect::<io::Result<Vec<_>>>()?;
        names.sort();
        Ok(names)
    }

    #[test]
    fn external_temp_layout_is_canonical_and_exact() {
        // Arrange
        let canonical_temp = std::env::temp_dir()
            .canonicalize()
            .expect("system temporary directory should canonicalize");
        let canonical_workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("workspace should canonicalize");
        let repository =
            FakeRepository::create("valid").expect("owned fake repository should be creatable");

        // Act
        let canonical_root = repository
            .path()
            .canonicalize()
            .expect("fake repository root should canonicalize");
        let output = repository.path().join("target/reference/oracle-debug");

        // Assert
        assert_eq!(canonical_root.parent(), Some(canonical_temp.as_path()));
        assert!(!canonical_root.starts_with(&canonical_workspace));
        assert_eq!(
            entry_names(repository.path()).expect("root should list"),
            ["target"]
        );
        assert_eq!(
            entry_names(&repository.path().join("target")).expect("target should list"),
            ["reference"]
        );
        assert_eq!(
            entry_names(&repository.path().join("target/reference"))
                .expect("reference should list"),
            ["oracle-debug"]
        );
        assert_eq!(
            entry_names(&output).expect("oracle output should list"),
            [
                OsString::from("behavior.txt"),
                OsString::from(oracle_file_name())
            ]
        );
    }

    #[test]
    fn unique_repositories_keep_unique_behavior_sidecars() {
        // Arrange
        let first = FakeRepository::create("valid")
            .expect("first owned fake repository should be creatable");
        let second = FakeRepository::create("malformed")
            .expect("second owned fake repository should be creatable");

        // Act
        let first_behavior = fs::read_to_string(
            first
                .path()
                .join("target/reference/oracle-debug/behavior.txt"),
        )
        .expect("first behavior should be readable");
        let second_behavior = fs::read_to_string(
            second
                .path()
                .join("target/reference/oracle-debug/behavior.txt"),
        )
        .expect("second behavior should be readable");

        // Assert
        assert_ne!(first.path(), second.path());
        assert_eq!(first_behavior, "valid");
        assert_eq!(second_behavior, "malformed");
    }

    #[test]
    fn preexisting_unique_root_is_rejected_without_claim_or_cleanup() {
        // Arrange
        let root = candidate_root().expect("candidate root should resolve");
        fs::create_dir(&root).expect("pre-existing root should be creatable");
        let sentinel = root.join("sentinel.bin");
        fs::write(&sentinel, b"preserve-me").expect("sentinel should be writable");

        // Act
        let result =
            FakeRepository::claim_and_populate(root.clone(), "valid", CleanupMode::RemoveOwnedRoot);
        let sentinel_bytes = fs::read(&sentinel).expect("sentinel should survive");

        // Assert
        assert!(matches!(
            result,
            Err(ref error) if error.kind() == io::ErrorKind::AlreadyExists
        ));
        assert_eq!(sentinel_bytes, b"preserve-me");
        fs::remove_dir_all(root).expect("unowned pre-existing root should be removable");
    }

    #[test]
    fn exact_root_cleanup_preserves_temp_siblings() {
        // Arrange
        let repository =
            FakeRepository::create("valid").expect("owned fake repository should be creatable");
        let root = repository.path().to_path_buf();
        let sibling = candidate_root().expect("sibling root should resolve");
        fs::create_dir(&sibling).expect("sibling should be creatable");
        let sentinel = sibling.join("sentinel.bin");
        fs::write(&sentinel, b"sibling-bytes").expect("sibling sentinel should be writable");

        // Act
        drop(repository);
        let sibling_bytes = fs::read(&sentinel).expect("sibling sentinel should survive");

        // Assert
        assert!(!root.exists());
        assert_eq!(sibling_bytes, b"sibling-bytes");
        fs::remove_dir_all(sibling).expect("unowned sibling should be removable");
    }

    #[test]
    fn unwind_cleanup_failure_preserves_original_panic_without_secondary_panic() {
        // Arrange
        const SENTINEL: &str = "plan-25-original-panic";
        let repository = FakeRepository::create_with_mode("valid", CleanupMode::InjectFailure)
            .expect("injected-cleanup repository should be creatable");
        let root = repository.path().to_path_buf();

        // Act
        let recovered = std::panic::catch_unwind(move || {
            let _repository = repository;
            std::panic::panic_any(SENTINEL);
        });

        // Assert
        let payload = recovered.expect_err("sentinel panic should be recovered");
        assert_eq!(payload.downcast_ref::<&str>(), Some(&SENTINEL));
        fs::remove_dir_all(root).expect("injected-cleanup leftover should be removable");
    }
}
