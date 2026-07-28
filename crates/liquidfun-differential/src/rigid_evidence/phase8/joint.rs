//! Native joint snapshot projection into the rigid evidence contract.

use liquidfun::{JointId, JointKind, JointLimitState, JointSnapshot, JointSpecificSnapshot};
use liquidfun_test_protocol::{
    FloatBits, RigidJointBranchState, RigidJointKind, RigidJointSnapshot, ScenarioId,
};

use crate::rigid_world::{NativeRigidWorldError, TimelineExecutor};

pub(crate) fn joint_observation(
    executor: &TimelineExecutor,
    joint_id: &ScenarioId,
    joint: JointId,
) -> Result<RigidJointSnapshot, NativeRigidWorldError> {
    let snapshot = executor.world.joint_snapshot(joint).map_err(|error| {
        NativeRigidWorldError::Declaration {
            checkpoint_id: joint_id.as_str().into(),
            message: error.to_string().into(),
        }
    })?;
    let [body_a, body_b] = snapshot.bodies();
    let inverse_timestep = 1.0 / f32::from_bits(liquidfun_test_protocol::RIGID_WORLD_TIMESTEP_BITS);
    let dependencies = dependencies(executor, &snapshot)?;
    let (branch_state, coordinate, speed) = joint_state(&snapshot);
    Ok(RigidJointSnapshot {
        joint_id: joint_id.clone(),
        joint_kind: joint_kind(snapshot.kind()),
        body_a_id: executor.semantic_body(body_a)?,
        body_b_id: executor.semantic_body(body_b)?,
        collide_connected: snapshot.collide_connected(),
        dependencies: dependencies.into_boxed_slice(),
        branch_state,
        coordinate_bits: FloatBits::from_f32(coordinate),
        speed_bits: FloatBits::from_f32(speed),
        reaction_force: {
            let value = executor
                .world
                .joint_reaction_force(joint, inverse_timestep)
                .map_err(|error| observation_error(joint_id, error))?;
            liquidfun_test_protocol::Vec2Bits {
                x_bits: FloatBits::from_f32(value.x),
                y_bits: FloatBits::from_f32(value.y),
            }
        },
        reaction_torque_bits: FloatBits::from_f32(
            executor
                .world
                .joint_reaction_torque(joint, inverse_timestep)
                .map_err(|error| observation_error(joint_id, error))?,
        ),
    })
}

fn dependencies(
    executor: &TimelineExecutor,
    snapshot: &JointSnapshot,
) -> Result<Vec<ScenarioId>, NativeRigidWorldError> {
    let JointSpecificSnapshot::Gear(gear) = snapshot.specific() else {
        return Ok(Vec::new());
    };
    gear.source_joints()
        .into_iter()
        .map(|joint| executor.semantic_joint(joint))
        .collect()
}

fn joint_state(snapshot: &JointSnapshot) -> (RigidJointBranchState, f32, f32) {
    match snapshot.specific() {
        JointSpecificSnapshot::Revolute(state) => (
            branch_state(state.limit_state()),
            state.angle(),
            state.speed(),
        ),
        JointSpecificSnapshot::Prismatic(state) => (
            branch_state(state.limit_state()),
            state.translation(),
            state.speed(),
        ),
        JointSpecificSnapshot::Distance(state) => {
            (RigidJointBranchState::Inactive, state.current_length(), 0.0)
        }
        JointSpecificSnapshot::Pulley(state) => (
            RigidJointBranchState::Inactive,
            state.current_length_a() + state.ratio() * state.current_length_b(),
            0.0,
        ),
        JointSpecificSnapshot::Mouse(_) | JointSpecificSnapshot::Friction(_) => {
            (RigidJointBranchState::Inactive, 0.0, 0.0)
        }
        JointSpecificSnapshot::Gear(state) => (
            RigidJointBranchState::Active,
            state.coordinate1() + state.ratio() * state.coordinate2(),
            0.0,
        ),
        JointSpecificSnapshot::Wheel(state) => (
            RigidJointBranchState::Active,
            state.translation(),
            state.speed(),
        ),
        JointSpecificSnapshot::Weld(_) => (RigidJointBranchState::Active, 0.0, 0.0),
        JointSpecificSnapshot::Rope(state) => (
            branch_state(state.limit_state()),
            state.current_length(),
            0.0,
        ),
        JointSpecificSnapshot::Motor(state) => {
            (RigidJointBranchState::Active, state.angular_error(), 0.0)
        }
        JointSpecificSnapshot::Pending => (RigidJointBranchState::Inactive, 0.0, 0.0),
    }
}

const fn branch_state(state: JointLimitState) -> RigidJointBranchState {
    match state {
        JointLimitState::Inactive => RigidJointBranchState::Inactive,
        JointLimitState::AtLower => RigidJointBranchState::AtLower,
        JointLimitState::AtUpper => RigidJointBranchState::AtUpper,
        JointLimitState::Equal => RigidJointBranchState::Equal,
    }
}

pub(crate) const fn joint_kind(kind: JointKind) -> RigidJointKind {
    match kind {
        JointKind::Revolute => RigidJointKind::Revolute,
        JointKind::Prismatic => RigidJointKind::Prismatic,
        JointKind::Distance => RigidJointKind::Distance,
        JointKind::Pulley => RigidJointKind::Pulley,
        JointKind::Mouse => RigidJointKind::Mouse,
        JointKind::Gear => RigidJointKind::Gear,
        JointKind::Wheel => RigidJointKind::Wheel,
        JointKind::Weld => RigidJointKind::Weld,
        JointKind::Friction => RigidJointKind::Friction,
        JointKind::Rope => RigidJointKind::Rope,
        JointKind::Motor => RigidJointKind::Motor,
    }
}

fn observation_error(
    joint_id: &ScenarioId,
    error: impl std::fmt::Display,
) -> NativeRigidWorldError {
    NativeRigidWorldError::Declaration {
        checkpoint_id: joint_id.as_str().into(),
        message: error.to_string().into(),
    }
}
