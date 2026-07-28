use super::staging::{
    stage_distance, stage_friction, stage_gear, stage_motor, stage_mouse, stage_prismatic,
    stage_pulley, stage_revolute, stage_rope, stage_weld, stage_wheel,
};
use super::{
    ContactSolveFailure, FamilyCandidate, JointConstraintInput, JointDef, JointRuntime,
    JointSolverLanes, JointVelocityConstraint, SolverBody,
};

#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive construction match keeps all eleven typed family pairings auditable"
)]
pub(crate) fn build_constraints(
    inputs: &[JointConstraintInput],
    bodies: &[SolverBody],
    time_step: f32,
    time_step_ratio: f32,
    warm_starting: bool,
) -> Result<Vec<JointVelocityConstraint>, ContactSolveFailure> {
    let mut constraints = Vec::new();
    constraints.try_reserve_exact(inputs.len()).map_err(|_| {
        ContactSolveFailure::CapacityExceeded {
            resource: "island joint constraints",
            limit: inputs.len(),
        }
    })?;
    for input in inputs {
        constraints.push(match (input.definition, input.runtime, input.lanes) {
            (
                JointDef::Revolute(definition),
                JointRuntime::Revolute(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Revolute(stage_revolute(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Prismatic(definition),
                JointRuntime::Prismatic(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Prismatic(stage_prismatic(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Distance(definition),
                JointRuntime::Distance(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Distance(stage_distance(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Pulley(definition),
                JointRuntime::Pulley(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Pulley(stage_pulley(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Mouse(definition),
                JointRuntime::Mouse(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Mouse(stage_mouse(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Gear(definition),
                JointRuntime::Gear(runtime),
                JointSolverLanes::Gear(lanes),
            ) => JointVelocityConstraint::Gear(stage_gear(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                warm_starting,
            )?),
            (
                JointDef::Wheel(definition),
                JointRuntime::Wheel(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Wheel(stage_wheel(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Weld(definition),
                JointRuntime::Weld(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Weld(stage_weld(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Friction(definition),
                JointRuntime::Friction(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Friction(stage_friction(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Rope(definition),
                JointRuntime::Rope(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Rope(stage_rope(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            (
                JointDef::Motor(definition),
                JointRuntime::Motor(runtime),
                JointSolverLanes::Ordinary(lanes),
            ) => JointVelocityConstraint::Motor(stage_motor(
                FamilyCandidate {
                    joint_id: input.joint_id,
                    lanes,
                    definition,
                    runtime,
                },
                bodies,
                time_step,
                time_step_ratio,
                warm_starting,
            )?),
            _ => return Err(ContactSolveFailure::UnsupportedTopology),
        });
    }
    Ok(constraints)
}
