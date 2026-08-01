//! Current-checkout oracle adapter and effective compile-command identity.

use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path},
};

use liquidfun_test_protocol::BuildIdentity;
use sha2::{Digest, Sha256};

const ADAPTER_INPUT_MANIFEST: &str = "tools/reference/adapter-inputs.txt";
const RESULT_TRANSLATION_UNITS: [&str; 4] = [
    "collision_probe.cpp",
    "math_probe.cpp",
    "protocol_bits.cpp",
    "rigid_world.cpp",
];
const REVIEWED_PRESETS: [&str; 3] = ["oracle-debug", "oracle-release", "oracle-asan-ubsan"];

/// Failure to independently bind an oracle identity to current repository inputs.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum OracleCheckoutIdentityError {
    /// The reviewed adapter-input manifest could not be read.
    #[error("oracle adapter input manifest is unreadable")]
    ManifestRead,
    /// A manifest entry was not one unique confined relative path.
    #[error("oracle adapter input manifest entry {line} is invalid or duplicated")]
    InvalidAdapterPath {
        /// One-based manifest line number.
        line: usize,
    },
    /// The manifest had no adapter inputs.
    #[error("oracle adapter input manifest is empty")]
    EmptyAdapterManifest,
    /// One reviewed adapter input could not be read.
    #[error("oracle adapter input {index} is unreadable")]
    AdapterInputRead {
        /// One-based reviewed input index.
        index: usize,
    },
    /// The caller requested a build directory outside the closed preset registry.
    #[error("oracle identity preset is not reviewed")]
    InvalidPreset,
    /// The selected preset's effective compile database could not be read.
    #[error("oracle effective compile database is unreadable")]
    CompileDatabaseRead,
    /// The selected preset's effective compile database was not valid JSON.
    #[error("oracle effective compile database is malformed")]
    CompileDatabaseMalformed,
    /// One reviewed result translation unit appeared more than once.
    #[error("oracle effective compile database duplicates {unit}")]
    DuplicateCompileUnit {
        /// Closed reviewed translation-unit filename.
        unit: &'static str,
    },
    /// One reviewed result translation unit was absent.
    #[error("oracle effective compile database is missing {unit}")]
    MissingCompileUnit {
        /// Closed reviewed translation-unit filename.
        unit: &'static str,
    },
    /// A reviewed result translation unit had no usable command representation.
    #[error("oracle effective compile command for {unit} is malformed")]
    MalformedCompileCommand {
        /// Closed reviewed translation-unit filename.
        unit: &'static str,
    },
    /// The four result units did not share one effective command shape.
    #[error("oracle result translation units use divergent effective compile flags")]
    DivergentCompileFlags,
    /// The child did not report the Phase 4 compile provenance extension.
    #[error("oracle handshake lacks Phase 4 compile provenance")]
    MissingPhase4Identity,
    /// The child adapter digest differs from current reviewed source bytes.
    #[error("oracle adapter digest differs from current checkout inputs")]
    AdapterDigestMismatch,
    /// The child compile digest differs from the selected preset's effective database.
    #[error("oracle compile-command digest differs from the current effective database")]
    CompileDigestMismatch,
}

/// Hashes the confined reviewed adapter inputs from current repository bytes.
///
/// # Errors
///
/// Returns [`OracleCheckoutIdentityError`] for an unreadable or invalid manifest or input.
pub fn adapter_source_digest(
    repository_root: &Path,
) -> Result<String, OracleCheckoutIdentityError> {
    let manifest = fs::read_to_string(repository_root.join(ADAPTER_INPUT_MANIFEST))
        .map_err(|_| OracleCheckoutIdentityError::ManifestRead)?;
    let mut seen = BTreeSet::new();
    let mut paths = Vec::new();
    for (line_index, line) in manifest.lines().enumerate() {
        let relative = line.trim();
        if relative.is_empty() || relative.starts_with('#') {
            continue;
        }
        let path = Path::new(relative);
        if path.is_absolute()
            || path
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
            || !seen.insert(relative.to_owned())
        {
            return Err(OracleCheckoutIdentityError::InvalidAdapterPath {
                line: line_index + 1,
            });
        }
        paths.push(relative.to_owned());
    }
    if paths.is_empty() {
        return Err(OracleCheckoutIdentityError::EmptyAdapterManifest);
    }

    let mut digest_input = Sha256::new();
    for (index, relative) in paths.iter().enumerate() {
        let bytes = fs::read(repository_root.join(relative))
            .map_err(|_| OracleCheckoutIdentityError::AdapterInputRead { index: index + 1 })?;
        let source_digest = Sha256::digest(bytes);
        digest_input.update(relative.as_bytes());
        digest_input.update(b"=");
        digest_input.update(format!("{source_digest:x}").as_bytes());
        digest_input.update(b"\n");
    }

    Ok(format!("{:x}", digest_input.finalize()))
}

/// Hashes the normalized effective commands for the four reviewed result units.
///
/// # Errors
///
/// Returns [`OracleCheckoutIdentityError`] for an unreviewed preset or invalid database.
pub fn effective_compile_command_sha256(
    repository_root: &Path,
    preset: &str,
) -> Result<String, OracleCheckoutIdentityError> {
    if !REVIEWED_PRESETS.contains(&preset) {
        return Err(OracleCheckoutIdentityError::InvalidPreset);
    }
    let build_directory = repository_root.join("target/reference").join(preset);
    let bytes = fs::read(build_directory.join("compile_commands.json"))
        .map_err(|_| OracleCheckoutIdentityError::CompileDatabaseRead)?;
    compile_database_sha256(&bytes, repository_root, &build_directory)
}

/// Recomputes and validates both current-checkout oracle identity digests.
///
/// # Errors
///
/// Returns [`OracleCheckoutIdentityError`] when local inputs are invalid or either digest differs.
pub fn validate_oracle_checkout_identity(
    repository_root: &Path,
    preset: &str,
    identity: &BuildIdentity,
) -> Result<(), OracleCheckoutIdentityError> {
    let expected_adapter = adapter_source_digest(repository_root)?;
    if identity.adapter_content_sha256().as_str() != expected_adapter {
        return Err(OracleCheckoutIdentityError::AdapterDigestMismatch);
    }
    let expected_compile = effective_compile_command_sha256(repository_root, preset)?;
    let phase4 = identity
        .maybe_phase4()
        .ok_or(OracleCheckoutIdentityError::MissingPhase4Identity)?;
    if phase4.compile_command_sha256() != expected_compile {
        return Err(OracleCheckoutIdentityError::CompileDigestMismatch);
    }
    Ok(())
}

fn compile_database_sha256(
    bytes: &[u8],
    repository_root: &Path,
    build_directory: &Path,
) -> Result<String, OracleCheckoutIdentityError> {
    let entries: Vec<serde_json::Value> = serde_json::from_slice(bytes)
        .map_err(|_| OracleCheckoutIdentityError::CompileDatabaseMalformed)?;
    let mut commands = Vec::new();
    let mut command_signatures = Vec::new();
    let mut found_units = BTreeSet::new();
    for entry in entries {
        let Some(source) = entry.get("file").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(filename) = Path::new(source)
            .file_name()
            .and_then(|value| value.to_str())
        else {
            continue;
        };
        let Some(unit) = RESULT_TRANSLATION_UNITS
            .iter()
            .copied()
            .find(|unit| *unit == filename)
        else {
            continue;
        };
        if !found_units.insert(unit) {
            return Err(OracleCheckoutIdentityError::DuplicateCompileUnit { unit });
        }
        let command = compile_database_command(&entry, unit)?;
        let normalized_source = normalize_compile_path(source, repository_root, build_directory);
        let normalized_command = normalize_compile_path(&command, repository_root, build_directory);
        let normalized_signature = normalized_command.replace(unit, "<result-unit>.cpp");
        command_signatures.push(normalize_result_target_directories(
            &normalized_signature,
            unit,
        )?);
        commands.push(format!("{normalized_source}\n{normalized_command}"));
    }
    for unit in RESULT_TRANSLATION_UNITS {
        if !found_units.contains(unit) {
            return Err(OracleCheckoutIdentityError::MissingCompileUnit { unit });
        }
    }
    let Some(first_signature) = command_signatures.first() else {
        return Err(OracleCheckoutIdentityError::MissingCompileUnit {
            unit: RESULT_TRANSLATION_UNITS[0],
        });
    };
    if command_signatures
        .iter()
        .any(|signature| signature != first_signature)
    {
        return Err(OracleCheckoutIdentityError::DivergentCompileFlags);
    }
    commands.sort_unstable();
    Ok(format!("{:x}", Sha256::digest(commands.join("\n"))))
}

fn compile_database_command(
    entry: &serde_json::Value,
    unit: &'static str,
) -> Result<String, OracleCheckoutIdentityError> {
    if let Some(command) = entry.get("command").and_then(serde_json::Value::as_str) {
        return Ok(command.to_owned());
    }
    let arguments = entry
        .get("arguments")
        .and_then(serde_json::Value::as_array)
        .ok_or(OracleCheckoutIdentityError::MalformedCompileCommand { unit })?;
    let mut command = String::new();
    for argument in arguments {
        let argument = argument
            .as_str()
            .ok_or(OracleCheckoutIdentityError::MalformedCompileCommand { unit })?;
        command.push_str(argument);
        command.push('\n');
    }
    Ok(command)
}

fn normalize_compile_path(value: &str, repository_root: &Path, build_directory: &Path) -> String {
    let build = build_directory.to_string_lossy();
    let repository = repository_root.to_string_lossy();
    value
        .replace(build.as_ref(), "<build>")
        .replace(repository.as_ref(), "<repo>")
}

fn normalize_result_target_directories(
    value: &str,
    unit: &'static str,
) -> Result<String, OracleCheckoutIdentityError> {
    const PREFIX: &str = "CMakeFiles/";
    const SUFFIX: &str = ".dir";

    let mut normalized = String::with_capacity(value.len());
    let mut remaining = value;
    while let Some(prefix_index) = remaining.find(PREFIX) {
        let target_start = prefix_index + PREFIX.len();
        normalized.push_str(&remaining[..target_start]);
        let target_and_remainder = &remaining[target_start..];
        let Some(suffix_index) = target_and_remainder.find(SUFFIX) else {
            return Err(OracleCheckoutIdentityError::MalformedCompileCommand { unit });
        };
        let target = &target_and_remainder[..suffix_index];
        if target.is_empty()
            || target
                .chars()
                .any(|character| character == '/' || character.is_whitespace())
        {
            return Err(OracleCheckoutIdentityError::MalformedCompileCommand { unit });
        }
        normalized.push_str("<result-target>.dir");
        remaining = &target_and_remainder[suffix_index + SUFFIX.len()..];
    }
    normalized.push_str(remaining);
    Ok(normalized)
}
