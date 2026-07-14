//! Source-ordered wheel-joint state and checked world authority.

use std::f32::consts::TAU;

use crate::math::Vec2;
use crate::{
    JointDef, JointId, JointKind, JointMutationError, JointQueryError, JointSnapshot,
    JointSpecificSnapshot, WheelJointDef, WheelJointSnapshot,
};

use super::{JointRecord, JointRuntime};
use crate::world::object::World;

#[derive(Debug, Clone, Copy)]
pub(crate) struct WheelRuntime {
    line_impulse: f32,
    motor_impulse: f32,
    spring_impulse: f32,
    axis: Vec2,
    perpendicular: Vec2,
    line_mass: f32,
    motor_mass: f32,
    spring_mass: f32,
    gamma: f32,
    bias: f32,
}

impl WheelRuntime {
    pub(super) fn new(definition: WheelJointDef) -> Self {
        let axis = definition.local_axis_a();
        Self {
            line_impulse: 0.0,
            motor_impulse: 0.0,
            spring_impulse: 0.0,
            axis,
            perpendicular: Vec2::scalar_cross(1.0, axis),
            line_mass: 0.0,
            motor_mass: 0.0,
            spring_mass: 0.0,
            gamma: 0.0,
            bias: 0.0,
        }
    }

    pub(super) fn reaction_force(self, inverse_timestep: f32) -> Vec2 {
        inverse_timestep
            * (self.line_impulse * self.perpendicular + self.spring_impulse * self.axis)
    }

    pub(super) fn reaction_torque(self, inverse_timestep: f32) -> f32 {
        inverse_timestep * self.motor_impulse
    }

    #[allow(
        clippy::too_many_arguments,
        dead_code,
        reason = "matches one pinned solver initialization"
    )]
    pub(super) fn initialize(
        &mut self,
        definition: WheelJointDef,
        timestep: f32,
        inverse_mass_a: f32,
        inverse_mass_b: f32,
        inverse_inertia_a: f32,
        inverse_inertia_b: f32,
        line_lever_a: f32,
        line_lever_b: f32,
        spring_lever_a: f32,
        spring_lever_b: f32,
        spring_translation: f32,
        world_axis: Vec2,
        maybe_warm_start_ratio: Option<f32>,
    ) -> Result<(), JointMutationError> {
        let previous = *self;
        let values = [
            timestep,
            inverse_mass_a,
            inverse_mass_b,
            inverse_inertia_a,
            inverse_inertia_b,
            line_lever_a,
            line_lever_b,
            spring_lever_a,
            spring_lever_b,
            spring_translation,
        ];
        if values.iter().any(|value| !value.is_finite())
            || timestep < 0.0
            || inverse_mass_a < 0.0
            || inverse_mass_b < 0.0
            || inverse_inertia_a < 0.0
            || inverse_inertia_b < 0.0
            || !world_axis.is_valid()
        {
            return Err(JointMutationError::InvalidValue);
        }

        self.axis = world_axis;
        self.perpendicular = Vec2::scalar_cross(1.0, world_axis);
        let line_inverse_mass = inverse_mass_a
            + inverse_mass_b
            + inverse_inertia_a * line_lever_a * line_lever_a
            + inverse_inertia_b * line_lever_b * line_lever_b;
        self.line_mass = invert_positive(line_inverse_mass);

        self.spring_mass = 0.0;
        self.gamma = 0.0;
        self.bias = 0.0;
        if definition.frequency() > 0.0 {
            let spring_inverse_mass = inverse_mass_a
                + inverse_mass_b
                + inverse_inertia_a * spring_lever_a * spring_lever_a
                + inverse_inertia_b * spring_lever_b * spring_lever_b;
            if spring_inverse_mass > 0.0 {
                let spring_mass = 1.0 / spring_inverse_mass;
                let omega = TAU * definition.frequency();
                let damping = 2.0 * spring_mass * definition.damping_ratio() * omega;
                let stiffness = spring_mass * omega * omega;
                self.gamma = timestep * (damping + timestep * stiffness);
                self.gamma = invert_positive(self.gamma);
                self.bias = spring_translation * timestep * stiffness * self.gamma;
                self.spring_mass = invert_positive(spring_inverse_mass + self.gamma);
            }
        } else {
            self.spring_impulse = 0.0;
        }

        self.motor_mass = if definition.is_motor_enabled() {
            invert_positive(inverse_inertia_a + inverse_inertia_b)
        } else {
            self.motor_impulse = 0.0;
            0.0
        };

        match maybe_warm_start_ratio {
            Some(ratio) if ratio.is_finite() && ratio >= 0.0 => {
                self.line_impulse *= ratio;
                self.spring_impulse *= ratio;
                self.motor_impulse *= ratio;
            }
            Some(_) => {
                *self = previous;
                return Err(JointMutationError::InvalidValue);
            }
            None => {
                self.line_impulse = 0.0;
                self.spring_impulse = 0.0;
                self.motor_impulse = 0.0;
            }
        }
        if !self.is_valid() {
            *self = previous;
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok(())
    }

    #[allow(dead_code, reason = "used by the mixed-island integration plan")]
    pub(super) fn solve_spring(&mut self, relative_speed: f32) -> Result<f32, JointMutationError> {
        if !relative_speed.is_finite() {
            return Err(JointMutationError::InvalidValue);
        }
        let applied =
            -self.spring_mass * (relative_speed + self.bias + self.gamma * self.spring_impulse);
        let candidate = self.spring_impulse + applied;
        if !candidate.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        self.spring_impulse = candidate;
        Ok(applied)
    }

    #[allow(dead_code, reason = "used by the mixed-island integration plan")]
    pub(super) fn solve_motor(
        &mut self,
        definition: WheelJointDef,
        timestep: f32,
        relative_speed: f32,
    ) -> Result<f32, JointMutationError> {
        if !timestep.is_finite() || timestep < 0.0 || !relative_speed.is_finite() {
            return Err(JointMutationError::InvalidValue);
        }
        let impulse = -self.motor_mass * (relative_speed - definition.motor_speed());
        let max_impulse = timestep * definition.max_motor_torque();
        let candidate = (self.motor_impulse + impulse).clamp(-max_impulse, max_impulse);
        if !candidate.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        let applied = candidate - self.motor_impulse;
        self.motor_impulse = candidate;
        Ok(applied)
    }

    #[allow(dead_code, reason = "used by the mixed-island integration plan")]
    pub(super) fn solve_line(&mut self, relative_speed: f32) -> Result<f32, JointMutationError> {
        if !relative_speed.is_finite() {
            return Err(JointMutationError::InvalidValue);
        }
        let applied = -self.line_mass * relative_speed;
        let candidate = self.line_impulse + applied;
        if !candidate.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        self.line_impulse = candidate;
        Ok(applied)
    }

    #[allow(dead_code, reason = "used by the mixed-island integration plan")]
    pub(super) fn solve_position(
        perpendicular_error: f32,
        inverse_effective_mass: f32,
    ) -> Result<(f32, bool), JointMutationError> {
        if !perpendicular_error.is_finite()
            || !inverse_effective_mass.is_finite()
            || inverse_effective_mass < 0.0
        {
            return Err(JointMutationError::InvalidValue);
        }
        let impulse = if inverse_effective_mass == 0.0 {
            0.0
        } else {
            -perpendicular_error / inverse_effective_mass
        };
        if !impulse.is_finite() {
            return Err(JointMutationError::NonFiniteDerivedState);
        }
        Ok((
            impulse,
            perpendicular_error.abs() <= crate::math::settings::LINEAR_SLOP,
        ))
    }

    fn is_valid(self) -> bool {
        self.axis.is_valid()
            && self.perpendicular.is_valid()
            && self.line_impulse.is_finite()
            && self.motor_impulse.is_finite()
            && self.spring_impulse.is_finite()
            && self.line_mass.is_finite()
            && self.motor_mass.is_finite()
            && self.spring_mass.is_finite()
            && self.gamma.is_finite()
            && self.bias.is_finite()
    }
}

pub(super) fn snapshot(
    world: &World,
    record: &JointRecord,
    runtime: WheelRuntime,
) -> Result<JointSnapshot, JointQueryError> {
    let JointDef::Wheel(definition) = record.definition else {
        return Err(wrong_kind(record.definition));
    };
    let body_a = world.bodies.get(record.bodies[0])?.state.snapshot();
    let body_b = world.bodies.get(record.bodies[1])?.state.snapshot();
    let transform_a = body_a.transform();
    let transform_b = body_b.transform();
    let anchor_a = transform_a.apply(definition.local_anchor_a());
    let anchor_b = transform_b.apply(definition.local_anchor_b());
    let axis = transform_a.rotation().apply(definition.local_axis_a());
    let translation = (anchor_b - anchor_a).dot(axis);
    let speed = body_b.angular_velocity() - body_a.angular_velocity();
    if !anchor_a.is_valid()
        || !anchor_b.is_valid()
        || !translation.is_finite()
        || !speed.is_finite()
    {
        return Err(JointQueryError::NonFiniteDerivedState);
    }
    Ok(
        JointSnapshot::from_definition(record.definition, record.bodies).with_runtime(
            anchor_a,
            anchor_b,
            JointSpecificSnapshot::Wheel(WheelJointSnapshot::new(
                translation,
                speed,
                definition.is_motor_enabled(),
                definition.motor_speed(),
                definition.max_motor_torque(),
                definition.frequency(),
                definition.damping_ratio(),
                runtime.gamma,
                runtime.bias,
            )),
        ),
    )
}

impl World {
    /// Returns the current wheel translation along body A's suspension axis.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid, poisoned, or wrong-kind joint.
    pub fn wheel_joint_translation(&self, joint: JointId) -> Result<f32, JointQueryError> {
        let snapshot = self.joint_snapshot_of_kind(joint, JointKind::Wheel)?;
        let JointSpecificSnapshot::Wheel(state) = snapshot.specific() else {
            return Err(wrong_kind(snapshot.definition()));
        };
        Ok(state.translation())
    }

    /// Returns the current relative angular wheel speed.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid, poisoned, or wrong-kind joint.
    pub fn wheel_joint_speed(&self, joint: JointId) -> Result<f32, JointQueryError> {
        let snapshot = self.joint_snapshot_of_kind(joint, JointKind::Wheel)?;
        let JointSpecificSnapshot::Wheel(state) = snapshot.specific() else {
            return Err(wrong_kind(snapshot.definition()));
        };
        Ok(state.speed())
    }

    /// Returns cached motor torque for an explicit inverse timestep.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid joint or inverse timestep.
    pub fn wheel_motor_torque(
        &self,
        joint: JointId,
        inverse_timestep: f32,
    ) -> Result<f32, JointQueryError> {
        self.validate_reaction_query(joint, inverse_timestep)?;
        let record = wheel_record(self, joint)?;
        let JointRuntime::Wheel(runtime) = record.runtime else {
            return Err(wrong_kind(record.definition));
        };
        Ok(inverse_timestep * runtime.motor_impulse)
    }

    /// Enables or disables the wheel motor and unconditionally wakes both bodies.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid identity, kind, lock, or poison.
    pub fn set_wheel_motor_enabled(
        &mut self,
        joint: JointId,
        enabled: bool,
    ) -> Result<(), JointMutationError> {
        self.mutate_wheel(joint, true, |definition| {
            definition.with_motor(
                enabled,
                definition.motor_speed(),
                definition.max_motor_torque(),
            )
        })
    }

    /// Sets motor speed and unconditionally wakes both bodies.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid value, identity, kind, lock, or poison.
    pub fn set_wheel_motor_speed(
        &mut self,
        joint: JointId,
        speed: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_wheel(joint, true, |definition| {
            definition.with_motor(
                definition.is_motor_enabled(),
                speed,
                definition.max_motor_torque(),
            )
        })
    }

    /// Sets maximum motor torque and unconditionally wakes both bodies.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid value, identity, kind, lock, or poison.
    pub fn set_wheel_max_motor_torque(
        &mut self,
        joint: JointId,
        torque: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_wheel(joint, true, |definition| {
            definition.with_motor(
                definition.is_motor_enabled(),
                definition.motor_speed(),
                torque,
            )
        })
    }

    /// Sets spring frequency without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid value, identity, kind, lock, or poison.
    pub fn set_wheel_frequency(
        &mut self,
        joint: JointId,
        frequency: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_wheel(joint, false, |definition| {
            definition.with_spring(frequency, definition.damping_ratio())
        })
    }

    /// Sets spring damping ratio without waking either body.
    ///
    /// # Errors
    ///
    /// Returns a no-effect error for invalid value, identity, kind, lock, or poison.
    pub fn set_wheel_damping_ratio(
        &mut self,
        joint: JointId,
        damping_ratio: f32,
    ) -> Result<(), JointMutationError> {
        self.mutate_wheel(joint, false, |definition| {
            definition.with_spring(definition.frequency(), damping_ratio)
        })
    }

    fn mutate_wheel(
        &mut self,
        joint: JointId,
        wake: bool,
        mutate: impl FnOnce(WheelJointDef) -> Result<WheelJointDef, crate::JointDefError>,
    ) -> Result<(), JointMutationError> {
        self.ensure_joint_mutable()?;
        let record = wheel_record(self, joint).map_err(as_mutation_error)?;
        let JointDef::Wheel(definition) = record.definition else {
            return Err(as_mutation_error(wrong_kind(record.definition)));
        };
        let candidate = mutate(definition).map_err(|_| JointMutationError::InvalidValue)?;
        let bodies = record.bodies;
        if wake {
            self.wake_joint_bodies(bodies);
        }
        self.joint_mut_after_validation(joint).definition = candidate.into();
        Ok(())
    }
}

fn invert_positive(value: f32) -> f32 {
    if value > 0.0 { 1.0 / value } else { 0.0 }
}

fn wheel_record(world: &World, joint: JointId) -> Result<&JointRecord, JointQueryError> {
    let record = world.joints.get(joint)?;
    if !matches!(record.definition, JointDef::Wheel(_)) {
        return Err(wrong_kind(record.definition));
    }
    Ok(record)
}

fn wrong_kind(definition: JointDef) -> JointQueryError {
    JointQueryError::WrongKind {
        expected: JointKind::Wheel,
        actual: JointKind::from_definition(definition),
    }
}

fn as_mutation_error(error: JointQueryError) -> JointMutationError {
    match error {
        JointQueryError::InvalidHandle(error) => JointMutationError::InvalidHandle(error),
        JointQueryError::WrongKind { expected, actual } => {
            JointMutationError::WrongKind { expected, actual }
        }
        JointQueryError::Poisoned => JointMutationError::Poisoned,
        JointQueryError::InvalidInverseTimestep | JointQueryError::NonFiniteDerivedState => {
            JointMutationError::InvalidValue
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BodyDef, World};

    fn definition() -> WheelJointDef {
        let mut world = World::new().expect("world");
        let body_a = world.create_body(&BodyDef::default()).expect("A");
        let body_b = world.create_body(&BodyDef::default()).expect("B");
        WheelJointDef::new(body_a, body_b).expect("definition")
    }

    #[test]
    fn initialization_covers_spring_motor_and_warm_start() {
        // Arrange
        let definition = definition()
            .with_motor(true, 4.0, 2.0)
            .expect("motor")
            .with_spring(2.0, 0.7)
            .expect("spring");
        let mut runtime = WheelRuntime::new(definition);
        runtime.line_impulse = 2.0;
        runtime.spring_impulse = 4.0;
        runtime.motor_impulse = 6.0;

        // Act
        runtime
            .initialize(
                definition,
                0.5,
                1.0,
                1.0,
                1.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
                Vec2::new(1.0, 0.0),
                Some(0.5),
            )
            .expect("initialize");

        // Assert
        assert!(runtime.gamma > 0.0);
        assert!(runtime.bias > 0.0);
        assert_eq!(runtime.line_impulse.to_bits(), 1.0_f32.to_bits());
        assert_eq!(runtime.spring_impulse.to_bits(), 2.0_f32.to_bits());
        assert_eq!(runtime.motor_impulse.to_bits(), 3.0_f32.to_bits());
    }

    #[test]
    fn motor_impulse_is_capped_and_zero_frequency_clears_spring_cache() {
        // Arrange
        let definition = definition()
            .with_motor(true, 10.0, 2.0)
            .expect("motor")
            .with_spring(0.0, 0.7)
            .expect("disabled spring");
        let mut runtime = WheelRuntime::new(definition);
        runtime.spring_impulse = 5.0;
        runtime
            .initialize(
                definition,
                0.25,
                1.0,
                1.0,
                1.0,
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                Vec2::new(1.0, 0.0),
                Some(1.0),
            )
            .expect("initialize");

        // Act
        let applied = runtime.solve_motor(definition, 0.25, 0.0).expect("solve");

        // Assert
        assert_eq!(applied.to_bits(), 0.5_f32.to_bits());
        assert_eq!(runtime.spring_impulse.to_bits(), 0.0_f32.to_bits());
    }
}
