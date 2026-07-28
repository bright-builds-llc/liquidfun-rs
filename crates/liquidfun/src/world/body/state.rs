use super::{
    AggregateMassError, BodyDef, BodyDefError, BodyFlags, BodyMassData, BodySnapshot, BodyState,
    BodyType, MassData, MassState, Sweep, SweepError, Transform, Vec2, aggregate_mass_state,
    checked_finite, initial_body_mass, initial_sweep, validate_body_transform,
};

impl BodyState {
    pub(in crate::world) fn from_definition(definition: &BodyDef) -> Self {
        let position = definition.position();
        let angle = definition.angle();
        let mass = initial_body_mass(definition.body_type());
        Self {
            snapshot: definition.snapshot(),
            transform: definition.transform(),
            sweep: initial_sweep(position, angle),
            linear_velocity: definition.linear_velocity(),
            angular_velocity: definition.angular_velocity(),
            inverse_mass: mass,
            inverse_inertia: 0.0,
            force: Vec2::ZERO,
            torque: 0.0,
            sleep_time: 0.0,
        }
    }

    pub(in crate::world) const fn snapshot(self) -> BodySnapshot {
        self.snapshot
    }

    pub(in crate::world) const fn transform(self) -> Transform {
        self.transform
    }

    pub(in crate::world) const fn sweep(self) -> Sweep {
        self.sweep
    }

    pub(in crate::world) fn candidate_equalize_sweep(
        mut self,
        fraction: f32,
    ) -> Result<Self, SweepError> {
        self.sweep.advance(fraction)?;
        Ok(self)
    }

    pub(in crate::world) fn candidate_advance_to(
        mut self,
        fraction: f32,
    ) -> Result<Self, SweepError> {
        self.sweep.advance(fraction)?;
        let local_center = self.sweep.local_center();
        let center = self.sweep.initial_center();
        let angle = self.sweep.initial_angle();
        self.sweep = Sweep::new(local_center, center, center, angle, angle, fraction)?;
        let transform = self.sweep.transform_at(0.0)?;
        self.transform = transform;
        self.snapshot.position = transform.position();
        self.snapshot.angle = angle;
        Ok(self)
    }

    pub(in crate::world) const fn solver_linear(self) -> Vec2 {
        self.linear_velocity
    }

    pub(in crate::world) const fn solver_angular(self) -> f32 {
        self.angular_velocity
    }

    pub(in crate::world) const fn inverse_mass(self) -> f32 {
        self.inverse_mass
    }

    pub(in crate::world) const fn inverse_inertia(self) -> f32 {
        self.inverse_inertia
    }

    pub(in crate::world) const fn accumulated_force(self) -> Vec2 {
        self.force
    }

    pub(in crate::world) const fn accumulated_torque(self) -> f32 {
        self.torque
    }

    pub(in crate::world) const fn sleep_time(self) -> f32 {
        self.sleep_time
    }

    pub(in crate::world) fn maybe_shifted_origin(self, shift: Vec2) -> Option<Self> {
        let position = self.snapshot.position - shift;
        if !position.is_valid() {
            return None;
        }
        let sweep = Sweep::new(
            self.sweep.local_center(),
            self.sweep.initial_center() - shift,
            self.sweep.center() - shift,
            self.sweep.initial_angle(),
            self.sweep.angle(),
            self.sweep.initial_fraction(),
        )
        .ok()?;

        let mut snapshot = self.snapshot;
        snapshot.position = position;
        Some(Self {
            snapshot,
            transform: Transform::from_position_angle(position, snapshot.angle),
            sweep,
            linear_velocity: self.linear_velocity,
            angular_velocity: self.angular_velocity,
            inverse_mass: self.inverse_mass,
            inverse_inertia: self.inverse_inertia,
            force: self.force,
            torque: self.torque,
            sleep_time: self.sleep_time,
        })
    }

    pub(in crate::world) fn candidate_set_sleep_time(mut self, sleep_time: f32) -> Self {
        self.sleep_time = sleep_time;
        self
    }

    #[cfg(test)]
    pub(in crate::world) fn set_solver_motion(
        &mut self,
        linear_velocity: Vec2,
        angular_velocity: f32,
    ) {
        self.linear_velocity = linear_velocity;
        self.angular_velocity = angular_velocity;
        self.snapshot.linear_velocity = linear_velocity;
        self.snapshot.angular_velocity = angular_velocity;
    }

    pub(in crate::world) fn candidate_set_solver_state(
        self,
        position: Vec2,
        angle: f32,
        linear_velocity: Vec2,
        angular_velocity: f32,
    ) -> Result<Self, BodyDefError> {
        validate_body_transform(position, angle)?;
        if !linear_velocity.is_valid() || !angular_velocity.is_finite() {
            return Err(BodyDefError::NonFiniteDerivedCenter);
        }
        let transform = Transform::from_position_angle(position, angle);
        let mut snapshot = self.snapshot;
        snapshot.position = position;
        snapshot.angle = angle;
        snapshot.linear_velocity = linear_velocity;
        snapshot.angular_velocity = angular_velocity;
        Ok(Self {
            snapshot,
            transform,
            sweep: Sweep::new(
                self.sweep.local_center(),
                self.sweep.center(),
                transform.apply(self.sweep.local_center()),
                self.sweep.angle(),
                angle,
                0.0,
            )
            .map_err(|_error| BodyDefError::NonFiniteDerivedCenter)?,
            linear_velocity,
            angular_velocity,
            inverse_mass: self.inverse_mass,
            inverse_inertia: self.inverse_inertia,
            force: self.force,
            torque: self.torque,
            sleep_time: self.sleep_time,
        })
    }

    pub(in crate::world) fn candidate_set_toi_solver_state(
        self,
        initial_center: Vec2,
        initial_angle: f32,
        position: Vec2,
        angle: f32,
        linear_velocity: Vec2,
        angular_velocity: f32,
    ) -> Result<Self, BodyDefError> {
        validate_body_transform(position, angle)?;
        if !initial_center.is_valid()
            || !initial_angle.is_finite()
            || !linear_velocity.is_valid()
            || !angular_velocity.is_finite()
        {
            return Err(BodyDefError::NonFiniteDerivedCenter);
        }
        let transform = Transform::from_position_angle(position, angle);
        let mut snapshot = self.snapshot;
        snapshot.position = position;
        snapshot.angle = angle;
        snapshot.linear_velocity = linear_velocity;
        snapshot.angular_velocity = angular_velocity;
        Ok(Self {
            snapshot,
            transform,
            sweep: Sweep::new(
                self.sweep.local_center(),
                initial_center,
                transform.apply(self.sweep.local_center()),
                initial_angle,
                angle,
                self.sweep.initial_fraction(),
            )
            .map_err(|_error| BodyDefError::NonFiniteDerivedCenter)?,
            linear_velocity,
            angular_velocity,
            inverse_mass: self.inverse_mass,
            inverse_inertia: self.inverse_inertia,
            force: self.force,
            torque: self.torque,
            sleep_time: self.sleep_time,
        })
    }

    pub(in crate::world) fn with_transform(
        self,
        position: Vec2,
        angle: f32,
    ) -> Result<Self, BodyDefError> {
        validate_body_transform(position, angle)?;
        let transform = Transform::from_position_angle(position, angle);
        let mut snapshot = self.snapshot;
        snapshot.position = position;
        snapshot.angle = angle;
        Ok(Self {
            snapshot,
            transform,
            sweep: Sweep::new(
                self.snapshot.local_center,
                transform.apply(self.snapshot.local_center),
                transform.apply(self.snapshot.local_center),
                angle,
                angle,
                0.0,
            )
            .map_err(|_error| BodyDefError::NonFiniteDerivedCenter)?,
            linear_velocity: self.linear_velocity,
            angular_velocity: self.angular_velocity,
            inverse_mass: self.inverse_mass,
            inverse_inertia: self.inverse_inertia,
            force: self.force,
            torque: self.torque,
            sleep_time: self.sleep_time,
        })
    }

    pub(in crate::world) fn with_body_type_and_reset_mass_data(
        mut self,
        body_type: BodyType,
        fixture_mass_data: &[MassData],
    ) -> Result<Self, AggregateMassError> {
        self.snapshot.body_type = body_type;
        if body_type == BodyType::Static {
            self.linear_velocity = Vec2::ZERO;
            self.angular_velocity = 0.0;
            self.snapshot.linear_velocity = Vec2::ZERO;
            self.snapshot.angular_velocity = 0.0;
        }
        let mass_state = aggregate_mass_state(
            body_type,
            self.snapshot.is_fixed_rotation(),
            fixture_mass_data,
        )?;
        self.with_mass_state(mass_state)
    }

    pub(in crate::world) fn set_active(&mut self, active: bool) {
        self.snapshot.flags.set(BodyFlags::ACTIVE, active);
    }

    pub(in crate::world) fn with_reset_mass_data(
        self,
        fixture_mass_data: &[MassData],
    ) -> Result<Self, AggregateMassError> {
        let mass_state = aggregate_mass_state(
            self.snapshot.body_type,
            self.snapshot.is_fixed_rotation(),
            fixture_mass_data,
        )?;
        self.with_mass_state(mass_state)
    }

    pub(in crate::world) fn with_custom_mass_data(
        self,
        data: BodyMassData,
    ) -> Result<Self, AggregateMassError> {
        if self.snapshot.body_type != BodyType::Dynamic {
            return Ok(self);
        }
        let mass = if data.mass() > 0.0 { data.mass() } else { 1.0 };
        let rotational_inertia =
            if !self.snapshot.is_fixed_rotation() && data.rotational_inertia() > 0.0 {
                data.centered_rotational_inertia()
            } else {
                0.0
            };
        let inverse_mass = checked_finite(1.0 / mass, AggregateMassError::NonFiniteInverseMass)?;
        let inverse_inertia = if rotational_inertia > 0.0 {
            checked_finite(
                1.0 / rotational_inertia,
                AggregateMassError::NonFiniteInverseInertia,
            )?
        } else {
            0.0
        };
        self.with_mass_state(MassState {
            mass,
            local_center: data.center(),
            rotational_inertia,
            inverse_mass,
            inverse_inertia,
        })
    }

    fn with_mass_state(self, mass_state: MassState) -> Result<Self, AggregateMassError> {
        let old_center = self.sweep.center();
        let current_center = self.transform.apply(mass_state.local_center);
        if !current_center.x.is_finite() || !current_center.y.is_finite() {
            return Err(AggregateMassError::NonFiniteDerivedCenter);
        }
        let sweep = Sweep::new(
            mass_state.local_center,
            current_center,
            current_center,
            self.snapshot.angle,
            self.snapshot.angle,
            0.0,
        )
        .map_err(|_error| AggregateMassError::NonFiniteDerivedCenter)?;
        let linear_velocity = self.linear_velocity
            + Vec2::scalar_cross(self.angular_velocity, current_center - old_center);
        if !linear_velocity.x.is_finite() || !linear_velocity.y.is_finite() {
            return Err(AggregateMassError::NonFiniteDerivedVelocity);
        }

        let mut snapshot = self.snapshot;
        snapshot.mass = mass_state.mass;
        snapshot.local_center = mass_state.local_center;
        snapshot.rotational_inertia = mass_state.rotational_inertia;
        snapshot.linear_velocity = linear_velocity;
        Ok(Self {
            snapshot,
            transform: self.transform,
            sweep,
            linear_velocity,
            angular_velocity: self.angular_velocity,
            inverse_mass: mass_state.inverse_mass,
            inverse_inertia: mass_state.inverse_inertia,
            force: self.force,
            torque: self.torque,
            sleep_time: self.sleep_time,
        })
    }
}
