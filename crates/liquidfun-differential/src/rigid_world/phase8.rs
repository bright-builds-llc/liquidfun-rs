//! Phase 8 joints, rope, callbacks, lifecycle, and semantic diagnostics.

mod callbacks;
mod inspection;
mod lifecycle;
mod mutation;
mod rope;

use inspection::{observe_diagnostics, observe_reconstruction};
use rope::{create_rope, observe_rope, rope_mut};

use liquidfun::rope::RopeIterations;
use liquidfun::{
    DestroyedId, JointDef, LifecycleEvent, StepConfiguration, StepError, StepLimits, StepReport,
};
use liquidfun::{
    DistanceJointDef, FrictionJointDef, GearJointDef, MotorJointDef, MouseJointDef,
    PrismaticJointDef, PulleyJointDef, RevoluteJointDef, RopeJointDef, WeldJointDef, WheelJointDef,
};
use liquidfun_test_protocol::{
    RigidJointDeclaration, RigidJointDefinition, RigidWorldAction, RigidWorldActionRecord,
    RigidWorldObservation, RigidWorldTimeline, ScenarioId,
};

use super::{NativeRigidWorldError, TimelineExecutor, action_error, vec2};

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
            mutation::mutate_joint(
                executor,
                executor.joint(joint_id, action)?,
                *mutation,
                action,
            )?;
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
    lifecycle::collect_mutation_lifecycle(executor, report.lifecycle())?;
    for record in &report {
        lifecycle::remove_destroyed_mapping(executor, record.destroyed());
    }
    Ok(())
}

pub(super) fn collect_step_lifecycle(
    executor: &mut TimelineExecutor,
    report: &StepReport,
) -> Result<(), NativeRigidWorldError> {
    lifecycle::collect_step_lifecycle(executor, report)
}

use callbacks::{directive_fixtures, pre_solve_directive, refresh_filter_pair, upsert_pair};

pub(super) fn step(
    executor: &mut TimelineExecutor,
    configuration: StepConfiguration,
    limits: StepLimits,
) -> Result<StepReport, StepError> {
    callbacks::step(executor, configuration, limits)
}

pub(super) fn collect_mutation_lifecycle(
    executor: &mut TimelineExecutor,
    events: &[LifecycleEvent],
) -> Result<(), NativeRigidWorldError> {
    lifecycle::collect_mutation_lifecycle(executor, events)
}

pub(super) fn remove_destroyed_mapping(executor: &mut TimelineExecutor, destroyed: DestroyedId) {
    lifecycle::remove_destroyed_mapping(executor, destroyed);
}
