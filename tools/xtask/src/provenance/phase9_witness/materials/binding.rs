use super::{
    BTreeSet, Command, Digest, EXPECTED_PRESET, EXPECTED_TARGET, MATERIALS_SCHEMA_VERSION,
    MaterialKey, MaterialsManifest, OsString, Path, ProvenanceError, Sha256, env, fs,
};

pub(super) fn validate_git_material(
    repository_root: &Path,
    repository_revision: &str,
    relative_path: &str,
) -> Result<(), ProvenanceError> {
    let git = env::var_os("LIQUIDFUN_XTASK_GIT").unwrap_or_else(|| OsString::from("git"));
    let object = format!("{repository_revision}:{relative_path}");
    let output = Command::new(git)
        .arg("-C")
        .arg(repository_root)
        .arg("show")
        .arg(&object)
        .output()
        .map_err(|error| {
            ProvenanceError::new(
                "repository",
                format!("failed to read repository material `{object}`: {error}"),
            )
        })?;
    if !output.status.success() {
        return Err(ProvenanceError::new(
            "repository",
            format!(
                "repository revision does not contain scoped material `{relative_path}`: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        ));
    }
    let current = fs::read(repository_root.join(relative_path)).map_err(|error| {
        ProvenanceError::new(
            "materials",
            format!("failed to read scoped material `{relative_path}`: {error}"),
        )
    })?;
    if Sha256::digest(&output.stdout) != Sha256::digest(&current) {
        return Err(ProvenanceError::new(
            "repository",
            format!(
                "scoped material `{relative_path}` differs from repository revision `{repository_revision}`"
            ),
        ));
    }
    Ok(())
}

pub(super) fn read_manifest(path: &Path) -> Result<MaterialsManifest, ProvenanceError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        ProvenanceError::new(
            "materials",
            format!(
                "failed to read materials manifest {}: {error}",
                path.display()
            ),
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        ProvenanceError::new(
            "materials",
            format!("invalid materials manifest {}: {error}", path.display()),
        )
    })
}

pub(super) fn validate_declaration(
    manifest: &MaterialsManifest,
) -> Result<BTreeSet<MaterialKey>, ProvenanceError> {
    if manifest.schema_version != MATERIALS_SCHEMA_VERSION
        || manifest.target != EXPECTED_TARGET
        || manifest.preset != EXPECTED_PRESET
    {
        return Err(ProvenanceError::new(
            "materials",
            "Phase 9 materials manifest has an unexpected schema, target, or preset",
        ));
    }
    if manifest.materials.is_empty() {
        return Err(ProvenanceError::new(
            "materials",
            "Phase 9 materials manifest must not be empty",
        ));
    }

    let keys = manifest
        .materials
        .iter()
        .map(MaterialKey::from)
        .collect::<Vec<_>>();
    for key in &keys {
        validate_identity(key)?;
    }
    let mut canonical = keys.clone();
    canonical.sort();
    if keys != canonical {
        return Err(ProvenanceError::new(
            "materials",
            "Phase 9 materials must use canonical kind and identity ordering",
        ));
    }
    let unique = keys.into_iter().collect::<BTreeSet<_>>();
    if unique.len() != manifest.materials.len() {
        return Err(ProvenanceError::new(
            "materials",
            "Phase 9 materials contain a duplicate kind and identity",
        ));
    }
    Ok(unique)
}

pub(super) fn validate_identity(key: &MaterialKey) -> Result<(), ProvenanceError> {
    let identity = key.identity.as_str();
    if identity.is_empty()
        || identity.contains('\\')
        || identity.contains('*')
        || identity.contains('?')
        || identity.split('/').any(|component| component == "..")
    {
        return Err(ProvenanceError::new(
            "materials",
            format!("noncanonical {} material `{identity}`", key.kind.as_str()),
        ));
    }
    if key.kind.is_file() && Path::new(identity).is_absolute() {
        return Err(ProvenanceError::new(
            "materials",
            format!("material path `{identity}` must be normalized and relative"),
        ));
    }
    Ok(())
}
