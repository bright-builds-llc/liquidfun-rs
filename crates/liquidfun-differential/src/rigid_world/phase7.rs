//! Phase 7 body controls, configured stepping, queries, rays, and origin shifting.

use liquidfun::collision::{Aabb, RayCastInput};
use liquidfun::{
    BodyId, QueryDirective, RayCastDirective, RayCastFraction, StepCompletion, StepConfiguration,
    StepError, StepLimits, WakePolicy,
};
use liquidfun_test_protocol::{
    FloatBits, RigidAabbBits, RigidBodyControlSnapshot, RigidFixtureChildOccurrence,
    RigidPartialProgressClassification, RigidQueryCompletion, RigidQueryDirective,
    RigidQueryDirectiveRule, RigidQueryObservation, RigidRayCompletion, RigidRayDirective,
    RigidRayDirectiveRule, RigidRayHitObservation, RigidRayObservation, RigidStepCompletion,
    RigidStepOutcome, RigidWakePolicy, RigidWorldAction, RigidWorldActionRecord,
    RigidWorldObservation, ScenarioId, Vec2Bits,
};

use super::{
    NativeHook, NativeRigidWorldError, TimelineExecutor, action_error, collect_direct_transitions,
    collect_step_report, observe_step, vec2, vec2_bits,
};

#[allow(
    clippy::too_many_lines,
    reason = "the closed protocol action dispatch stays centralized to preserve source ordering"
)]
pub(super) fn execute_action(
    executor: &mut TimelineExecutor,
    record: &RigidWorldActionRecord,
) -> Result<bool, NativeRigidWorldError> {
    match record.action() {
        RigidWorldAction::SetLinearVelocity { body_id, velocity } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .set_body_linear_velocity(body, vec2(*velocity))
                .map_err(|error| action_error(record, error))?;
            observe_body_control(executor, body_id, body, record)?;
        }
        RigidWorldAction::SetAngularVelocity {
            body_id,
            angular_velocity_bits,
        } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .set_body_angular_velocity(body, angular_velocity_bits.to_f32())
                .map_err(|error| action_error(record, error))?;
            observe_body_control(executor, body_id, body, record)?;
        }
        RigidWorldAction::ApplyForce {
            body_id,
            force,
            point,
            wake_policy,
        } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .apply_body_force(
                    body,
                    vec2(*force),
                    vec2(*point),
                    wake_policy_value(*wake_policy),
                )
                .map_err(|error| action_error(record, error))?;
            observe_body_control(executor, body_id, body, record)?;
        }
        RigidWorldAction::ApplyTorque {
            body_id,
            torque_bits,
            wake_policy,
        } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .apply_body_torque(body, torque_bits.to_f32(), wake_policy_value(*wake_policy))
                .map_err(|error| action_error(record, error))?;
            observe_body_control(executor, body_id, body, record)?;
        }
        RigidWorldAction::ApplyLinearImpulse {
            body_id,
            impulse,
            point,
            wake_policy,
        } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .apply_body_linear_impulse(
                    body,
                    vec2(*impulse),
                    vec2(*point),
                    wake_policy_value(*wake_policy),
                )
                .map_err(|error| action_error(record, error))?;
            observe_body_control(executor, body_id, body, record)?;
        }
        RigidWorldAction::ApplyAngularImpulse {
            body_id,
            impulse_bits,
            wake_policy,
        } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .apply_body_angular_impulse(
                    body,
                    impulse_bits.to_f32(),
                    wake_policy_value(*wake_policy),
                )
                .map_err(|error| action_error(record, error))?;
            observe_body_control(executor, body_id, body, record)?;
        }
        RigidWorldAction::SetBodyDamping {
            body_id,
            linear_damping_bits,
            angular_damping_bits,
        } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .set_body_linear_damping(body, linear_damping_bits.to_f32())
                .and_then(|()| {
                    executor
                        .world
                        .set_body_angular_damping(body, angular_damping_bits.to_f32())
                })
                .map_err(|error| action_error(record, error))?;
            observe_body_control(executor, body_id, body, record)?;
        }
        RigidWorldAction::SetGravityScale {
            body_id,
            gravity_scale_bits,
        } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .set_body_gravity_scale(body, gravity_scale_bits.to_f32())
                .map_err(|error| action_error(record, error))?;
            observe_body_control(executor, body_id, body, record)?;
        }
        RigidWorldAction::SetFixedRotation {
            body_id,
            fixed_rotation,
        } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .set_body_fixed_rotation(body, *fixed_rotation)
                .map_err(|error| action_error(record, error))?;
            observe_body_control(executor, body_id, body, record)?;
        }
        RigidWorldAction::SetSleepingAllowed {
            body_id,
            sleeping_allowed,
        } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .set_body_sleeping_allowed(body, *sleeping_allowed)
                .map_err(|error| action_error(record, error))?;
            observe_body_control(executor, body_id, body, record)?;
        }
        RigidWorldAction::SetAwake { body_id, awake } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .set_body_awake(body, *awake)
                .map_err(|error| action_error(record, error))?;
            observe_body_control(executor, body_id, body, record)?;
        }
        RigidWorldAction::SetBullet { body_id, bullet } => {
            let body = executor.body(body_id, record)?;
            executor
                .world
                .set_body_bullet(body, *bullet)
                .map_err(|error| action_error(record, error))?;
            observe_body_control(executor, body_id, body, record)?;
        }
        RigidWorldAction::SetWorldGravity { gravity } => executor
            .world
            .set_gravity(vec2(*gravity))
            .map_err(|error| action_error(record, error))?,
        RigidWorldAction::SetAutomaticForceClearing { enabled } => executor
            .world
            .set_automatic_force_clearing_enabled(*enabled)
            .map_err(|error| action_error(record, error))?,
        RigidWorldAction::SetWarmStarting { enabled } => executor
            .world
            .set_warm_starting_enabled(*enabled)
            .map_err(|error| action_error(record, error))?,
        RigidWorldAction::SetContinuousPhysics { enabled } => executor
            .world
            .set_continuous_physics_enabled(*enabled)
            .map_err(|error| action_error(record, error))?,
        RigidWorldAction::SetSubStepping { enabled } => executor
            .world
            .set_sub_stepping_enabled(*enabled)
            .map_err(|error| action_error(record, error))?,
        RigidWorldAction::ClearForces => executor
            .world
            .clear_forces()
            .map_err(|error| action_error(record, error))?,
        RigidWorldAction::ConfiguredStep {
            timestep_bits,
            velocity_iterations,
            position_iterations,
            continuous_work_budget,
        } => execute_configured_step(
            executor,
            record,
            *timestep_bits,
            *velocity_iterations,
            *position_iterations,
            *continuous_work_budget,
        )?,
        RigidWorldAction::QueryAabb {
            aabb,
            directive_rules,
        } => execute_query(executor, record, *aabb, directive_rules)?,
        RigidWorldAction::RayCast {
            start,
            end,
            directive_rules,
        } => execute_ray_cast(executor, record, *start, *end, directive_rules)?,
        RigidWorldAction::ShiftOrigin { shift } => {
            executor
                .world
                .shift_origin(vec2(*shift))
                .map_err(|error| action_error(record, error))?;
            executor
                .semantic_observations
                .push(RigidWorldObservation::OriginShift { shift: *shift });
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn observe_body_control(
    executor: &mut TimelineExecutor,
    body_id: &ScenarioId,
    body: BodyId,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let snapshot = executor
        .world
        .body_snapshot(body)
        .map_err(|error| action_error(action, error))?;
    executor
        .semantic_observations
        .push(RigidWorldObservation::BodyState {
            state: RigidBodyControlSnapshot {
                body_id: body_id.clone(),
                linear_velocity: vec2_bits(snapshot.linear_velocity()),
                angular_velocity_bits: FloatBits::from_f32(snapshot.angular_velocity()),
                awake: snapshot.is_awake(),
                bullet: snapshot.is_bullet(),
                sleeping_allowed: snapshot.is_sleeping_allowed(),
                fixed_rotation: snapshot.is_fixed_rotation(),
                linear_damping_bits: FloatBits::from_f32(snapshot.linear_damping()),
                angular_damping_bits: FloatBits::from_f32(snapshot.angular_damping()),
                gravity_scale_bits: FloatBits::from_f32(snapshot.gravity_scale()),
            },
        });
    Ok(())
}

fn execute_configured_step(
    executor: &mut TimelineExecutor,
    action: &RigidWorldActionRecord,
    timestep_bits: FloatBits,
    velocity_iterations: u32,
    position_iterations: u32,
    continuous_work_budget: u32,
) -> Result<(), NativeRigidWorldError> {
    let configuration = StepConfiguration::new(
        timestep_bits.to_f32(),
        velocity_iterations,
        position_iterations,
    )
    .map_err(|error| action_error(action, error))?;
    let work_limit =
        usize::try_from(continuous_work_budget).map_err(|error| action_error(action, error))?;
    let limits = StepLimits::default()
        .with_continuous_work_limit(work_limit)
        .map_err(|error| action_error(action, error))?;
    match executor.world.step(configuration, &mut NativeHook, limits) {
        Ok(report) => {
            collect_step_report(executor, &report)?;
            observe_step(executor, action.phase());
            let completion = match report.completion() {
                StepCompletion::Complete => RigidStepCompletion::Complete,
                StepCompletion::ContinuousPending => RigidStepCompletion::ContinuousPending,
            };
            executor
                .semantic_observations
                .push(RigidWorldObservation::Step {
                    outcome: RigidStepOutcome::Completed { completion },
                });
        }
        Err(StepError::ContinuousWorkLimitExceeded { .. }) => {
            collect_direct_transitions(executor)?;
            executor
                .semantic_observations
                .push(RigidWorldObservation::Step {
                    outcome: RigidStepOutcome::Partial {
                        classification:
                            RigidPartialProgressClassification::ContinuousWorkBudgetExhausted,
                    },
                });
        }
        Err(error) => return Err(action_error(action, error)),
    }
    Ok(())
}

fn execute_query(
    executor: &mut TimelineExecutor,
    action: &RigidWorldActionRecord,
    aabb: RigidAabbBits,
    rules: &[RigidQueryDirectiveRule],
) -> Result<(), NativeRigidWorldError> {
    let query = Aabb::new(vec2(aabb.lower), vec2(aabb.upper))
        .map_err(|error| action_error(action, error))?;
    let fixture_ids = executor.fixtures.clone();
    let mut terminated = false;
    let mut unmapped = false;
    let mut occurrences = Vec::new();
    executor.world.query_aabb(query, |occurrence| {
        let maybe_semantic = fixture_ids
            .iter()
            .find_map(|(id, fixture)| (*fixture == occurrence.fixture()).then(|| id.clone()));
        let Some(fixture_id) = maybe_semantic else {
            unmapped = true;
            return QueryDirective::Terminate;
        };
        let Ok(child_index) = u32::try_from(occurrence.child_index().get()) else {
            unmapped = true;
            return QueryDirective::Terminate;
        };
        occurrences.push(RigidFixtureChildOccurrence {
            fixture_id: fixture_id.clone(),
            child_index,
        });
        let directive = rules
            .iter()
            .find(|rule| {
                rule.target.fixture_id == fixture_id && rule.target.child_index == child_index
            })
            .map_or(RigidQueryDirective::Continue, |rule| rule.directive);
        match directive {
            RigidQueryDirective::Continue => QueryDirective::Continue,
            RigidQueryDirective::Terminate => {
                terminated = true;
                QueryDirective::Terminate
            }
        }
    });
    if unmapped {
        return Err(action_error(action, "query returned an unmapped fixture"));
    }
    executor
        .semantic_observations
        .push(RigidWorldObservation::Query {
            observation: RigidQueryObservation {
                completion: if terminated {
                    RigidQueryCompletion::Terminated
                } else {
                    RigidQueryCompletion::Exhausted
                },
                occurrences: occurrences.into_boxed_slice(),
            },
        });
    Ok(())
}

fn execute_ray_cast(
    executor: &mut TimelineExecutor,
    action: &RigidWorldActionRecord,
    start: Vec2Bits,
    end: Vec2Bits,
    rules: &[RigidRayDirectiveRule],
) -> Result<(), NativeRigidWorldError> {
    let input = RayCastInput::new(vec2(start), vec2(end), 1.0)
        .map_err(|error| action_error(action, error))?;
    let fixture_ids = executor.fixtures.clone();
    let mut terminated = false;
    let mut invalid = false;
    let mut hits = Vec::new();
    let result = executor.world.ray_cast(input, |hit| {
        let maybe_semantic = fixture_ids
            .iter()
            .find_map(|(id, fixture)| (*fixture == hit.fixture()).then(|| id.clone()));
        let Some(fixture_id) = maybe_semantic else {
            invalid = true;
            return RayCastDirective::Terminate;
        };
        let Ok(child_index) = u32::try_from(hit.child_index().get()) else {
            invalid = true;
            return RayCastDirective::Terminate;
        };
        hits.push(RigidRayHitObservation {
            fixture_id: fixture_id.clone(),
            child_index,
            point: vec2_bits(hit.point()),
            normal: vec2_bits(hit.normal()),
            fraction_bits: FloatBits::from_f32(hit.fraction().get()),
        });
        let directive = rules
            .iter()
            .find(|rule| {
                rule.target.fixture_id == fixture_id && rule.target.child_index == child_index
            })
            .map_or(RigidRayDirective::Continue, |rule| rule.directive);
        match directive {
            RigidRayDirective::Ignore => RayCastDirective::Ignore,
            RigidRayDirective::Terminate => {
                terminated = true;
                RayCastDirective::Terminate
            }
            RigidRayDirective::Continue => RayCastDirective::Continue,
            RigidRayDirective::Clip { fraction_bits } => {
                match RayCastFraction::new(fraction_bits.to_f32()) {
                    Ok(fraction) => RayCastDirective::Clip(fraction),
                    Err(_error) => {
                        invalid = true;
                        RayCastDirective::Terminate
                    }
                }
            }
        }
    });
    if invalid {
        return Err(action_error(
            action,
            "ray cast returned invalid semantic data",
        ));
    }
    result.map_err(|error| action_error(action, error))?;
    executor
        .semantic_observations
        .push(RigidWorldObservation::RayCast {
            observation: RigidRayObservation {
                completion: if terminated {
                    RigidRayCompletion::Terminated
                } else {
                    RigidRayCompletion::Exhausted
                },
                hits: hits.into_boxed_slice(),
            },
        });
    Ok(())
}

const fn wake_policy_value(policy: RigidWakePolicy) -> WakePolicy {
    match policy {
        RigidWakePolicy::Wake => WakePolicy::Wake,
        RigidWakePolicy::PreserveSleep => WakePolicy::PreserveSleep,
    }
}
