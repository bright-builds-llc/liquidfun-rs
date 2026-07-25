//! Target-scoped material resolution for the Phase 9 lifecycle/contact witness.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::ProvenanceError;

pub(super) const MATERIALS_PATH: &str =
    "tools/reference/phase9-lifecycle-contact-witness.materials.json";
const MATERIALS_SCHEMA_VERSION: u64 = 1;
const EXPECTED_TARGET: &str = "phase9-lifecycle-contact-witness";
const EXPECTED_PRESET: &str = "oracle-debug";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
enum MaterialKind {
    BuildRule,
    CompileDefinition,
    CompileFragment,
    GeneratedInput,
    Header,
    IncludePath,
    LinkFragment,
    LinkInput,
    PresetValue,
    Source,
}

impl MaterialKind {
    const fn as_str(self) -> &'static str {
        match self {
            Self::BuildRule => "build_rule",
            Self::CompileDefinition => "compile_definition",
            Self::CompileFragment => "compile_fragment",
            Self::GeneratedInput => "generated_input",
            Self::Header => "header",
            Self::IncludePath => "include_path",
            Self::LinkFragment => "link_fragment",
            Self::LinkInput => "link_input",
            Self::PresetValue => "preset_value",
            Self::Source => "source",
        }
    }

    const fn is_file(self) -> bool {
        matches!(
            self,
            Self::BuildRule | Self::GeneratedInput | Self::Header | Self::Source
        )
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct DeclaredMaterial {
    kind: MaterialKind,
    identity: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MaterialsManifest {
    schema_version: u64,
    target: String,
    preset: String,
    materials: Vec<DeclaredMaterial>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct MaterialKey {
    kind: MaterialKind,
    identity: String,
}

impl From<&DeclaredMaterial> for MaterialKey {
    fn from(material: &DeclaredMaterial) -> Self {
        Self {
            kind: material.kind,
            identity: material.identity.clone(),
        }
    }
}

#[derive(Debug)]
pub(super) struct MaterialsDerivation {
    pub(super) reply_index: PathBuf,
    pub(super) reply_directory: PathBuf,
    pub(super) build_directory: PathBuf,
    pub(super) presets_path: PathBuf,
    pub(super) depfiles: Vec<PathBuf>,
}

#[derive(Debug)]
pub(super) struct ResolvedMaterials {
    pub(super) digest: String,
    pub(super) count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileApiIndex {
    objects: Vec<FileApiObject>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileApiObject {
    kind: String,
    #[serde(rename = "jsonFile")]
    json_file: String,
    version: FileApiVersion,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileApiVersion {
    major: u64,
    minor: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Codemodel {
    configurations: Vec<CodemodelConfiguration>,
    kind: String,
    paths: CodemodelPaths,
    version: FileApiVersion,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CodemodelPaths {
    build: String,
    source: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CodemodelConfiguration {
    name: String,
    directories: Vec<serde_json::Value>,
    projects: Vec<serde_json::Value>,
    targets: Vec<CodemodelTargetReference>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CodemodelTargetReference {
    id: String,
    #[serde(rename = "jsonFile")]
    json_file: String,
    name: String,
    #[serde(rename = "directoryIndex")]
    directory_index: usize,
    #[serde(rename = "projectIndex")]
    project_index: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetCodemodel {
    name: String,
    #[serde(rename = "type")]
    target_type: String,
    paths: TargetPaths,
    #[serde(default)]
    dependencies: Vec<TargetDependency>,
    #[serde(default)]
    sources: Vec<TargetSource>,
    #[serde(default, rename = "compileGroups")]
    compile_groups: Vec<CompileGroup>,
    link: Option<TargetLink>,
    #[serde(rename = "backtraceGraph")]
    backtrace_graph: BacktraceGraph,
    #[serde(flatten)]
    remaining: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetPaths {
    build: String,
    source: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetDependency {
    id: String,
    #[serde(default)]
    backtrace: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetSource {
    path: String,
    #[serde(default, rename = "isGenerated")]
    is_generated: bool,
    #[serde(default, rename = "compileGroupIndex")]
    compile_group_index: Option<usize>,
    #[serde(default)]
    backtrace: Option<usize>,
    #[serde(default, rename = "sourceGroupIndex")]
    source_group_index: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompileGroup {
    #[serde(default, rename = "compileCommandFragments")]
    compile_command_fragments: Vec<Fragment>,
    #[serde(default)]
    defines: Vec<Definition>,
    #[serde(default)]
    includes: Vec<IncludePath>,
    language: String,
    #[serde(default, rename = "languageStandard")]
    language_standard: Option<serde_json::Value>,
    #[serde(default, rename = "sourceIndexes")]
    source_indexes: Vec<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Fragment {
    #[serde(rename = "fragment")]
    value: String,
    #[serde(default)]
    backtrace: Option<usize>,
    #[serde(default)]
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Definition {
    define: String,
    #[serde(default)]
    backtrace: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct IncludePath {
    path: String,
    #[serde(default, rename = "isSystem")]
    is_system: bool,
    #[serde(default)]
    backtrace: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TargetLink {
    #[serde(default, rename = "commandFragments")]
    command_fragments: Vec<Fragment>,
    language: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BacktraceGraph {
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    commands: Vec<String>,
    #[serde(default)]
    nodes: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CmakePresets {
    version: u64,
    #[serde(rename = "configurePresets")]
    configure_presets: Vec<ConfigurePreset>,
    #[serde(flatten)]
    remaining: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigurePreset {
    name: String,
    #[serde(default)]
    hidden: bool,
    #[serde(default)]
    inherits: Option<String>,
    #[serde(default, rename = "cacheVariables")]
    cache_variables: BTreeMap<String, serde_json::Value>,
    #[serde(flatten)]
    remaining: BTreeMap<String, serde_json::Value>,
}

pub(super) fn validate_target_scoped_materials(
    repository_root: &Path,
    manifest_path: &Path,
    derivation: &MaterialsDerivation,
) -> Result<ResolvedMaterials, ProvenanceError> {
    let manifest = read_manifest(manifest_path)?;
    let declared = validate_declaration(&manifest)?;
    let derived = derive_materials(repository_root, derivation, &manifest)?;
    compare_material_sets(&declared, &derived)?;
    digest_materials(repository_root, derivation, &derived)
}

pub(super) fn resolve_declared_materials(
    repository_root: &Path,
) -> Result<ResolvedMaterials, ProvenanceError> {
    let manifest = read_manifest(&repository_root.join(MATERIALS_PATH))?;
    let declared = validate_declaration(&manifest)?;
    digest_materials(
        repository_root,
        &MaterialsDerivation {
            reply_index: PathBuf::new(),
            reply_directory: PathBuf::new(),
            build_directory: PathBuf::new(),
            presets_path: PathBuf::new(),
            depfiles: Vec::new(),
        },
        &declared,
    )
}

fn read_manifest(path: &Path) -> Result<MaterialsManifest, ProvenanceError> {
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

fn validate_declaration(
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

fn validate_identity(key: &MaterialKey) -> Result<(), ProvenanceError> {
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

fn derive_materials(
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

fn collect_target_materials(
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

fn collect_preset_materials(
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

fn collect_depfile_materials(
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

fn normalize_path(
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

fn normalize_absolute_material(
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

fn lexical_absolute(path: &Path) -> Result<PathBuf, ProvenanceError> {
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

fn normalize_metadata(
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

fn path_text(path: &Path) -> Result<String, ProvenanceError> {
    let value = path.to_str().ok_or_else(|| {
        ProvenanceError::new(
            "materials",
            format!("material path {} is not UTF-8", path.display()),
        )
    })?;
    Ok(value.replace('\\', "/"))
}

fn compare_material_sets(
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

fn digest_materials(
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

fn material_path(
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

fn update_length_prefixed(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path, label: &str) -> Result<T, ProvenanceError> {
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
