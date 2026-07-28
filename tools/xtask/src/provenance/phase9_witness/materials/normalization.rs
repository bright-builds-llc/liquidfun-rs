use super::{
    BTreeSet, Component, Deserialize, Digest, MaterialKey, MaterialsDerivation, Path, PathBuf,
    ProvenanceError, ResolvedMaterials, Sha256, fs,
};

pub(super) fn normalize_path(
    repository_root: &Path,
    derivation: &MaterialsDerivation,
    codemodel_source: &str,
    target_source: &str,
    value: &str,
) -> Result<String, ProvenanceError> {
    let value_path = Path::new(value);
    let absolute = if value_path.is_absolute() {
        value_path.to_path_buf()
    } else {
        let target_source = Path::new(target_source);
        let base = if target_source.is_absolute() {
            target_source.to_path_buf()
        } else {
            Path::new(codemodel_source).join(target_source)
        };
        base.join(value_path)
    };
    normalize_absolute_material(repository_root, derivation, &absolute)?.ok_or_else(|| {
        ProvenanceError::new(
            "materials",
            format!("target material `{value}` resolves outside repository and build roots"),
        )
    })
}

pub(super) fn normalize_absolute_material(
    repository_root: &Path,
    derivation: &MaterialsDerivation,
    value: &Path,
) -> Result<Option<String>, ProvenanceError> {
    let absolute = lexical_absolute(value)?;
    let root = lexical_absolute(repository_root)?;
    let build = lexical_absolute(&derivation.build_directory)?;
    if !derivation.build_directory.as_os_str().is_empty()
        && let Ok(relative) = absolute.strip_prefix(&build)
    {
        return Ok(Some(format!("<build>/{}", path_text(relative)?)));
    }
    if let Ok(relative) = absolute.strip_prefix(&root) {
        return Ok(Some(path_text(relative)?));
    }
    Ok(None)
}

pub(super) fn lexical_absolute(path: &Path) -> Result<PathBuf, ProvenanceError> {
    if !path.is_absolute() {
        return Err(ProvenanceError::new(
            "materials",
            format!("material path {} is not absolute", path.display()),
        ));
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(ProvenanceError::new(
                        "materials",
                        format!("material path {} escapes its root", path.display()),
                    ));
                }
            }
            Component::Normal(value) => normalized.push(value),
        }
    }
    Ok(normalized)
}

pub(super) fn normalize_metadata(
    repository_root: &Path,
    derivation: &MaterialsDerivation,
    value: &str,
) -> String {
    if Path::new(value).is_absolute()
        && let Ok(Some(normalized)) =
            normalize_absolute_material(repository_root, derivation, Path::new(value))
    {
        return if let Some(relative) = normalized.strip_prefix("<build>/") {
            format!("<build>/{relative}")
        } else {
            format!("<repo>/{normalized}")
        };
    }
    let root = repository_root.to_string_lossy();
    let build = derivation.build_directory.to_string_lossy();
    value
        .replace(build.as_ref(), "<build>")
        .replace(root.as_ref(), "<repo>")
        .replace('\\', "/")
}

pub(super) fn path_text(path: &Path) -> Result<String, ProvenanceError> {
    let value = path.to_str().ok_or_else(|| {
        ProvenanceError::new(
            "materials",
            format!("material path {} is not UTF-8", path.display()),
        )
    })?;
    Ok(value.replace('\\', "/"))
}

pub(super) fn compare_material_sets(
    declared: &BTreeSet<MaterialKey>,
    derived: &BTreeSet<MaterialKey>,
) -> Result<(), ProvenanceError> {
    if let Some(unexpected) = derived.difference(declared).next() {
        return Err(ProvenanceError::new(
            "materials",
            format!(
                "unexpected {} material `{}`",
                unexpected.kind.as_str(),
                unexpected.identity
            ),
        ));
    }
    if let Some(missing) = declared.difference(derived).next() {
        return Err(ProvenanceError::new(
            "materials",
            format!(
                "declared {} material `{}` was not derived",
                missing.kind.as_str(),
                missing.identity
            ),
        ));
    }
    Ok(())
}

pub(super) fn digest_materials(
    repository_root: &Path,
    derivation: &MaterialsDerivation,
    materials: &BTreeSet<MaterialKey>,
) -> Result<ResolvedMaterials, ProvenanceError> {
    let mut digest = Sha256::new();
    for material in materials {
        update_length_prefixed(&mut digest, material.kind.as_str().as_bytes());
        update_length_prefixed(&mut digest, material.identity.as_bytes());
        if material.kind.is_file() {
            let path = material_path(repository_root, derivation, &material.identity)?;
            let bytes = fs::read(&path).map_err(|error| {
                ProvenanceError::new(
                    "materials",
                    format!(
                        "failed to read {} material `{}` at {}: {error}",
                        material.kind.as_str(),
                        material.identity,
                        path.display()
                    ),
                )
            })?;
            update_length_prefixed(&mut digest, &bytes);
        }
    }
    Ok(ResolvedMaterials {
        digest: format!("{:x}", digest.finalize()),
        count: materials.len(),
    })
}

pub(super) fn material_path(
    repository_root: &Path,
    derivation: &MaterialsDerivation,
    identity: &str,
) -> Result<PathBuf, ProvenanceError> {
    if let Some(relative) = identity.strip_prefix("<build>/") {
        if derivation.build_directory.as_os_str().is_empty() {
            return Err(ProvenanceError::new(
                "materials",
                format!("generated material `{identity}` requires a configured build"),
            ));
        }
        return Ok(derivation.build_directory.join(relative));
    }
    let path = Path::new(identity);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(ProvenanceError::new(
            "materials",
            format!("material path `{identity}` is not repository-confined"),
        ));
    }
    Ok(repository_root.join(path))
}

pub(super) fn update_length_prefixed(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

pub(super) fn read_json<T: for<'de> Deserialize<'de>>(
    path: &Path,
    label: &str,
) -> Result<T, ProvenanceError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        ProvenanceError::new(
            "materials",
            format!("failed to read {label} {}: {error}", path.display()),
        )
    })?;
    serde_json::from_str(&contents).map_err(|error| {
        ProvenanceError::new(
            "materials",
            format!("invalid {label} {}: {error}", path.display()),
        )
    })
}
