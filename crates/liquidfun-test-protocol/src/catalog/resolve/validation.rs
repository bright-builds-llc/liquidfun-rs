use super::super::model::{
    ActionSchedule, CATALOG_MAXIMUM_ACTIONS, CATALOG_MAXIMUM_CHECKPOINTS, CATALOG_MAXIMUM_ENTITIES,
    CatalogError, CatalogErrorKind, ResolvedEntity, ResolvedPayload, ScheduledAction,
    deterministic_action_id, deterministic_checkpoint_id, deterministic_entity_id,
};
use super::particles::{CatalogSemanticState, validate_group_operation, validate_particle_action};
use crate::{
    FloatBits, RigidJointMutation, RigidRayDirective, RigidWorldAction, ScenarioId,
    SemanticEntityKind, Vec2Bits,
};

pub(super) fn validate_payload(payload: &ResolvedPayload) -> Result<(), CatalogError> {
    if payload.actions.is_empty()
        || payload.actions.len() > CATALOG_MAXIMUM_ACTIONS
        || payload.entities.len() > CATALOG_MAXIMUM_ENTITIES
        || payload.checkpoints.len() > CATALOG_MAXIMUM_CHECKPOINTS
    {
        return Err(CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded));
    }
    for (ordinal, entity) in payload.entities.iter().enumerate() {
        let ordinal = u32::try_from(ordinal)
            .map_err(|_| CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded))?;
        if entity.semantic_id().ordinal() != ordinal {
            return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
        }
        let expected = deterministic_entity_id(entity.semantic_id().kind(), ordinal)?;
        if &expected != entity {
            return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
        }
    }
    let mut expected_setup_ordinal = 0_u32;
    let mut expected_logical_step = 1_u32;
    let mut logical_action_ordinals = Vec::new();
    let mut semantic_state = CatalogSemanticState::default();
    for (ordinal, action) in payload.actions.iter().enumerate() {
        let ordinal = u32::try_from(ordinal)
            .map_err(|_| CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded))?;
        if action.action_id() != &deterministic_action_id(ordinal)? {
            return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
        }
        match action.schedule() {
            ActionSchedule::Setup { ordinal } if expected_logical_step == 1 => {
                if ordinal != expected_setup_ordinal {
                    return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
                }
                expected_setup_ordinal = expected_setup_ordinal
                    .checked_add(1)
                    .ok_or_else(|| CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded))?;
            }
            ActionSchedule::LogicalStep {
                ordinal: logical_step,
            } => {
                if logical_step != expected_logical_step {
                    return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
                }
                logical_action_ordinals.push(ordinal);
                expected_logical_step = expected_logical_step
                    .checked_add(1)
                    .ok_or_else(|| CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded))?;
            }
            ActionSchedule::Setup { .. } => {
                return Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier));
            }
        }
        validate_action(
            action,
            payload.identity.settings,
            &payload.entities,
            &mut semantic_state,
        )?;
    }
    if payload.checkpoints.len() != logical_action_ordinals.len() {
        return Err(CatalogError::new(
            CatalogErrorKind::InvalidCheckpointReference,
        ));
    }
    for (index, checkpoint) in payload.checkpoints.iter().enumerate() {
        let logical_step = u32::try_from(index)
            .ok()
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| CatalogError::new(CatalogErrorKind::ResolvedLimitExceeded))?;
        if checkpoint.logical_step() != logical_step
            || checkpoint.after_action_id()
                != &deterministic_action_id(logical_action_ordinals[index])?
            || checkpoint.checkpoint_id() != &deterministic_checkpoint_id(logical_step)?
        {
            return Err(CatalogError::new(
                CatalogErrorKind::InvalidCheckpointReference,
            ));
        }
    }
    if expected_setup_ordinal == 0 || expected_logical_step == 1 {
        return Err(CatalogError::new(
            CatalogErrorKind::InvalidCheckpointReference,
        ));
    }
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "an explicit closed action whitelist keeps canonical replay fail-closed"
)]
fn validate_action(
    action: &ScheduledAction,
    settings: super::super::RunSettings,
    entities: &[ResolvedEntity],
    semantic_state: &mut CatalogSemanticState,
) -> Result<(), CatalogError> {
    if let RigidWorldAction::SetWorldGravity { gravity } = action.action() {
        if !gravity.x_bits.to_f32().is_finite() || !gravity.y_bits.to_f32().is_finite() {
            return Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings));
        }
        return Ok(());
    }

    match action.action() {
        RigidWorldAction::CreateBody { body_id }
        | RigidWorldAction::InspectBody { body_id }
        | RigidWorldAction::SetSleepingAllowed { body_id, .. }
        | RigidWorldAction::SetAwake { body_id, .. }
        | RigidWorldAction::SetBullet { body_id, .. }
        | RigidWorldAction::SetBodyType { body_id, .. }
        | RigidWorldAction::DestroyBody { body_id } => {
            require_entity(entities, body_id, SemanticEntityKind::Body)
        }
        RigidWorldAction::SetLinearVelocity { body_id, velocity } => {
            require_entity(entities, body_id, SemanticEntityKind::Body)?;
            require_finite_vec2(*velocity)
        }
        RigidWorldAction::ApplyForce {
            body_id,
            force,
            point,
            ..
        } => {
            require_entity(entities, body_id, SemanticEntityKind::Body)?;
            require_finite_vec2(*force)?;
            require_finite_vec2(*point)
        }
        RigidWorldAction::CreateFixture { fixture_id }
        | RigidWorldAction::InspectFixture { fixture_id }
        | RigidWorldAction::SetFixtureSensor { fixture_id, .. }
        | RigidWorldAction::SetFixtureFilter { fixture_id, .. }
        | RigidWorldAction::DestroyFixture { fixture_id } => {
            require_entity(entities, fixture_id, SemanticEntityKind::Fixture)
        }
        RigidWorldAction::SetFixtureMaterial {
            fixture_id,
            friction_bits,
            restitution_bits,
        } => {
            require_entity(entities, fixture_id, SemanticEntityKind::Fixture)?;
            require_nonnegative_finite(*friction_bits)?;
            require_nonnegative_finite(*restitution_bits)
        }
        RigidWorldAction::ConfiguredStep {
            timestep_bits,
            velocity_iterations,
            position_iterations,
            continuous_work_budget,
        } if *timestep_bits != settings.timestep_bits()
            || *velocity_iterations != settings.velocity_iterations()
            || *position_iterations != settings.position_iterations()
            || *continuous_work_budget != 1 =>
        {
            Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings))
        }
        RigidWorldAction::SetContinuousPhysics { .. } | RigidWorldAction::ConfiguredStep { .. } => {
            Ok(())
        }
        RigidWorldAction::QueryAabb {
            aabb,
            directive_rules,
        } => {
            require_finite_vec2(aabb.lower)?;
            require_finite_vec2(aabb.upper)?;
            if aabb.lower.x_bits.to_f32() > aabb.upper.x_bits.to_f32()
                || aabb.lower.y_bits.to_f32() > aabb.upper.y_bits.to_f32()
            {
                return Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings));
            }
            for rule in directive_rules {
                require_entity(
                    entities,
                    &rule.target.fixture_id,
                    SemanticEntityKind::Fixture,
                )?;
            }
            Ok(())
        }
        RigidWorldAction::RayCast {
            start,
            end,
            directive_rules,
        } => {
            require_finite_vec2(*start)?;
            require_finite_vec2(*end)?;
            if start == end {
                return Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings));
            }
            for rule in directive_rules {
                require_entity(
                    entities,
                    &rule.target.fixture_id,
                    SemanticEntityKind::Fixture,
                )?;
                if let RigidRayDirective::Clip { fraction_bits } = rule.directive {
                    let fraction = fraction_bits.to_f32();
                    if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
                        return Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings));
                    }
                }
            }
            Ok(())
        }
        RigidWorldAction::CreateJoint { joint_id }
        | RigidWorldAction::InspectJoint { joint_id } => {
            require_entity(entities, joint_id, SemanticEntityKind::Joint)
        }
        RigidWorldAction::MutateJoint { joint_id, mutation } => {
            require_entity(entities, joint_id, SemanticEntityKind::Joint)?;
            validate_joint_mutation(*mutation)
        }
        RigidWorldAction::SetRopeAngle {
            rope_id,
            angle_bits,
        } => {
            require_entity(entities, rope_id, SemanticEntityKind::Rope)?;
            require_finite(*angle_bits)
        }
        RigidWorldAction::StepRope {
            rope_id,
            timestep_bits,
            iterations,
        } if *timestep_bits != settings.timestep_bits()
            || *iterations != settings.particle_iterations() =>
        {
            Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings))
        }
        RigidWorldAction::CreateRope { rope_id }
        | RigidWorldAction::InspectRope { rope_id }
        | RigidWorldAction::StepRope { rope_id, .. } => {
            require_entity(entities, rope_id, SemanticEntityKind::Rope)
        }
        RigidWorldAction::SetContactFilterDirective { target, .. } => {
            require_entity(entities, &target.fixture_a_id, SemanticEntityKind::Fixture)?;
            require_entity(entities, &target.fixture_b_id, SemanticEntityKind::Fixture)
        }
        RigidWorldAction::SetPreSolveDirective { target, directive } => {
            require_entity(entities, &target.fixture_a_id, SemanticEntityKind::Fixture)?;
            require_entity(entities, &target.fixture_b_id, SemanticEntityKind::Fixture)?;
            for maybe_value in [
                directive.maybe_friction_bits,
                directive.maybe_restitution_bits,
                directive.maybe_tangent_speed_bits,
            ]
            .into_iter()
            .flatten()
            {
                require_finite(maybe_value)?;
            }
            if directive
                .maybe_friction_bits
                .is_some_and(|value| value.to_f32() < 0.0)
                || directive
                    .maybe_restitution_bits
                    .is_some_and(|value| value.to_f32() < 0.0)
            {
                return Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings));
            }
            Ok(())
        }
        RigidWorldAction::Particle { action } => {
            validate_particle_action(action, entities, semantic_state)
        }
        RigidWorldAction::ParticleGroup { operation } => {
            validate_group_operation(operation, settings, entities, semantic_state)
        }
        _ => Err(CatalogError::new(CatalogErrorKind::CanonicalEncoding)),
    }
}

pub(super) fn require_entity(
    entities: &[ResolvedEntity],
    scenario_id: &ScenarioId,
    kind: SemanticEntityKind,
) -> Result<(), CatalogError> {
    if entities
        .iter()
        .any(|entity| entity.scenario_id() == scenario_id && entity.semantic_id().kind() == kind)
    {
        return Ok(());
    }
    Err(CatalogError::new(CatalogErrorKind::InvalidIdentifier))
}

fn require_finite(value: FloatBits) -> Result<(), CatalogError> {
    if value.to_f32().is_finite() {
        return Ok(());
    }
    Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings))
}

fn require_nonnegative_finite(value: FloatBits) -> Result<(), CatalogError> {
    require_finite(value)?;
    if value.to_f32() >= 0.0 {
        return Ok(());
    }
    Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings))
}

pub(super) fn require_finite_vec2(value: Vec2Bits) -> Result<(), CatalogError> {
    require_finite(value.x_bits)?;
    require_finite(value.y_bits)
}

fn validate_joint_mutation(mutation: RigidJointMutation) -> Result<(), CatalogError> {
    match mutation {
        RigidJointMutation::MotorEnabled { .. } => Ok(()),
        RigidJointMutation::Limits {
            lower_bits,
            upper_bits,
        } => {
            require_finite(lower_bits)?;
            require_finite(upper_bits)?;
            if lower_bits.to_f32() <= upper_bits.to_f32() {
                return Ok(());
            }
            Err(CatalogError::new(CatalogErrorKind::InvalidRunSettings))
        }
        RigidJointMutation::Length { length_bits }
        | RigidJointMutation::RopeMaxLength {
            max_length_bits: length_bits,
        } => require_nonnegative_finite(length_bits),
        RigidJointMutation::MouseTarget { target }
        | RigidJointMutation::LinearOffset { offset: target } => require_finite_vec2(target),
        RigidJointMutation::GearRatio { ratio_bits } => require_finite(ratio_bits),
        RigidJointMutation::MaxMotorTorque { torque_bits } => {
            require_nonnegative_finite(torque_bits)
        }
        RigidJointMutation::Frequency { frequency_bits } => {
            require_nonnegative_finite(frequency_bits)
        }
        RigidJointMutation::MaxForce { force_bits } => require_nonnegative_finite(force_bits),
        RigidJointMutation::CorrectionFactor { factor_bits } => {
            require_nonnegative_finite(factor_bits)
        }
        _ => Err(CatalogError::new(CatalogErrorKind::CanonicalEncoding)),
    }
}
