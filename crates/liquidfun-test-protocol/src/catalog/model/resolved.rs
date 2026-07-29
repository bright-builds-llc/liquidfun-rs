use serde::{Deserialize, Serialize};

use super::{
    CatalogError, CatalogErrorKind, CatalogProgram, CatalogProgramKind, CatalogSchemaVersion,
    CheckpointDeclaration, GeneratorId, GeneratorVersion, ResolvedEntity, RunSettings,
    ScenarioActionId, ScenarioVersion, ScheduledAction,
};
use crate::{CheckpointId, ScenarioId, SemanticEntityId, SemanticEntityKind, Sha256Hex, Vec2Bits};

/// Hash-independent portion of one resolved run identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanonicalRunIdentity {
    pub(crate) catalog_schema_version: CatalogSchemaVersion,
    pub(crate) slug: super::CatalogSlug,
    pub(crate) scenario_version: ScenarioVersion,
    pub(crate) generator_id: GeneratorId,
    pub(crate) generator_version: GeneratorVersion,
    pub(crate) maybe_seed: Option<u64>,
    pub(crate) settings: RunSettings,
}

/// Complete immutable run identity, including exact resolved-byte hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunIdentity {
    canonical: CanonicalRunIdentity,
    content_sha256: Sha256Hex,
}

impl RunIdentity {
    pub(crate) const fn new(canonical: CanonicalRunIdentity, content_sha256: Sha256Hex) -> Self {
        Self {
            canonical,
            content_sha256,
        }
    }

    /// Returns the resolved catalog schema version.
    #[must_use]
    pub const fn catalog_schema_version(&self) -> CatalogSchemaVersion {
        self.canonical.catalog_schema_version
    }

    /// Returns the stable catalog slug.
    #[must_use]
    pub const fn slug(&self) -> &super::CatalogSlug {
        &self.canonical.slug
    }

    /// Returns the scenario definition version.
    #[must_use]
    pub const fn scenario_version(&self) -> ScenarioVersion {
        self.canonical.scenario_version
    }

    /// Returns the deterministic generator identity.
    #[must_use]
    pub const fn generator_id(&self) -> &GeneratorId {
        &self.canonical.generator_id
    }

    /// Returns the deterministic generator version.
    #[must_use]
    pub const fn generator_version(&self) -> GeneratorVersion {
        self.canonical.generator_version
    }

    /// Returns the exact seed when the definition is seeded.
    #[must_use]
    pub const fn maybe_seed(&self) -> Option<u64> {
        self.canonical.maybe_seed
    }

    /// Returns the exact validated run settings.
    #[must_use]
    pub const fn settings(&self) -> RunSettings {
        self.canonical.settings
    }

    /// Returns SHA-256 over the exact canonical resolved bytes.
    #[must_use]
    pub const fn content_sha256(&self) -> &Sha256Hex {
        &self.content_sha256
    }
}

/// Canonical hash payload decoded under fixed collection bounds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ResolvedPayload {
    pub(crate) identity: CanonicalRunIdentity,
    pub(crate) entities: Vec<ResolvedEntity>,
    pub(crate) actions: Vec<ScheduledAction>,
    pub(crate) checkpoints: Vec<CheckpointDeclaration>,
}

/// Immutable resolved plan and the exact bytes that identify it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedScenario {
    identity: RunIdentity,
    entities: Box<[ResolvedEntity]>,
    actions: Box<[ScheduledAction]>,
    checkpoints: Box<[CheckpointDeclaration]>,
    canonical_bytes: Box<[u8]>,
}

impl ResolvedScenario {
    pub(crate) fn from_payload(
        payload: ResolvedPayload,
        canonical_bytes: Vec<u8>,
        content_sha256: Sha256Hex,
    ) -> Self {
        Self {
            identity: RunIdentity::new(payload.identity, content_sha256),
            entities: payload.entities.into_boxed_slice(),
            actions: payload.actions.into_boxed_slice(),
            checkpoints: payload.checkpoints.into_boxed_slice(),
            canonical_bytes: canonical_bytes.into_boxed_slice(),
        }
    }

    /// Returns the complete replay identity.
    #[must_use]
    pub const fn identity(&self) -> &RunIdentity {
        &self.identity
    }

    /// Returns entities in deterministic semantic ordinal order.
    #[must_use]
    pub fn entities(&self) -> &[ResolvedEntity] {
        &self.entities
    }

    /// Returns actions in explicit execution order.
    #[must_use]
    pub fn actions(&self) -> &[ScheduledAction] {
        &self.actions
    }

    /// Returns checkpoints in logical-step order.
    #[must_use]
    pub fn checkpoints(&self) -> &[CheckpointDeclaration] {
        &self.checkpoints
    }

    /// Returns the exact persistable canonical JSON bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

pub(crate) fn deterministic_entity_id(
    kind: SemanticEntityKind,
    ordinal: u32,
) -> Result<ResolvedEntity, CatalogError> {
    let kind_name = match kind {
        SemanticEntityKind::Body => "body",
        SemanticEntityKind::Fixture => "fixture",
        SemanticEntityKind::Joint => "joint",
        SemanticEntityKind::Rope => "rope",
        SemanticEntityKind::ParticleSystem => "particle-system",
        SemanticEntityKind::ParticleGroup => "particle-group",
        SemanticEntityKind::Particle => "particle",
    };
    let scenario_id = ScenarioId::new(format!("entity-{kind_name}-{ordinal:04}"))
        .map_err(|_| CatalogError::new(CatalogErrorKind::InvalidIdentifier))?;
    Ok(ResolvedEntity::new(
        SemanticEntityId::new(kind, ordinal),
        scenario_id,
    ))
}

pub(crate) fn deterministic_action_id(ordinal: u32) -> Result<ScenarioActionId, CatalogError> {
    ScenarioActionId::new(format!("action-{ordinal:04}"))
}

pub(crate) fn deterministic_checkpoint_id(logical_step: u32) -> Result<CheckpointId, CatalogError> {
    CheckpointId::new(format!("checkpoint-{logical_step:04}"))
        .map_err(|_| CatalogError::new(CatalogErrorKind::InvalidIdentifier))
}

pub(crate) fn exact_gravity(program: &CatalogProgram) -> Option<Vec2Bits> {
    match program.kind() {
        CatalogProgramKind::ExactGravity(gravity) => Some(*gravity),
        CatalogProgramKind::SeededGravityChoices(_) | CatalogProgramKind::ExactActions { .. } => {
            None
        }
    }
}

pub(crate) fn gravity_choices(program: &CatalogProgram) -> Option<&[Vec2Bits]> {
    match program.kind() {
        CatalogProgramKind::SeededGravityChoices(choices) => Some(choices),
        CatalogProgramKind::ExactGravity(_) | CatalogProgramKind::ExactActions { .. } => None,
    }
}
