use serde::{Deserialize, Serialize};

use crate::{
    CheckpointId, RigidWorldAction, ScenarioId, SemanticEntityId, SemanticEntityKind, Vec2Bits,
};

mod identity;
pub use identity::*;
mod metadata;
pub use metadata::*;
mod resolved;
pub(crate) use resolved::{
    CanonicalRunIdentity, ResolvedPayload, deterministic_action_id, deterministic_checkpoint_id,
    deterministic_entity_id, exact_gravity, gravity_choices,
};
pub use resolved::{ResolvedScenario, RunIdentity};

/// Maximum catalog definitions accepted by one resolver call.
pub const CATALOG_MAXIMUM_DEFINITIONS: usize = 256;
/// Maximum semantic entities in one resolved scenario.
pub const CATALOG_MAXIMUM_ENTITIES: usize = 4_096;
/// Maximum ordered actions in one resolved scenario.
pub const CATALOG_MAXIMUM_ACTIONS: usize = 128;
/// Maximum checkpoint declarations in one resolved scenario.
pub const CATALOG_MAXIMUM_CHECKPOINTS: usize = 128;
/// Maximum exact choices in one seeded catalog program.
pub const CATALOG_MAXIMUM_GENERATOR_CHOICES: usize = 128;
/// Maximum canonical resolved-scenario byte length.
pub const CATALOG_MAXIMUM_CANONICAL_BYTES: usize = 1024 * 1024;
/// Maximum reviewed solver iteration count.
pub const CATALOG_MAXIMUM_ITERATIONS: u32 = 1_024;

/// Stable failure categories for catalog definition, resolution, and replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogErrorKind {
    /// A stable slug or semantic identifier is malformed.
    InvalidIdentifier,
    /// A display title is empty or exceeds its presentation-only bound.
    InvalidDisplayTitle,
    /// A version does not match the supported catalog contract.
    UnsupportedVersion,
    /// A run setting is zero, non-finite, or outside reviewed bounds.
    InvalidRunSettings,
    /// A catalog definition collection exceeds its reviewed bound.
    TooManyDefinitions,
    /// Two definitions use the same stable slug.
    DuplicateSlug,
    /// Two definitions use the same stable slug and scenario version.
    DuplicateScenarioIdentity,
    /// Two consumer mappings target the same stable scenario identity.
    DuplicateMapping,
    /// A registered scenario has no consumer mapping.
    MissingMapping,
    /// A consumer mapping does not resolve to a registered scenario.
    UnknownMapping,
    /// A mapped public test identity is not in the reviewed test registry.
    StaleTestId,
    /// A mapped evidence identity is outside its sealed authority.
    StaleEvidence,
    /// A mapped upstream corpus identity is absent from the checked authority.
    StaleUpstreamCorpusId,
    /// A mapped compatibility reference is absent from the checked ledger.
    StaleCompatibilityRef,
    /// Consumer eligibility disagrees with the scenario definition metadata.
    ContradictoryEligibility,
    /// Presentation text was supplied where a stable catalog slug was required.
    TitleAsIdentity,
    /// A seeded scenario omitted a stable generator identity or version.
    SeedGeneratorMissing,
    /// No definition has the requested stable slug.
    UnknownSlug,
    /// A named-only definition received a seed.
    SeedNotAllowed,
    /// A seeded definition did not receive a seed.
    SeedRequired,
    /// A resolved collection exceeds a reviewed count bound.
    ResolvedLimitExceeded,
    /// A catalog program has no exact choice to generate from.
    EmptyGeneratorChoices,
    /// Required catalog metadata is empty or internally inconsistent.
    InvalidMetadata,
    /// A checkpoint names an action outside the resolved schedule.
    InvalidCheckpointReference,
    /// Canonical JSON encoding or strict decoding failed.
    CanonicalEncoding,
    /// Resolved bytes exceed their reviewed byte limit.
    CanonicalBytesExceeded,
    /// Persisted bytes are valid JSON but are not the canonical encoding.
    NonCanonicalBytes,
    /// Persisted bytes do not match their asserted SHA-256 identity.
    HashMismatch,
    /// Tracked presentation bytes differ from the typed in-memory projection.
    ProjectionMismatch,
}

/// Bounded semantic catalog error without raw record disclosure.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("scenario catalog failure: {kind:?}")]
pub struct CatalogError {
    kind: CatalogErrorKind,
}

impl CatalogError {
    pub(crate) const fn new(kind: CatalogErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(&self) -> CatalogErrorKind {
        self.kind
    }
}

/// Closed private catalog program resolved without effects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogProgram {
    kind: CatalogProgramKind,
    step_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CatalogProgramKind {
    ExactGravity(Vec2Bits),
    SeededGravityChoices(Box<[Vec2Bits]>),
    ExactActions {
        setup_actions: Box<[RigidWorldAction]>,
        logical_actions: Box<[RigidWorldAction]>,
    },
}

impl CatalogProgram {
    /// Creates a named exact-gravity program.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError`] when the requested schedule exceeds the action bound.
    pub fn exact_gravity(gravity: Vec2Bits, step_count: u32) -> Result<Self, CatalogError> {
        Self::validate_step_count(step_count)?;
        Ok(Self {
            kind: CatalogProgramKind::ExactGravity(gravity),
            step_count,
        })
    }

    /// Creates a seeded program that selects one exact gravity vector with `ChaCha8`.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError`] for an empty or oversized choice set or action schedule.
    pub fn seeded_gravity_choices(
        choices: Vec<Vec2Bits>,
        step_count: u32,
    ) -> Result<Self, CatalogError> {
        Self::validate_step_count(step_count)?;
        if choices.is_empty() {
            return Err(CatalogError::new(CatalogErrorKind::EmptyGeneratorChoices));
        }
        if choices.len() > CATALOG_MAXIMUM_GENERATOR_CHOICES {
            return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
        }
        Ok(Self {
            kind: CatalogProgramKind::SeededGravityChoices(choices.into_boxed_slice()),
            step_count,
        })
    }

    /// Creates a named program from exact closed setup and logical-step actions.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError`] when either schedule is empty or the combined schedule exceeds
    /// the reviewed action or checkpoint bounds.
    pub(crate) fn exact_actions(
        setup_actions: Vec<RigidWorldAction>,
        logical_actions: Vec<RigidWorldAction>,
    ) -> Result<Self, CatalogError> {
        if setup_actions.is_empty() || logical_actions.is_empty() {
            return Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings));
        }
        let Some(action_count) = setup_actions.len().checked_add(logical_actions.len()) else {
            return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
        };
        if action_count > CATALOG_MAXIMUM_ACTIONS
            || logical_actions.len() > CATALOG_MAXIMUM_CHECKPOINTS
        {
            return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
        }
        let step_count = u32::try_from(logical_actions.len())
            .map_err(|_| CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded))?;
        Ok(Self {
            kind: CatalogProgramKind::ExactActions {
                setup_actions: setup_actions.into_boxed_slice(),
                logical_actions: logical_actions.into_boxed_slice(),
            },
            step_count,
        })
    }

    fn validate_step_count(step_count: u32) -> Result<(), CatalogError> {
        let action_count = usize::try_from(step_count)
            .ok()
            .and_then(|steps| steps.checked_add(1));
        if step_count == 0
            || !matches!(action_count, Some(count) if count <= CATALOG_MAXIMUM_ACTIONS)
        {
            return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
        }
        Ok(())
    }

    pub(crate) const fn step_count(&self) -> u32 {
        self.step_count
    }

    pub(crate) const fn kind(&self) -> &CatalogProgramKind {
        &self.kind
    }
}

/// One private, versioned catalog definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogDefinition {
    slug: CatalogSlug,
    display_title: Box<str>,
    scenario_version: ScenarioVersion,
    generator_id: GeneratorId,
    generator_version: GeneratorVersion,
    eligibility: ScenarioEligibility,
    entity_kinds: Box<[SemanticEntityKind]>,
    program: CatalogProgram,
    maybe_metadata: Option<CatalogMetadata>,
}

impl CatalogDefinition {
    /// Creates and validates one catalog definition.
    ///
    /// # Errors
    ///
    /// Returns [`CatalogError`] when the title or entity count violates reviewed bounds.
    #[allow(
        clippy::too_many_arguments,
        reason = "the definition has seven identity and program fields"
    )]
    pub fn new(
        slug: CatalogSlug,
        display_title: impl Into<String>,
        scenario_version: ScenarioVersion,
        generator_id: GeneratorId,
        generator_version: GeneratorVersion,
        eligibility: ScenarioEligibility,
        entity_kinds: Vec<SemanticEntityKind>,
        program: CatalogProgram,
    ) -> Result<Self, CatalogError> {
        let display_title = display_title.into();
        if display_title.is_empty() || display_title.len() > 256 {
            return Err(CatalogError::new(CatalogErrorKind::InvalidDisplayTitle));
        }
        if entity_kinds.len() > CATALOG_MAXIMUM_ENTITIES {
            return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
        }
        let program_matches = matches!(
            (eligibility, program.kind()),
            (
                ScenarioEligibility::NamedOnly,
                CatalogProgramKind::ExactGravity(_) | CatalogProgramKind::ExactActions { .. }
            ) | (
                ScenarioEligibility::SeedRequired,
                CatalogProgramKind::SeededGravityChoices(_)
            )
        );
        if !program_matches {
            return Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings));
        }
        Ok(Self {
            slug,
            display_title: display_title.into_boxed_str(),
            scenario_version,
            generator_id,
            generator_version,
            eligibility,
            entity_kinds: entity_kinds.into_boxed_slice(),
            program,
            maybe_metadata: None,
        })
    }

    /// Attaches validated discovery, default-setting, coverage, and eligibility metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: CatalogMetadata) -> Self {
        self.maybe_metadata = Some(metadata);
        self
    }

    /// Returns the stable lookup slug.
    #[must_use]
    pub const fn slug(&self) -> &CatalogSlug {
        &self.slug
    }

    /// Returns presentation-only title text.
    #[must_use]
    pub fn display_title(&self) -> &str {
        &self.display_title
    }

    /// Returns native scenario metadata when this is a discoverable catalog definition.
    #[must_use]
    pub const fn metadata(&self) -> Option<&CatalogMetadata> {
        self.maybe_metadata.as_ref()
    }

    /// Returns the stable scenario contract version.
    #[must_use]
    pub const fn scenario_version(&self) -> ScenarioVersion {
        self.scenario_version
    }

    pub(crate) const fn generator_id(&self) -> &GeneratorId {
        &self.generator_id
    }

    pub(crate) const fn generator_version(&self) -> GeneratorVersion {
        self.generator_version
    }

    pub(crate) const fn eligibility(&self) -> ScenarioEligibility {
        self.eligibility
    }

    pub(crate) fn entity_kinds(&self) -> &[SemanticEntityKind] {
        &self.entity_kinds
    }

    pub(crate) const fn program(&self) -> &CatalogProgram {
        &self.program
    }
}

/// Boundary input selecting a stable catalog definition and exact settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveRequest {
    slug: CatalogSlug,
    maybe_seed: Option<u64>,
    settings: RunSettings,
}

impl ResolveRequest {
    /// Creates a validated resolver request.
    #[must_use]
    pub const fn new(slug: CatalogSlug, maybe_seed: Option<u64>, settings: RunSettings) -> Self {
        Self {
            slug,
            maybe_seed,
            settings,
        }
    }

    pub(crate) const fn slug(&self) -> &CatalogSlug {
        &self.slug
    }

    pub(crate) const fn maybe_seed(&self) -> Option<u64> {
        self.maybe_seed
    }

    pub(crate) const fn settings(&self) -> RunSettings {
        self.settings
    }
}

/// One deterministic semantic entity identity in a resolved plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedEntity {
    semantic_id: SemanticEntityId,
    scenario_id: ScenarioId,
}

impl ResolvedEntity {
    pub(crate) const fn new(semantic_id: SemanticEntityId, scenario_id: ScenarioId) -> Self {
        Self {
            semantic_id,
            scenario_id,
        }
    }

    /// Returns the engine-neutral kind and deterministic ordinal.
    #[must_use]
    pub const fn semantic_id(&self) -> SemanticEntityId {
        self.semantic_id
    }

    /// Returns the stable protocol identity used by closed actions.
    #[must_use]
    pub const fn scenario_id(&self) -> &ScenarioId {
        &self.scenario_id
    }
}

/// Explicit placement of an action in setup or logical-step order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ActionSchedule {
    /// An ordered action applied before the first logical step.
    Setup {
        /// Zero-based setup ordinal.
        ordinal: u32,
    },
    /// One action applying one logical simulation step.
    LogicalStep {
        /// One-based logical step ordinal.
        ordinal: u32,
    },
}

/// One closed action with stable identity and explicit order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScheduledAction {
    action_id: ScenarioActionId,
    schedule: ActionSchedule,
    action: RigidWorldAction,
}

impl ScheduledAction {
    pub(crate) const fn new(
        action_id: ScenarioActionId,
        schedule: ActionSchedule,
        action: RigidWorldAction,
    ) -> Self {
        Self {
            action_id,
            schedule,
            action,
        }
    }

    /// Returns the stable action identity.
    #[must_use]
    pub const fn action_id(&self) -> &ScenarioActionId {
        &self.action_id
    }

    /// Returns the explicit action placement.
    #[must_use]
    pub const fn schedule(&self) -> ActionSchedule {
        self.schedule
    }

    /// Returns the existing closed rigid-world protocol action.
    #[must_use]
    pub const fn action(&self) -> &RigidWorldAction {
        &self.action
    }
}

/// One deterministic checkpoint bound to an action and logical step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointDeclaration {
    checkpoint_id: CheckpointId,
    after_action_id: ScenarioActionId,
    logical_step: u32,
}

impl CheckpointDeclaration {
    pub(crate) const fn new(
        checkpoint_id: CheckpointId,
        after_action_id: ScenarioActionId,
        logical_step: u32,
    ) -> Self {
        Self {
            checkpoint_id,
            after_action_id,
            logical_step,
        }
    }

    /// Returns the stable checkpoint identity.
    #[must_use]
    pub const fn checkpoint_id(&self) -> &CheckpointId {
        &self.checkpoint_id
    }

    /// Returns the action boundary captured by this checkpoint.
    #[must_use]
    pub const fn after_action_id(&self) -> &ScenarioActionId {
        &self.after_action_id
    }

    /// Returns the one-based logical step ordinal.
    #[must_use]
    pub const fn logical_step(&self) -> u32 {
        self.logical_step
    }
}
