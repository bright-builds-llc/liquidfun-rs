//! Semantic object construction and destruction helpers.

use liquidfun::collision::shape::CircleShape;
use liquidfun::collision::{FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::rope::{Rope, RopeDef, RopeIterations};
use liquidfun::{BodyDef, BodyType, FixtureDef, RevoluteJointDef};
use liquidfun_test_protocol::{RigidJointMutation, RigidWorldAction, ScenarioId};

use crate::SessionBackendError;

use super::{NativeSession, action_failure, protocol_failure};

impl NativeSession {
    pub(super) fn create_body(&mut self, id: &ScenarioId) -> Result<(), SessionBackendError> {
        if self.bodies.iter().any(|(candidate, _)| candidate == id) {
            return Err(protocol_failure());
        }
        let ordinal = u16::try_from(self.bodies.len()).map_err(|_error| action_failure())?;
        let position = Vec2::new(f32::from(ordinal) * 0.75, 0.0);
        let definition = BodyDef::new(BodyType::Dynamic, position, 0.0, true)
            .map_err(|_error| action_failure())?;
        let body = self
            .world
            .create_body(&definition)
            .map_err(|_error| action_failure())?;
        self.bodies.push((id.clone(), body));
        Ok(())
    }

    pub(super) fn create_fixture(&mut self, id: &ScenarioId) -> Result<(), SessionBackendError> {
        if self.fixtures.iter().any(|(candidate, _)| candidate == id) {
            return Err(protocol_failure());
        }
        let body = self
            .bodies
            .get(self.fixtures.len() % self.bodies.len().max(1))
            .map(|(_, body)| *body)
            .ok_or_else(action_failure)?;
        let shape = CircleShape::new(Vec2::ZERO, 0.5).map_err(|_error| action_failure())?;
        let definition = FixtureDef::new(
            Shape::from(shape),
            1.0,
            0.3,
            0.0,
            false,
            FilterData::default(),
        )
        .map_err(|_error| action_failure())?;
        let fixture = self
            .world
            .create_fixture(body, &definition)
            .map_err(|_error| action_failure())?;
        self.fixtures.push((id.clone(), fixture));
        Ok(())
    }

    pub(super) fn destroy_fixture(&mut self, id: &ScenarioId) -> Result<(), SessionBackendError> {
        let fixture = self.fixture(id)?;
        self.world
            .destroy_fixture(fixture)
            .map_err(|_error| action_failure())?;
        self.fixtures.retain(|(_, candidate)| *candidate != fixture);
        Ok(())
    }

    pub(super) fn destroy_body(&mut self, id: &ScenarioId) -> Result<(), SessionBackendError> {
        let body = self.body(id)?;
        self.world
            .destroy_body(body)
            .map_err(|_error| action_failure())?;
        self.bodies.retain(|(_, candidate)| *candidate != body);
        self.fixtures
            .retain(|(_, fixture)| self.world.contains_fixture(*fixture));
        self.joints
            .retain(|(_, joint)| self.world.contains_joint(*joint));
        Ok(())
    }

    pub(super) fn execute_object(
        &mut self,
        action: &RigidWorldAction,
    ) -> Result<(), SessionBackendError> {
        match action {
            RigidWorldAction::CreateJoint { joint_id } => self.create_joint(joint_id),
            RigidWorldAction::InspectJoint { joint_id } => self
                .world
                .joint_snapshot(self.joint(joint_id)?)
                .map(|_snapshot| ())
                .map_err(|_error| action_failure()),
            RigidWorldAction::MutateJoint { joint_id, mutation } => {
                self.mutate_joint(joint_id, *mutation)
            }
            RigidWorldAction::DestroyJoint { joint_id } => {
                let joint = self.joint(joint_id)?;
                self.world
                    .destroy_joint(joint)
                    .map_err(|_error| action_failure())?;
                self.joints.retain(|(_, candidate)| *candidate != joint);
                Ok(())
            }
            RigidWorldAction::CreateRope { rope_id } => self.create_rope(rope_id),
            RigidWorldAction::SetRopeAngle {
                rope_id,
                angle_bits,
            } => self
                .rope_mut(rope_id)?
                .set_angle(angle_bits.to_f32())
                .map_err(|_error| action_failure()),
            RigidWorldAction::StepRope {
                rope_id,
                timestep_bits,
                iterations,
            } => {
                let iterations = usize::try_from(*iterations)
                    .ok()
                    .and_then(|value| RopeIterations::new(value).ok())
                    .ok_or_else(action_failure)?;
                self.rope_mut(rope_id)?
                    .step(timestep_bits.to_f32(), iterations)
                    .map_err(|_error| action_failure())?;
                self.simulation_time += timestep_bits.to_f32();
                Ok(())
            }
            RigidWorldAction::InspectRope { rope_id } => {
                self.rope_mut(rope_id)?;
                Ok(())
            }
            RigidWorldAction::DestroyRope { rope_id } => {
                let position = self
                    .ropes
                    .iter()
                    .position(|(candidate, _)| candidate == rope_id)
                    .ok_or_else(action_failure)?;
                self.ropes.remove(position);
                Ok(())
            }
            _ => Err(action_failure()),
        }
    }

    fn create_joint(&mut self, id: &ScenarioId) -> Result<(), SessionBackendError> {
        if self.joints.iter().any(|(candidate, _)| candidate == id) {
            return Err(protocol_failure());
        }
        let [body_a, body_b] = self
            .bodies
            .get(0..2)
            .and_then(|bodies| match bodies {
                [(_, body_a), (_, body_b)] => Some([*body_a, *body_b]),
                _ => None,
            })
            .ok_or_else(action_failure)?;
        let definition = RevoluteJointDef::new(body_a, body_b)
            .map_err(|_error| action_failure())?
            .into();
        let joint = self
            .world
            .create_joint(definition)
            .map_err(|_error| action_failure())?;
        self.joints.push((id.clone(), joint));
        Ok(())
    }

    fn mutate_joint(
        &mut self,
        id: &ScenarioId,
        mutation: RigidJointMutation,
    ) -> Result<(), SessionBackendError> {
        let joint = self.joint(id)?;
        match mutation {
            RigidJointMutation::LimitEnabled { enabled } => self
                .world
                .set_revolute_limit_enabled(joint, enabled)
                .map_err(|_error| action_failure()),
            RigidJointMutation::Limits {
                lower_bits,
                upper_bits,
            } => self
                .world
                .set_revolute_limits(joint, lower_bits.to_f32(), upper_bits.to_f32())
                .map_err(|_error| action_failure()),
            RigidJointMutation::MotorEnabled { enabled } => self
                .world
                .set_revolute_motor_enabled(joint, enabled)
                .map_err(|_error| action_failure()),
            RigidJointMutation::MotorSpeed { speed_bits } => self
                .world
                .set_revolute_motor_speed(joint, speed_bits.to_f32())
                .map_err(|_error| action_failure()),
            RigidJointMutation::MaxMotorTorque { torque_bits } => self
                .world
                .set_revolute_max_motor_torque(joint, torque_bits.to_f32())
                .map_err(|_error| action_failure()),
            _ => Ok(()),
        }
    }

    fn create_rope(&mut self, id: &ScenarioId) -> Result<(), SessionBackendError> {
        if self.ropes.iter().any(|(candidate, _)| candidate == id) {
            return Err(protocol_failure());
        }
        let rope = RopeDef::new(
            vec![
                Vec2::new(0.0, 2.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 0.0),
            ],
            vec![0.0, 1.0, 1.0],
            Vec2::new(0.0, -10.0),
            0.1,
            1.0,
            0.5,
        )
        .and_then(Rope::new)
        .map_err(|_error| action_failure())?;
        self.ropes.push((id.clone(), rope));
        Ok(())
    }
}
