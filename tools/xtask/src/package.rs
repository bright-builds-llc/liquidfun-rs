use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Display, Formatter};
use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::read::GzDecoder;

const USAGE: &str = "Usage: cargo xtask package verify";
const FORBIDDEN_PREFIXES: [&str; 3] = ["tools", "third_party", "reference"];
const FORBIDDEN_EXTENSIONS: [&str; 11] = [
    "c", "cc", "cpp", "cxx", "h", "hh", "hpp", "hxx", "s", "asm", "cmake",
];
static TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PackageError {
    category: &'static str,
    message: String,
}

impl PackageError {
    fn new(category: &'static str, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }

    fn usage(message: impl Into<String>) -> Self {
        Self::new("usage", format!("{}\n\n{USAGE}", message.into()))
    }
}

impl Display for PackageError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "package/{}: {}", self.category, self.message)
    }
}

impl Error for PackageError {}

struct PackageIdentity {
    name: String,
    version: String,
}

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn create() -> Result<Self, PackageError> {
        let id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| PackageError::new("filesystem", error.to_string()))?
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "liquidfun-package-verify-{}-{timestamp}-{id}",
            std::process::id()
        ));
        fs::create_dir(&path).map_err(|error| {
            PackageError::new(
                "filesystem",
                format!("failed to create {}: {error}", path.display()),
            )
        })?;
        Ok(Self { path })
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _cleanup_result = fs::remove_dir_all(&self.path);
    }
}

pub(crate) fn run(args: &[String]) -> Result<(), PackageError> {
    if args != ["verify"] {
        return Err(PackageError::usage("expected `verify`"));
    }
    let repository_root = repository_root()?;
    verify(&repository_root)
}

fn verify(repository_root: &Path) -> Result<(), PackageError> {
    let identity = read_package_identity(repository_root)?;
    let archive_path = package_archive(repository_root, &identity)?;
    let package_prefix = format!("{}-{}", identity.name, identity.version);
    let entries = inspect_archive(&archive_path, &package_prefix)?;
    let temporary_directory = TemporaryDirectory::create()?;
    extract_archive(&archive_path, &temporary_directory.path)?;
    let unpacked_crate = temporary_directory.path.join(&package_prefix);
    if !unpacked_crate.join("Cargo.toml").is_file() {
        return Err(PackageError::new(
            "archive",
            format!("archive did not contain {package_prefix}/Cargo.toml"),
        ));
    }
    verify_license(repository_root, &unpacked_crate)?;
    build_and_test(&unpacked_crate, &temporary_directory.path)?;
    println!(
        "package verified: {} entries built and tested outside the repository",
        entries.len()
    );
    Ok(())
}

fn package_archive(
    repository_root: &Path,
    identity: &PackageIdentity,
) -> Result<PathBuf, PackageError> {
    if let Some(path) = env::var_os("LIQUIDFUN_XTASK_TEST_PACKAGE_ARCHIVE") {
        return Ok(PathBuf::from(path));
    }
    let cargo = cargo_program();
    run_process(
        &cargo,
        [
            OsStr::new("package"),
            OsStr::new("-p"),
            OsStr::new(&identity.name),
            OsStr::new("--allow-dirty"),
        ],
        repository_root,
        None,
        "create Cargo package",
    )?;
    let path = repository_root.join(format!(
        "target/package/{}-{}.crate",
        identity.name, identity.version
    ));
    if !path.is_file() {
        return Err(PackageError::new(
            "archive",
            format!("cargo package did not create {}", path.display()),
        ));
    }
    Ok(path)
}

fn inspect_archive(
    archive_path: &Path,
    package_prefix: &str,
) -> Result<Vec<PathBuf>, PackageError> {
    let file = File::open(archive_path).map_err(|error| {
        PackageError::new(
            "archive",
            format!("failed to open {}: {error}", archive_path.display()),
        )
    })?;
    let mut archive = tar::Archive::new(GzDecoder::new(file));
    let mut paths = Vec::new();
    let mut unique_paths = BTreeSet::new();
    let entries = archive.entries().map_err(|error| {
        PackageError::new(
            "archive",
            format!("failed to read {}: {error}", archive_path.display()),
        )
    })?;
    for maybe_entry in entries {
        let entry = maybe_entry.map_err(|error| {
            PackageError::new("archive", format!("invalid archive entry: {error}"))
        })?;
        let entry_type = entry.header().entry_type();
        if !entry_type.is_file() && !entry_type.is_dir() {
            return Err(PackageError::new(
                "archive-type",
                "package archives may contain only regular files and directories",
            ));
        }
        let path = entry
            .path()
            .map_err(|error| PackageError::new("archive-path", error.to_string()))?
            .into_owned();
        let relative = validate_archive_path(&path, package_prefix)?;
        validate_package_content(&relative)?;
        if !unique_paths.insert(path.clone()) {
            return Err(PackageError::new(
                "archive-path",
                format!("duplicate archive path `{}`", path.display()),
            ));
        }
        paths.push(path);
    }
    if paths.is_empty() {
        return Err(PackageError::new("archive", "package archive is empty"));
    }
    let license_path = Path::new(package_prefix).join("LICENSE");
    if !unique_paths.contains(&license_path) {
        return Err(PackageError::new(
            "required-content",
            format!("archive did not contain {package_prefix}/LICENSE"),
        ));
    }
    Ok(paths)
}

fn verify_license(repository_root: &Path, unpacked_crate: &Path) -> Result<(), PackageError> {
    let root_license_path = repository_root.join("LICENSE");
    let packaged_license_path = unpacked_crate.join("LICENSE");
    let root_license = fs::read(&root_license_path).map_err(|error| {
        PackageError::new(
            "license",
            format!("failed to read {}: {error}", root_license_path.display()),
        )
    })?;
    let packaged_license = fs::read(&packaged_license_path).map_err(|error| {
        PackageError::new(
            "license",
            format!(
                "failed to read {}: {error}",
                packaged_license_path.display()
            ),
        )
    })?;
    if packaged_license != root_license {
        return Err(PackageError::new(
            "license",
            "packaged LICENSE differs from the repository LICENSE",
        ));
    }
    Ok(())
}

fn validate_archive_path(path: &Path, package_prefix: &str) -> Result<PathBuf, PackageError> {
    if path.as_os_str().is_empty()
        || path.to_str().is_none()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(PackageError::new(
            "archive-path",
            format!(
                "archive path `{}` must be normalized, relative, UTF-8, and traversal-free",
                path.display()
            ),
        ));
    }
    let relative = path.strip_prefix(package_prefix).map_err(|_| {
        PackageError::new(
            "archive-path",
            format!(
                "archive path `{}` is outside package root `{package_prefix}`",
                path.display()
            ),
        )
    })?;
    Ok(relative.to_path_buf())
}

fn validate_package_content(relative: &Path) -> Result<(), PackageError> {
    let maybe_first = relative
        .components()
        .next()
        .and_then(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        });
    if maybe_first.is_some_and(|first| FORBIDDEN_PREFIXES.contains(&first)) {
        return Err(PackageError::new(
            "forbidden-content",
            format!("forbidden package prefix `{}`", relative.display()),
        ));
    }
    let maybe_file_name = relative.file_name().and_then(OsStr::to_str);
    if maybe_file_name == Some("CMakeLists.txt") || maybe_file_name == Some("build.rs") {
        return Err(PackageError::new(
            "forbidden-content",
            format!("forbidden package file `{}`", relative.display()),
        ));
    }
    let maybe_extension = relative
        .extension()
        .and_then(OsStr::to_str)
        .map(str::to_ascii_lowercase);
    if maybe_extension
        .as_deref()
        .is_some_and(|extension| FORBIDDEN_EXTENSIONS.contains(&extension))
    {
        return Err(PackageError::new(
            "forbidden-content",
            format!("forbidden native source `{}`", relative.display()),
        ));
    }
    Ok(())
}

fn extract_archive(archive_path: &Path, destination: &Path) -> Result<(), PackageError> {
    let file = File::open(archive_path).map_err(|error| {
        PackageError::new(
            "extract",
            format!("failed to reopen {}: {error}", archive_path.display()),
        )
    })?;
    let mut archive = tar::Archive::new(GzDecoder::new(file));
    archive.unpack(destination).map_err(|error| {
        PackageError::new(
            "extract",
            format!("failed to unpack {}: {error}", archive_path.display()),
        )
    })
}

fn build_and_test(unpacked_crate: &Path, temporary_root: &Path) -> Result<(), PackageError> {
    let cargo = cargo_program();
    let toolchain =
        env::var("LIQUIDFUN_XTASK_PACKAGE_TOOLCHAIN").unwrap_or_else(|_| "1.92.0".to_owned());
    let toolchain_argument = format!("+{toolchain}");
    let target_dir = temporary_root.join("target");
    for (action, arguments) in [
        (
            "build unpacked package",
            ["build", "--all-targets", "--all-features", "--locked"],
        ),
        (
            "test unpacked package",
            ["test", "--all-features", "--locked", "--no-fail-fast"],
        ),
    ] {
        let mut command_arguments = vec![OsString::from(&toolchain_argument)];
        command_arguments.extend(arguments.map(OsString::from));
        run_process(
            &cargo,
            command_arguments.iter().map(OsString::as_os_str),
            unpacked_crate,
            Some(&target_dir),
            action,
        )?;
    }
    Ok(())
}

fn read_package_identity(repository_root: &Path) -> Result<PackageIdentity, PackageError> {
    let path = repository_root.join("crates/liquidfun/Cargo.toml");
    let contents = fs::read_to_string(&path).map_err(|error| {
        PackageError::new(
            "manifest",
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    let manifest: toml::Value = toml::from_str(&contents).map_err(|error| {
        PackageError::new("manifest", format!("invalid {}: {error}", path.display()))
    })?;
    let package = manifest.get("package").and_then(toml::Value::as_table);
    let Some(name) = package
        .and_then(|table| table.get("name"))
        .and_then(toml::Value::as_str)
    else {
        return Err(PackageError::new("manifest", "package.name is missing"));
    };
    let Some(version) = package
        .and_then(|table| table.get("version"))
        .and_then(toml::Value::as_str)
    else {
        return Err(PackageError::new("manifest", "package.version is missing"));
    };
    Ok(PackageIdentity {
        name: name.to_owned(),
        version: version.to_owned(),
    })
}

fn run_process<'a>(
    program: &OsStr,
    args: impl IntoIterator<Item = &'a OsStr>,
    current_dir: &Path,
    maybe_target_dir: Option<&Path>,
    action: &str,
) -> Result<(), PackageError> {
    let mut command = Command::new(program);
    command.args(args).current_dir(current_dir);
    if let Some(target_dir) = maybe_target_dir {
        command.env("CARGO_TARGET_DIR", target_dir);
    }
    let output = command
        .output()
        .map_err(|error| PackageError::new("process", format!("failed to {action}: {error}")))?;
    if output.status.success() {
        return Ok(());
    }
    Err(PackageError::new(
        "process",
        format!(
            "failed to {action} with {}:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    ))
}

fn cargo_program() -> OsString {
    env::var_os("LIQUIDFUN_XTASK_CARGO").unwrap_or_else(|| OsString::from("cargo"))
}

fn repository_root() -> Result<PathBuf, PackageError> {
    if let Some(root) = env::var_os("LIQUIDFUN_XTASK_ROOT") {
        return Ok(PathBuf::from(root));
    }
    let current_dir = env::current_dir().map_err(|error| {
        PackageError::new(
            "filesystem",
            format!("failed to read current directory: {error}"),
        )
    })?;
    let Some(root) = current_dir.ancestors().find(|candidate| {
        candidate.join("Cargo.toml").is_file()
            && candidate.join("crates/liquidfun/Cargo.toml").is_file()
    }) else {
        return Err(PackageError::new(
            "repository",
            "could not find the liquidfun Cargo workspace",
        ));
    };
    Ok(root.to_path_buf())
}
