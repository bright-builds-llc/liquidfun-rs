//! Rigid-body, fixture, world, query, and callback action dispatch.

use liquidfun::collision::FilterData;
use liquidfun::{BodyMassData, BodyType, WakePolicy};
use liquidfun_test_protocol::{ResolvedScenario, RigidBodyKind, RigidWakePolicy, RigidWorldAction};

use crate::SessionBackendError;

use super::{NativeSession, action_failure, resource_failure, vec2};

impl NativeSession {
    #[allow(
        clippy::too_many_lines,
        reason = "the closed rigid action vocabulary is clearest as one exhaustive dispatcher"
    )]
    pub(super) fn execute_rigid(
        &mut self,
        resolved: &ResolvedScenario,
        action: &RigidWorldAction,
    ) -> Result<(), SessionBackendError> {
        if is_object_action(action) {
            return self.execute_object(action);
        }
        match action {
            RigidWorldAction::CreateBody { body_id } => self.create_body(body_id),
            RigidWorldAction::CreateFixture { fixture_id } => self.create_fixture(fixture_id),
            RigidWorldAction::InspectBody { body_id } => self
                .world
                .body_snapshot(self.body(body_id)?)
                .map(|_snapshot| ())
                .map_err(|_error| action_failure()),
            RigidWorldAction::InspectFixture { fixture_id } => self
                .world
                .fixture_snapshot(self.fixture(fixture_id)?)
                .map(|_snapshot| ())
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetBodyTransform { body_id, transform } => self
                .world
                .set_body_transform(
                    self.body(body_id)?,
                    vec2(transform.position),
                    transform.angle_bits.to_f32(),
                )
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetBodyType { body_id, body_kind } => self
                .world
                .set_body_type(self.body(body_id)?, body_type(*body_kind))
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetBodyActive { body_id, active } => self
                .world
                .set_body_active(self.body(body_id)?, *active)
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetLinearVelocity { body_id, velocity } => self
                .world
                .set_body_linear_velocity(self.body(body_id)?, vec2(*velocity))
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetAngularVelocity {
                body_id,
                angular_velocity_bits,
            } => self
                .world
                .set_body_angular_velocity(self.body(body_id)?, angular_velocity_bits.to_f32())
                .map_err(|_error| action_failure()),
            RigidWorldAction::ApplyForce {
                body_id,
                force,
                point,
                wake_policy,
            } => self
                .world
                .apply_body_force(
                    self.body(body_id)?,
                    vec2(*force),
                    vec2(*point),
                    native_wake(*wake_policy),
                )
                .map_err(|_error| action_failure()),
            RigidWorldAction::ApplyTorque {
                body_id,
                torque_bits,
                wake_policy,
            } => self
                .world
                .apply_body_torque(
                    self.body(body_id)?,
                    torque_bits.to_f32(),
                    native_wake(*wake_policy),
                )
                .map_err(|_error| action_failure()),
            RigidWorldAction::ApplyLinearImpulse {
                body_id,
                impulse,
                point,
                wake_policy,
            } => self
                .world
                .apply_body_linear_impulse(
                    self.body(body_id)?,
                    vec2(*impulse),
                    vec2(*point),
                    native_wake(*wake_policy),
                )
                .map_err(|_error| action_failure()),
            RigidWorldAction::ApplyAngularImpulse {
                body_id,
                impulse_bits,
                wake_policy,
            } => self
                .world
                .apply_body_angular_impulse(
                    self.body(body_id)?,
                    impulse_bits.to_f32(),
                    native_wake(*wake_policy),
                )
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetBodyDamping {
                body_id,
                linear_damping_bits,
                angular_damping_bits,
            } => {
                let body = self.body(body_id)?;
                self.world
                    .set_body_linear_damping(body, linear_damping_bits.to_f32())
                    .and_then(|()| {
                        self.world
                            .set_body_angular_damping(body, angular_damping_bits.to_f32())
                    })
                    .map_err(|_error| action_failure())
            }
            RigidWorldAction::SetGravityScale {
                body_id,
                gravity_scale_bits,
            } => self
                .world
                .set_body_gravity_scale(self.body(body_id)?, gravity_scale_bits.to_f32())
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetFixedRotation {
                body_id,
                fixed_rotation,
            } => self
                .world
                .set_body_fixed_rotation(self.body(body_id)?, *fixed_rotation)
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetSleepingAllowed {
                body_id,
                sleeping_allowed,
            } => self
                .world
                .set_body_sleeping_allowed(self.body(body_id)?, *sleeping_allowed)
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetAwake { body_id, awake } => self
                .world
                .set_body_awake(self.body(body_id)?, *awake)
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetBullet { body_id, bullet } => self
                .world
                .set_body_bullet(self.body(body_id)?, *bullet)
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetFixtureSensor { fixture_id, sensor } => self
                .world
                .set_fixture_sensor(self.fixture(fixture_id)?, *sensor)
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetFixtureMaterial {
                fixture_id,
                friction_bits,
                restitution_bits,
            } => {
                let fixture = self.fixture(fixture_id)?;
                self.world
                    .set_fixture_friction(fixture, friction_bits.to_f32())
                    .and_then(|()| {
                        self.world
                            .set_fixture_restitution(fixture, restitution_bits.to_f32())
                    })
                    .map_err(|_error| action_failure())
            }
            RigidWorldAction::SetFixtureFilter { fixture_id, filter } => self
                .world
                .set_fixture_filter(
                    self.fixture(fixture_id)?,
                    FilterData::new(
                        filter.category_bits(),
                        filter.mask_bits(),
                        filter.group_index(),
                    ),
                )
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetFixtureDensity {
                fixture_id,
                density_bits,
            } => self
                .world
                .set_fixture_density(self.fixture(fixture_id)?, density_bits.to_f32())
                .map_err(|_error| action_failure()),
            RigidWorldAction::ResetMassData { body_id } => self
                .world
                .reset_body_mass_data(self.body(body_id)?)
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetCustomMassData {
                body_id,
                mass_bits,
                center,
                inertia_bits,
            } => {
                let data =
                    BodyMassData::new(mass_bits.to_f32(), vec2(*center), inertia_bits.to_f32())
                        .map_err(|_error| action_failure())?;
                self.world
                    .set_body_mass_data(self.body(body_id)?, data)
                    .map_err(|_error| action_failure())
            }
            RigidWorldAction::Step {
                timestep_bits,
                velocity_iterations,
                position_iterations,
            }
            | RigidWorldAction::ConfiguredStep {
                timestep_bits,
                velocity_iterations,
                position_iterations,
                ..
            } => self.step(
                timestep_bits.to_f32(),
                *velocity_iterations,
                *position_iterations,
                resolved.identity().settings().particle_iterations(),
            ),
            RigidWorldAction::SetWorldGravity { gravity } => self
                .world
                .set_gravity(vec2(*gravity))
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetAutomaticForceClearing { enabled } => self
                .world
                .set_automatic_force_clearing_enabled(*enabled)
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetWarmStarting { enabled } => self
                .world
                .set_warm_starting_enabled(*enabled)
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetContinuousPhysics { enabled } => self
                .world
                .set_continuous_physics_enabled(*enabled)
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetSubStepping { enabled } => self
                .world
                .set_sub_stepping_enabled(*enabled)
                .map_err(|_error| action_failure()),
            RigidWorldAction::ClearForces => {
                self.world.clear_forces().map_err(|_error| action_failure())
            }
            RigidWorldAction::QueryAabb {
                directive_rules, ..
            } => {
                for rule in directive_rules {
                    self.fixture(&rule.target.fixture_id)?;
                }
                Ok(())
            }
            RigidWorldAction::RayCast {
                directive_rules, ..
            } => {
                for rule in directive_rules {
                    self.fixture(&rule.target.fixture_id)?;
                }
                Ok(())
            }
            RigidWorldAction::ShiftOrigin { shift } => self
                .world
                .shift_origin(vec2(*shift))
                .map_err(|_error| action_failure()),
            RigidWorldAction::SetContactFilterDirective { target, .. }
            | RigidWorldAction::SetPreSolveDirective { target, .. } => {
                self.fixture(&target.fixture_a_id)?;
                self.fixture(&target.fixture_b_id)?;
                Ok(())
            }
            RigidWorldAction::RequestReconstruction | RigidWorldAction::RequestDiagnostics => self
                .world
                .world_observation(liquidfun::WorldObservationLimits::reviewed())
                .map(|_observation| ())
                .map_err(|_error| resource_failure()),
            RigidWorldAction::DestroyFixture { fixture_id } => self.destroy_fixture(fixture_id),
            RigidWorldAction::DestroyBody { body_id } => self.destroy_body(body_id),
            RigidWorldAction::Particle { .. } | RigidWorldAction::ParticleGroup { .. } => {
                Err(action_failure())
            }
            _ => Err(action_failure()),
        }
    }
}

const fn body_type(value: RigidBodyKind) -> BodyType {
    match value {
        RigidBodyKind::Static => BodyType::Static,
        RigidBodyKind::Kinematic => BodyType::Kinematic,
        RigidBodyKind::Dynamic => BodyType::Dynamic,
    }
}

const fn native_wake(value: RigidWakePolicy) -> WakePolicy {
    match value {
        RigidWakePolicy::Wake => WakePolicy::Wake,
        RigidWakePolicy::PreserveSleep => WakePolicy::PreserveSleep,
    }
}

const fn is_object_action(action: &RigidWorldAction) -> bool {
    matches!(
        action,
        RigidWorldAction::CreateJoint { .. }
            | RigidWorldAction::InspectJoint { .. }
            | RigidWorldAction::MutateJoint { .. }
            | RigidWorldAction::DestroyJoint { .. }
            | RigidWorldAction::CreateRope { .. }
            | RigidWorldAction::SetRopeAngle { .. }
            | RigidWorldAction::StepRope { .. }
            | RigidWorldAction::InspectRope { .. }
            | RigidWorldAction::DestroyRope { .. }
    )
}
