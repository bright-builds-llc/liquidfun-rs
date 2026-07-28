#[allow(
    clippy::wildcard_imports,
    reason = "this split module shares its parent private contract"
)]
use super::*;

pub(super) const fn build_evidence_label(tier: BuildEvidenceTier) -> &'static str {
    match tier {
        BuildEvidenceTier::D1Canonical => "d1_canonical",
        BuildEvidenceTier::D2Supported => "d2_supported",
        BuildEvidenceTier::D3Exploratory => "d3_exploratory",
    }
}

pub(super) fn native_source_manifest_sha256(
    repository_root: &Path,
) -> Result<String, DifferentialError> {
    let manifest_path = repository_root.join(NATIVE_SOURCE_MANIFEST);
    let manifest = fs::read_to_string(&manifest_path).map_err(|error| {
        DifferentialError::new(
            "identity",
            format!("failed to read {}: {error}", manifest_path.display()),
        )
    })?;
    native_source_digest_from_manifest(repository_root, &manifest)
}

pub(super) fn native_source_digest_from_manifest(
    repository_root: &Path,
    manifest: &str,
) -> Result<String, DifferentialError> {
    let mut hasher = Sha256::new();
    for relative in manifest
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let path = Path::new(relative);
        if path.is_absolute()
            || path
                .components()
                .any(|part| matches!(part, std::path::Component::ParentDir))
        {
            return Err(DifferentialError::new(
                "identity",
                format!("invalid native source manifest path `{relative}`"),
            ));
        }
        let bytes = fs::read(repository_root.join(path)).map_err(|error| {
            DifferentialError::new(
                "identity",
                format!("failed to hash native source `{relative}`: {error}"),
            )
        })?;
        let relative_len = u64::try_from(relative.len())
            .map_err(|_| DifferentialError::new("identity", "native source path is too long"))?;
        hasher.update(relative_len.to_be_bytes());
        hasher.update(relative.as_bytes());
        hasher.update(Sha256::digest(bytes));
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub(super) fn read_regular_file(
    repository_root: &Path,
    relative: &str,
) -> Result<Vec<u8>, DifferentialError> {
    let path = repository_root.join(relative);
    let metadata = fs::symlink_metadata(&path).map_err(|error| {
        DifferentialError::new(
            "filesystem",
            format!("failed to inspect {}: {error}", path.display()),
        )
    })?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(DifferentialError::new(
            "filesystem",
            format!("{} must be a regular checked-in file", path.display()),
        ));
    }
    fs::read(&path).map_err(|error| {
        DifferentialError::new(
            "filesystem",
            format!("failed to read {}: {error}", path.display()),
        )
    })
}

pub(super) struct MathProbeCapture {
    pub(super) results: Vec<MathProbeResult>,
    pub(super) response_bytes: Vec<u8>,
    pub(super) oracle_identity: BuildIdentity,
}

pub(super) struct CollisionProbeCapture {
    pub(super) results: Vec<CollisionProbeResult>,
    pub(super) response_bytes: Vec<u8>,
}

pub(super) fn execute_collision_probe_once(
    repository_root: &Path,
    request: &CollisionProbeRequestRecord,
    preset: &str,
) -> Result<CollisionProbeCapture, DifferentialError> {
    let oracle_preset = match preset {
        "oracle-debug" => OraclePreset::Debug,
        "oracle-release" => OraclePreset::Release,
        _ => {
            return Err(DifferentialError::usage(
                "unregistered collision-probe preset",
            ));
        }
    };
    let executable = OracleExecutable::resolve(repository_root, oracle_preset)
        .map_err(|error| DifferentialError::new("oracle", error.to_string()))?;
    let capture = execute_collision_probe_process(&executable, request, ORACLE_REVISION).map_err(
        |error| {
            DifferentialError::process(format!(
                "{}; stderr bytes {}, killed {}, reaped {}: {}",
                error,
                error.stderr_bytes(),
                error.child_killed(),
                error.child_reaped(),
                String::from_utf8_lossy(error.retained_stderr()).trim_end()
            ))
        },
    )?;
    if capture.identity().cmake_preset() != preset || capture.identity().maybe_phase4().is_none() {
        return Err(DifferentialError::new(
            "identity",
            "oracle handshake lacks the requested collision build identity",
        ));
    }
    let expected_adapter_digest = liquidfun_differential::adapter_source_digest(repository_root)
        .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    if capture.identity().adapter_content_sha256().as_str() != expected_adapter_digest {
        return Err(DifferentialError::new(
            "identity",
            "oracle adapter digest differs from checked-in inputs",
        ));
    }
    let expected_compile_digest =
        liquidfun_differential::effective_compile_command_sha256(repository_root, preset)
            .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    if capture
        .identity()
        .maybe_phase4()
        .is_none_or(|identity| identity.compile_command_sha256() != expected_compile_digest)
    {
        return Err(DifferentialError::new(
            "identity",
            "oracle collision compile-command digest differs from the effective database",
        ));
    }
    Ok(CollisionProbeCapture {
        results: capture.results().to_vec(),
        response_bytes: capture.response_bytes().to_vec(),
    })
}

pub(super) fn execute_math_probe_once(
    repository_root: &Path,
    request: &MathProbeRequestRecord,
    preset: &str,
) -> Result<MathProbeCapture, DifferentialError> {
    let oracle_preset = match preset {
        "oracle-debug" => OraclePreset::Debug,
        "oracle-release" => OraclePreset::Release,
        _ => return Err(DifferentialError::usage("unregistered math-probe preset")),
    };
    let executable = OracleExecutable::resolve(repository_root, oracle_preset)
        .map_err(|error| DifferentialError::new("oracle", error.to_string()))?;
    let capture =
        execute_math_probe_process(&executable, request, ORACLE_REVISION).map_err(|error| {
            let stderr = String::from_utf8_lossy(error.retained_stderr());
            DifferentialError::process(format!(
                "{}; stderr bytes {}, killed {}, reaped {}: {}",
                error,
                error.stderr_bytes(),
                error.child_killed(),
                error.child_reaped(),
                stderr.trim_end()
            ))
        })?;
    if capture.identity().cmake_preset() != preset || capture.identity().maybe_phase4().is_none() {
        return Err(DifferentialError::new(
            "identity",
            "oracle handshake lacks the requested Phase 4 build identity",
        ));
    }
    let expected_adapter_digest = liquidfun_differential::adapter_source_digest(repository_root)
        .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    if capture.identity().adapter_content_sha256().as_str() != expected_adapter_digest {
        return Err(DifferentialError::new(
            "identity",
            "oracle adapter digest differs from independently hashed checked-in inputs",
        ));
    }
    let expected_compile_digest =
        liquidfun_differential::effective_compile_command_sha256(repository_root, preset)
            .map_err(|error| DifferentialError::new("identity", error.to_string()))?;
    let phase4_identity = capture
        .identity()
        .maybe_phase4()
        .ok_or_else(|| DifferentialError::new("identity", "Phase 4 identity is missing"))?;
    if phase4_identity.compile_command_sha256() != expected_compile_digest {
        return Err(DifferentialError::new(
            "identity",
            "oracle compile-command digest differs from the effective compile database",
        ));
    }
    Ok(MathProbeCapture {
        results: capture.results().to_vec(),
        response_bytes: capture.response_bytes().to_vec(),
        oracle_identity: capture.identity().clone(),
    })
}
