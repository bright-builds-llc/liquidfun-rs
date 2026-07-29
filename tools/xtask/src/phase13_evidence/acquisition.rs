use super::replay::{MaterialsManifest, persist_acquisition_failure};
use super::support::{
    absolute_path, file_sha256, json_bytes, path_text, provider_digest, require_options, required,
    run_process, sha256, update_length_prefixed, validate_relative_path,
};
use super::{
    BTreeMap, BTreeSet, BuildEvidenceTier, CatalogComparisonSurface, CatalogDefinition,
    CatalogFailureBundleRequest, CatalogFailureKind, CatalogOracleSupervisor, CatalogRunOutcome,
    CatalogRunRequest, CatalogSlug, ClosureEntry, ClosureIdentity, Command, ComparisonState,
    Component, Digest, EvidenceTier, MATERIALS_MANIFEST, ORACLE_PRESET, OracleExecutable,
    OraclePreset, Path, Phase13EvidenceError, Phase13EvidenceErrorKind, RIGID_STACK_CATALOG_SLUG,
    ReplayDriftClass, ReplayProjectionVersion, RequestId, ResolveRequest,
    RunProvenanceRequirements, ScenarioCatalog, SessionProfile, Sha256, UPSTREAM_REVISION,
    WITNESS_EXECUTABLE, WITNESS_REPOSITORY_PREFIXES, check_bundle, closure_digest,
    compare_catalog_physics_projection, execute_catalog_native, fs,
    legacy_physics_checkpoint_sha256, persist_catalog_failure_bundle, replay_catalog_regressions,
    resolve_catalog, reviewed_scenario_catalog,
};

pub(super) struct WitnessOutput {
    pub(super) bytes: Vec<u8>,
    pub(super) repeat_sha256: [String; 2],
    pub(super) invocation: Vec<String>,
}

pub(super) fn produce_witness(
    repository_root: &Path,
    temporary_root: &Path,
) -> Result<WitnessOutput, Phase13EvidenceError> {
    let executable = repository_root.join(WITNESS_EXECUTABLE);
    let output_path = temporary_root.join("witness.json");
    let provenance_path = temporary_root.join("witness.provenance.raw.json");
    let invocation = vec![
        WITNESS_EXECUTABLE.to_owned(),
        "--output".to_owned(),
        path_text(&output_path)?,
        "--provenance".to_owned(),
        path_text(&provenance_path)?,
    ];
    let mut repeat_sha256 = [String::new(), String::new()];
    let mut bytes = Vec::new();
    for digest in &mut repeat_sha256 {
        run_process(
            Command::new(&executable)
                .current_dir(repository_root)
                .args(&invocation[1..]),
            "run the Phase 9 witness oracle",
        )?;
        bytes = fs::read(&output_path).map_err(|error| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Filesystem,
                format!("failed to read generated witness: {error}"),
            )
        })?;
        *digest = sha256(&bytes);
    }
    Ok(WitnessOutput {
        bytes,
        repeat_sha256,
        invocation,
    })
}

pub(super) struct ReplayOutput {
    pub(super) sealed_input_sha256: String,
    pub(super) native_repeat_sha256: [String; 2],
    pub(super) oracle_identity_sha256: String,
    pub(super) d1_passed: bool,
    pub(super) d1_diagnostic: Option<String>,
    pub(super) diagnosis: serde_json::Value,
}

pub(super) struct ReplayAcquisition {
    pub(super) output: ReplayOutput,
}

pub(crate) fn select_rigid_stack_definition(
    catalog: &ScenarioCatalog,
) -> Result<(&CatalogDefinition, CatalogSlug), Phase13EvidenceError> {
    let slug = CatalogSlug::new(RIGID_STACK_CATALOG_SLUG.to_owned()).map_err(|error| {
        Phase13EvidenceError::new(Phase13EvidenceErrorKind::Protocol, error.to_string())
    })?;
    let definition = catalog
        .definitions()
        .iter()
        .find(|candidate| candidate.slug() == &slug)
        .ok_or_else(|| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Protocol,
                "rigid-stack-v1 is absent from the reviewed catalog",
            )
        })?;
    Ok((definition, slug))
}

#[allow(
    clippy::too_many_lines,
    reason = "the D0/D1 authority sequence remains linear so no comparison gate can be skipped"
)]
pub(super) fn acquire_replay(
    repository_root: &Path,
    persist_failures: bool,
) -> Result<ReplayAcquisition, Phase13EvidenceError> {
    let catalog = reviewed_scenario_catalog().map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("reviewed catalog is invalid: {error}"),
        )
    })?;
    let (definition, slug) = super::select_rigid_stack_definition(&catalog)?;
    let metadata = definition.metadata().ok_or_else(|| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            "rigid-stack-v1 has no reviewed metadata",
        )
    })?;
    let resolved = resolve_catalog(
        catalog.definitions(),
        &ResolveRequest::new(slug, None, metadata.default_settings()),
    )
    .map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("failed to resolve rigid-stack-v1: {error}"),
        )
    })?;
    let executable =
        OracleExecutable::resolve(repository_root, OraclePreset::Debug).map_err(|error| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Oracle,
                format!("canonical oracle is unavailable: {error}"),
            )
        })?;
    let mut supervisor =
        CatalogOracleSupervisor::new(executable, SessionProfile::Reuse, UPSTREAM_REVISION);
    let oracle_identity = supervisor.discover_identity().map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Oracle,
            format!("failed to discover oracle identity: {error}"),
        )
    })?;
    if oracle_identity.evidence_tier() != BuildEvidenceTier::D1Canonical {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Environment,
            "oracle build identity is not canonical D1",
        ));
    }
    let request_id = RequestId::new(format!(
        "phase13-{}",
        &resolved.identity().content_sha256().as_str()[..16]
    ))
    .map_err(|error| {
        Phase13EvidenceError::new(Phase13EvidenceErrorKind::Protocol, error.to_string())
    })?;
    let request = CatalogRunRequest::new(
        request_id,
        resolved,
        RunProvenanceRequirements::new(
            oracle_identity.identity_sha256().clone(),
            supervisor.limits_profile_sha256(),
            EvidenceTier::D1Canonical,
        ),
    )
    .map_err(|error| {
        Phase13EvidenceError::new(Phase13EvidenceErrorKind::Protocol, error.to_string())
    })?;
    let first = execute_catalog_native(&request).map_err(|error| {
        persist_acquisition_failure(
            repository_root,
            persist_failures,
            &request,
            error.kind(),
            &format!("first native D0 failed: {error}"),
            &[],
        )
    })?;
    let second = execute_catalog_native(&request).map_err(|error| {
        persist_acquisition_failure(
            repository_root,
            persist_failures,
            &request,
            error.kind(),
            &format!("second native D0 failed: {error}"),
            &[],
        )
    })?;
    let replay = replay_catalog_regressions(repository_root).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("reviewed replay diagnosis failed: {error}"),
        )
    })?;
    let replay_entry = replay
        .entries()
        .iter()
        .find(|entry| entry.fixture_id() == "rigid-stack-v1")
        .ok_or_else(|| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Protocol,
                "rigid-stack-v1 replay result is absent",
            )
        })?;
    let diagnosis = replay_entry.maybe_diagnosis().ok_or_else(|| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            "rigid-stack-v1 capture-schema diagnosis is absent",
        )
    })?;
    if diagnosis.drift_class() != ReplayDriftClass::CaptureSchemaDrift
        || diagnosis.reviewed_schema().projection_version()
            != ReplayProjectionVersion::LegacyPhysicsV1
        || diagnosis.current_schema().projection_version()
            != ReplayProjectionVersion::ExpandedCheckpointV1
    {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            "rigid-stack-v1 did not select the reviewed legacy physics projection",
        ));
    }
    let native_repeat_sha256 = [
        legacy_physics_checkpoint_sha256(first.checkpoints())
            .map_err(|error| {
                Phase13EvidenceError::new(
                    Phase13EvidenceErrorKind::Protocol,
                    format!("first legacy D0 projection failed: {error}"),
                )
            })?
            .as_str()
            .to_owned(),
        legacy_physics_checkpoint_sha256(second.checkpoints())
            .map_err(|error| {
                Phase13EvidenceError::new(
                    Phase13EvidenceErrorKind::Protocol,
                    format!("second legacy D0 projection failed: {error}"),
                )
            })?
            .as_str()
            .to_owned(),
    ];
    let oracle = supervisor.execute(&request).map_err(|error| {
        persist_acquisition_failure(
            repository_root,
            persist_failures,
            &request,
            error.kind(),
            &format!("pinned-oracle D1 execution failed: {error}"),
            error.retained_stderr(),
        )
    })?;
    let outcome =
        compare_catalog_physics_projection(&first, oracle.capture()).map_err(|error| {
            persist_acquisition_failure(
                repository_root,
                persist_failures,
                &request,
                error.kind(),
                &format!("D1 comparison failed: {error}"),
                &[],
            )
        })?;
    let d1_passed = matches!(outcome, CatalogRunOutcome::Match(_));
    let d1_diagnostic = match &outcome {
        CatalogRunOutcome::Match(_) => None,
        CatalogRunOutcome::PhysicsMismatch(mismatch) => {
            let comparison = mismatch.first_mismatch();
            let maybe_entry = comparison.entries().iter().find(|entry| {
                !matches!(
                    entry.state(),
                    ComparisonState::ExactMatch | ComparisonState::WithinPolicy
                )
            });
            Some(maybe_entry.map_or_else(
                || {
                    format!(
                        "D1 physics mismatch at checkpoint {}",
                        comparison.checkpoint_id().as_str()
                    )
                },
                |entry| {
                    format!(
                        "D1 physics mismatch at checkpoint {} path {} ({:?}): Rust={:?}, C++={:?}",
                        comparison.checkpoint_id().as_str(),
                        entry.semantic_path(),
                        entry.state(),
                        entry.maybe_rust_value(),
                        entry.maybe_oracle_value()
                    )
                },
            ))
        }
        CatalogRunOutcome::HarnessFailure(kind) => {
            Some(format!("D1 comparison reported harness failure {kind:?}"))
        }
    };
    if persist_failures && !d1_passed {
        let controller = json_bytes(&serde_json::json!({
            "controller_state": "physics_mismatch",
            "diagnostic": d1_diagnostic.as_deref(),
        }))?;
        let bundle = CatalogFailureBundleRequest::from_projection_captures(
            CatalogFailureKind::PhysicsMismatch,
            CatalogComparisonSurface::LegacyPhysicsV1,
            &request,
            &first,
            oracle.capture(),
            &[],
            &controller,
        )
        .map_err(|error| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Bundle,
                format!("failed to construct D1 mismatch evidence: {error}"),
            )
        })?;
        persist_catalog_failure_bundle(repository_root, &bundle).map_err(|error| {
            Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Bundle,
                format!("failed to persist D1 mismatch evidence: {error}"),
            )
        })?;
    }
    Ok(ReplayAcquisition {
        output: ReplayOutput {
            sealed_input_sha256: request
                .resolved()
                .identity()
                .content_sha256()
                .as_str()
                .to_owned(),
            native_repeat_sha256,
            oracle_identity_sha256: oracle_identity.identity_sha256().as_str().to_owned(),
            d1_passed,
            d1_diagnostic,
            diagnosis: serde_json::to_value(diagnosis).map_err(|error| {
                Phase13EvidenceError::new(
                    Phase13EvidenceErrorKind::Protocol,
                    format!("failed to encode replay diagnosis: {error}"),
                )
            })?,
        },
    })
}

pub(crate) fn witness_materials_identity(
    repository_root: &Path,
) -> Result<(String, usize), Phase13EvidenceError> {
    let manifest_bytes = fs::read(repository_root.join(MATERIALS_MANIFEST)).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            format!("failed to read scoped witness materials: {error}"),
        )
    })?;
    let manifest: MaterialsManifest = serde_json::from_slice(&manifest_bytes).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("invalid scoped witness materials: {error}"),
        )
    })?;
    if manifest.schema_version != 1
        || manifest.target != "phase9-lifecycle-contact-witness"
        || manifest.preset != ORACLE_PRESET
    {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            "scoped witness materials have the wrong schema, target, or preset",
        ));
    }
    let materials = manifest
        .materials
        .into_iter()
        .map(|material| (material.kind, material.identity))
        .collect::<BTreeSet<_>>();
    let mut digest = Sha256::new();
    for (kind, identity) in &materials {
        if !matches!(
            kind.as_str(),
            "build_rule"
                | "compile_definition"
                | "compile_fragment"
                | "generated_input"
                | "header"
                | "include_path"
                | "link_fragment"
                | "link_input"
                | "preset_value"
                | "source"
        ) {
            return Err(Phase13EvidenceError::new(
                Phase13EvidenceErrorKind::Protocol,
                format!("scoped witness material `{identity}` has unknown kind `{kind}`"),
            ));
        }
        update_length_prefixed(&mut digest, kind.as_bytes());
        update_length_prefixed(&mut digest, identity.as_bytes());
        if matches!(
            kind.as_str(),
            "build_rule" | "generated_input" | "header" | "source"
        ) {
            let path = Path::new(identity);
            if path.is_absolute()
                || path
                    .components()
                    .any(|component| matches!(component, Component::ParentDir))
                || identity.starts_with("<build>/")
            {
                return Err(Phase13EvidenceError::new(
                    Phase13EvidenceErrorKind::Protocol,
                    format!("scoped witness material `{identity}` is not repository-confined"),
                ));
            }
            let bytes = fs::read(repository_root.join(path)).map_err(|error| {
                Phase13EvidenceError::new(
                    Phase13EvidenceErrorKind::Filesystem,
                    format!("failed to read scoped witness material `{identity}`: {error}"),
                )
            })?;
            update_length_prefixed(&mut digest, &bytes);
        }
    }
    Ok((format!("{:x}", digest.finalize()), materials.len()))
}

pub(super) fn derive_witness_closure(
    repository_root: &Path,
    producer_sha: &str,
) -> Result<ClosureIdentity, Phase13EvidenceError> {
    let manifest_bytes = fs::read(repository_root.join(MATERIALS_MANIFEST)).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Filesystem,
            format!("failed to read witness materials: {error}"),
        )
    })?;
    let manifest: MaterialsManifest = serde_json::from_slice(&manifest_bytes).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            format!("invalid witness materials: {error}"),
        )
    })?;
    if manifest.schema_version != 1
        || manifest.target != "phase9-lifecycle-contact-witness"
        || manifest.preset != ORACLE_PRESET
    {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Protocol,
            "witness materials manifest has the wrong schema, target, or preset",
        ));
    }
    let mut entries =
        derive_git_entries(repository_root, producer_sha, WITNESS_REPOSITORY_PREFIXES)?;
    for material in manifest.materials {
        let path = material.identity;
        if !matches!(material.kind.as_str(), "source" | "header" | "build_rule") {
            continue;
        }
        let candidate = repository_root.join(&path);
        if candidate.is_file() {
            entries.insert(path, file_sha256(&candidate)?);
        }
    }
    closure_from_entries("witness", entries)
}

pub(super) fn derive_git_closure(
    repository_root: &Path,
    producer_sha: &str,
    label: &str,
    prefixes: &[&str],
) -> Result<ClosureIdentity, Phase13EvidenceError> {
    closure_from_entries(
        label,
        derive_git_entries(repository_root, producer_sha, prefixes)?,
    )
}

fn derive_git_entries(
    repository_root: &Path,
    producer_sha: &str,
    prefixes: &[&str],
) -> Result<BTreeMap<String, String>, Phase13EvidenceError> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repository_root)
        .args(["ls-tree", "-r", "--name-only", producer_sha, "--"])
        .args(prefixes);
    let output = run_process(&mut command, "enumerate producer-affecting Git inputs")?;
    let names = String::from_utf8(output.stdout).map_err(|error| {
        Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Git,
            format!("Git returned non-UTF-8 paths: {error}"),
        )
    })?;
    let mut entries = BTreeMap::new();
    for path in names.lines() {
        validate_relative_path(path)?;
        let object = format!("{producer_sha}:{path}");
        let output = run_process(
            Command::new("git")
                .arg("-C")
                .arg(repository_root)
                .args(["show", &object]),
            "read producer-affecting Git input",
        )?;
        entries.insert(path.to_owned(), sha256(&output.stdout));
    }
    Ok(entries)
}

fn closure_from_entries(
    label: &str,
    entries: BTreeMap<String, String>,
) -> Result<ClosureIdentity, Phase13EvidenceError> {
    if entries.is_empty() {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Git,
            format!("{label} producer-affecting closure is empty"),
        ));
    }
    let entries = entries
        .into_iter()
        .map(|(path, sha256)| ClosureEntry { path, sha256 })
        .collect::<Vec<_>>();
    let digest = closure_digest(label, &entries);
    Ok(ClosureIdentity {
        schema_version: 1,
        label: label.to_owned(),
        digest,
        entries,
    })
}

pub(super) fn acquire_check(
    repository_root: &Path,
    options: &BTreeMap<String, String>,
) -> Result<(), Phase13EvidenceError> {
    require_options(
        options,
        &[
            "--staging-root",
            "--run-id",
            "--artifact-id",
            "--artifact-name",
            "--provider-digest",
            "--expected-producer-sha",
            "--expected-bundle-sha256",
        ],
    )?;
    for key in ["--run-id", "--artifact-id"] {
        if required(options, key)?
            .parse::<u64>()
            .ok()
            .is_none_or(|value| value == 0)
        {
            return Err(Phase13EvidenceError::usage(format!(
                "`{key}` must be a positive decimal identifier"
            )));
        }
    }
    let producer_sha = required(options, "--expected-producer-sha")?;
    let artifact_name = required(options, "--artifact-name")?;
    if !artifact_name.contains(producer_sha)
        || !artifact_name.contains(required(options, "--run-id")?)
        || !provider_digest(required(options, "--provider-digest")?)
    {
        return Err(Phase13EvidenceError::new(
            Phase13EvidenceErrorKind::Bundle,
            "artifact provider metadata does not bind the expected run and P",
        ));
    }
    let staging_root = absolute_path(repository_root, required(options, "--staging-root")?);
    check_bundle(
        &staging_root,
        producer_sha,
        required(options, "--expected-bundle-sha256")?,
        None,
        None,
    )
    .map_err(|error| {
        Phase13EvidenceError::new(Phase13EvidenceErrorKind::Bundle, error.to_string())
    })?;
    println!(
        "phase13 acquisition verified: run={} artifact={} name={artifact_name}",
        required(options, "--run-id")?,
        required(options, "--artifact-id")?
    );
    Ok(())
}
