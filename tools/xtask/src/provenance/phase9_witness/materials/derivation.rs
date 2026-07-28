use super::{
    BTreeMap, BTreeSet, CmakePresets, Codemodel, ConfigurePreset, FileApiIndex, MaterialKey,
    MaterialKind, MaterialsDerivation, MaterialsManifest, Path, ProvenanceError, TargetCodemodel,
    fs, normalize_absolute_material, normalize_metadata, normalize_path, read_json,
};

pub(super) fn derive_materials(
    repository_root: &Path,
    derivation: &MaterialsDerivation,
    manifest: &MaterialsManifest,
) -> Result<BTreeSet<MaterialKey>, ProvenanceError> {
    let index: FileApiIndex = read_json(&derivation.reply_index, "CMake File API index")?;
    let codemodel_object = index
        .objects
        .iter()
        .find(|object| object.kind == "codemodel" && object.version.major == 2)
        .ok_or_else(|| {
            ProvenanceError::new(
                "materials",
                "CMake File API index does not contain codemodel v2",
            )
        })?;
    let codemodel: Codemodel = read_json(
        &derivation.reply_directory.join(&codemodel_object.json_file),
        "CMake codemodel",
    )?;
    if codemodel.kind != "codemodel"
        || codemodel.version.major != 2
        || codemodel.configurations.len() != 1
    {
        return Err(ProvenanceError::new(
            "materials",
            "CMake codemodel must contain exactly one v2 configuration",
        ));
    }
    let configuration = &codemodel.configurations[0];
    let target_references = configuration
        .targets
        .iter()
        .map(|target| (target.id.as_str(), target))
        .collect::<BTreeMap<_, _>>();
    let root_target = configuration
        .targets
        .iter()
        .find(|target| target.name == manifest.target)
        .ok_or_else(|| {
            ProvenanceError::new(
                "materials",
                format!("CMake codemodel is missing target `{}`", manifest.target),
            )
        })?;

    let mut derived = BTreeSet::new();
    let mut pending = vec![root_target.id.clone()];
    let mut visited = BTreeSet::new();
    while let Some(target_id) = pending.pop() {
        if !visited.insert(target_id.clone()) {
            continue;
        }
        let reference = target_references.get(target_id.as_str()).ok_or_else(|| {
            ProvenanceError::new(
                "materials",
                format!("CMake codemodel references unknown target `{target_id}`"),
            )
        })?;
        let target: TargetCodemodel = read_json(
            &derivation.reply_directory.join(&reference.json_file),
            "CMake target codemodel",
        )?;
        collect_target_materials(
            repository_root,
            derivation,
            &codemodel,
            &target,
            &mut derived,
        )?;
        for dependency in &target.dependencies {
            let dependency_name = dependency
                .id
                .split_once("::")
                .map_or(dependency.id.as_str(), |(name, _)| name);
            derived.insert(MaterialKey {
                kind: MaterialKind::LinkInput,
                identity: format!("{}->{dependency_name}", target.name),
            });
            pending.push(dependency.id.clone());
        }
    }

    collect_preset_materials(&derivation.presets_path, &manifest.preset, &mut derived)?;
    for depfile in &derivation.depfiles {
        collect_depfile_materials(repository_root, derivation, depfile, &mut derived)?;
    }
    Ok(derived)
}

pub(super) fn collect_target_materials(
    repository_root: &Path,
    derivation: &MaterialsDerivation,
    codemodel: &Codemodel,
    target: &TargetCodemodel,
    derived: &mut BTreeSet<MaterialKey>,
) -> Result<(), ProvenanceError> {
    if target.name.is_empty() || target.target_type.is_empty() {
        return Err(ProvenanceError::new(
            "materials",
            "CMake target codemodel has empty target identity",
        ));
    }
    for source in &target.sources {
        if Path::new(&source.path)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("rule"))
        {
            continue;
        }
        let normalized = normalize_path(
            repository_root,
            derivation,
            &codemodel.paths.source,
            &target.paths.source,
            &source.path,
        )?;
        let kind = if source.is_generated {
            MaterialKind::GeneratedInput
        } else if source.compile_group_index.is_some() {
            MaterialKind::Source
        } else {
            MaterialKind::Header
        };
        derived.insert(MaterialKey {
            kind,
            identity: normalized,
        });
    }
    for file in &target.backtrace_graph.files {
        let normalized = normalize_path(
            repository_root,
            derivation,
            &codemodel.paths.source,
            &target.paths.source,
            file,
        )?;
        derived.insert(MaterialKey {
            kind: MaterialKind::BuildRule,
            identity: normalized,
        });
    }
    for group in &target.compile_groups {
        for fragment in &group.compile_command_fragments {
            derived.insert(MaterialKey {
                kind: MaterialKind::CompileFragment,
                identity: format!(
                    "{}:{}",
                    target.name,
                    normalize_metadata(repository_root, derivation, &fragment.value)
                ),
            });
        }
        for definition in &group.defines {
            derived.insert(MaterialKey {
                kind: MaterialKind::CompileDefinition,
                identity: format!("{}:{}", target.name, definition.define),
            });
        }
        for include in &group.includes {
            let system = if include.is_system {
                "system"
            } else {
                "ordinary"
            };
            derived.insert(MaterialKey {
                kind: MaterialKind::IncludePath,
                identity: format!(
                    "{}:{system}:{}",
                    target.name,
                    normalize_metadata(repository_root, derivation, &include.path)
                ),
            });
        }
    }
    if let Some(link) = &target.link {
        for fragment in &link.command_fragments {
            derived.insert(MaterialKey {
                kind: MaterialKind::LinkFragment,
                identity: format!(
                    "{}:{}:{}",
                    target.name,
                    fragment.role.as_deref().unwrap_or("unknown"),
                    normalize_metadata(repository_root, derivation, &fragment.value)
                ),
            });
        }
    }
    Ok(())
}

pub(super) fn collect_preset_materials(
    presets_path: &Path,
    preset_name: &str,
    derived: &mut BTreeSet<MaterialKey>,
) -> Result<(), ProvenanceError> {
    let presets: CmakePresets = read_json(presets_path, "CMake presets")?;
    if presets.version < 4 {
        return Err(ProvenanceError::new(
            "materials",
            "CMake presets version must support inherited configure presets",
        ));
    }
    let by_name = presets
        .configure_presets
        .iter()
        .map(|preset| (preset.name.as_str(), preset))
        .collect::<BTreeMap<_, _>>();
    let mut values = BTreeMap::new();
    collect_preset_values(preset_name, &by_name, &mut BTreeSet::new(), &mut values)?;
    derived.insert(MaterialKey {
        kind: MaterialKind::PresetValue,
        identity: format!("preset={preset_name}"),
    });
    for (name, value) in values {
        let canonical = match value {
            serde_json::Value::String(value) => value,
            other => other.to_string(),
        };
        derived.insert(MaterialKey {
            kind: MaterialKind::PresetValue,
            identity: format!("{name}={canonical}"),
        });
    }
    Ok(())
}

fn collect_preset_values<'a>(
    name: &'a str,
    presets: &BTreeMap<&'a str, &'a ConfigurePreset>,
    visiting: &mut BTreeSet<&'a str>,
    values: &mut BTreeMap<String, serde_json::Value>,
) -> Result<(), ProvenanceError> {
    if !visiting.insert(name) {
        return Err(ProvenanceError::new(
            "materials",
            format!("CMake preset inheritance cycle at `{name}`"),
        ));
    }
    let preset = presets.get(name).ok_or_else(|| {
        ProvenanceError::new("materials", format!("missing CMake preset `{name}`"))
    })?;
    if let Some(parent) = preset.inherits.as_deref() {
        collect_preset_values(parent, presets, visiting, values)?;
    }
    values.extend(preset.cache_variables.clone());
    visiting.remove(name);
    Ok(())
}

pub(super) fn collect_depfile_materials(
    repository_root: &Path,
    derivation: &MaterialsDerivation,
    depfile: &Path,
    derived: &mut BTreeSet<MaterialKey>,
) -> Result<(), ProvenanceError> {
    let contents = fs::read_to_string(depfile).map_err(|error| {
        ProvenanceError::new(
            "materials",
            format!(
                "failed to read compiler depfile {}: {error}",
                depfile.display()
            ),
        )
    })?;
    let logical = contents.replace("\\\n", " ");
    let (_, dependencies) = logical.split_once(':').ok_or_else(|| {
        ProvenanceError::new(
            "materials",
            format!(
                "compiler depfile {} has no target separator",
                depfile.display()
            ),
        )
    })?;
    for dependency in dependencies.split_ascii_whitespace() {
        let unescaped = dependency.replace("\\ ", " ");
        let path = Path::new(&unescaped);
        let normalized = normalize_absolute_material(repository_root, derivation, path)?;
        let Some(normalized) = normalized else {
            continue;
        };
        let kind = if normalized.starts_with("<build>/") {
            MaterialKind::GeneratedInput
        } else if matches!(
            Path::new(&normalized)
                .extension()
                .and_then(|extension| extension.to_str()),
            Some("c" | "cc" | "cpp" | "cxx")
        ) {
            MaterialKind::Source
        } else {
            MaterialKind::Header
        };
        derived.insert(MaterialKey {
            kind,
            identity: normalized,
        });
    }
    Ok(())
}
