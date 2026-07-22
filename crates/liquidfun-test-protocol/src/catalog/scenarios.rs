use crate::{
    CatalogCoverage, CatalogDefinition, CatalogError, CatalogEvidence, CatalogMetadata,
    CatalogProgram, CatalogSlug, FloatBits, GeneratorId, GeneratorVersion, RigidJointKind,
    RigidWorldAction, RigidWorldWitness, RunSettings, ScenarioEligibility, ScenarioId,
    ScenarioVersion, SemanticEntityKind, Vec2Bits,
};

/// Native scenarios covering particle-group topology and mutation behavior.
pub mod groups;
/// Native scenarios covering all eleven supported joint kinds.
pub mod joints;
/// Native scenarios covering particle storage, lifecycle, and solver behavior.
pub mod particles;
/// Native scenarios covering particle queries, callbacks, and rejected mutations.
pub mod queries_callbacks;
/// Native scenarios covering representative rigid-world behaviors.
pub mod rigid;
/// Native standalone-rope scenarios.
pub mod rope;

/// Composes every reviewed native family into stable slug/version order.
///
/// # Errors
///
/// Returns [`CatalogError`] if any family definition violates its typed invariants.
pub fn scenario_definitions() -> Result<Vec<CatalogDefinition>, CatalogError> {
    let mut definitions = rigid::definitions()?;
    definitions.extend(joints::definitions()?);
    definitions.extend(rope::definitions()?);
    definitions.extend(particles::definitions()?);
    definitions.extend(groups::definitions()?);
    definitions.extend(queries_callbacks::definitions()?);
    definitions.sort_unstable_by(|left, right| {
        (left.slug(), left.scenario_version()).cmp(&(right.slug(), right.scenario_version()))
    });
    Ok(definitions)
}

fn bits(value: f32) -> FloatBits {
    FloatBits::from_f32(value)
}

fn vec2(x: f32, y: f32) -> Vec2Bits {
    Vec2Bits {
        x_bits: bits(x),
        y_bits: bits(y),
    }
}

fn entity_id(kind: SemanticEntityKind, ordinal: u32) -> Result<ScenarioId, CatalogError> {
    let kind_name = match kind {
        SemanticEntityKind::Body => "body",
        SemanticEntityKind::Fixture => "fixture",
        SemanticEntityKind::Joint => "joint",
        SemanticEntityKind::Rope => "rope",
        SemanticEntityKind::ParticleSystem => "particle-system",
        SemanticEntityKind::ParticleGroup => "particle-group",
        SemanticEntityKind::Particle => "particle",
    };
    ScenarioId::new(format!("entity-{kind_name}-{ordinal:04}"))
        .map_err(|_| CatalogError::new(super::CatalogErrorKind::InvalidIdentifier))
}

fn default_settings(particle_iterations: u32) -> Result<RunSettings, CatalogError> {
    RunSettings::new(bits(1.0 / 60.0), 8, 3, particle_iterations)
}

#[allow(
    clippy::too_many_arguments,
    reason = "native definitions carry explicit identity, schedule, and coverage"
)]
fn definition(
    slug: &str,
    title: &str,
    generator_id: &str,
    tags: &[&str],
    test_id: &str,
    evidence_leaves: &[RigidWorldWitness],
    maybe_joint_kind: Option<RigidJointKind>,
    entity_kinds: Vec<SemanticEntityKind>,
    setup_actions: Vec<RigidWorldAction>,
    logical_actions: Vec<RigidWorldAction>,
    particle_iterations: u32,
) -> Result<CatalogDefinition, CatalogError> {
    definition_with_evidence(
        slug,
        title,
        generator_id,
        tags,
        test_id,
        evidence_leaves
            .iter()
            .copied()
            .map(CatalogEvidence::Rigid)
            .collect(),
        maybe_joint_kind,
        entity_kinds,
        setup_actions,
        logical_actions,
        particle_iterations,
    )
}

#[allow(
    clippy::too_many_arguments,
    reason = "native definitions carry explicit identity, schedule, and coverage"
)]
fn definition_with_evidence(
    slug: &str,
    title: &str,
    generator_id: &str,
    tags: &[&str],
    test_id: &str,
    evidence_leaves: Vec<CatalogEvidence>,
    maybe_joint_kind: Option<RigidJointKind>,
    entity_kinds: Vec<SemanticEntityKind>,
    setup_actions: Vec<RigidWorldAction>,
    logical_actions: Vec<RigidWorldAction>,
    particle_iterations: u32,
) -> Result<CatalogDefinition, CatalogError> {
    let settings = default_settings(particle_iterations)?;
    let coverage = CatalogCoverage::new(
        vec![CatalogSlug::new(test_id)?],
        evidence_leaves,
        true,
        true,
        true,
    )?;
    let metadata = CatalogMetadata::new(
        tags.iter()
            .map(|tag| CatalogSlug::new(*tag))
            .collect::<Result<Vec<_>, _>>()?,
        settings,
        coverage,
        maybe_joint_kind,
    )?;
    CatalogDefinition::new(
        CatalogSlug::new(slug)?,
        title,
        ScenarioVersion::CURRENT,
        GeneratorId::new(generator_id)?,
        GeneratorVersion::CURRENT,
        ScenarioEligibility::NamedOnly,
        entity_kinds,
        CatalogProgram::exact_actions(setup_actions, logical_actions)?,
    )
    .map(|definition| definition.with_metadata(metadata))
}

fn configured_steps(settings: RunSettings, count: usize) -> Vec<RigidWorldAction> {
    (0..count)
        .map(|_| RigidWorldAction::ConfiguredStep {
            timestep_bits: settings.timestep_bits(),
            velocity_iterations: settings.velocity_iterations(),
            position_iterations: settings.position_iterations(),
            continuous_work_budget: 1,
        })
        .collect()
}
