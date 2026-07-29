//! Target-scoped material resolution for the Phase 9 lifecycle/contact witness.
#![allow(dead_code)]

#[path = "materials/binding.rs"]
mod binding;
#[path = "materials/derivation.rs"]
mod derivation;
#[path = "materials/normalization.rs"]
mod normalization;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::ProvenanceError;
use binding::{read_manifest, validate_declaration, validate_git_material};
use derivation::derive_materials;
use normalization::{
    compare_material_sets, digest_materials, normalize_absolute_material, normalize_metadata,
    normalize_path, read_json,
};

pub(crate) const MATERIALS_PATH: &str =
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

pub(super) fn validate_repository_binding(
    repository_root: &Path,
    repository_revision: &str,
) -> Result<(), ProvenanceError> {
    let manifest = read_manifest(&repository_root.join(MATERIALS_PATH))?;
    validate_declaration(&manifest)?;

    validate_git_material(repository_root, repository_revision, MATERIALS_PATH)?;
    for material in &manifest.materials {
        if !material.kind.is_file()
            || material.identity.starts_with("<build>/")
            || material.identity.starts_with("third_party/")
        {
            continue;
        }
        validate_git_material(repository_root, repository_revision, &material.identity)?;
    }
    Ok(())
}
