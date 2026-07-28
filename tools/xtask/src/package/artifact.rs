use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod validation;

#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use validation::*;

use super::metadata::ConsumerManifest;
use super::{
    PackageError, TemporaryDirectory, cargo_program, extract_archive, inspect_archive,
    read_archive, read_package_identity, run_process, verify_license,
};

const ARTIFACT_SCHEMA_VERSION: u8 = 1;
const PACKAGE_NAME: &str = "liquidfun";
const REQUIRED_RUST_VERSION: &str = "1.92";
const CREATION_TOOLCHAIN: &str = "1.97.0";
const NATIVE_TOOLCHAIN: &str = "1.97.0";
const MSRV_TOOLCHAIN: &str = "1.92.0";
const CANONICAL_TARGET: &str = "x86_64-unknown-linux-gnu";
const CONDITIONAL_TARGET: &str = "x86_64-apple-darwin";
const MAX_EVIDENCE_AGE_DAYS: u64 = 90;
const SECONDS_PER_DAY: u64 = 86_400;
const DURABLE_TARGETS: [&str; 4] = [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
];
const REQUIRED_FEATURES: [&str; 2] = ["default", "differential-internals"];
const REQUIRED_NORMAL_DEPENDENCIES: [&str; 1] = ["bitflags"];
const SCALAR_MODE: &str = "strict_f32";
const COMPILER_CLASS: &str = "rustc-platform-native";
const TOLERANCE_PROFILE: &str = "phase4-v1";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PackageArtifactIdentity {
    schema_version: u8,
    archive_sha256: String,
    archive_bytes: u64,
    package: String,
    version: String,
    rust_version: String,
    features: Vec<String>,
    normal_dependencies: Vec<String>,
    source_files: Vec<String>,
    license_files: Vec<String>,
    notice_files: Vec<String>,
    candidate_commit: String,
    created_with_toolchain: String,
    scalar_mode: String,
    compiler_class: String,
    tolerance_profile: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlatformSupport {
    schema_version: u8,
    evidence_tier: String,
    durable_targets: Vec<String>,
    conditional_targets: Vec<ConditionalTarget>,
    conditional_evidence_policy: ConditionalEvidencePolicy,
    scalar_mode: String,
    compiler_class: String,
    tolerance_profile: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConditionalTarget {
    target: String,
    tier: String,
    native_evidence: Option<NativeEvidence>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeEvidence {
    runner: String,
    recorded_at_unix: u64,
    expires_at_unix: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConditionalEvidencePolicy {
    max_age_days: u64,
    missing_or_expired_outcome: String,
}

struct ArchiveContents {
    manifest: ConsumerManifest,
    source_files: Vec<String>,
    license_files: Vec<String>,
    notice_files: Vec<String>,
    unpacked_crate: PathBuf,
    temporary: TemporaryDirectory,
}

pub(super) fn create(repository_root: &Path, args: &[String]) -> Result<(), PackageError> {
    let options = parse_options(args, &["--archive", "--identity", "--candidate-commit"])?;
    let archive_destination = PathBuf::from(required_option(&options, "--archive")?);
    let identity_destination = PathBuf::from(required_option(&options, "--identity")?);
    let candidate_commit = required_option(&options, "--candidate-commit")?;
    validate_candidate_commit(candidate_commit)?;
    validate_candidate_matches_repository(repository_root, candidate_commit)?;

    super::metadata::verify_workspace(repository_root, &cargo_program())?;
    let package = read_package_identity(repository_root)?;
    let generated_archive = create_archive(repository_root, &package.name, &package.version)?;
    let archive_bytes = read_archive(&generated_archive)?;
    write_bytes(&archive_destination, &archive_bytes, "artifact archive")?;
    let contents = inspect_contents(
        repository_root,
        &archive_destination,
        &archive_bytes,
        &package.name,
        &package.version,
    )?;
    let identity = identity_from_contents(
        &archive_bytes,
        package.name,
        package.version,
        candidate_commit,
        &contents,
    )?;
    let identity_bytes = serde_json::to_vec_pretty(&identity).map_err(|error| {
        PackageError::new(
            "artifact-identity",
            format!("failed to serialize artifact identity: {error}"),
        )
    })?;
    write_bytes(&identity_destination, &identity_bytes, "artifact identity")?;
    println!(
        "package artifact created: {} ({})",
        archive_destination.display(),
        identity.archive_sha256
    );
    Ok(())
}

pub(super) fn verify(repository_root: &Path, args: &[String]) -> Result<(), PackageError> {
    let options = parse_options(
        args,
        &["--archive", "--identity", "--toolchain", "--target"],
    )?;
    let archive_path = PathBuf::from(required_option(&options, "--archive")?);
    let identity_path = PathBuf::from(required_option(&options, "--identity")?);
    let toolchain = required_option(&options, "--toolchain")?;
    let target = required_option(&options, "--target")?;

    let identity = read_identity(&identity_path)?;
    validate_identity(&identity)?;
    validate_candidate_matches_repository(repository_root, &identity.candidate_commit)?;
    let repository_package = read_package_identity(repository_root)?;
    if identity.package != repository_package.name || identity.version != repository_package.version
    {
        return Err(PackageError::new(
            "artifact-identity",
            "artifact package/version differs from the candidate repository",
        ));
    }
    validate_platform(repository_root, toolchain, target)?;
    let archive_bytes = read_archive(&archive_path)?;
    verify_hash_and_size(&identity, &archive_bytes)?;
    let contents = inspect_contents(
        repository_root,
        &archive_path,
        &archive_bytes,
        &identity.package,
        &identity.version,
    )?;
    verify_contents(&identity, &contents)?;
    build_and_test_artifact(
        &contents.unpacked_crate,
        &contents.temporary.path,
        toolchain,
        target,
    )?;
    let evidence_tier = if toolchain == MSRV_TOOLCHAIN && target == CANONICAL_TARGET {
        "d1_canonical"
    } else {
        "d2_supported"
    };
    println!(
        "package artifact verified: {} {} {target} {evidence_tier}",
        identity.package, identity.archive_sha256
    );
    Ok(())
}

fn parse_options<'a>(
    args: &'a [String],
    expected: &[&str],
) -> Result<BTreeMap<&'a str, &'a str>, PackageError> {
    if !args.len().is_multiple_of(2) {
        return Err(PackageError::usage(
            "package artifact options require flag/value pairs",
        ));
    }
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        let flag = pair[0].as_str();
        if !expected.contains(&flag) || options.insert(flag, pair[1].as_str()).is_some() {
            return Err(PackageError::usage(format!(
                "unknown or duplicate package artifact option `{flag}`"
            )));
        }
    }
    if options.len() != expected.len() {
        return Err(PackageError::usage(
            "package artifact command is missing a required option",
        ));
    }
    Ok(options)
}

fn required_option<'a>(
    options: &'a BTreeMap<&str, &'a str>,
    name: &str,
) -> Result<&'a str, PackageError> {
    options
        .get(name)
        .copied()
        .ok_or_else(|| PackageError::usage(format!("missing `{name}`")))
}

fn create_archive(
    repository_root: &Path,
    package: &str,
    version: &str,
) -> Result<PathBuf, PackageError> {
    let toolchain = format!("+{CREATION_TOOLCHAIN}");
    run_process(
        &cargo_program(),
        [
            OsStr::new(&toolchain),
            OsStr::new("package"),
            OsStr::new("-p"),
            OsStr::new(package),
            OsStr::new("--allow-dirty"),
            OsStr::new("--no-verify"),
        ],
        repository_root,
        None,
        "create reusable Cargo package artifact",
    )?;
    let target_directory = cargo_target_directory(repository_root);
    let archive = target_directory.join(format!("package/{package}-{version}.crate"));
    if !archive.is_file() {
        return Err(PackageError::new(
            "archive",
            format!("cargo package did not create {}", archive.display()),
        ));
    }
    Ok(archive)
}

fn cargo_target_directory(repository_root: &Path) -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR").map_or_else(
        || repository_root.join("target"),
        |path| {
            let path = PathBuf::from(path);
            if path.is_absolute() {
                path
            } else {
                repository_root.join(path)
            }
        },
    )
}

fn write_bytes(path: &Path, bytes: &[u8], name: &str) -> Result<(), PackageError> {
    let Some(parent) = path.parent() else {
        return Err(PackageError::new(
            "filesystem",
            format!("{name} path has no parent: {}", path.display()),
        ));
    };
    if !parent.as_os_str().is_empty() {
        fs::create_dir_all(parent).map_err(|error| {
            PackageError::new(
                "filesystem",
                format!("failed to create {}: {error}", parent.display()),
            )
        })?;
    }
    fs::write(path, bytes).map_err(|error| {
        PackageError::new(
            "filesystem",
            format!("failed to write {}: {error}", path.display()),
        )
    })
}

fn inspect_contents(
    repository_root: &Path,
    archive_path: &Path,
    archive_bytes: &[u8],
    package: &str,
    version: &str,
) -> Result<ArchiveContents, PackageError> {
    let package_prefix = format!("{package}-{version}");
    let entries = inspect_archive(archive_bytes, archive_path, &package_prefix)?;
    let temporary = TemporaryDirectory::create()?;
    extract_archive(archive_bytes, archive_path, &temporary.path)?;
    let unpacked_crate = temporary.path.join(&package_prefix);
    let manifest_path = unpacked_crate.join("Cargo.toml");
    if !manifest_path.is_file() {
        return Err(PackageError::new(
            "archive",
            format!("archive did not contain {package_prefix}/Cargo.toml"),
        ));
    }
    let manifest = super::metadata::verify_packaged_manifest(&manifest_path)?;
    verify_license(repository_root, &unpacked_crate)?;
    let mut source_files = entries
        .iter()
        .filter_map(|entry| {
            let relative = entry.strip_prefix(&package_prefix).ok()?;
            unpacked_crate
                .join(relative)
                .is_file()
                .then(|| relative.to_string_lossy().replace('\\', "/"))
        })
        .collect::<Vec<_>>();
    source_files.sort();
    let license_files = source_files
        .iter()
        .filter(|path| path.rsplit('/').next() == Some("LICENSE"))
        .cloned()
        .collect();
    let notice_files = source_files
        .iter()
        .filter(|path| {
            path.rsplit('/')
                .next()
                .is_some_and(|name| name.to_ascii_uppercase().contains("NOTICE"))
        })
        .cloned()
        .collect();
    Ok(ArchiveContents {
        manifest,
        source_files,
        license_files,
        notice_files,
        unpacked_crate,
        temporary,
    })
}

fn identity_from_contents(
    archive_bytes: &[u8],
    package: String,
    version: String,
    candidate_commit: &str,
    contents: &ArchiveContents,
) -> Result<PackageArtifactIdentity, PackageError> {
    let archive_bytes_length = u64::try_from(archive_bytes.len()).map_err(|_| {
        PackageError::new(
            "artifact-identity",
            "archive byte length is not representable",
        )
    })?;
    Ok(PackageArtifactIdentity {
        schema_version: ARTIFACT_SCHEMA_VERSION,
        archive_sha256: sha256(archive_bytes),
        archive_bytes: archive_bytes_length,
        package,
        version,
        rust_version: contents.manifest.rust_version.clone(),
        features: contents.manifest.features.clone(),
        normal_dependencies: contents.manifest.normal_dependencies.clone(),
        source_files: contents.source_files.clone(),
        license_files: contents.license_files.clone(),
        notice_files: contents.notice_files.clone(),
        candidate_commit: candidate_commit.to_owned(),
        created_with_toolchain: CREATION_TOOLCHAIN.to_owned(),
        scalar_mode: SCALAR_MODE.to_owned(),
        compiler_class: COMPILER_CLASS.to_owned(),
        tolerance_profile: TOLERANCE_PROFILE.to_owned(),
    })
}

fn read_identity(path: &Path) -> Result<PackageArtifactIdentity, PackageError> {
    let bytes = fs::read(path).map_err(|error| {
        PackageError::new(
            "artifact-identity",
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        PackageError::new(
            "artifact-identity",
            format!("invalid {}: {error}", path.display()),
        )
    })
}

fn build_and_test_artifact(
    unpacked_crate: &Path,
    temporary_root: &Path,
    toolchain: &str,
    target: &str,
) -> Result<(), PackageError> {
    let cargo = cargo_program();
    let toolchain_argument = format!("+{toolchain}");
    let target_dir = temporary_root.join("target");
    for (action, arguments) in [
        (
            "build exact package artifact",
            vec![
                "build",
                "--all-targets",
                "--all-features",
                "--locked",
                "--target",
                target,
            ],
        ),
        (
            "test exact package artifact",
            vec![
                "test",
                "--all-features",
                "--locked",
                "--no-fail-fast",
                "--target",
                target,
            ],
        ),
    ] {
        let mut command_arguments = vec![OsString::from(&toolchain_argument)];
        command_arguments.extend(arguments.into_iter().map(OsString::from));
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
