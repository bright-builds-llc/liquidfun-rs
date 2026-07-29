use std::collections::HashSet;

use rand_chacha::{
    ChaCha8Rng,
    rand_core::{Rng, SeedableRng},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::model::{
    ActionSchedule, CATALOG_MAXIMUM_ACTIONS, CATALOG_MAXIMUM_CANONICAL_BYTES,
    CATALOG_MAXIMUM_CHECKPOINTS, CATALOG_MAXIMUM_DEFINITIONS, CATALOG_MAXIMUM_ENTITIES,
    CanonicalRunIdentity, CatalogDefinition, CatalogError, CatalogErrorKind, CatalogProgramKind,
    CatalogSchemaVersion, CheckpointDeclaration, ResolveRequest, ResolvedEntity, ResolvedPayload,
    ResolvedScenario, ScenarioEligibility, ScheduledAction, deterministic_action_id,
    deterministic_checkpoint_id, deterministic_entity_id, exact_gravity, gravity_choices,
};
use crate::{Phase10Operation, RigidWorldAction, Sha256Hex, codec::BoundedVec};

mod particles;
mod validation;

/// Resolves one catalog request into bounded immutable canonical bytes.
///
/// # Errors
///
/// Returns [`CatalogError`] for duplicate or unknown slugs, invalid seed eligibility, limits,
/// identifier construction, or canonical encoding failures.
pub fn resolve_catalog(
    definitions: &[CatalogDefinition],
    request: &ResolveRequest,
) -> Result<ResolvedScenario, CatalogError> {
    let definition = select_definition(definitions, request)?;
    let maybe_gravity = resolve_gravity(definition, request.maybe_seed())?;
    let entities = resolve_entities(definition)?;
    let (actions, checkpoints) = resolve_schedule(definition, request, maybe_gravity)?;
    let payload = ResolvedPayload {
        identity: CanonicalRunIdentity {
            catalog_schema_version: CatalogSchemaVersion::CURRENT,
            slug: definition.slug().clone(),
            scenario_version: definition.scenario_version(),
            generator_id: definition.generator_id().clone(),
            generator_version: definition.generator_version(),
            maybe_seed: request.maybe_seed(),
            settings: request.settings(),
        },
        entities,
        actions,
        checkpoints,
    };
    encode_payload(payload)
}

fn select_definition<'a>(
    definitions: &'a [CatalogDefinition],
    request: &ResolveRequest,
) -> Result<&'a CatalogDefinition, CatalogError> {
    if definitions.len() > CATALOG_MAXIMUM_DEFINITIONS {
        return Err(CatalogError::new(CatalogErrorKind::TooManyDefinitions));
    }
    let mut slugs = HashSet::with_capacity(definitions.len());
    for definition in definitions {
        if !slugs.insert(definition.slug().as_str()) {
            return Err(CatalogError::new(CatalogErrorKind::DuplicateSlug));
        }
    }
    definitions
        .iter()
        .find(|definition| definition.slug() == request.slug())
        .ok_or_else(|| CatalogError::new(CatalogErrorKind::UnknownSlug))
}

fn resolve_gravity(
    definition: &CatalogDefinition,
    maybe_seed: Option<u64>,
) -> Result<Option<crate::Vec2Bits>, CatalogError> {
    match (definition.eligibility(), maybe_seed) {
        (ScenarioEligibility::NamedOnly, Some(_)) => {
            Err(CatalogError::new(CatalogErrorKind::SeedNotAllowed))
        }
        (ScenarioEligibility::NamedOnly, None) => match definition.program().kind() {
            CatalogProgramKind::ExactGravity(_) => exact_gravity(definition.program())
                .map(Some)
                .ok_or_else(|| CatalogError::new(CatalogErrorKind::InvalidRunSettings)),
            CatalogProgramKind::ExactActions { .. } => Ok(None),
            CatalogProgramKind::SeededGravityChoices(_) => {
                Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings))
            }
        },
        (ScenarioEligibility::SeedRequired, None) => {
            Err(CatalogError::new(CatalogErrorKind::SeedRequired))
        }
        (ScenarioEligibility::SeedRequired, Some(seed)) => {
            let choices = gravity_choices(definition.program())
                .ok_or_else(|| CatalogError::new(CatalogErrorKind::InvalidRunSettings))?;
            let mut generator = ChaCha8Rng::from_seed(expand_seed(seed));
            let choice = usize::try_from(generator.next_u32())
                .map_err(|_| CatalogError::new(CatalogErrorKind::InvalidRunSettings))?
                % choices.len();
            Ok(Some(choices[choice]))
        }
    }
}

fn expand_seed(seed: u64) -> [u8; 32] {
    let mut expanded = [0_u8; 32];
    let components = [
        seed,
        !seed,
        seed.rotate_left(17) ^ 0x6c69_7175_6964_6675,
        seed.rotate_right(11) ^ 0x6e2d_6361_7461_6c6f,
    ];
    for (destination, component) in expanded.chunks_exact_mut(8).zip(components) {
        destination.copy_from_slice(&component.to_le_bytes());
    }
    expanded
}

fn resolve_entities(definition: &CatalogDefinition) -> Result<Vec<ResolvedEntity>, CatalogError> {
    if definition.entity_kinds().len() > CATALOG_MAXIMUM_ENTITIES {
        return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
    }
    definition
        .entity_kinds()
        .iter()
        .copied()
        .enumerate()
        .map(|(ordinal, kind)| {
            let ordinal = u32::try_from(ordinal)
                .map_err(|_| CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded))?;
            deterministic_entity_id(kind, ordinal)
        })
        .collect()
}

fn resolve_schedule(
    definition: &CatalogDefinition,
    request: &ResolveRequest,
    maybe_gravity: Option<crate::Vec2Bits>,
) -> Result<(Vec<ScheduledAction>, Vec<CheckpointDeclaration>), CatalogError> {
    if let CatalogProgramKind::ExactActions {
        setup_actions,
        logical_actions,
    } = definition.program().kind()
    {
        return resolve_exact_actions(setup_actions, logical_actions, request.settings());
    }
    let gravity =
        maybe_gravity.ok_or_else(|| CatalogError::new(CatalogErrorKind::InvalidRunSettings))?;
    let step_count = definition.program().step_count();
    let action_capacity = usize::try_from(step_count)
        .ok()
        .and_then(|count| count.checked_add(1))
        .ok_or_else(|| CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded))?;
    if action_capacity > CATALOG_MAXIMUM_ACTIONS {
        return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
    }

    let gravity_action_id = deterministic_action_id(0)?;
    let mut actions = Vec::with_capacity(action_capacity);
    actions.push(ScheduledAction::new(
        gravity_action_id,
        ActionSchedule::Setup { ordinal: 0 },
        RigidWorldAction::SetWorldGravity { gravity },
    ));

    let checkpoint_capacity = usize::try_from(step_count)
        .map_err(|_| CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded))?;
    if checkpoint_capacity > CATALOG_MAXIMUM_CHECKPOINTS {
        return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
    }
    let mut checkpoints = Vec::with_capacity(checkpoint_capacity);
    for logical_step in 1..=step_count {
        let action_id = deterministic_action_id(logical_step)?;
        actions.push(ScheduledAction::new(
            action_id.clone(),
            ActionSchedule::LogicalStep {
                ordinal: logical_step,
            },
            RigidWorldAction::ConfiguredStep {
                timestep_bits: request.settings().timestep_bits(),
                velocity_iterations: request.settings().velocity_iterations(),
                position_iterations: request.settings().position_iterations(),
                continuous_work_budget: 1,
            },
        ));
        checkpoints.push(CheckpointDeclaration::new(
            deterministic_checkpoint_id(logical_step)?,
            action_id,
            logical_step,
        ));
    }
    Ok((actions, checkpoints))
}

fn resolve_exact_actions(
    setup_actions: &[RigidWorldAction],
    logical_actions: &[RigidWorldAction],
    settings: super::RunSettings,
) -> Result<(Vec<ScheduledAction>, Vec<CheckpointDeclaration>), CatalogError> {
    let action_capacity = setup_actions
        .len()
        .checked_add(logical_actions.len())
        .ok_or_else(|| CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded))?;
    if setup_actions.is_empty()
        || logical_actions.is_empty()
        || action_capacity > CATALOG_MAXIMUM_ACTIONS
        || logical_actions.len() > CATALOG_MAXIMUM_CHECKPOINTS
    {
        return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
    }

    let mut actions = Vec::with_capacity(action_capacity);
    for (ordinal, action) in setup_actions.iter().enumerate() {
        let ordinal = u32::try_from(ordinal)
            .map_err(|_| CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded))?;
        actions.push(ScheduledAction::new(
            deterministic_action_id(ordinal)?,
            ActionSchedule::Setup { ordinal },
            materialize_action(action, settings),
        ));
    }

    let mut checkpoints = Vec::with_capacity(logical_actions.len());
    for (index, action) in logical_actions.iter().enumerate() {
        let logical_step = u32::try_from(index)
            .ok()
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded))?;
        let action_ordinal = setup_actions
            .len()
            .checked_add(index)
            .ok_or_else(|| CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded))?;
        let action_ordinal = u32::try_from(action_ordinal)
            .map_err(|_| CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded))?;
        let action_id = deterministic_action_id(action_ordinal)?;
        actions.push(ScheduledAction::new(
            action_id.clone(),
            ActionSchedule::LogicalStep {
                ordinal: logical_step,
            },
            materialize_action(action, settings),
        ));
        checkpoints.push(CheckpointDeclaration::new(
            deterministic_checkpoint_id(logical_step)?,
            action_id,
            logical_step,
        ));
    }
    Ok((actions, checkpoints))
}

fn materialize_action(action: &RigidWorldAction, settings: super::RunSettings) -> RigidWorldAction {
    match action {
        RigidWorldAction::ConfiguredStep {
            continuous_work_budget,
            ..
        } => RigidWorldAction::ConfiguredStep {
            timestep_bits: settings.timestep_bits(),
            velocity_iterations: settings.velocity_iterations(),
            position_iterations: settings.position_iterations(),
            continuous_work_budget: *continuous_work_budget,
        },
        RigidWorldAction::StepRope { rope_id, .. } => RigidWorldAction::StepRope {
            rope_id: rope_id.clone(),
            timestep_bits: settings.timestep_bits(),
            iterations: settings.particle_iterations(),
        },
        RigidWorldAction::ParticleGroup {
            operation: Phase10Operation::Step { .. },
        } => RigidWorldAction::ParticleGroup {
            operation: Phase10Operation::Step {
                timestep_bits: settings.timestep_bits(),
                velocity_iterations: settings.velocity_iterations(),
                position_iterations: settings.position_iterations(),
                particle_iterations: settings.particle_iterations(),
            },
        },
        _ => action.clone(),
    }
}

fn encode_payload(payload: ResolvedPayload) -> Result<ResolvedScenario, CatalogError> {
    let canonical_bytes = serde_json::to_vec(&payload)
        .map_err(|_| CatalogError::new(CatalogErrorKind::CanonicalEncoding))?;
    if canonical_bytes.len() > CATALOG_MAXIMUM_CANONICAL_BYTES {
        return Err(CatalogError::new(CatalogErrorKind::CanonicalBytesExceeded));
    }
    let content_sha256 = Sha256Hex::from_digest(Sha256::digest(&canonical_bytes).into());
    Ok(ResolvedScenario::from_payload(
        payload,
        canonical_bytes,
        content_sha256,
    ))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawResolvedPayload {
    identity: CanonicalRunIdentity,
    entities: BoundedVec<ResolvedEntity, CATALOG_MAXIMUM_ENTITIES>,
    actions: BoundedVec<ScheduledAction, CATALOG_MAXIMUM_ACTIONS>,
    checkpoints: BoundedVec<CheckpointDeclaration, CATALOG_MAXIMUM_CHECKPOINTS>,
}

/// Decodes canonical resolved bytes and verifies their asserted SHA-256 identity.
///
/// # Errors
///
/// Returns [`CatalogError`] for oversized, malformed, noncanonical, out-of-bound, internally
/// inconsistent, or hash-mismatched bytes.
pub fn decode_resolved_scenario(
    canonical_bytes: &[u8],
    expected_sha256: &Sha256Hex,
) -> Result<ResolvedScenario, CatalogError> {
    if canonical_bytes.len() > CATALOG_MAXIMUM_CANONICAL_BYTES {
        return Err(CatalogError::new(CatalogErrorKind::CanonicalBytesExceeded));
    }
    let actual_sha256 = Sha256Hex::from_digest(Sha256::digest(canonical_bytes).into());
    if &actual_sha256 != expected_sha256 {
        return Err(CatalogError::new(CatalogErrorKind::HashMismatch));
    }

    let raw: RawResolvedPayload = serde_json::from_slice(canonical_bytes)
        .map_err(|_| CatalogError::new(CatalogErrorKind::CanonicalEncoding))?;
    let payload = ResolvedPayload {
        identity: raw.identity,
        entities: raw.entities.into_vec(),
        actions: raw.actions.into_vec(),
        checkpoints: raw.checkpoints.into_vec(),
    };
    validation::validate_payload(&payload)?;
    let reencoded = serde_json::to_vec(&payload)
        .map_err(|_| CatalogError::new(CatalogErrorKind::CanonicalEncoding))?;
    if reencoded != canonical_bytes {
        return Err(CatalogError::new(CatalogErrorKind::NonCanonicalBytes));
    }
    Ok(ResolvedScenario::from_payload(
        payload,
        canonical_bytes.to_vec(),
        actual_sha256,
    ))
}
