use super::{
    BTreeMap, CanonicalEnvironment, Command, Component, Digest, EvidenceMetadata, ORACLE_PRESET,
    Output, Path, PathBuf, Phase13EvidenceError, Phase13EvidenceErrorKind, ProductionGate,
    Serialize, Sha256, UPSTREAM_REVISION, env, fs,
};

pub(super) fn bounded_text(value: &str, maximum_chars: usize) -> String {
    value.chars().take(maximum_chars).collect()
}

pub(super) fn canonical_environment() -> Result<CanonicalEnvironment, Phase13EvidenceError> {
    if env::consts::OS != "linux" || env::consts::ARCH != "x86_64" {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Environment,
            "canonical producer runs on x86_64 Linux only",
        ));
    }
    let rust = command_text(Command::new("rustc").arg("-vV"), "read rustc identity")?;
    let cmake = command_text(
        Command::new("cmake").arg("--version"),
        "read CMake identity",
    )?;
    let ninja = command_text(
        Command::new("ninja").arg("--version"),
        "read Ninja identity",
    )?;
    let clang = command_text(
        Command::new(env::var_os("CXX").unwrap_or_else(|| "clang++-22".into())).arg("--version"),
        "read Clang identity",
    )?;
    let rust_version = field_after(&rust, "release: ")?;
    let rust_target = field_after(&rust, "host: ")?;
    let environment = CanonicalEnvironment {
        operating_system: env::consts::OS.to_owned(),
        architecture: env::consts::ARCH.to_owned(),
        rust_target,
        rust_version,
        cmake_version: token_after(&cmake, "cmake version ")?,
        ninja_version: ninja.lines().next().unwrap_or_default().trim().to_owned(),
        clang_version: token_after(&clang, "clang version ")?,
        cmake_preset: ORACLE_PRESET.to_owned(),
    };
    let dummy = ProductionGate {
        producer_sha: "a".repeat(40),
        upstream_revision: UPSTREAM_REVISION.to_owned(),
        environment: environment.clone(),
        witness_repeat_sha256: ["a".repeat(64), "a".repeat(64)],
        native_d0_repeat_sha256: ["a".repeat(64), "a".repeat(64)],
        d1_oracle_passed: true,
        sealed_input_sha256: "a".repeat(64),
        d1_input_sha256: "a".repeat(64),
    };
    dummy.validate().map_err(|error| {
        Phase13EvidenceError::new(Phase13EvidenceErrorKind::Environment, error.to_string())
    })?;
    Ok(environment)
}

pub(super) fn run_xtask(repository_root: &Path, args: &[&str]) -> Result<(), Phase13EvidenceError> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    run_process(
        Command::new(cargo)
            .current_dir(repository_root)
            .arg("xtask")
            .args(args),
        "run nested repository orchestration",
    )
    .map(|_output| ())
}

pub(super) fn require_upstream_revision(
    repository_root: &Path,
) -> Result<(), Phase13EvidenceError> {
    let revision = git_text(
        &repository_root.join("third_party/liquidfun"),
        &["rev-parse", "HEAD"],
    )?;
    if revision != UPSTREAM_REVISION {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Git,
            "upstream checkout does not equal the pinned revision",
        ));
    }
    Ok(())
}

pub(super) fn parse_options(
    args: &[String],
) -> Result<BTreeMap<String, String>, Phase13EvidenceError> {
    if !args.len().is_multiple_of(2) {
        return Err(Phase13EvidenceError::usage(
            "every option requires exactly one value",
        ));
    }
    let mut options = BTreeMap::new();
    for pair in args.chunks_exact(2) {
        if !pair[0].starts_with("--") || options.insert(pair[0].clone(), pair[1].clone()).is_some()
        {
            return Err(Phase13EvidenceError::usage(
                "options must be unique option/value pairs",
            ));
        }
    }
    Ok(options)
}

pub(super) fn require_options(
    options: &BTreeMap<String, String>,
    required_names: &[&str],
) -> Result<(), Phase13EvidenceError> {
    require_allowed_options(options, required_names, &[])
}

pub(super) fn require_allowed_options(
    options: &BTreeMap<String, String>,
    required_names: &[&str],
    optional_names: &[&str],
) -> Result<(), Phase13EvidenceError> {
    let exact = required_names
        .iter()
        .all(|name| options.contains_key(*name))
        && options.keys().all(|name| {
            required_names.contains(&name.as_str()) || optional_names.contains(&name.as_str())
        });
    if exact {
        return Ok(());
    }
    Err(Phase13EvidenceError::usage(
        "command options do not match the closed contract",
    ))
}

pub(super) fn required<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, Phase13EvidenceError> {
    options
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| Phase13EvidenceError::usage(format!("missing `{name}`")))
}

pub(super) fn repository_root() -> Result<PathBuf, Phase13EvidenceError> {
    let current = env::current_dir().map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            format!("failed to read current directory: {error}"),
        )
    })?;
    current
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file()
                && candidate.join("crates/liquidfun/Cargo.toml").is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Filesystem,
                "repository root is unavailable",
            )
        })
}

pub(super) fn absolute_path(repository_root: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repository_root.join(path)
    }
}

pub(super) fn lexical_absolute(path: &Path) -> Result<PathBuf, Phase13EvidenceError> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|error| {
                Phase13EvidenceError::new(
                    Phase13EvidenceErrorKind::Filesystem,
                    format!("failed to read current directory: {error}"),
                )
            })?
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(Phase13EvidenceError::new(
                        Phase13EvidenceErrorKind::Filesystem,
                        "path normalization escaped its root",
                    ));
                }
            }
        }
    }
    Ok(normalized)
}

pub(super) fn git_text(
    repository_root: &Path,
    args: &[&str],
) -> Result<String, Phase13EvidenceError> {
    let output = run_process(
        Command::new("git")
            .arg("-C")
            .arg(repository_root)
            .args(args),
        "query Git identity",
    )?;
    String::from_utf8(output.stdout)
        .map(|text| text.trim().to_owned())
        .map_err(|error| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Git,
                format!("Git returned non-UTF-8 output: {error}"),
            )
        })
}

pub(super) fn run_process(
    command: &mut Command,
    action: &str,
) -> Result<Output, Phase13EvidenceError> {
    let output = command.output().map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Process,
            format!("failed to {action}: {error}"),
        )
    })?;
    if !output.status.success() {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Process,
            format!(
                "failed to {action} with {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    Ok(output)
}

pub(super) fn command_text(
    command: &mut Command,
    action: &str,
) -> Result<String, Phase13EvidenceError> {
    let output = run_process(command, action)?;
    String::from_utf8(output.stdout).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Process,
            format!("{action} returned non-UTF-8 output: {error}"),
        )
    })
}

pub(super) fn field_after(text: &str, prefix: &str) -> Result<String, Phase13EvidenceError> {
    text.lines()
        .find_map(|line| line.strip_prefix(prefix))
        .map(str::to_owned)
        .ok_or_else(|| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Environment,
                format!("tool identity omitted `{prefix}`"),
            )
        })
}

pub(super) fn token_after(text: &str, prefix: &str) -> Result<String, Phase13EvidenceError> {
    text.lines()
        .find_map(|line| line.split_once(prefix).map(|(_, tail)| tail))
        .and_then(|tail| tail.split_whitespace().next())
        .map(str::to_owned)
        .ok_or_else(|| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Environment,
                format!("tool identity omitted `{prefix}`"),
            )
        })
}

pub(super) fn file_sha256(path: &Path) -> Result<String, Phase13EvidenceError> {
    fs::read(path).map(|bytes| sha256(&bytes)).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            format!("failed to hash {}: {error}", path.display()),
        )
    })
}

pub(super) fn update_length_prefixed(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

pub(super) fn metadata(record_class: &str, source_path: &str) -> EvidenceMetadata {
    let (derivation_kind, alteration_summary) = match record_class {
        "witness" => (
            "generated-semantic-oracle-witness",
            "Repository-authored semantic observations generated from the pinned upstream oracle without copying source, raw object memory, or Rust-produced expectations.",
        ),
        "replay_evidence" => (
            "repository-authored-replay-verification",
            "Repository-authored replay results derived from a canonical oracle bundle; no upstream source, raw object memory, or Rust-produced expectations are copied.",
        ),
        _ => (
            "repository-authored-staged-evidence-bundle",
            "Repository-authored immutable bundle metadata assembling oracle evidence and provenance records; no upstream source or raw object memory is copied.",
        ),
    };
    EvidenceMetadata {
        record_class: record_class.to_owned(),
        source_revision: UPSTREAM_REVISION.to_owned(),
        source_path: source_path.to_owned(),
        derivation_kind: derivation_kind.to_owned(),
        alteration_summary: alteration_summary.to_owned(),
        notice_refs: vec!["THIRD_PARTY_NOTICES.md".to_owned()],
    }
}

pub(super) fn json_bytes(value: &impl Serialize) -> Result<Vec<u8>, Phase13EvidenceError> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("failed to encode evidence record: {error}"),
        )
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub(super) fn validate_relative_path(value: &str) -> Result<(), Phase13EvidenceError> {
    if !value.is_empty()
        && !value.contains('\\')
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Ok(());
    }
    Err(Phase13EvidenceError::new(
        Phase13EvidenceErrorKind::Filesystem,
        format!("unsafe producer-affecting path `{value}`"),
    ))
}

pub(super) fn provider_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(valid_digest)
}

pub(super) fn path_text(path: &Path) -> Result<String, Phase13EvidenceError> {
    path.to_str().map(str::to_owned).ok_or_else(|| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            "path is not valid UTF-8",
        )
    })
}

pub(super) fn valid_revision(value: &str) -> bool {
    value.len() == 40 && lower_hex(value)
}

pub(super) fn valid_digest(value: &str) -> bool {
    value.len() == 64 && lower_hex(value)
}

fn lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
