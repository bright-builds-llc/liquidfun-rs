use super::{
    FloatBits, HashMap, HashSet, RIGID_WORLD_MAXIMUM_DIRECTIVES, RawCheckpoint, RawSource,
    RigidAabbBits, RigidContactDirectiveTarget, RigidContactIdentity, RigidExpectedCheckpoint,
    RigidExpectedTransition, RigidFixtureChildSelector, RigidFixtureShape, RigidPreSolveDirective,
    RigidQueryDirectiveRule, RigidRayDirective, RigidRayDirectiveRule, RigidWorldActionRecord,
    RigidWorldDecodeError, RigidWorldErrorKind, RigidWorldWitnessFamily, ScenarioId,
    ScenarioSource, validate_nonnegative, validate_positive, validate_vec2, validation,
};

pub(super) fn validate_unit_interval(bits: FloatBits) -> Result<(), RigidWorldDecodeError> {
    let value = bits.to_f32();
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(validation(RigidWorldErrorKind::InvalidJointDefinition));
    }
    Ok(())
}

pub(super) fn validate_nonzero_vector(
    vector: crate::Vec2Bits,
    kind: RigidWorldErrorKind,
) -> Result<(), RigidWorldDecodeError> {
    let x = vector.x_bits.to_f32();
    let y = vector.y_bits.to_f32();
    if x == 0.0 && y == 0.0 {
        return Err(validation(kind));
    }
    Ok(())
}

pub(super) fn validate_contact_directive_target(
    target: &RigidContactDirectiveTarget,
    live_fixtures: &HashSet<ScenarioId>,
) -> Result<(), RigidWorldDecodeError> {
    if target.fixture_a_id == target.fixture_b_id
        || !live_fixtures.contains(&target.fixture_a_id)
        || !live_fixtures.contains(&target.fixture_b_id)
    {
        return Err(validation(RigidWorldErrorKind::InvalidContactDirective));
    }
    Ok(())
}

pub(super) fn validate_pre_solve_directive(
    directive: RigidPreSolveDirective,
) -> Result<(), RigidWorldDecodeError> {
    for bits in [
        directive.maybe_friction_bits,
        directive.maybe_restitution_bits,
    ]
    .into_iter()
    .flatten()
    {
        validate_nonnegative(bits)
            .map_err(|_| validation(RigidWorldErrorKind::InvalidContactDirective))?;
    }
    if let Some(bits) = directive.maybe_tangent_speed_bits {
        validate_finite(bits, RigidWorldErrorKind::InvalidContactDirective)?;
    }
    Ok(())
}

pub(super) fn validate_finite(
    value: FloatBits,
    kind: RigidWorldErrorKind,
) -> Result<(), RigidWorldDecodeError> {
    if !value.to_f32().is_finite() {
        return Err(validation(kind));
    }
    Ok(())
}

pub(super) fn validate_aabb(aabb: RigidAabbBits) -> Result<(), RigidWorldDecodeError> {
    validate_vec2(aabb.lower)?;
    validate_vec2(aabb.upper)?;
    if aabb.lower.x_bits.to_f32() > aabb.upper.x_bits.to_f32()
        || aabb.lower.y_bits.to_f32() > aabb.upper.y_bits.to_f32()
    {
        return Err(validation(RigidWorldErrorKind::InvalidQueryDirective));
    }
    Ok(())
}

pub(super) fn validate_query_rules(
    rules: &[RigidQueryDirectiveRule],
    live_fixtures: &HashSet<ScenarioId>,
    fixture_shapes: &HashMap<ScenarioId, RigidFixtureShape>,
) -> Result<(), RigidWorldDecodeError> {
    if rules.len() > RIGID_WORLD_MAXIMUM_DIRECTIVES {
        return Err(validation(RigidWorldErrorKind::AggregateLimitExceeded));
    }
    validate_unique_selectors(
        rules.iter().map(|rule| &rule.target),
        live_fixtures,
        fixture_shapes,
        RigidWorldErrorKind::InvalidQueryDirective,
    )
}

pub(super) fn validate_ray_geometry(
    start: crate::Vec2Bits,
    end: crate::Vec2Bits,
) -> Result<(), RigidWorldDecodeError> {
    let direction_x = end.x_bits.to_f32() - start.x_bits.to_f32();
    let direction_y = end.y_bits.to_f32() - start.y_bits.to_f32();
    let squared_x = direction_x * direction_x;
    let squared_y = direction_y * direction_y;
    let length_squared = squared_x + squared_y;
    if !direction_x.is_finite()
        || !direction_y.is_finite()
        || !squared_x.is_finite()
        || !squared_y.is_finite()
        || !length_squared.is_finite()
        || length_squared == 0.0
    {
        return Err(validation(RigidWorldErrorKind::InvalidRayDirective));
    }
    Ok(())
}

pub(super) fn validate_ray_rules(
    rules: &[RigidRayDirectiveRule],
    live_fixtures: &HashSet<ScenarioId>,
    fixture_shapes: &HashMap<ScenarioId, RigidFixtureShape>,
) -> Result<(), RigidWorldDecodeError> {
    if rules.len() > RIGID_WORLD_MAXIMUM_DIRECTIVES {
        return Err(validation(RigidWorldErrorKind::AggregateLimitExceeded));
    }
    validate_unique_selectors(
        rules.iter().map(|rule| &rule.target),
        live_fixtures,
        fixture_shapes,
        RigidWorldErrorKind::InvalidRayDirective,
    )?;
    for rule in rules {
        if let RigidRayDirective::Clip { fraction_bits } = rule.directive {
            let fraction = fraction_bits.to_f32();
            if !fraction.is_finite() || fraction <= 0.0 || fraction > 1.0 {
                return Err(validation(RigidWorldErrorKind::InvalidRayDirective));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_unique_selectors<'a>(
    selectors: impl Iterator<Item = &'a RigidFixtureChildSelector>,
    live_fixtures: &HashSet<ScenarioId>,
    fixture_shapes: &HashMap<ScenarioId, RigidFixtureShape>,
    kind: RigidWorldErrorKind,
) -> Result<(), RigidWorldDecodeError> {
    let mut unique = HashSet::new();
    for selector in selectors {
        let maybe_shape = fixture_shapes.get(&selector.fixture_id);
        if !live_fixtures.contains(&selector.fixture_id)
            || maybe_shape.is_none_or(|shape| selector.child_index >= shape_child_count(shape))
            || !unique.insert(selector.clone())
        {
            return Err(validation(kind));
        }
    }
    Ok(())
}

pub(super) const fn shape_child_count(shape: &RigidFixtureShape) -> u32 {
    match shape {
        RigidFixtureShape::Circle { .. } | RigidFixtureShape::Polygon { .. } => 1,
    }
}

pub(super) fn validate_custom_mass(
    mass_bits: FloatBits,
    center: crate::Vec2Bits,
    inertia_bits: FloatBits,
) -> Result<(), RigidWorldDecodeError> {
    validate_positive(mass_bits)?;
    validate_vec2(center)?;
    validate_nonnegative(inertia_bits)?;

    let mass = mass_bits.to_f32();
    let origin_inertia = inertia_bits.to_f32();
    if origin_inertia == 0.0 {
        return Ok(());
    }
    let center_x = center.x_bits.to_f32();
    let center_y = center.y_bits.to_f32();
    let squared_center = [center_x * center_x, center_y * center_y];
    let center_dot = squared_center[0] + squared_center[1];
    let parallel_axis = mass * center_dot;
    let centered_inertia = origin_inertia - parallel_axis;
    if !squared_center[0].is_finite()
        || !squared_center[1].is_finite()
        || !center_dot.is_finite()
        || !parallel_axis.is_finite()
        || !centered_inertia.is_finite()
        || centered_inertia <= 0.0
    {
        return Err(validation(RigidWorldErrorKind::InvalidGeometry));
    }
    Ok(())
}

pub(super) fn validate_checkpoints(
    raw_checkpoints: Vec<RawCheckpoint>,
    family: RigidWorldWitnessFamily,
    actions: &[RigidWorldActionRecord],
    body_ids: &HashSet<ScenarioId>,
    fixture_owners: &HashMap<ScenarioId, ScenarioId>,
) -> Result<Vec<RigidExpectedCheckpoint>, RigidWorldDecodeError> {
    if raw_checkpoints.is_empty() {
        return Err(validation(RigidWorldErrorKind::InvalidCheckpointOrder));
    }
    let action_positions = actions
        .iter()
        .enumerate()
        .map(|(index, action)| (action.action_id.clone(), index))
        .collect::<HashMap<_, _>>();
    let live_counts = action_live_counts(actions, fixture_owners);
    let mut checkpoint_ids = HashSet::with_capacity(raw_checkpoints.len());
    let mut witnesses = HashSet::new();
    let mut previous_action_index = None;
    let mut checkpoints = Vec::with_capacity(raw_checkpoints.len());

    for raw in raw_checkpoints {
        if !checkpoint_ids.insert(raw.checkpoint_id.clone()) {
            return Err(validation(RigidWorldErrorKind::DuplicateCheckpointId));
        }
        let Some(&action_index) = action_positions.get(&raw.after_action_id) else {
            return Err(validation(RigidWorldErrorKind::InvalidCheckpointOrder));
        };
        if previous_action_index.is_some_and(|previous| action_index <= previous) {
            return Err(validation(RigidWorldErrorKind::InvalidCheckpointOrder));
        }
        previous_action_index = Some(action_index);
        let phase = raw.phase.into_string();
        if phase.trim().is_empty() || phase.as_str() != actions[action_index].phase.as_ref() {
            return Err(validation(RigidWorldErrorKind::CheckpointPhaseMismatch));
        }
        let (body_count, fixture_count) = live_counts[action_index];
        if raw.counts.bodies != checked_u32(body_count)?
            || raw.counts.fixtures != checked_u32(fixture_count)?
            || raw.counts.manifold_points > raw.counts.contacts.saturating_mul(2)
            || (family == RigidWorldWitnessFamily::NonCollidingBodyFixtureLifecycle
                && (raw.counts.contacts != 0 || raw.counts.manifold_points != 0))
            || (family == RigidWorldWitnessFamily::SingleContactLifecycle
                && raw.counts.contacts > 1)
        {
            return Err(validation(RigidWorldErrorKind::ExpectedCountMismatch));
        }
        let transitions = raw
            .transitions
            .into_vec()
            .into_iter()
            .map(|transition| {
                if !witnesses.insert(transition.witness) {
                    return Err(validation(RigidWorldErrorKind::DuplicateWitness));
                }
                if transition.witness.requires_contact_identity()
                    != transition.maybe_contact.is_some()
                {
                    return Err(validation(RigidWorldErrorKind::InvalidContactIdentity));
                }
                if let Some(contact) = &transition.maybe_contact {
                    validate_contact_identity(contact, body_ids, fixture_owners)?;
                }
                Ok(RigidExpectedTransition {
                    witness: transition.witness,
                    maybe_contact: transition.maybe_contact,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        checkpoints.push(RigidExpectedCheckpoint {
            checkpoint_id: raw.checkpoint_id,
            after_action_id: raw.after_action_id,
            phase: phase.into_boxed_str(),
            counts: raw.counts,
            transitions: transitions.into_boxed_slice(),
        });
    }

    let required = family.required_witnesses();
    if required.iter().any(|witness| !witnesses.contains(witness)) {
        return Err(validation(RigidWorldErrorKind::MissingWitness));
    }
    if witnesses.iter().any(|witness| !required.contains(witness)) {
        return Err(validation(RigidWorldErrorKind::UnexpectedWitness));
    }
    Ok(checkpoints)
}

pub(super) fn action_live_counts(
    actions: &[RigidWorldActionRecord],
    fixture_owners: &HashMap<ScenarioId, ScenarioId>,
) -> Vec<(usize, usize)> {
    let mut bodies = HashSet::new();
    let mut fixtures = HashSet::new();
    let mut counts = Vec::with_capacity(actions.len());
    for action in actions {
        super::super::types::apply_lifecycle_action(
            action.action(),
            fixture_owners,
            &mut bodies,
            &mut fixtures,
        );
        counts.push((bodies.len(), fixtures.len()));
    }
    counts
}

pub(super) fn validate_contact_identity(
    contact: &RigidContactIdentity,
    _body_ids: &HashSet<ScenarioId>,
    fixture_owners: &HashMap<ScenarioId, ScenarioId>,
) -> Result<(), RigidWorldDecodeError> {
    if contact.fixture_a_id() == contact.fixture_b_id()
        || !fixture_owners.contains_key(contact.fixture_a_id())
        || !fixture_owners.contains_key(contact.fixture_b_id())
        || contact.child_a() != 0
        || contact.child_b() != 0
        || contact.occurrence() == 0
    {
        return Err(validation(RigidWorldErrorKind::InvalidContactIdentity));
    }
    Ok(())
}

pub(super) fn validate_source(raw: RawSource) -> Result<ScenarioSource, RigidWorldDecodeError> {
    match raw {
        RawSource::Named { name } => {
            let name = name.into_string();
            if name.trim().is_empty() {
                return Err(validation(RigidWorldErrorKind::InvalidSource));
            }
            Ok(ScenarioSource::Named {
                name: name.into_boxed_str(),
            })
        }
        RawSource::Seeded {
            generator_id,
            generator_version,
            seed,
        } => {
            let generator_id = generator_id.into_string();
            if generator_id.trim().is_empty() || generator_version == 0 {
                return Err(validation(RigidWorldErrorKind::InvalidSource));
            }
            Ok(ScenarioSource::Seeded {
                generator_id: generator_id.into_boxed_str(),
                generator_version,
                seed,
            })
        }
    }
}

pub(super) fn require_live(
    id: &ScenarioId,
    live: &HashSet<ScenarioId>,
    kind: RigidWorldErrorKind,
) -> Result<(), RigidWorldDecodeError> {
    if !live.contains(id) {
        return Err(validation(kind));
    }
    Ok(())
}

pub(super) fn checked_u32(value: usize) -> Result<u32, RigidWorldDecodeError> {
    u32::try_from(value).map_err(|_| validation(RigidWorldErrorKind::AggregateLimitExceeded))
}
