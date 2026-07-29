use super::{
    HashMap, HashSet, PHASE9_MAXIMUM_IDENTITIES, PHASE9_MAXIMUM_PARTICLES, Phase9ActionState,
    Phase9ParticleAction, Phase9ParticleDeclaration, Phase9ParticleSystemDeclaration,
    Phase10ActionState, Phase10Operation, RIGID_WORLD_MAXIMUM_CONTINUOUS_WORK,
    RIGID_WORLD_MAXIMUM_ITERATIONS, RIGID_WORLD_POSITION_ITERATIONS, RIGID_WORLD_TIMESTEP_BITS,
    RIGID_WORLD_VELOCITY_ITERATIONS, RigidAabbBits, RigidFixtureShape, RigidJointDefinition,
    RigidWorldAction, RigidWorldDecodeError, RigidWorldErrorKind, ScenarioId,
    joint_mutation_changes_definition, remove_joint_cascade, require_live, validate_aabb,
    validate_contact_directive_target, validate_custom_mass, validate_finite,
    validate_joint_mutation, validate_nonnegative, validate_positive, validate_pre_solve_directive,
    validate_query_rules, validate_ray_geometry, validate_ray_rules, validate_transform,
    validate_vec2, validation,
};

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "one explicit lifecycle transition function keeps the closed action registry auditable"
)]
pub(super) fn validate_action(
    action: &RigidWorldAction,
    body_ids: &HashSet<ScenarioId>,
    fixture_owners: &HashMap<ScenarioId, ScenarioId>,
    fixture_shapes: &HashMap<ScenarioId, RigidFixtureShape>,
    live_bodies: &mut HashSet<ScenarioId>,
    live_fixtures: &mut HashSet<ScenarioId>,
    created_bodies: &mut HashSet<ScenarioId>,
    created_fixtures: &mut HashSet<ScenarioId>,
    joint_ids: &HashSet<ScenarioId>,
    joint_definitions: &HashMap<ScenarioId, RigidJointDefinition>,
    joint_bodies: &HashMap<ScenarioId, [ScenarioId; 2]>,
    gear_dependents: &HashMap<ScenarioId, Vec<ScenarioId>>,
    rope_ids: &HashSet<ScenarioId>,
    live_joints: &mut HashSet<ScenarioId>,
    live_ropes: &mut HashSet<ScenarioId>,
    created_joints: &mut HashSet<ScenarioId>,
    created_ropes: &mut HashSet<ScenarioId>,
    particle_system_ids: &HashSet<ScenarioId>,
    particle_owners: &HashMap<ScenarioId, ScenarioId>,
    phase9_state: &mut Phase9ActionState,
    reserved_semantic_ids: &HashSet<ScenarioId>,
    phase10_state: &mut Phase10ActionState,
) -> Result<(), RigidWorldDecodeError> {
    match action {
        RigidWorldAction::Particle { action } => {
            if let Phase9ParticleAction::DestroySystem { system_id } = action
                && phase10_state.has_live_group_in_system(system_id)
            {
                return Err(validation(RigidWorldErrorKind::InvalidParticleGroupAction));
            }
            validate_phase9_action(action, particle_system_ids, particle_owners, phase9_state)?;
        }
        RigidWorldAction::ParticleGroup { operation } => {
            validate_phase10_action(
                operation,
                particle_system_ids,
                &phase9_state.live_systems,
                reserved_semantic_ids,
                phase10_state,
            )?;
        }
        RigidWorldAction::CreateBody { body_id } => {
            if !body_ids.contains(body_id)
                || !created_bodies.insert(body_id.clone())
                || !live_bodies.insert(body_id.clone())
            {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        RigidWorldAction::CreateFixture { fixture_id } => {
            let Some(owner) = fixture_owners.get(fixture_id) else {
                return Err(validation(RigidWorldErrorKind::UnknownFixture));
            };
            if !live_bodies.contains(owner)
                || !created_fixtures.insert(fixture_id.clone())
                || !live_fixtures.insert(fixture_id.clone())
            {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        RigidWorldAction::InspectBody { body_id }
        | RigidWorldAction::ResetMassData { body_id }
        | RigidWorldAction::SetBodyType { body_id, .. }
        | RigidWorldAction::SetBodyActive { body_id, .. }
        | RigidWorldAction::SetFixedRotation { body_id, .. }
        | RigidWorldAction::SetSleepingAllowed { body_id, .. }
        | RigidWorldAction::SetAwake { body_id, .. }
        | RigidWorldAction::SetBullet { body_id, .. } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
        }
        RigidWorldAction::SetLinearVelocity { body_id, velocity } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
            validate_vec2(*velocity)?;
        }
        RigidWorldAction::SetAngularVelocity {
            body_id,
            angular_velocity_bits,
        }
        | RigidWorldAction::ApplyTorque {
            body_id,
            torque_bits: angular_velocity_bits,
            ..
        }
        | RigidWorldAction::ApplyAngularImpulse {
            body_id,
            impulse_bits: angular_velocity_bits,
            ..
        }
        | RigidWorldAction::SetGravityScale {
            body_id,
            gravity_scale_bits: angular_velocity_bits,
        } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
            validate_finite(
                *angular_velocity_bits,
                RigidWorldErrorKind::InvalidBodyControl,
            )?;
        }
        RigidWorldAction::ApplyForce {
            body_id,
            force,
            point,
            ..
        }
        | RigidWorldAction::ApplyLinearImpulse {
            body_id,
            impulse: force,
            point,
            ..
        } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
            validate_vec2(*force)?;
            validate_vec2(*point)?;
        }
        RigidWorldAction::SetBodyDamping {
            body_id,
            linear_damping_bits,
            angular_damping_bits,
        } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
            validate_nonnegative(*linear_damping_bits)?;
            validate_nonnegative(*angular_damping_bits)?;
        }
        RigidWorldAction::SetBodyTransform { body_id, transform } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
            validate_transform(*transform)?;
        }
        RigidWorldAction::SetCustomMassData {
            body_id,
            mass_bits,
            center,
            inertia_bits,
        } => {
            require_live(body_id, live_bodies, RigidWorldErrorKind::UnknownBody)?;
            validate_custom_mass(*mass_bits, *center, *inertia_bits)?;
        }
        RigidWorldAction::InspectFixture { fixture_id }
        | RigidWorldAction::SetFixtureSensor { fixture_id, .. }
        | RigidWorldAction::SetFixtureFilter { fixture_id, .. } => {
            require_live(
                fixture_id,
                live_fixtures,
                RigidWorldErrorKind::UnknownFixture,
            )?;
        }
        RigidWorldAction::SetFixtureMaterial {
            fixture_id,
            friction_bits,
            restitution_bits,
        } => {
            require_live(
                fixture_id,
                live_fixtures,
                RigidWorldErrorKind::UnknownFixture,
            )?;
            validate_nonnegative(*friction_bits)?;
            validate_nonnegative(*restitution_bits)?;
        }
        RigidWorldAction::SetFixtureDensity {
            fixture_id,
            density_bits,
        } => {
            require_live(
                fixture_id,
                live_fixtures,
                RigidWorldErrorKind::UnknownFixture,
            )?;
            validate_nonnegative(*density_bits)?;
        }
        RigidWorldAction::Step {
            timestep_bits,
            velocity_iterations,
            position_iterations,
        } => {
            if timestep_bits.bits() != RIGID_WORLD_TIMESTEP_BITS
                || *velocity_iterations != RIGID_WORLD_VELOCITY_ITERATIONS
                || *position_iterations != RIGID_WORLD_POSITION_ITERATIONS
            {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        RigidWorldAction::SetWorldGravity { gravity }
        | RigidWorldAction::ShiftOrigin { shift: gravity } => {
            validate_vec2(*gravity)?;
        }
        RigidWorldAction::SetAutomaticForceClearing { .. }
        | RigidWorldAction::SetWarmStarting { .. }
        | RigidWorldAction::SetContinuousPhysics { .. }
        | RigidWorldAction::SetSubStepping { .. }
        | RigidWorldAction::ClearForces
        | RigidWorldAction::RequestReconstruction
        | RigidWorldAction::RequestDiagnostics => {}
        RigidWorldAction::ConfiguredStep {
            timestep_bits,
            velocity_iterations,
            position_iterations,
            continuous_work_budget,
        } => {
            let timestep = timestep_bits.to_f32();
            if !timestep.is_finite()
                || timestep <= 0.0
                || !(1..=RIGID_WORLD_MAXIMUM_ITERATIONS).contains(velocity_iterations)
                || !(1..=RIGID_WORLD_MAXIMUM_ITERATIONS).contains(position_iterations)
                || !(1..=RIGID_WORLD_MAXIMUM_CONTINUOUS_WORK).contains(continuous_work_budget)
            {
                return Err(validation(RigidWorldErrorKind::InvalidStepConfiguration));
            }
        }
        RigidWorldAction::QueryAabb {
            aabb,
            directive_rules,
        } => {
            validate_aabb(*aabb)?;
            validate_query_rules(directive_rules, live_fixtures, fixture_shapes)?;
        }
        RigidWorldAction::RayCast {
            start,
            end,
            directive_rules,
        } => {
            validate_vec2(*start)?;
            validate_vec2(*end)?;
            validate_ray_geometry(*start, *end)?;
            validate_ray_rules(directive_rules, live_fixtures, fixture_shapes)?;
        }
        RigidWorldAction::CreateJoint { joint_id } => {
            if !joint_ids.contains(joint_id)
                || !created_joints.insert(joint_id.clone())
                || !live_joints.insert(joint_id.clone())
            {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        RigidWorldAction::InspectJoint { joint_id } => {
            require_live(joint_id, live_joints, RigidWorldErrorKind::UnknownJoint)?;
        }
        RigidWorldAction::MutateJoint { joint_id, mutation } => {
            require_live(joint_id, live_joints, RigidWorldErrorKind::UnknownJoint)?;
            let Some(joint_definition) = joint_definitions.get(joint_id) else {
                return Err(validation(RigidWorldErrorKind::UnknownJoint));
            };
            validate_joint_mutation(joint_definition.joint_kind(), *mutation)?;
            if !joint_mutation_changes_definition(joint_definition, *mutation) {
                return Err(validation(RigidWorldErrorKind::InvalidJointDefinition));
            }
        }
        RigidWorldAction::DestroyJoint { joint_id } => {
            if !live_joints.contains(joint_id) {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
            remove_joint_cascade(joint_id, live_joints, gear_dependents);
        }
        RigidWorldAction::CreateRope { rope_id } => {
            if !rope_ids.contains(rope_id)
                || !created_ropes.insert(rope_id.clone())
                || !live_ropes.insert(rope_id.clone())
            {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        RigidWorldAction::SetRopeAngle {
            rope_id,
            angle_bits,
        } => {
            require_live(rope_id, live_ropes, RigidWorldErrorKind::UnknownRope)?;
            validate_finite(*angle_bits, RigidWorldErrorKind::InvalidRopeDefinition)?;
        }
        RigidWorldAction::StepRope {
            rope_id,
            timestep_bits,
            iterations,
        } => {
            require_live(rope_id, live_ropes, RigidWorldErrorKind::UnknownRope)?;
            let timestep = timestep_bits.to_f32();
            if !timestep.is_finite()
                || timestep <= 0.0
                || !(1..=RIGID_WORLD_MAXIMUM_ITERATIONS).contains(iterations)
            {
                return Err(validation(RigidWorldErrorKind::InvalidRopeDefinition));
            }
        }
        RigidWorldAction::InspectRope { rope_id } => {
            require_live(rope_id, live_ropes, RigidWorldErrorKind::UnknownRope)?;
        }
        RigidWorldAction::DestroyRope { rope_id } => {
            if !live_ropes.remove(rope_id) {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        RigidWorldAction::SetContactFilterDirective { target, .. }
        | RigidWorldAction::SetPreSolveDirective { target, .. } => {
            validate_contact_directive_target(target, live_fixtures)?;
            if let RigidWorldAction::SetPreSolveDirective { directive, .. } = action {
                validate_pre_solve_directive(*directive)?;
            }
        }
        RigidWorldAction::DestroyFixture { fixture_id } => {
            if !live_fixtures.remove(fixture_id) {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
        }
        RigidWorldAction::DestroyBody { body_id } => {
            if !live_bodies.remove(body_id) {
                return Err(validation(RigidWorldErrorKind::InvalidActionOrder));
            }
            live_fixtures.retain(|fixture_id| fixture_owners.get(fixture_id) != Some(body_id));
            let attached = live_joints
                .iter()
                .filter(|joint_id| {
                    joint_bodies
                        .get(*joint_id)
                        .is_some_and(|endpoints| endpoints.contains(body_id))
                })
                .cloned()
                .collect::<Vec<_>>();
            for joint_id in attached {
                remove_joint_cascade(&joint_id, live_joints, gear_dependents);
            }
        }
    }
    Ok(())
}

pub(super) fn validate_phase10_action(
    operation: &Phase10Operation,
    particle_system_ids: &HashSet<ScenarioId>,
    live_system_ids: &HashSet<ScenarioId>,
    particle_ids: &HashSet<ScenarioId>,
    state: &mut Phase10ActionState,
) -> Result<(), RigidWorldDecodeError> {
    state
        .apply(
            operation,
            particle_system_ids,
            live_system_ids,
            particle_ids,
        )
        .map_err(|_| validation(RigidWorldErrorKind::InvalidParticleGroupAction))
}

pub(super) fn validate_phase9_declarations(
    systems: &[Phase9ParticleSystemDeclaration],
    particles: &[Phase9ParticleDeclaration],
) -> Result<(), RigidWorldDecodeError> {
    let mut system_ids = HashSet::with_capacity(systems.len());
    for system in systems {
        let capacity = system.buffer_mode.capacity();
        if !system_ids.insert(system.system_id.clone())
            || capacity == 0
            || capacity > PHASE9_MAXIMUM_PARTICLES
            || system
                .maximum_count
                .is_some_and(|maximum| maximum == 0 || maximum > PHASE9_MAXIMUM_PARTICLES)
        {
            return Err(validation(RigidWorldErrorKind::InvalidParticleDefinition));
        }
        validate_positive(system.density_bits)?;
        validate_finite(
            system.gravity_scale_bits,
            RigidWorldErrorKind::InvalidParticleDefinition,
        )?;
        validate_positive(system.radius_bits)?;
        validate_nonnegative(system.damping_bits)?;
        validate_positive(system.lifetime_granularity_bits)?;
    }

    let mut particle_ids = HashSet::with_capacity(particles.len());
    for particle in particles {
        if !particle_ids.insert(particle.particle_id.clone())
            || !system_ids.contains(&particle.system_id)
        {
            return Err(validation(RigidWorldErrorKind::InvalidParticleDefinition));
        }
        validate_vec2(particle.position)?;
        validate_vec2(particle.velocity)?;
        validate_finite(
            particle.lifetime_bits,
            RigidWorldErrorKind::InvalidParticleDefinition,
        )?;
    }
    Ok(())
}

pub(super) fn validate_phase9_action(
    action: &Phase9ParticleAction,
    system_ids: &HashSet<ScenarioId>,
    particle_owners: &HashMap<ScenarioId, ScenarioId>,
    state: &mut Phase9ActionState,
) -> Result<(), RigidWorldDecodeError> {
    validate_phase9_action_shape(action)?;
    match action {
        Phase9ParticleAction::CreateSystem { system_id } => {
            if !system_ids.contains(system_id)
                || !state.created_systems.insert(system_id.clone())
                || !state.live_systems.insert(system_id.clone())
            {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            }
        }
        Phase9ParticleAction::DestroySystem { system_id } => {
            if !state.live_systems.remove(system_id) {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            }
            state
                .live_particles
                .retain(|particle_id| particle_owners.get(particle_id) != Some(system_id));
            state
                .pending_particles
                .retain(|particle_id| particle_owners.get(particle_id) != Some(system_id));
        }
        Phase9ParticleAction::CreateParticle { particle_id } => {
            let Some(owner) = particle_owners.get(particle_id) else {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            };
            if !state.live_systems.contains(owner)
                || !state.created_particles.insert(particle_id.clone())
                || !state.live_particles.insert(particle_id.clone())
            {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            }
        }
        Phase9ParticleAction::InspectSystem { system_id }
        | Phase9ParticleAction::InspectParticleContact { system_id, .. }
        | Phase9ParticleAction::InspectBodyContact { system_id, .. }
        | Phase9ParticleAction::SetPaused { system_id, .. }
        | Phase9ParticleAction::Compact { system_id }
        | Phase9ParticleAction::RequestStatistics { system_id } => {
            require_live_phase9_system(system_id, state)?;
            if matches!(action, Phase9ParticleAction::Compact { .. }) {
                state
                    .pending_particles
                    .retain(|particle_id| particle_owners.get(particle_id) != Some(system_id));
            }
        }
        Phase9ParticleAction::InspectParticle { particle_id }
        | Phase9ParticleAction::SetPosition { particle_id, .. }
        | Phase9ParticleAction::SetVelocity { particle_id, .. } => {
            require_live_phase9_particle(particle_id, particle_owners, state)?;
        }
        Phase9ParticleAction::MarkForDestruction { particle_id } => {
            require_live_phase9_particle(particle_id, particle_owners, state)?;
            state.live_particles.remove(particle_id);
            state.pending_particles.insert(particle_id.clone());
        }
        Phase9ParticleAction::ApplyForce { particle_ids, .. }
        | Phase9ParticleAction::ApplyImpulse { particle_ids, .. } => {
            let mut maybe_owner: Option<&ScenarioId> = None;
            for particle_id in particle_ids {
                let owner = require_live_phase9_particle(particle_id, particle_owners, state)?;
                if maybe_owner.is_some_and(|expected| expected != owner) {
                    return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
                }
                maybe_owner = Some(owner);
            }
        }
        Phase9ParticleAction::QueryAabb { system_id, .. }
        | Phase9ParticleAction::RayCast { system_id, .. } => {
            if let Some(system_id) = system_id {
                require_live_phase9_system(system_id, state)?;
            }
        }
        Phase9ParticleAction::InspectOccurrence { .. } => {}
    }
    Ok(())
}

pub(super) fn validate_phase9_action_shape(
    action: &Phase9ParticleAction,
) -> Result<(), RigidWorldDecodeError> {
    match action {
        Phase9ParticleAction::SetPosition { position, .. }
        | Phase9ParticleAction::SetVelocity {
            velocity: position, ..
        } => validate_vec2(*position)?,
        Phase9ParticleAction::ApplyForce {
            particle_ids,
            force,
        }
        | Phase9ParticleAction::ApplyImpulse {
            particle_ids,
            impulse: force,
        } => {
            if particle_ids.is_empty() || particle_ids.len() > PHASE9_MAXIMUM_IDENTITIES {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            }
            let mut ids = HashSet::with_capacity(particle_ids.len());
            if particle_ids.iter().any(|id| !ids.insert(id)) {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            }
            validate_vec2(*force)?;
        }
        Phase9ParticleAction::QueryAabb { lower, upper, .. } => {
            validate_aabb(RigidAabbBits {
                lower: *lower,
                upper: *upper,
            })?;
        }
        Phase9ParticleAction::RayCast { start, end, .. } => {
            validate_ray_geometry(*start, *end)?;
        }
        Phase9ParticleAction::InspectParticleContact { contact_index, .. }
        | Phase9ParticleAction::InspectBodyContact { contact_index, .. } => {
            if *contact_index >= PHASE9_MAXIMUM_IDENTITIES {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            }
        }
        Phase9ParticleAction::InspectOccurrence { occurrence_index } => {
            if *occurrence_index >= PHASE9_MAXIMUM_IDENTITIES {
                return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
            }
        }
        Phase9ParticleAction::CreateSystem { .. }
        | Phase9ParticleAction::DestroySystem { .. }
        | Phase9ParticleAction::CreateParticle { .. }
        | Phase9ParticleAction::InspectSystem { .. }
        | Phase9ParticleAction::InspectParticle { .. }
        | Phase9ParticleAction::SetPaused { .. }
        | Phase9ParticleAction::MarkForDestruction { .. }
        | Phase9ParticleAction::Compact { .. }
        | Phase9ParticleAction::RequestStatistics { .. } => {}
    }
    Ok(())
}

pub(super) fn require_live_phase9_system(
    system_id: &ScenarioId,
    state: &Phase9ActionState,
) -> Result<(), RigidWorldDecodeError> {
    if !state.live_systems.contains(system_id) {
        return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
    }
    Ok(())
}

pub(super) fn require_live_phase9_particle<'a>(
    particle_id: &ScenarioId,
    particle_owners: &'a HashMap<ScenarioId, ScenarioId>,
    state: &Phase9ActionState,
) -> Result<&'a ScenarioId, RigidWorldDecodeError> {
    let Some(owner) = particle_owners.get(particle_id) else {
        return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
    };
    if !state.live_systems.contains(owner) || !state.live_particles.contains(particle_id) {
        return Err(validation(RigidWorldErrorKind::InvalidParticleAction));
    }
    Ok(owner)
}
