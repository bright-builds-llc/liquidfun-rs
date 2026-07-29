use super::{
    Arena, ArenaInsertError, Body, BodyActivationError, BodyControlError, BodyDef, BodyId,
    BodySnapshot, BodyState, BodyTransformError, BodyType, BodyTypeChangeError, ContactManager,
    ContinuousStepState, HandleError, StepState, Vec2, WakePolicy, World, WorldConfiguration,
    WorldConfigurationError, WorldKey, WorldKeyError, new_world_broad_phase,
};

impl World {
    /// Creates an empty world with a process-unique identity scope.
    ///
    /// # Errors
    ///
    /// Returns [`WorldKeyError::Exhausted`] if process-unique world identities are exhausted.
    pub fn new() -> Result<Self, WorldKeyError> {
        let world = WorldKey::fresh()?;
        Ok(Self {
            scope_key: world,
            bodies: Arena::new(world, usize::MAX),
            body_order: Vec::new(),
            fixtures: Arena::new(world, usize::MAX),
            joints: Arena::new(world, usize::MAX),
            particle_systems: Arena::new(world, usize::MAX),
            particle_system_order: Vec::new(),
            particle_groups: Arena::new(world, usize::MAX),
            broad_phase: new_world_broad_phase(),
            contact_manager: ContactManager::new(),
            continuous_step_state: ContinuousStepState::new(),
            next_diagnostic_id: Some(1),
            step_state: StepState::new(),
            configuration: WorldConfiguration::default(),
        })
    }

    pub(in crate::world) fn allocate_diagnostic_id(&mut self) -> Result<u64, ArenaInsertError> {
        let Some(id) = self.next_diagnostic_id else {
            return Err(ArenaInsertError::DiagnosticIdExhausted);
        };
        self.next_diagnostic_id = id.checked_add(1);
        Ok(id)
    }

    pub(in crate::world) fn preflight_diagnostic_ids(
        &self,
        count: usize,
    ) -> Result<(u64, Option<u64>), ArenaInsertError> {
        let Some(first) = self.next_diagnostic_id else {
            return Err(ArenaInsertError::DiagnosticIdExhausted);
        };
        let last_offset = count
            .checked_sub(1)
            .and_then(|offset| u64::try_from(offset).ok())
            .ok_or(ArenaInsertError::DiagnosticIdExhausted)?;
        let last = first
            .checked_add(last_offset)
            .ok_or(ArenaInsertError::DiagnosticIdExhausted)?;
        Ok((first, last.checked_add(1)))
    }

    pub(in crate::world) fn commit_next_diagnostic_id(&mut self, next: Option<u64>) {
        self.next_diagnostic_id = next;
    }

    #[cfg(test)]
    pub(super) fn set_next_diagnostic_id_for_test(&mut self, next: u64) {
        self.next_diagnostic_id = Some(next);
    }

    /// Creates a body from a reusable checked definition.
    ///
    /// # Errors
    ///
    /// Returns an arena error if body storage is exhausted.
    pub fn create_body(&mut self, definition: &BodyDef) -> Result<BodyId, ArenaInsertError> {
        self.ensure_not_poisoned_for_insert()?;
        let diagnostic_id = self.allocate_diagnostic_id()?;
        let body = self.bodies.insert(Body {
            diagnostic_id,
            state: BodyState::from_definition(definition),
            fixtures: Vec::new(),
            joints: Vec::new(),
            contacts: Vec::new(),
            pending_contact_destruction: false,
            pending_wake: false,
        })?;
        self.body_order.insert(0, body);
        self.debug_assert_body_order_invariant();
        Ok(body)
    }

    /// Returns an owned semantic snapshot of a live body.
    ///
    /// # Errors
    ///
    /// Returns a handle error when `body` is foreign, stale, or destroyed.
    pub fn body_snapshot(&self, body: BodyId) -> Result<BodySnapshot, HandleError> {
        self.ensure_not_poisoned_for_handle()?;
        self.bodies.get(body).map(|record| record.state.snapshot())
    }

    /// Clears accumulated force and torque from every live body.
    ///
    /// Use this after an application-managed sequence of sub-steps when
    /// automatic force clearing is disabled.
    ///
    /// # Errors
    ///
    /// Returns a typed no-effect error for a poisoned or locked world.
    pub fn clear_forces(&mut self) -> Result<(), WorldConfigurationError> {
        self.ensure_configuration_mutable()?;
        self.clear_force_accumulators();
        Ok(())
    }

    /// Sets a live body's linear velocity.
    ///
    /// A static body accepts this call without changing state. A nonzero
    /// velocity wakes a non-static body; zero velocity preserves its wake state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle or non-finite velocity.
    pub fn set_body_linear_velocity(
        &mut self,
        body: BodyId,
        velocity: Vec2,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| state.candidate_set_linear_velocity(velocity))
    }

    /// Sets a live body's angular velocity.
    ///
    /// A static body accepts this call without changing state. A nonzero
    /// velocity wakes a non-static body; zero velocity preserves its wake state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle or non-finite velocity.
    pub fn set_body_angular_velocity(
        &mut self,
        body: BodyId,
        angular_velocity: f32,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_set_angular_velocity(angular_velocity)
        })
    }

    /// Applies force at a world point using the requested wake policy.
    ///
    /// Static and kinematic bodies, plus asleep bodies under
    /// [`WakePolicy::PreserveSleep`], accept this call without changing state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle, invalid input, or
    /// non-finite derived accumulation.
    pub fn apply_body_force(
        &mut self,
        body: BodyId,
        force: Vec2,
        point: Vec2,
        wake_policy: WakePolicy,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_apply_force(force, point, wake_policy)
        })
    }

    /// Applies force at a body's center of mass using the requested wake policy.
    ///
    /// Static and kinematic bodies, plus asleep bodies under
    /// [`WakePolicy::PreserveSleep`], accept this call without changing state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle, invalid input, or
    /// non-finite force accumulation.
    pub fn apply_body_force_to_center(
        &mut self,
        body: BodyId,
        force: Vec2,
        wake_policy: WakePolicy,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_apply_force_to_center(force, wake_policy)
        })
    }

    /// Applies torque using the requested wake policy.
    ///
    /// Static and kinematic bodies, plus asleep bodies under
    /// [`WakePolicy::PreserveSleep`], accept this call without changing state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle, invalid input, or
    /// non-finite torque accumulation.
    pub fn apply_body_torque(
        &mut self,
        body: BodyId,
        torque: f32,
        wake_policy: WakePolicy,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_apply_torque(torque, wake_policy)
        })
    }

    /// Applies linear impulse at a world point using the requested wake policy.
    ///
    /// Static and kinematic bodies, plus asleep bodies under
    /// [`WakePolicy::PreserveSleep`], accept this call without changing state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle, invalid input, or
    /// non-finite derived velocity.
    pub fn apply_body_linear_impulse(
        &mut self,
        body: BodyId,
        impulse: Vec2,
        point: Vec2,
        wake_policy: WakePolicy,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_apply_linear_impulse(impulse, point, wake_policy)
        })
    }

    /// Applies linear impulse at a body's center using the requested wake policy.
    ///
    /// Static and kinematic bodies, plus asleep bodies under
    /// [`WakePolicy::PreserveSleep`], accept this call without changing state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle, invalid input, or
    /// non-finite derived velocity.
    pub fn apply_body_linear_impulse_to_center(
        &mut self,
        body: BodyId,
        impulse: Vec2,
        wake_policy: WakePolicy,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_apply_linear_impulse_to_center(impulse, wake_policy)
        })
    }

    /// Applies angular impulse using the requested wake policy.
    ///
    /// Static and kinematic bodies, plus asleep bodies under
    /// [`WakePolicy::PreserveSleep`], accept this call without changing state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle, invalid input, or
    /// non-finite derived angular velocity.
    pub fn apply_body_angular_impulse(
        &mut self,
        body: BodyId,
        impulse: f32,
        wake_policy: WakePolicy,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_apply_angular_impulse(impulse, wake_policy)
        })
    }

    /// Changes whether a live body is awake.
    ///
    /// Sleeping clears velocity, accumulated force, accumulated torque, and
    /// sleep time atomically.
    ///
    /// # Errors
    ///
    /// Returns a typed error when `body` is foreign, stale, destroyed, or its
    /// world is poisoned.
    pub fn set_body_awake(&mut self, body: BodyId, awake: bool) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| Ok(state.candidate_set_awake(awake)))
    }

    /// Changes whether a live body may sleep automatically.
    ///
    /// Disabling sleep wakes the body immediately.
    ///
    /// # Errors
    ///
    /// Returns a typed error when `body` is foreign, stale, destroyed, or its
    /// world is poisoned.
    pub fn set_body_sleeping_allowed(
        &mut self,
        body: BodyId,
        sleeping_allowed: bool,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            Ok(state.candidate_set_sleeping_allowed(sleeping_allowed))
        })
    }

    /// Sets a live body's linear damping without changing wake state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle or non-finite or negative damping.
    pub fn set_body_linear_damping(
        &mut self,
        body: BodyId,
        damping: f32,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| state.candidate_set_linear_damping(damping))
    }

    /// Sets a live body's angular damping without changing wake state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle or non-finite or negative damping.
    pub fn set_body_angular_damping(
        &mut self,
        body: BodyId,
        damping: f32,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| state.candidate_set_angular_damping(damping))
    }

    /// Sets a live body's gravity scale without changing wake state.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle or non-finite gravity scale.
    pub fn set_body_gravity_scale(
        &mut self,
        body: BodyId,
        gravity_scale: f32,
    ) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| {
            state.candidate_set_gravity_scale(gravity_scale)
        })
    }

    /// Changes whether a live body receives continuous bullet treatment.
    ///
    /// # Errors
    ///
    /// Returns a typed error when `body` is foreign, stale, destroyed, or its
    /// world is poisoned.
    pub fn set_body_bullet(&mut self, body: BodyId, bullet: bool) -> Result<(), BodyControlError> {
        self.update_body_state(body, |state| Ok(state.candidate_set_bullet(bullet)))
    }

    /// Changes whether a live body has fixed rotation.
    ///
    /// A changed setting clears angular velocity and recomputes fixture-derived
    /// mass state before replacing the body state once.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid handle or invalid aggregate mass.
    pub fn set_body_fixed_rotation(
        &mut self,
        body: BodyId,
        fixed_rotation: bool,
    ) -> Result<(), BodyControlError> {
        self.ensure_not_poisoned_for_handle()?;
        let state = self.bodies.get(body)?.state;
        let fixture_mass_data = self.collect_fixture_mass_data(body, None, None);
        let candidate = state.candidate_set_fixed_rotation(fixed_rotation, &fixture_mass_data)?;
        self.body_mut_after_validation(body).state = candidate;
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    /// Changes the motion type of a live body.
    ///
    /// # Errors
    ///
    /// Returns [`BodyTypeChangeError::InvalidHandle`] when `body` is foreign, stale, or
    /// destroyed, or [`BodyTypeChangeError::InvalidAggregateMass`] when the complete target-type
    /// fixture aggregate is invalid. Either failure leaves body, contact, proxy, and fixture state
    /// unchanged.
    pub fn set_body_type(
        &mut self,
        body: BodyId,
        target_type: BodyType,
    ) -> Result<(), BodyTypeChangeError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.bodies.get(body)?;
        if record.state.snapshot().body_type() == target_type {
            return Ok(());
        }
        let fixtures = record.fixtures.clone();
        let fixture_mass_data = self.collect_fixture_mass_data(body, None, None);
        let candidate = record
            .state
            .with_body_type_and_reset_mass_data(target_type, &fixture_mass_data)?;
        self.destroy_contacts_for_body(body);
        {
            let record = self.body_mut_after_validation(body);
            record.state = candidate;
            record.pending_contact_destruction = true;
            record.pending_wake = true;
        }
        self.touch_body_fixture_entries(body, &fixtures);
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    /// Changes a live body's position and angle after validating the complete candidate state.
    ///
    /// Accepted values retain their exact `f32` bits. A failure leaves the prior body state
    /// unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`BodyTransformError::InvalidHandle`] when `body` is invalid, or
    /// [`BodyTransformError::InvalidTransform`] when a position coordinate or angle is
    /// non-finite.
    pub fn set_body_transform(
        &mut self,
        body: BodyId,
        position: Vec2,
        angle: f32,
    ) -> Result<(), BodyTransformError> {
        self.ensure_not_poisoned_for_handle()?;
        let candidate = self
            .bodies
            .get(body)?
            .state
            .with_transform(position, angle)?;
        let record = self.bodies.get(body)?;
        let previous = record.state.transform();
        let fixtures = record.fixtures.clone();
        let active = record.state.snapshot().is_active();
        let synchronizations = if active {
            self.prepare_body_synchronizations(body, &fixtures, previous, candidate.transform())?
        } else {
            Vec::new()
        };
        self.apply_body_synchronizations(synchronizations);
        self.body_mut_after_validation(body).state = candidate;
        self.invalidate_continuous_for_body(body);
        Ok(())
    }

    /// Changes whether a live body participates in simulation.
    ///
    /// # Errors
    ///
    /// Returns a handle error without mutation when `body` is foreign, stale, or destroyed.
    pub fn set_body_active(
        &mut self,
        body: BodyId,
        active: bool,
    ) -> Result<(), BodyActivationError> {
        self.ensure_not_poisoned_for_handle()?;
        let record = self.bodies.get(body)?;
        if record.state.snapshot().is_active() == active {
            return Ok(());
        }
        let transform = record.state.transform();
        let fixtures = record.fixtures.clone();
        if active {
            let creations = self.prepare_body_fixture_creations(&fixtures, transform)?;
            self.create_body_fixture_entries(body, creations);
        } else {
            self.destroy_contacts_for_body(body);
            self.destroy_body_fixture_entries(body, fixtures);
        }
        let record = self.body_mut_after_validation(body);
        record.state.set_active(active);
        if !active {
            record.pending_contact_destruction = true;
        }
        self.invalidate_continuous_for_body(body);
        Ok(())
    }
}
