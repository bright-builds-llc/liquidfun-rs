//! Phase 8 joints, rope, callbacks, lifecycle, and semantic diagnostics.

use liquidfun::rope::{Rope, RopeDef, RopeIterations};
use liquidfun::{
    CollisionDecisionHook, CollisionDirective, DestroyedId, FixtureId, FixturePairView, JointDef,
    LifecycleEvent, PreSolveDirective, PreSolveView, StepConfiguration, StepError,
    StepLifecycleEvent, StepLimits, StepReport,
};
use liquidfun::{
    DistanceJointDef, FrictionJointDef, GearJointDef, MotorJointDef, MouseJointDef,
    PrismaticJointDef, PulleyJointDef, RevoluteJointDef, RopeJointDef, WeldJointDef, WheelJointDef,
};
use liquidfun_test_protocol::{
    FloatBits, RigidContactDirectiveTarget, RigidDiagnosticsObservation, RigidJointDeclaration,
    RigidJointDefinition, RigidJointMutation, RigidLifecycleObservation,
    RigidLifecycleObservationKind, RigidPreSolveDirective, RigidReconstructionKind,
    RigidReconstructionObservation, RigidReconstructionSupport, RigidRopeDeclaration,
    RigidRopeSnapshot, RigidWorldAction, RigidWorldActionRecord, RigidWorldObservation,
    RigidWorldTimeline, RigidWorldWitnessFamily, ScenarioId,
};

use super::{NativeRigidWorldError, TimelineExecutor, action_error, checked_u32, vec2, vec2_bits};

pub(super) fn execute_action(
    executor: &mut TimelineExecutor,
    timeline: &RigidWorldTimeline,
    action: &RigidWorldActionRecord,
) -> Result<bool, NativeRigidWorldError> {
    match action.action() {
        RigidWorldAction::CreateJoint { joint_id } => {
            create_joint(executor, timeline, joint_id, action)?;
        }
        RigidWorldAction::InspectJoint { joint_id } => observe_joint(executor, joint_id, action)?,
        RigidWorldAction::MutateJoint { joint_id, mutation } => {
            mutate_joint(executor, joint_id, *mutation, action)?;
            observe_joint(executor, joint_id, action)?;
        }
        RigidWorldAction::DestroyJoint { joint_id } => destroy_joint(executor, joint_id, action)?,
        RigidWorldAction::CreateRope { rope_id } => {
            create_rope(executor, timeline, rope_id, action)?;
        }
        RigidWorldAction::SetRopeAngle {
            rope_id,
            angle_bits,
        } => {
            let rope = rope_mut(executor, rope_id, action)?;
            rope.set_angle(angle_bits.to_f32())
                .map_err(|error| action_error(action, error))?;
            observe_rope(executor, rope_id, action)?;
        }
        RigidWorldAction::StepRope {
            rope_id,
            timestep_bits,
            iterations,
        } => {
            let count =
                usize::try_from(*iterations).map_err(|error| action_error(action, error))?;
            let iterations =
                RopeIterations::new(count).map_err(|error| action_error(action, error))?;
            rope_mut(executor, rope_id, action)?
                .step(timestep_bits.to_f32(), iterations)
                .map_err(|error| action_error(action, error))?;
            observe_rope(executor, rope_id, action)?;
        }
        RigidWorldAction::InspectRope { rope_id } => observe_rope(executor, rope_id, action)?,
        RigidWorldAction::DestroyRope { rope_id } => {
            let before = executor.ropes.len();
            executor.ropes.retain(|(id, _rope)| id != rope_id);
            if executor.ropes.len() == before {
                return Err(action_error(action, format!("unknown rope `{rope_id}`")));
            }
        }
        RigidWorldAction::SetContactFilterDirective {
            target,
            should_collide,
        } => {
            let [fixture_a, fixture_b] = directive_fixtures(executor, target, action)?;
            upsert_pair(
                &mut executor.filter_directives,
                fixture_a,
                fixture_b,
                *should_collide,
            );
            refresh_filter_pair(executor, [fixture_a, fixture_b], action)?;
        }
        RigidWorldAction::SetPreSolveDirective { target, directive } => {
            let [fixture_a, fixture_b] = directive_fixtures(executor, target, action)?;
            let directive = pre_solve_directive(*directive, action)?;
            upsert_pair(
                &mut executor.pre_solve_directives,
                fixture_a,
                fixture_b,
                directive,
            );
        }
        RigidWorldAction::RequestReconstruction => observe_reconstruction(executor, action)?,
        RigidWorldAction::RequestDiagnostics => observe_diagnostics(executor)?,
        _ => return Ok(false),
    }
    Ok(true)
}

pub(super) fn step(
    executor: &mut TimelineExecutor,
    configuration: StepConfiguration,
    limits: StepLimits,
) -> Result<StepReport, StepError> {
    let mut hook = Phase8Hook {
        filter_directives: executor.filter_directives.clone(),
        pre_solve_directives: executor.pre_solve_directives.clone(),
        allow_unconfigured_contacts: captures_contact_behavior(executor.family),
    };
    executor.world.step(configuration, &mut hook, limits)
}

fn captures_contact_behavior(family: RigidWorldWitnessFamily) -> bool {
    // Solver-only timelines use fixtures solely to give moving bodies mass. Rejecting their
    // undeclared cross-family pairs keeps joint evidence independent of incidental contacts;
    // the C++ adapter must mirror this typed-family rule.
    matches!(
        family,
        RigidWorldWitnessFamily::MixedJointIslandOrderAndCollisionSuppression
            | RigidWorldWitnessFamily::ContactFilterListenerAndPreSolveTiming
            | RigidWorldWitnessFamily::DestructionListenerAndDependencyCascades
    ) || !RigidWorldWitnessFamily::PHASE8_REQUIRED.contains(&family)
}

fn refresh_filter_pair(
    executor: &mut TimelineExecutor,
    fixtures: [FixtureId; 2],
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    for fixture in fixtures {
        let filter = executor
            .world
            .fixture_snapshot(fixture)
            .map_err(|error| action_error(action, error))?
            .filter_data();
        executor
            .world
            .set_fixture_filter(fixture, filter)
            .map_err(|error| action_error(action, error))?;
    }
    Ok(())
}

struct Phase8Hook {
    filter_directives: Vec<(FixtureId, FixtureId, bool)>,
    pre_solve_directives: Vec<(FixtureId, FixtureId, PreSolveDirective)>,
    allow_unconfigured_contacts: bool,
}

impl CollisionDecisionHook for Phase8Hook {
    fn should_collide(&mut self, pair: FixturePairView<'_>) -> CollisionDirective {
        pair_value(&self.filter_directives, pair.fixtures()).map_or_else(
            || {
                if self.allow_unconfigured_contacts {
                    CollisionDirective::Collide
                } else {
                    CollisionDirective::Ignore
                }
            },
            |should_collide| {
                if *should_collide {
                    CollisionDirective::Collide
                } else {
                    CollisionDirective::Ignore
                }
            },
        )
    }

    fn pre_solve(&mut self, contact: PreSolveView<'_>) -> PreSolveDirective {
        pair_value(&self.pre_solve_directives, contact.fixtures())
            .copied()
            .unwrap_or(PreSolveDirective::Enable)
    }
}

fn create_joint(
    executor: &mut TimelineExecutor,
    timeline: &RigidWorldTimeline,
    joint_id: &ScenarioId,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let declaration = timeline
        .joints()
        .iter()
        .find(|value| &value.joint_id == joint_id)
        .ok_or_else(|| {
            action_error(
                action,
                format!("missing declaration for joint `{joint_id}`"),
            )
        })?;
    let definition = joint_definition(executor, declaration, action)?;
    let joint = executor
        .world
        .create_joint(definition)
        .map_err(|error| action_error(action, error))?;
    executor.joints.push((joint_id.clone(), joint));
    observe_joint(executor, joint_id, action)?;
    Ok(())
}

#[allow(
    clippy::too_many_lines,
    reason = "closed protocol-to-joint definition mapping is exhaustive"
)]
fn joint_definition(
    executor: &TimelineExecutor,
    declaration: &RigidJointDeclaration,
    action: &RigidWorldActionRecord,
) -> Result<JointDef, NativeRigidWorldError> {
    let body_a = executor.body(&declaration.body_a_id, action)?;
    let body_b = executor.body(&declaration.body_b_id, action)?;
    let collide = declaration.collide_connected;
    let definition = match &declaration.definition {
        RigidJointDefinition::Revolute {
            local_anchor_a,
            local_anchor_b,
            reference_angle_bits,
            lower_angle_bits,
            upper_angle_bits,
            motor_speed_bits,
            max_motor_torque_bits,
            limit_enabled,
            motor_enabled,
        } => RevoluteJointDef::new(body_a, body_b)
            .and_then(|value| {
                value.with_frame(
                    vec2(*local_anchor_a),
                    vec2(*local_anchor_b),
                    reference_angle_bits.to_f32(),
                )
            })
            .and_then(|value| {
                value.with_limits(
                    *limit_enabled,
                    lower_angle_bits.to_f32(),
                    upper_angle_bits.to_f32(),
                )
            })
            .and_then(|value| {
                value.with_motor(
                    *motor_enabled,
                    motor_speed_bits.to_f32(),
                    max_motor_torque_bits.to_f32(),
                )
            })
            .map(|value| JointDef::from(value.with_collide_connected(collide))),
        RigidJointDefinition::Prismatic {
            local_anchor_a,
            local_anchor_b,
            local_axis_a,
            reference_angle_bits,
            lower_translation_bits,
            upper_translation_bits,
            motor_speed_bits,
            max_motor_force_bits,
            limit_enabled,
            motor_enabled,
        } => PrismaticJointDef::new(body_a, body_b)
            .and_then(|value| {
                value.with_frame(
                    vec2(*local_anchor_a),
                    vec2(*local_anchor_b),
                    vec2(*local_axis_a),
                    reference_angle_bits.to_f32(),
                )
            })
            .and_then(|value| {
                value.with_limits(
                    *limit_enabled,
                    lower_translation_bits.to_f32(),
                    upper_translation_bits.to_f32(),
                )
            })
            .and_then(|value| {
                value.with_motor(
                    *motor_enabled,
                    motor_speed_bits.to_f32(),
                    max_motor_force_bits.to_f32(),
                )
            })
            .map(|value| JointDef::from(value.with_collide_connected(collide))),
        RigidJointDefinition::Distance {
            local_anchor_a,
            local_anchor_b,
            length_bits,
            frequency_bits,
            damping_ratio_bits,
        } => DistanceJointDef::new(body_a, body_b)
            .and_then(|value| value.with_anchors(vec2(*local_anchor_a), vec2(*local_anchor_b)))
            .and_then(|value| value.with_length(length_bits.to_f32()))
            .and_then(|value| value.with_frequency(frequency_bits.to_f32()))
            .and_then(|value| value.with_damping_ratio(damping_ratio_bits.to_f32()))
            .map(|value| JointDef::from(value.with_collide_connected(collide))),
        RigidJointDefinition::Pulley {
            ground_anchor_a,
            ground_anchor_b,
            local_anchor_a,
            local_anchor_b,
            length_a_bits,
            length_b_bits,
            ratio_bits,
        } => PulleyJointDef::new(body_a, body_b)
            .and_then(|value| {
                value.with_geometry(
                    vec2(*ground_anchor_a),
                    vec2(*ground_anchor_b),
                    vec2(*local_anchor_a),
                    vec2(*local_anchor_b),
                    length_a_bits.to_f32(),
                    length_b_bits.to_f32(),
                    ratio_bits.to_f32(),
                )
            })
            .map(|value| JointDef::from(value.with_collide_connected(collide))),
        RigidJointDefinition::Mouse {
            target,
            max_force_bits,
            frequency_bits,
            damping_ratio_bits,
        } => MouseJointDef::new(body_a, body_b)
            .and_then(|value| value.with_target(vec2(*target)))
            .and_then(|value| value.with_max_force(max_force_bits.to_f32()))
            .and_then(|value| value.with_frequency(frequency_bits.to_f32()))
            .and_then(|value| value.with_damping_ratio(damping_ratio_bits.to_f32()))
            .map(|value| JointDef::from(value.with_collide_connected(collide))),
        RigidJointDefinition::Gear {
            joint_a_id,
            joint_b_id,
            ratio_bits,
        } => GearJointDef::new(
            executor.joint(joint_a_id, action)?,
            executor.joint(joint_b_id, action)?,
        )
        .and_then(|value| value.with_ratio(ratio_bits.to_f32()))
        .map(|value| JointDef::from(value.with_collide_connected(collide))),
        RigidJointDefinition::Wheel {
            local_anchor_a,
            local_anchor_b,
            local_axis_a,
            motor_speed_bits,
            max_motor_torque_bits,
            frequency_bits,
            damping_ratio_bits,
            motor_enabled,
        } => WheelJointDef::new(body_a, body_b)
            .and_then(|value| {
                value.with_frame(
                    vec2(*local_anchor_a),
                    vec2(*local_anchor_b),
                    vec2(*local_axis_a),
                )
            })
            .and_then(|value| {
                value.with_motor(
                    *motor_enabled,
                    motor_speed_bits.to_f32(),
                    max_motor_torque_bits.to_f32(),
                )
            })
            .and_then(|value| {
                value.with_spring(frequency_bits.to_f32(), damping_ratio_bits.to_f32())
            })
            .map(|value| JointDef::from(value.with_collide_connected(collide))),
        RigidJointDefinition::Weld {
            local_anchor_a,
            local_anchor_b,
            reference_angle_bits,
            frequency_bits,
            damping_ratio_bits,
        } => WeldJointDef::new(body_a, body_b)
            .and_then(|value| {
                value.with_frame(
                    vec2(*local_anchor_a),
                    vec2(*local_anchor_b),
                    reference_angle_bits.to_f32(),
                )
            })
            .and_then(|value| value.with_frequency(frequency_bits.to_f32()))
            .and_then(|value| value.with_damping_ratio(damping_ratio_bits.to_f32()))
            .map(|value| JointDef::from(value.with_collide_connected(collide))),
        RigidJointDefinition::Friction {
            local_anchor_a,
            local_anchor_b,
            max_force_bits,
            max_torque_bits,
        } => FrictionJointDef::new(body_a, body_b)
            .and_then(|value| value.with_anchors(vec2(*local_anchor_a), vec2(*local_anchor_b)))
            .and_then(|value| value.with_max_force(max_force_bits.to_f32()))
            .and_then(|value| value.with_max_torque(max_torque_bits.to_f32()))
            .map(|value| JointDef::from(value.with_collide_connected(collide))),
        RigidJointDefinition::Rope {
            local_anchor_a,
            local_anchor_b,
            max_length_bits,
        } => RopeJointDef::new(body_a, body_b)
            .and_then(|value| value.with_anchors(vec2(*local_anchor_a), vec2(*local_anchor_b)))
            .and_then(|value| value.with_max_length(max_length_bits.to_f32()))
            .map(|value| JointDef::from(value.with_collide_connected(collide))),
        RigidJointDefinition::Motor {
            linear_offset,
            angular_offset_bits,
            max_force_bits,
            max_torque_bits,
            correction_factor_bits,
        } => MotorJointDef::new(body_a, body_b)
            .and_then(|value| {
                value.with_offsets(vec2(*linear_offset), angular_offset_bits.to_f32())
            })
            .and_then(|value| value.with_caps(max_force_bits.to_f32(), max_torque_bits.to_f32()))
            .and_then(|value| value.with_correction_factor(correction_factor_bits.to_f32()))
            .map(|value| JointDef::from(value.with_collide_connected(collide))),
    };
    definition.map_err(|error| action_error(action, error))
}

fn observe_joint(
    executor: &mut TimelineExecutor,
    joint_id: &ScenarioId,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let joint = executor.joint(joint_id, action)?;
    let snapshot = crate::rigid_evidence::phase8::joint_observation(executor, joint_id, joint)?;
    executor
        .semantic_observations
        .push(RigidWorldObservation::Joint { snapshot });
    Ok(())
}

fn mutate_joint(
    executor: &mut TimelineExecutor,
    joint_id: &ScenarioId,
    mutation: RigidJointMutation,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let joint = executor.joint(joint_id, action)?;
    match mutation {
        RigidJointMutation::LimitEnabled { enabled } => {
            set_limit_enabled(executor, joint, enabled, action)
        }
        RigidJointMutation::Limits {
            lower_bits,
            upper_bits,
        } => set_limits(
            executor,
            joint,
            lower_bits.to_f32(),
            upper_bits.to_f32(),
            action,
        ),
        RigidJointMutation::MotorEnabled { enabled } => {
            set_motor_enabled(executor, joint, enabled, action)
        }
        RigidJointMutation::MotorSpeed { speed_bits } => {
            set_motor_speed(executor, joint, speed_bits.to_f32(), action)
        }
        RigidJointMutation::MaxMotorForce { force_bits } => apply_mutation(
            executor
                .world
                .set_prismatic_max_motor_force(joint, force_bits.to_f32()),
            action,
        ),
        RigidJointMutation::MaxMotorTorque { torque_bits } => {
            set_max_motor_torque(executor, joint, torque_bits.to_f32(), action)
        }
        RigidJointMutation::Length { length_bits } => apply_mutation(
            executor
                .world
                .set_distance_length(joint, length_bits.to_f32()),
            action,
        ),
        RigidJointMutation::Frequency { frequency_bits } => {
            set_frequency(executor, joint, frequency_bits.to_f32(), action)
        }
        RigidJointMutation::DampingRatio { damping_ratio_bits } => {
            set_damping_ratio(executor, joint, damping_ratio_bits.to_f32(), action)
        }
        RigidJointMutation::MouseTarget { target } => {
            apply_mutation(executor.world.set_mouse_target(joint, vec2(target)), action)
        }
        RigidJointMutation::MaxForce { force_bits } => {
            set_max_force(executor, joint, force_bits.to_f32(), action)
        }
        RigidJointMutation::MaxTorque { torque_bits } => {
            set_max_torque(executor, joint, torque_bits.to_f32(), action)
        }
        RigidJointMutation::GearRatio { ratio_bits } => apply_mutation(
            executor.world.set_gear_ratio(joint, ratio_bits.to_f32()),
            action,
        ),
        RigidJointMutation::RopeMaxLength { max_length_bits } => apply_mutation(
            executor
                .world
                .set_rope_joint_max_length(joint, max_length_bits.to_f32()),
            action,
        ),
        RigidJointMutation::LinearOffset { offset } => apply_mutation(
            executor.world.set_motor_linear_offset(joint, vec2(offset)),
            action,
        ),
        RigidJointMutation::AngularOffset { offset_bits } => apply_mutation(
            executor
                .world
                .set_motor_angular_offset(joint, offset_bits.to_f32()),
            action,
        ),
        RigidJointMutation::CorrectionFactor { factor_bits } => apply_mutation(
            executor
                .world
                .set_motor_correction_factor(joint, factor_bits.to_f32()),
            action,
        ),
    }
}

fn joint_kind(
    executor: &TimelineExecutor,
    joint: liquidfun::JointId,
) -> Option<liquidfun::JointKind> {
    executor
        .world
        .joint_snapshot(joint)
        .ok()
        .map(liquidfun::JointSnapshot::kind)
}

fn apply_mutation(
    result: Result<(), liquidfun::JointMutationError>,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    result.map_err(|error| action_error(action, error))
}

fn set_limit_enabled(
    executor: &mut TimelineExecutor,
    joint: liquidfun::JointId,
    enabled: bool,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let result = match joint_kind(executor, joint) {
        Some(liquidfun::JointKind::Revolute) => {
            executor.world.set_revolute_limit_enabled(joint, enabled)
        }
        Some(liquidfun::JointKind::Prismatic) => {
            executor.world.set_prismatic_limit_enabled(joint, enabled)
        }
        _ => {
            return Err(action_error(
                action,
                "unsupported limit-enabled mutation kind",
            ));
        }
    };
    apply_mutation(result, action)
}

fn set_limits(
    executor: &mut TimelineExecutor,
    joint: liquidfun::JointId,
    lower: f32,
    upper: f32,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let result = match joint_kind(executor, joint) {
        Some(liquidfun::JointKind::Revolute) => {
            executor.world.set_revolute_limits(joint, lower, upper)
        }
        Some(liquidfun::JointKind::Prismatic) => {
            executor.world.set_prismatic_limits(joint, lower, upper)
        }
        _ => return Err(action_error(action, "unsupported limits mutation kind")),
    };
    apply_mutation(result, action)
}

fn set_motor_enabled(
    executor: &mut TimelineExecutor,
    joint: liquidfun::JointId,
    enabled: bool,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let result = match joint_kind(executor, joint) {
        Some(liquidfun::JointKind::Revolute) => {
            executor.world.set_revolute_motor_enabled(joint, enabled)
        }
        Some(liquidfun::JointKind::Prismatic) => {
            executor.world.set_prismatic_motor_enabled(joint, enabled)
        }
        Some(liquidfun::JointKind::Wheel) => executor.world.set_wheel_motor_enabled(joint, enabled),
        _ => {
            return Err(action_error(
                action,
                "unsupported motor-enabled mutation kind",
            ));
        }
    };
    apply_mutation(result, action)
}

fn set_motor_speed(
    executor: &mut TimelineExecutor,
    joint: liquidfun::JointId,
    speed: f32,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let result = match joint_kind(executor, joint) {
        Some(liquidfun::JointKind::Revolute) => {
            executor.world.set_revolute_motor_speed(joint, speed)
        }
        Some(liquidfun::JointKind::Prismatic) => {
            executor.world.set_prismatic_motor_speed(joint, speed)
        }
        Some(liquidfun::JointKind::Wheel) => executor.world.set_wheel_motor_speed(joint, speed),
        _ => {
            return Err(action_error(
                action,
                "unsupported motor-speed mutation kind",
            ));
        }
    };
    apply_mutation(result, action)
}

fn set_max_motor_torque(
    executor: &mut TimelineExecutor,
    joint: liquidfun::JointId,
    torque: f32,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let result = match joint_kind(executor, joint) {
        Some(liquidfun::JointKind::Revolute) => {
            executor.world.set_revolute_max_motor_torque(joint, torque)
        }
        Some(liquidfun::JointKind::Wheel) => {
            executor.world.set_wheel_max_motor_torque(joint, torque)
        }
        _ => {
            return Err(action_error(
                action,
                "unsupported motor-torque mutation kind",
            ));
        }
    };
    apply_mutation(result, action)
}

fn set_frequency(
    executor: &mut TimelineExecutor,
    joint: liquidfun::JointId,
    frequency: f32,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let result = match joint_kind(executor, joint) {
        Some(liquidfun::JointKind::Distance) => {
            executor.world.set_distance_frequency(joint, frequency)
        }
        Some(liquidfun::JointKind::Mouse) => executor.world.set_mouse_frequency(joint, frequency),
        Some(liquidfun::JointKind::Wheel) => executor.world.set_wheel_frequency(joint, frequency),
        Some(liquidfun::JointKind::Weld) => executor.world.set_weld_frequency(joint, frequency),
        _ => return Err(action_error(action, "unsupported frequency mutation kind")),
    };
    apply_mutation(result, action)
}

fn set_damping_ratio(
    executor: &mut TimelineExecutor,
    joint: liquidfun::JointId,
    damping_ratio: f32,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let result = match joint_kind(executor, joint) {
        Some(liquidfun::JointKind::Distance) => executor
            .world
            .set_distance_damping_ratio(joint, damping_ratio),
        Some(liquidfun::JointKind::Mouse) => {
            executor.world.set_mouse_damping_ratio(joint, damping_ratio)
        }
        Some(liquidfun::JointKind::Wheel) => {
            executor.world.set_wheel_damping_ratio(joint, damping_ratio)
        }
        Some(liquidfun::JointKind::Weld) => {
            executor.world.set_weld_damping_ratio(joint, damping_ratio)
        }
        _ => return Err(action_error(action, "unsupported damping mutation kind")),
    };
    apply_mutation(result, action)
}

fn set_max_force(
    executor: &mut TimelineExecutor,
    joint: liquidfun::JointId,
    force: f32,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let result = match joint_kind(executor, joint) {
        Some(liquidfun::JointKind::Mouse) => executor.world.set_mouse_max_force(joint, force),
        Some(liquidfun::JointKind::Friction) => executor.world.set_friction_max_force(joint, force),
        Some(liquidfun::JointKind::Motor) => executor.world.set_motor_max_force(joint, force),
        _ => return Err(action_error(action, "unsupported force mutation kind")),
    };
    apply_mutation(result, action)
}

fn set_max_torque(
    executor: &mut TimelineExecutor,
    joint: liquidfun::JointId,
    torque: f32,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let result = match joint_kind(executor, joint) {
        Some(liquidfun::JointKind::Friction) => {
            executor.world.set_friction_max_torque(joint, torque)
        }
        Some(liquidfun::JointKind::Motor) => executor.world.set_motor_max_torque(joint, torque),
        _ => return Err(action_error(action, "unsupported torque mutation kind")),
    };
    apply_mutation(result, action)
}

fn destroy_joint(
    executor: &mut TimelineExecutor,
    joint_id: &ScenarioId,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let joint = executor.joint(joint_id, action)?;
    let report = executor
        .world
        .destroy_joint(joint)
        .map_err(|error| action_error(action, error))?;
    collect_mutation_lifecycle(executor, report.lifecycle())?;
    for record in &report {
        remove_destroyed_mapping(executor, record.destroyed());
    }
    Ok(())
}

fn create_rope(
    executor: &mut TimelineExecutor,
    timeline: &RigidWorldTimeline,
    rope_id: &ScenarioId,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let declaration = timeline
        .ropes()
        .iter()
        .find(|value| &value.rope_id == rope_id)
        .ok_or_else(|| action_error(action, format!("missing declaration for rope `{rope_id}`")))?;
    let rope =
        Rope::new(rope_definition(declaration).map_err(|error| action_error(action, error))?)
            .map_err(|error| action_error(action, error))?;
    executor.ropes.push((rope_id.clone(), rope));
    Ok(())
}

fn rope_definition(
    declaration: &RigidRopeDeclaration,
) -> Result<RopeDef, liquidfun::rope::RopeError> {
    RopeDef::new(
        declaration.vertices.iter().copied().map(vec2).collect(),
        declaration
            .masses_bits
            .iter()
            .map(|bits| bits.to_f32())
            .collect(),
        vec2(declaration.gravity),
        declaration.damping_bits.to_f32(),
        declaration.stretch_stiffness_bits.to_f32(),
        declaration.bend_stiffness_bits.to_f32(),
    )
}

fn observe_rope(
    executor: &mut TimelineExecutor,
    rope_id: &ScenarioId,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let rope = executor
        .ropes
        .iter()
        .find_map(|(id, rope)| (id == rope_id).then_some(rope))
        .ok_or_else(|| action_error(action, format!("unknown rope `{rope_id}`")))?;
    let snapshot = RigidRopeSnapshot {
        rope_id: rope_id.clone(),
        vertices: rope.vertices().iter().copied().map(vec2_bits).collect(),
    };
    executor
        .semantic_observations
        .push(RigidWorldObservation::Rope { snapshot });
    Ok(())
}

fn rope_mut<'a>(
    executor: &'a mut TimelineExecutor,
    rope_id: &ScenarioId,
    action: &RigidWorldActionRecord,
) -> Result<&'a mut Rope, NativeRigidWorldError> {
    executor
        .ropes
        .iter_mut()
        .find_map(|(id, rope)| (id == rope_id).then_some(rope))
        .ok_or_else(|| action_error(action, format!("unknown rope `{rope_id}`")))
}

fn observe_diagnostics(executor: &mut TimelineExecutor) -> Result<(), NativeRigidWorldError> {
    let diagnostics = executor.world.world_diagnostics();
    executor
        .semantic_observations
        .push(RigidWorldObservation::Diagnostics {
            snapshot: RigidDiagnosticsObservation {
                body_count: checked_u32(diagnostics.body_count(), "diagnostic-body-count")?,
                fixture_count: checked_u32(
                    diagnostics.fixture_count(),
                    "diagnostic-fixture-count",
                )?,
                joint_count: checked_u32(diagnostics.joint_count(), "diagnostic-joint-count")?,
                contact_count: checked_u32(
                    diagnostics.contact_count(),
                    "diagnostic-contact-count",
                )?,
                tree_height: u32::try_from(diagnostics.tree_height()).map_err(|error| {
                    NativeRigidWorldError::Declaration {
                        checkpoint_id: "diagnostic-tree-height".into(),
                        message: error.to_string().into(),
                    }
                })?,
                tree_max_balance: u32::try_from(diagnostics.tree_balance()).map_err(|error| {
                    NativeRigidWorldError::Declaration {
                        checkpoint_id: "diagnostic-tree-balance".into(),
                        message: error.to_string().into(),
                    }
                })?,
                tree_quality_bits: FloatBits::from_f32(diagnostics.tree_quality()),
            },
        });
    Ok(())
}

fn observe_reconstruction(
    executor: &mut TimelineExecutor,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
    let reconstruction = executor
        .world
        .semantic_reconstruction()
        .map_err(|error| action_error(action, error))?;
    let body_ids = executor
        .bodies
        .iter()
        .rev()
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    let mut body_ids = body_ids.into_iter();
    let mut ordinal = 0_u32;
    for body in reconstruction.bodies() {
        let body_id = body_ids
            .next()
            .ok_or_else(|| action_error(action, "reconstruction body mapping exhausted"))?;
        let native_body = executor.body(&body_id, action)?;
        push_reconstruction(
            executor,
            &mut ordinal,
            RigidReconstructionKind::Body,
            body_id,
            RigidReconstructionSupport::Supported,
            Vec::new(),
        )?;
        let fixture_ids = executor
            .fixtures
            .iter()
            .rev()
            .filter_map(|(id, fixture)| {
                executor
                    .fixture_owners
                    .iter()
                    .find_map(|(candidate, owner)| {
                        (*candidate == *fixture && *owner == native_body).then(|| id.clone())
                    })
            })
            .collect::<Vec<_>>();
        let mut fixture_ids = fixture_ids.into_iter();
        for _fixture in body.fixtures() {
            let fixture_id = fixture_ids
                .next()
                .ok_or_else(|| action_error(action, "reconstruction fixture mapping exhausted"))?;
            push_reconstruction(
                executor,
                &mut ordinal,
                RigidReconstructionKind::Fixture,
                fixture_id,
                RigidReconstructionSupport::Supported,
                Vec::new(),
            )?;
        }
    }
    let ordered_joints = reconstruction.joints();
    let joint_by_index = executor
        .joints
        .iter()
        .rev()
        .map(|(id, _)| id.clone())
        .collect::<Vec<_>>();
    for joint in ordered_joints {
        let position =
            usize::try_from(joint.index().get()).map_err(|error| action_error(action, error))?;
        let entity_id = joint_by_index
            .get(position)
            .cloned()
            .ok_or_else(|| action_error(action, "reconstruction joint index was unmapped"))?;
        let dependencies = joint
            .maybe_source_joint_indices()
            .map_or_else(Vec::new, |indices| {
                indices
                    .into_iter()
                    .filter_map(|index| {
                        usize::try_from(index.get())
                            .ok()
                            .and_then(|value| joint_by_index.get(value).cloned())
                    })
                    .collect()
            });
        let support = match joint.support() {
            liquidfun::ReconstructionSupport::Supported(_) => RigidReconstructionSupport::Supported,
            liquidfun::ReconstructionSupport::Unsupported(
                liquidfun::ReconstructionUnsupported::MouseJoint,
            ) => RigidReconstructionSupport::UnsupportedMouseJoint,
        };
        push_reconstruction(
            executor,
            &mut ordinal,
            RigidReconstructionKind::Joint,
            entity_id,
            support,
            dependencies,
        )?;
    }
    Ok(())
}

fn push_reconstruction(
    executor: &mut TimelineExecutor,
    ordinal: &mut u32,
    kind: RigidReconstructionKind,
    entity_id: ScenarioId,
    support: RigidReconstructionSupport,
    dependencies: Vec<ScenarioId>,
) -> Result<(), NativeRigidWorldError> {
    executor
        .semantic_observations
        .push(RigidWorldObservation::Reconstruction {
            record: RigidReconstructionObservation {
                ordinal: *ordinal,
                kind,
                entity_id,
                support,
                dependency_ids: dependencies.into_boxed_slice(),
            },
        });
    *ordinal = ordinal
        .checked_add(1)
        .ok_or_else(|| NativeRigidWorldError::Declaration {
            checkpoint_id: "reconstruction-ordinal".into(),
            message: "reconstruction ordinal exceeded the protocol representation".into(),
        })?;
    Ok(())
}

pub(super) fn collect_step_lifecycle(
    executor: &mut TimelineExecutor,
    report: &StepReport,
) -> Result<(), NativeRigidWorldError> {
    if !captures_lifecycle_evidence(executor.family) {
        return Ok(());
    }
    collect_mutation_lifecycle(executor, report.lifecycle())
}

pub(super) fn collect_mutation_lifecycle(
    executor: &mut TimelineExecutor,
    lifecycle: &[LifecycleEvent],
) -> Result<(), NativeRigidWorldError> {
    if !captures_lifecycle_evidence(executor.family) {
        return Ok(());
    }
    for event in lifecycle {
        match event {
            StepLifecycleEvent::Filter(filter) => {
                let fixture = executor.semantic_fixture(filter.fixtures()[0])?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::FilterDecision,
                    None,
                    Some(fixture),
                )?;
            }
            StepLifecycleEvent::Contact(transition) => {
                collect_contact_lifecycle(executor, transition)?;
            }
            StepLifecycleEvent::ContactDestruction(transition) => {
                let contact = executor.contact_identity(transition.contact())?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::EndContact,
                    Some(contact.clone()),
                    None,
                )?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::ContactDestroyed,
                    Some(contact),
                    None,
                )?;
            }
            StepLifecycleEvent::Hook(event) if event.maybe_pre_solve().is_some() => {
                let contact = executor.contact_identity(event.contact())?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::PreSolve,
                    Some(contact),
                    None,
                )?;
            }
            StepLifecycleEvent::Solve(solve) | StepLifecycleEvent::ContinuousSolve(solve) => {
                let contact = executor.contact_identity(solve.contact())?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::PostSolve,
                    Some(contact),
                    None,
                )?;
            }
            StepLifecycleEvent::JointGoodbye(record) => {
                let entity = semantic_destroyed(executor, record.destroyed())?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::JointGoodbye,
                    None,
                    Some(entity),
                )?;
            }
            StepLifecycleEvent::FixtureGoodbye(record) => {
                let entity = semantic_destroyed(executor, record.destroyed())?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::FixtureGoodbye,
                    None,
                    Some(entity),
                )?;
            }
            StepLifecycleEvent::Destruction(record)
                if matches!(record.destroyed(), DestroyedId::Body(_)) =>
            {
                let entity = semantic_destroyed(executor, record.destroyed())?;
                push_lifecycle(
                    executor,
                    RigidLifecycleObservationKind::BodyDestroyed,
                    None,
                    Some(entity),
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn collect_contact_lifecycle(
    executor: &mut TimelineExecutor,
    transition: &liquidfun::ContactTransition,
) -> Result<(), NativeRigidWorldError> {
    let maybe_kind = match transition.kind() {
        liquidfun::ContactTransitionKind::Begin => {
            Some(RigidLifecycleObservationKind::BeginContact)
        }
        liquidfun::ContactTransitionKind::End => Some(RigidLifecycleObservationKind::EndContact),
        _ => None,
    };
    let Some(kind) = maybe_kind else {
        return Ok(());
    };
    let occurrence = transition.contact().differential_occurrence();
    let contact = executor.contact_identity(transition.contact())?;
    if kind == RigidLifecycleObservationKind::BeginContact
        && !executor.seen_lifecycle_occurrences.contains(&occurrence)
    {
        executor.seen_lifecycle_occurrences.push(occurrence);
        push_lifecycle(
            executor,
            RigidLifecycleObservationKind::ContactCreated,
            Some(contact.clone()),
            None,
        )?;
    }
    push_lifecycle(executor, kind, Some(contact), None)
}

fn captures_lifecycle_evidence(family: RigidWorldWitnessFamily) -> bool {
    matches!(
        family,
        RigidWorldWitnessFamily::ContactFilterListenerAndPreSolveTiming
            | RigidWorldWitnessFamily::DestructionListenerAndDependencyCascades
    )
}

fn push_lifecycle(
    executor: &mut TimelineExecutor,
    kind: RigidLifecycleObservationKind,
    maybe_contact: Option<liquidfun_test_protocol::RigidContactIdentity>,
    maybe_entity_id: Option<ScenarioId>,
) -> Result<(), NativeRigidWorldError> {
    let ordinal = executor.next_lifecycle_ordinal;
    executor.next_lifecycle_ordinal =
        ordinal
            .checked_add(1)
            .ok_or_else(|| NativeRigidWorldError::Declaration {
                checkpoint_id: "lifecycle-ordinal".into(),
                message: "lifecycle ordinal exceeded the protocol representation".into(),
            })?;
    executor
        .semantic_observations
        .push(RigidWorldObservation::Lifecycle {
            event: RigidLifecycleObservation {
                ordinal,
                kind,
                maybe_contact,
                maybe_entity_id,
            },
        });
    Ok(())
}

pub(super) fn remove_destroyed_mapping(executor: &mut TimelineExecutor, destroyed: DestroyedId) {
    if let DestroyedId::Joint(joint) = destroyed {
        executor.joints.retain(|(_, value)| *value != joint);
    }
}

fn semantic_destroyed(
    executor: &TimelineExecutor,
    destroyed: DestroyedId,
) -> Result<ScenarioId, NativeRigidWorldError> {
    match destroyed {
        DestroyedId::Body(id) => executor.semantic_body(id),
        DestroyedId::Fixture(id) => executor.semantic_fixture(id),
        DestroyedId::Joint(id) => executor.semantic_joint(id),
        _ => Err(NativeRigidWorldError::Declaration {
            checkpoint_id: "lifecycle-map".into(),
            message: "unsupported lifecycle entity".into(),
        }),
    }
}

fn directive_fixtures(
    executor: &TimelineExecutor,
    target: &RigidContactDirectiveTarget,
    action: &RigidWorldActionRecord,
) -> Result<[FixtureId; 2], NativeRigidWorldError> {
    Ok([
        executor.fixture(&target.fixture_a_id, action)?,
        executor.fixture(&target.fixture_b_id, action)?,
    ])
}

fn pre_solve_directive(
    directive: RigidPreSolveDirective,
    action: &RigidWorldActionRecord,
) -> Result<PreSolveDirective, NativeRigidWorldError> {
    let mut value = if directive.enabled {
        PreSolveDirective::Enable
    } else {
        PreSolveDirective::Disable
    };
    if let Some(bits) = directive.maybe_friction_bits {
        value = value
            .with_friction(bits.to_f32())
            .map_err(|error| action_error(action, error))?;
    }
    if let Some(bits) = directive.maybe_restitution_bits {
        value = value
            .with_restitution(bits.to_f32())
            .map_err(|error| action_error(action, error))?;
    }
    if let Some(bits) = directive.maybe_tangent_speed_bits {
        value = value
            .with_tangent_speed(bits.to_f32())
            .map_err(|error| action_error(action, error))?;
    }
    Ok(value)
}

fn upsert_pair<T: Copy>(
    entries: &mut Vec<(FixtureId, FixtureId, T)>,
    a: FixtureId,
    b: FixtureId,
    value: T,
) {
    if let Some(entry) = entries
        .iter_mut()
        .find(|(x, y, _)| same_pair([*x, *y], [a, b]))
    {
        entry.2 = value;
    } else {
        entries.push((a, b, value));
    }
}

fn pair_value<T>(entries: &[(FixtureId, FixtureId, T)], pair: [FixtureId; 2]) -> Option<&T> {
    entries
        .iter()
        .find_map(|(a, b, value)| same_pair([*a, *b], pair).then_some(value))
}

fn same_pair(first: [FixtureId; 2], second: [FixtureId; 2]) -> bool {
    first == second || first == [second[1], second[0]]
}
