//! Read-only authority boundary for checked catalog regressions.

use std::{
    collections::HashSet,
    fs,
    path::{Component, Path, PathBuf},
};

use liquidfun_test_protocol::{
    CATALOG_MAXIMUM_CANONICAL_BYTES, CatalogSchemaVersion, CatalogSlug, GeneratorVersion,
    RequestId, ResolveRequest, RunSettings, ScenarioVersion, Sha256Hex, decode_resolved_scenario,
    resolve_catalog, scenarios::scenario_definitions,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::execute_resolved_catalog_native;

use super::diagnosis::{
    CheckpointSemanticDocuments, ReplayDiagnosis, ReplayDriftClass, ReplayProjectionVersion,
    ReplaySchemaIdentity, ReplaySemanticDocument, checkpoint_semantic_documents,
    diagnose_replay_drift,
};

mod evidence;

use evidence::validate_rigid_stack_replay_evidence;

const MANIFEST_PATH: &str = "scenarios/regressions/catalog-manifest.json";
const RIGID_STACK_REPLAY_EVIDENCE_PATH: &str =
    "reference/artifacts/catalog/rigid-stack-v1.replay-evidence.json";
const MANIFEST_SCHEMA_VERSION: u32 = 1;
const MAXIMUM_MANIFEST_BYTES: usize = 128 * 1024;
const MAXIMUM_REGRESSIONS: usize = 16;
const PINNED_UPSTREAM_REVISION: &str = "7f20402173fd143a3988c921bc384459c6a858f2";

/// Stable rejection categories for the checked catalog-regression boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogRegressionErrorKind {
    /// Manifest bytes are missing, oversized, malformed, or incomplete.
    InvalidManifest,
    /// A schema, scenario, or generator version is not supported.
    UnsupportedVersion,
    /// A fixture path is absolute, escaping, linked, or outside the reviewed directory.
    UnsafePath,
    /// A path or content identity occurs more than once.
    DuplicateIdentity,
    /// Exact fixture bytes, canonical form, or hash do not validate.
    FixtureMismatch,
    /// Persisted metadata or bytes disagree with the current closed typed catalog.
    CatalogMismatch,
    /// Native deterministic replay disagrees with the reviewed D0 identity.
    NativeMismatch,
}

/// Bounded catalog-regression failure without path or record disclosure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("catalog regression failure: {kind:?}")]
pub struct CatalogRegressionError {
    kind: CatalogRegressionErrorKind,
}

impl CatalogRegressionError {
    const fn new(kind: CatalogRegressionErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(self) -> CatalogRegressionErrorKind {
        self.kind
    }
}

/// One successfully validated and natively replayed checked fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogRegressionReplayEntry {
    fixture_id: Box<str>,
    slug: Box<str>,
    resolved_sha256: Sha256Hex,
    native_d0_sha256: Sha256Hex,
    maybe_diagnosis: Option<ReplayDiagnosis>,
}

impl CatalogRegressionReplayEntry {
    /// Returns the stable manifest fixture identity.
    #[must_use]
    pub fn fixture_id(&self) -> &str {
        &self.fixture_id
    }

    /// Returns the stable catalog slug.
    #[must_use]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// Returns the exact canonical resolved-byte identity.
    #[must_use]
    pub const fn resolved_sha256(&self) -> &Sha256Hex {
        &self.resolved_sha256
    }

    /// Returns the repeated native semantic-checkpoint identity.
    #[must_use]
    pub const fn native_d0_sha256(&self) -> &Sha256Hex {
        &self.native_d0_sha256
    }

    /// Returns structured drift evidence when a legacy projection preserves review.
    #[must_use]
    pub const fn maybe_diagnosis(&self) -> Option<&ReplayDiagnosis> {
        self.maybe_diagnosis.as_ref()
    }
}

/// Complete checked catalog-regression replay result in manifest order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogRegressionReplay {
    entries: Box<[CatalogRegressionReplayEntry]>,
}

impl CatalogRegressionReplay {
    /// Returns validated replay records in reviewed manifest order.
    #[must_use]
    pub fn entries(&self) -> &[CatalogRegressionReplayEntry] {
        &self.entries
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RegressionManifest {
    schema_version: u32,
    upstream_revision: String,
    authority: String,
    entries: Vec<ManifestEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestEntry {
    fixture_id: String,
    path: String,
    resolved_sha256: Sha256Hex,
    run_identity: ManifestRunIdentity,
    action_ids: Vec<String>,
    checkpoint_ids: Vec<String>,
    provenance: ManifestProvenance,
    expected_native_d0_sha256: Sha256Hex,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestRunIdentity {
    catalog_schema_version: u32,
    slug: String,
    scenario_version: u32,
    generator_id: String,
    generator_version: u32,
    maybe_seed: Option<u64>,
    settings: RunSettings,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestProvenance {
    source: String,
    authority: String,
    upstream_revision: String,
    review_status: String,
}

/// Loads, validates, catalog-binds, and natively replays every checked catalog regression.
///
/// Validation is complete before [`liquidfun::World`] construction. The function is read-only and
/// never regenerates a fixture from seed or mutable catalog state as a replay substitute.
///
/// # Errors
///
/// Returns [`CatalogRegressionError`] for malformed manifests, unsafe filesystem entries,
/// canonical/hash drift, catalog mismatch, or D0 replay mismatch.
pub fn replay_catalog_regressions(
    repository_root: &Path,
) -> Result<CatalogRegressionReplay, CatalogRegressionError> {
    let canonical_root = fs::canonicalize(repository_root).map_err(|_error| {
        CatalogRegressionError::new(CatalogRegressionErrorKind::InvalidManifest)
    })?;
    let manifest_bytes = read_regular_confined(
        &canonical_root,
        Path::new(MANIFEST_PATH),
        MAXIMUM_MANIFEST_BYTES,
        CatalogRegressionErrorKind::InvalidManifest,
    )?;
    let manifest: RegressionManifest =
        serde_json::from_slice(&manifest_bytes).map_err(|_error| {
            CatalogRegressionError::new(CatalogRegressionErrorKind::InvalidManifest)
        })?;
    validate_manifest_header(&manifest)?;
    validate_rigid_stack_replay_evidence(&canonical_root, &manifest)?;

    let definitions = scenario_definitions().map_err(|_error| {
        CatalogRegressionError::new(CatalogRegressionErrorKind::CatalogMismatch)
    })?;
    let validated = validate_regression_files(&canonical_root, &manifest, &definitions)?;

    // Native construction happens only after the complete manifest/filesystem/catalog gate passes.
    let entries = validated
        .into_iter()
        .map(|(entry, resolved)| {
            let first = native_replay_capture(&resolved)?;
            let second = native_replay_capture(&resolved)?;
            if first.semantic_documents.legacy_physics_sha256
                != second.semantic_documents.legacy_physics_sha256
                || first.semantic_documents.physics_projection
                    != second.semantic_documents.physics_projection
            {
                return Err(CatalogRegressionError::new(
                    CatalogRegressionErrorKind::NativeMismatch,
                ));
            }
            let (native_d0_sha256, maybe_diagnosis) =
                classify_native_identity(entry, &first, &resolved)?;
            let (repeated_native_d0_sha256, repeated_diagnosis) =
                classify_native_identity(entry, &second, &resolved)?;
            if native_d0_sha256 != repeated_native_d0_sha256
                || maybe_diagnosis != repeated_diagnosis
            {
                return Err(CatalogRegressionError::new(
                    CatalogRegressionErrorKind::NativeMismatch,
                ));
            }
            Ok(CatalogRegressionReplayEntry {
                fixture_id: entry.fixture_id.clone().into_boxed_str(),
                slug: entry.run_identity.slug.clone().into_boxed_str(),
                resolved_sha256: entry.resolved_sha256.clone(),
                native_d0_sha256,
                maybe_diagnosis,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CatalogRegressionReplay {
        entries: entries.into_boxed_slice(),
    })
}

fn validate_regression_files<'a>(
    canonical_root: &Path,
    manifest: &'a RegressionManifest,
    definitions: &[liquidfun_test_protocol::CatalogDefinition],
) -> Result<
    Vec<(&'a ManifestEntry, liquidfun_test_protocol::ResolvedScenario)>,
    CatalogRegressionError,
> {
    let mut paths = HashSet::with_capacity(manifest.entries.len());
    let mut hashes = HashSet::with_capacity(manifest.entries.len());
    let mut fixture_ids = HashSet::with_capacity(manifest.entries.len());
    let mut validated = Vec::with_capacity(manifest.entries.len());

    // This loop performs only bounded reads, strict decoding, and pure catalog resolution.
    for entry in &manifest.entries {
        validate_entry_identity(entry)?;
        if !fixture_ids.insert(entry.fixture_id.as_str())
            || !paths.insert(entry.path.as_str())
            || !hashes.insert(entry.resolved_sha256.as_str())
        {
            return Err(CatalogRegressionError::new(
                CatalogRegressionErrorKind::DuplicateIdentity,
            ));
        }
        let relative = validate_fixture_path(&entry.path)?;
        let bytes = read_regular_confined(
            canonical_root,
            &relative,
            CATALOG_MAXIMUM_CANONICAL_BYTES,
            CatalogRegressionErrorKind::FixtureMismatch,
        )?;
        let resolved =
            decode_resolved_scenario(&bytes, &entry.resolved_sha256).map_err(|_error| {
                CatalogRegressionError::new(CatalogRegressionErrorKind::FixtureMismatch)
            })?;
        validate_manifest_mapping(entry, &resolved)?;
        let definition = definitions
            .iter()
            .find(|definition| definition.slug().as_str() == entry.run_identity.slug)
            .ok_or_else(|| {
                CatalogRegressionError::new(CatalogRegressionErrorKind::CatalogMismatch)
            })?;
        let metadata = definition.metadata().ok_or_else(|| {
            CatalogRegressionError::new(CatalogRegressionErrorKind::CatalogMismatch)
        })?;
        if metadata.default_settings() != entry.run_identity.settings {
            return Err(CatalogRegressionError::new(
                CatalogRegressionErrorKind::CatalogMismatch,
            ));
        }
        let candidate = resolve_catalog(
            definitions,
            &ResolveRequest::new(
                CatalogSlug::new(entry.run_identity.slug.clone()).map_err(|_error| {
                    CatalogRegressionError::new(CatalogRegressionErrorKind::CatalogMismatch)
                })?,
                entry.run_identity.maybe_seed,
                entry.run_identity.settings,
            ),
        )
        .map_err(|_error| {
            CatalogRegressionError::new(CatalogRegressionErrorKind::CatalogMismatch)
        })?;
        if candidate.canonical_bytes() != bytes
            || candidate.identity().content_sha256() != &entry.resolved_sha256
            || candidate != resolved
        {
            return Err(CatalogRegressionError::new(
                CatalogRegressionErrorKind::CatalogMismatch,
            ));
        }
        validated.push((entry, resolved));
    }
    if validated.len() != 3 {
        return Err(CatalogRegressionError::new(
            CatalogRegressionErrorKind::InvalidManifest,
        ));
    }

    Ok(validated)
}

fn validate_manifest_header(manifest: &RegressionManifest) -> Result<(), CatalogRegressionError> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(CatalogRegressionError::new(
            CatalogRegressionErrorKind::UnsupportedVersion,
        ));
    }
    if manifest.upstream_revision != PINNED_UPSTREAM_REVISION
        || manifest.authority != "native-d0-reviewed"
        || manifest.entries.is_empty()
        || manifest.entries.len() > MAXIMUM_REGRESSIONS
    {
        return Err(CatalogRegressionError::new(
            CatalogRegressionErrorKind::InvalidManifest,
        ));
    }
    Ok(())
}

fn validate_entry_identity(entry: &ManifestEntry) -> Result<(), CatalogRegressionError> {
    let valid_id = !entry.fixture_id.is_empty()
        && entry.fixture_id.len() <= 80
        && entry
            .fixture_id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if !valid_id
        || entry.provenance.source != "typed-catalog-resolved-bytes"
        || entry.provenance.authority != "native-d0"
        || entry.provenance.upstream_revision != PINNED_UPSTREAM_REVISION
        || entry.provenance.review_status != "reviewed"
        || entry.action_ids.is_empty()
        || entry.checkpoint_ids.is_empty()
        || entry.action_ids.len() > 128
        || entry.checkpoint_ids.len() > 128
    {
        return Err(CatalogRegressionError::new(
            CatalogRegressionErrorKind::InvalidManifest,
        ));
    }
    Ok(())
}

fn validate_fixture_path(raw: &str) -> Result<PathBuf, CatalogRegressionError> {
    let path = Path::new(raw);
    let components = path.components().collect::<Vec<_>>();
    let is_reviewed = matches!(
        components.as_slice(),
        [Component::Normal(first), Component::Normal(second), Component::Normal(file)]
            if *first == "scenarios"
                && *second == "catalog"
                && Path::new(file).extension().is_some_and(|extension| extension == "json")
    );
    if path.is_absolute() || !is_reviewed {
        return Err(CatalogRegressionError::new(
            CatalogRegressionErrorKind::UnsafePath,
        ));
    }
    Ok(path.to_path_buf())
}

fn read_regular_confined(
    canonical_root: &Path,
    relative: &Path,
    maximum_bytes: usize,
    invalid_kind: CatalogRegressionErrorKind,
) -> Result<Vec<u8>, CatalogRegressionError> {
    reject_linked_components(canonical_root, relative)?;
    let path = canonical_root.join(relative);
    let metadata =
        fs::symlink_metadata(&path).map_err(|_error| CatalogRegressionError::new(invalid_kind))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(CatalogRegressionError::new(
            CatalogRegressionErrorKind::UnsafePath,
        ));
    }
    let length = usize::try_from(metadata.len())
        .map_err(|_error| CatalogRegressionError::new(invalid_kind))?;
    if length == 0 || length > maximum_bytes {
        return Err(CatalogRegressionError::new(invalid_kind));
    }
    let canonical =
        fs::canonicalize(&path).map_err(|_error| CatalogRegressionError::new(invalid_kind))?;
    if !canonical.starts_with(canonical_root) {
        return Err(CatalogRegressionError::new(
            CatalogRegressionErrorKind::UnsafePath,
        ));
    }
    let bytes = fs::read(canonical).map_err(|_error| CatalogRegressionError::new(invalid_kind))?;
    if bytes.len() != length {
        return Err(CatalogRegressionError::new(invalid_kind));
    }
    Ok(bytes)
}

fn reject_linked_components(
    canonical_root: &Path,
    relative: &Path,
) -> Result<(), CatalogRegressionError> {
    let mut current = canonical_root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(CatalogRegressionError::new(
                CatalogRegressionErrorKind::UnsafePath,
            ));
        };
        current.push(component);
        let metadata = fs::symlink_metadata(&current).map_err(|_error| {
            CatalogRegressionError::new(CatalogRegressionErrorKind::UnsafePath)
        })?;
        if metadata.file_type().is_symlink() {
            return Err(CatalogRegressionError::new(
                CatalogRegressionErrorKind::UnsafePath,
            ));
        }
    }
    Ok(())
}

fn validate_manifest_mapping(
    entry: &ManifestEntry,
    resolved: &liquidfun_test_protocol::ResolvedScenario,
) -> Result<(), CatalogRegressionError> {
    let identity = resolved.identity();
    if identity.catalog_schema_version() != CatalogSchemaVersion::CURRENT
        || entry.run_identity.catalog_schema_version != identity.catalog_schema_version().get()
        || entry.run_identity.slug != identity.slug().as_str()
        || identity.scenario_version() != ScenarioVersion::CURRENT
        || entry.run_identity.scenario_version != identity.scenario_version().get()
        || entry.run_identity.generator_id != identity.generator_id().as_str()
        || identity.generator_version() != GeneratorVersion::CURRENT
        || entry.run_identity.generator_version != identity.generator_version().get()
        || entry.run_identity.maybe_seed != identity.maybe_seed()
        || entry.run_identity.settings != identity.settings()
        || entry.resolved_sha256 != *identity.content_sha256()
    {
        return Err(CatalogRegressionError::new(
            CatalogRegressionErrorKind::CatalogMismatch,
        ));
    }
    let action_ids = resolved
        .actions()
        .iter()
        .map(|action| action.action_id().as_str())
        .collect::<Vec<_>>();
    let checkpoint_ids = resolved
        .checkpoints()
        .iter()
        .map(|checkpoint| checkpoint.checkpoint_id().as_str())
        .collect::<Vec<_>>();
    if action_ids
        != entry
            .action_ids
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
        || checkpoint_ids
            != entry
                .checkpoint_ids
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
    {
        return Err(CatalogRegressionError::new(
            CatalogRegressionErrorKind::CatalogMismatch,
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NativeReplayCapture {
    expanded_sha256: Sha256Hex,
    semantic_documents: CheckpointSemanticDocuments,
}

fn native_replay_capture(
    resolved: &liquidfun_test_protocol::ResolvedScenario,
) -> Result<NativeReplayCapture, CatalogRegressionError> {
    let request_id = RequestId::new("catalog-native-request").map_err(|_error| {
        CatalogRegressionError::new(CatalogRegressionErrorKind::NativeMismatch)
    })?;
    let capture = execute_resolved_catalog_native(&request_id, resolved).map_err(|_error| {
        CatalogRegressionError::new(CatalogRegressionErrorKind::NativeMismatch)
    })?;
    let mut hasher = Sha256::new();
    for bytes in capture.canonical_checkpoint_bytes() {
        hasher.update(bytes);
    }
    let semantic_documents =
        checkpoint_semantic_documents(capture.checkpoints()).map_err(|_error| {
            CatalogRegressionError::new(CatalogRegressionErrorKind::NativeMismatch)
        })?;
    Ok(NativeReplayCapture {
        expanded_sha256: Sha256Hex::from_digest(hasher.finalize().into()),
        semantic_documents,
    })
}

fn classify_native_identity(
    entry: &ManifestEntry,
    capture: &NativeReplayCapture,
    resolved: &liquidfun_test_protocol::ResolvedScenario,
) -> Result<(Sha256Hex, Option<ReplayDiagnosis>), CatalogRegressionError> {
    if capture.expanded_sha256 == entry.expected_native_d0_sha256 {
        return Ok((capture.expanded_sha256.clone(), None));
    }
    if capture.semantic_documents.legacy_physics_sha256 != entry.expected_native_d0_sha256 {
        return Err(CatalogRegressionError::new(
            CatalogRegressionErrorKind::NativeMismatch,
        ));
    }

    let catalog_schema_version = resolved.identity().catalog_schema_version().get();
    let reviewed_schema = ReplaySchemaIdentity::new(
        catalog_schema_version,
        1,
        ReplayProjectionVersion::LegacyPhysicsV1,
    );
    let current_schema = ReplaySchemaIdentity::new(
        catalog_schema_version,
        1,
        ReplayProjectionVersion::ExpandedCheckpointV1,
    );
    let reviewed = ReplaySemanticDocument::new(
        reviewed_schema,
        capture.semantic_documents.physics_projection.clone(),
        capture.semantic_documents.physics_projection.clone(),
    );
    let current = ReplaySemanticDocument::new(
        current_schema,
        capture.semantic_documents.physics_projection.clone(),
        capture.semantic_documents.expanded_checkpoint.clone(),
    );
    let diagnosis = diagnose_replay_drift(
        resolved.canonical_bytes(),
        resolved.canonical_bytes(),
        &reviewed,
        &current,
    )
    .map_err(|_error| CatalogRegressionError::new(CatalogRegressionErrorKind::NativeMismatch))?
    .filter(|diagnosis| diagnosis.drift_class() == ReplayDriftClass::CaptureSchemaDrift)
    .ok_or_else(|| CatalogRegressionError::new(CatalogRegressionErrorKind::NativeMismatch))?;
    Ok((
        capture.semantic_documents.legacy_physics_sha256.clone(),
        Some(diagnosis),
    ))
}
