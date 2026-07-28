//! Joint mutation dispatch for Phase 8 timelines.

use liquidfun_test_protocol::{RigidJointMutation, RigidWorldActionRecord};

use super::super::{NativeRigidWorldError, TimelineExecutor, action_error, vec2};

pub(super) fn mutate_joint(
    executor: &mut TimelineExecutor,
    joint: liquidfun::JointId,
    mutation: RigidJointMutation,
    action: &RigidWorldActionRecord,
) -> Result<(), NativeRigidWorldError> {
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
