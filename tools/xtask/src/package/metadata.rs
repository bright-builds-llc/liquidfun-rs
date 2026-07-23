use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use super::PackageError;

const FORBIDDEN_DEPENDENCIES: &[&str] = &[
    "autocxx",
    "bevy",
    "bindgen",
    "cc",
    "cmake",
    "criterion",
    "cxx",
    "egui",
    "ggez",
    "glfw",
    "liquidfun-benchmarks",
    "liquidfun-differential",
    "liquidfun-test-protocol",
    "macroquad",
    "miniquad",
    "pixels",
    "raylib",
    "sdl2",
    "wgpu",
    "winit",
];
const FORBIDDEN_FEATURE_TERMS: &[&str] = &[
    "benchmark",
    "cpp",
    "cxx",
    "oracle",
    "protocol",
    "reference",
    "renderer",
    "testbed",
    "visual",
    "window",
];
const FORBIDDEN_DEPENDENCY_TERMS: &[&str] = &[
    "benchmark",
    "cpp",
    "cxx",
    "game-engine",
    "oracle",
    "protocol",
    "reference",
    "renderer",
    "testbed",
    "visual",
    "window",
];

#[derive(Deserialize)]
struct CargoMetadata {
    packages: Vec<MetadataPackage>,
    workspace_members: Vec<String>,
    workspace_default_members: Vec<String>,
}

#[derive(Deserialize)]
struct MetadataPackage {
    id: String,
    name: String,
    publish: Option<Vec<String>>,
    manifest_path: String,
    rust_version: Option<String>,
    dependencies: Vec<MetadataDependency>,
    features: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct MetadataDependency {
    name: String,
    path: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ConsumerManifest {
    pub(super) name: String,
    pub(super) version: String,
    pub(super) rust_version: String,
    pub(super) features: Vec<String>,
    pub(super) normal_dependencies: Vec<String>,
    pub(super) license: String,
}

pub(super) fn verify_workspace(repository_root: &Path, cargo: &OsStr) -> Result<(), PackageError> {
    let bytes = if let Some(path) = env::var_os("LIQUIDFUN_XTASK_TEST_METADATA") {
        fs::read(&path).map_err(|error| {
            PackageError::new(
                "metadata",
                format!("failed to read {}: {error}", Path::new(&path).display()),
            )
        })?
    } else {
        let output = Command::new(cargo)
            .args(["metadata", "--format-version", "1", "--no-deps"])
            .current_dir(repository_root)
            .output()
            .map_err(|error| {
                PackageError::new("metadata", format!("failed to run cargo metadata: {error}"))
            })?;
        if !output.status.success() {
            return Err(PackageError::new(
                "metadata",
                format!(
                    "cargo metadata failed with {}: {}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            ));
        }
        output.stdout
    };
    let metadata: CargoMetadata = serde_json::from_slice(&bytes)
        .map_err(|error| PackageError::new("metadata", format!("invalid metadata: {error}")))?;
    verify_metadata(repository_root, &metadata)
}

fn verify_metadata(repository_root: &Path, metadata: &CargoMetadata) -> Result<(), PackageError> {
    let package = metadata
        .packages
        .iter()
        .find(|package| package.name == "liquidfun")
        .ok_or_else(|| PackageError::new("metadata", "liquidfun package is absent"))?;
    let expected_manifest = repository_root.join("crates/liquidfun/Cargo.toml");
    if Path::new(&package.manifest_path) != expected_manifest {
        return Err(PackageError::new(
            "metadata",
            "liquidfun manifest is outside the reviewed workspace path",
        ));
    }
    if package.publish.as_ref().is_some_and(Vec::is_empty) {
        return Err(PackageError::new(
            "publish-policy",
            "liquidfun must remain publishable",
        ));
    }
    if package.rust_version.as_deref() != Some("1.92") {
        return Err(PackageError::new(
            "rust-version",
            "liquidfun package rust-version must be exactly 1.92",
        ));
    }
    if metadata.workspace_default_members != [package.id.clone()]
        || !metadata.workspace_members.contains(&package.id)
    {
        return Err(PackageError::new(
            "default-members",
            "liquidfun must be the sole workspace default member",
        ));
    }
    for workspace_package in metadata.packages.iter().filter(|candidate| {
        candidate.id != package.id && metadata.workspace_members.contains(&candidate.id)
    }) {
        if !workspace_package
            .publish
            .as_ref()
            .is_some_and(Vec::is_empty)
        {
            return Err(PackageError::new(
                "publish-policy",
                format!(
                    "workspace package `{}` must remain unpublished",
                    workspace_package.name
                ),
            ));
        }
    }
    if package
        .dependencies
        .iter()
        .any(|dependency| dependency.path.is_some())
    {
        return Err(PackageError::new(
            "dependency-graph",
            "consumer package metadata contains a repository path dependency",
        ));
    }
    verify_dependencies(
        package
            .dependencies
            .iter()
            .map(|dependency| dependency.name.as_str()),
    )?;
    verify_features(&package.features)
}

pub(super) fn verify_packaged_manifest(path: &Path) -> Result<ConsumerManifest, PackageError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        PackageError::new(
            "manifest",
            format!("failed to read {}: {error}", path.display()),
        )
    })?;
    let manifest: toml::Value = toml::from_str(&contents).map_err(|error| {
        PackageError::new("manifest", format!("invalid {}: {error}", path.display()))
    })?;
    let dependencies = ["dependencies", "dev-dependencies", "build-dependencies"]
        .into_iter()
        .filter_map(|section| manifest.get(section).and_then(toml::Value::as_table))
        .flat_map(|table| table.keys().map(String::as_str));
    verify_dependencies(dependencies)?;
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if manifest
            .get(section)
            .and_then(toml::Value::as_table)
            .is_some_and(|table| {
                table.values().any(|dependency| {
                    dependency
                        .as_table()
                        .is_some_and(|fields| fields.contains_key("path"))
                })
            })
        {
            return Err(PackageError::new(
                "dependency-graph",
                "packaged manifest contains a path dependency",
            ));
        }
    }
    let features = manifest
        .get("features")
        .and_then(toml::Value::as_table)
        .map(|table| {
            table
                .iter()
                .map(|(name, values)| {
                    let values = values
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(toml::Value::as_str)
                        .map(str::to_owned)
                        .collect();
                    (name.clone(), values)
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    verify_features(&features)?;
    let package = manifest
        .get("package")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| PackageError::new("manifest", "packaged package table is missing"))?;
    let required_string = |field: &str| {
        package
            .get(field)
            .and_then(toml::Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| {
                PackageError::new(
                    "manifest",
                    format!("packaged package.{field} must be a string"),
                )
            })
    };
    let rust_version = required_string("rust-version")?;
    if rust_version != "1.92" {
        return Err(PackageError::new(
            "rust-version",
            "packaged package rust-version must be exactly 1.92",
        ));
    }
    let license = required_string("license")?;
    if license != "MIT" {
        return Err(PackageError::new(
            "license",
            "packaged package license must be exactly MIT",
        ));
    }
    let normal_dependencies = manifest
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .map(|dependencies| dependencies.keys().cloned().collect())
        .unwrap_or_default();
    Ok(ConsumerManifest {
        name: required_string("name")?,
        version: required_string("version")?,
        rust_version,
        features: features.keys().cloned().collect(),
        normal_dependencies,
        license,
    })
}

fn verify_dependencies<'a>(
    dependencies: impl Iterator<Item = &'a str>,
) -> Result<(), PackageError> {
    for dependency in dependencies {
        let normalized = dependency.replace('_', "-");
        if FORBIDDEN_DEPENDENCIES.contains(&normalized.as_str())
            || FORBIDDEN_DEPENDENCY_TERMS
                .iter()
                .any(|term| normalized.contains(term))
        {
            return Err(PackageError::new(
                "dependency-graph",
                format!("consumer package has forbidden dependency `{dependency}`"),
            ));
        }
    }
    Ok(())
}

fn verify_features(features: &BTreeMap<String, Vec<String>>) -> Result<(), PackageError> {
    let actual = features.keys().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = ["default", "differential-internals"]
        .into_iter()
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(PackageError::new(
            "feature-graph",
            "consumer features must be exactly `default` and `differential-internals`",
        ));
    }
    if features
        .get("default")
        .is_some_and(|members| !members.is_empty())
    {
        return Err(PackageError::new(
            "feature-graph",
            "consumer default features must remain empty",
        ));
    }
    for (feature, members) in features {
        let normalized = feature.replace('_', "-").to_ascii_lowercase();
        if FORBIDDEN_FEATURE_TERMS
            .iter()
            .any(|term| normalized.contains(term))
            || members.iter().any(|member| {
                let normalized = member.replace('_', "-").to_ascii_lowercase();
                FORBIDDEN_DEPENDENCIES
                    .iter()
                    .any(|dependency| normalized.contains(dependency))
            })
        {
            return Err(PackageError::new(
                "feature-graph",
                format!("consumer package has forbidden feature `{feature}`"),
            ));
        }
    }
    Ok(())
}
