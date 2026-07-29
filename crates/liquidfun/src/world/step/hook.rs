use super::{
    CollisionDirective, CollisionFilterEvent, ContactEvent, ContactSolve, ContactTransition,
    ContactView, DestructionRecord, FixturePairSnapshot, FixturePairView, FixtureParticleView,
    LifecycleEvent, ManagedContactSnapshot, ParticleBodyContact, ParticleBodyContactEffect,
    ParticleContact, ParticleContactEffect, ParticlePairContactView, PreSolveDirective,
    PreSolveView, StepError, StepLimits, WorldCommand, check_capacity,
};

/// Restricted synchronous step hooks.
///
/// Hooks receive only borrow-scoped read-only contact views and return narrow
/// decisions. They do not receive mutable world access:
///
/// ```compile_fail
/// use liquidfun::{CollisionDecisionHook, PreSolveDirective, PreSolveView, World};
///
/// struct MutatingHook;
///
/// impl CollisionDecisionHook for MutatingHook {
///     fn pre_solve(
///         &mut self,
///         _world: &mut World,
///         _contact: PreSolveView<'_>,
///     ) -> PreSolveDirective {
///         PreSolveDirective::Enable
///     }
/// }
/// ```
pub trait CollisionDecisionHook {
    /// Decides whether a broad-phase pair may create or retain a contact.
    fn should_collide(&mut self, _pair: FixturePairView<'_>) -> CollisionDirective {
        CollisionDirective::Collide
    }

    /// Decides whether one flag-gated fixture-particle candidate is retained.
    fn should_collide_fixture_particle(
        &mut self,
        _contact: FixtureParticleView<'_>,
    ) -> CollisionDirective {
        CollisionDirective::Collide
    }

    /// Decides whether one flag-gated particle-pair candidate is retained.
    fn should_collide_particle_pair(
        &mut self,
        _contact: ParticlePairContactView<'_>,
    ) -> CollisionDirective {
        CollisionDirective::Collide
    }

    /// Decides source-supported controls immediately after one solid update.
    fn pre_solve(&mut self, _contact: PreSolveView<'_>) -> PreSolveDirective {
        PreSolveDirective::Enable
    }

    /// Observes one non-filtered occurrence without mutable world access.
    fn observe(&mut self, _contact: ContactView<'_>) {}

    /// Optionally requests one owned mutation after observing an occurrence.
    fn command(&mut self, _contact: ContactView<'_>) -> Option<WorldCommand> {
        None
    }
}

/// Explicit no-op decision maker used to replace or unregister prior behavior.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoDecisionHook;

impl CollisionDecisionHook for NoDecisionHook {}

/// Legacy observation and deferred-command hook contract.
///
/// Implementations receive the new source-timed behavior through the blanket
/// [`CollisionDecisionHook`] adapter. New filtering code should implement
/// `CollisionDecisionHook` directly so it can inspect [`FixturePairView`].
pub trait StepHook {
    /// Decides whether one supported solid occurrence remains enabled.
    fn pre_solve(&mut self, _contact: ContactView<'_>) -> PreSolveDirective {
        PreSolveDirective::Enable
    }

    /// Observes one non-filtered occurrence without mutable world access.
    fn observe(&mut self, _contact: ContactView<'_>) {}

    /// Optionally requests one owned mutation after observing an occurrence.
    fn command(&mut self, _contact: ContactView<'_>) -> Option<WorldCommand> {
        None
    }
}

impl<H: StepHook + ?Sized> CollisionDecisionHook for H {
    fn pre_solve(&mut self, contact: PreSolveView<'_>) -> PreSolveDirective {
        StepHook::pre_solve(self, contact.contact())
    }

    fn observe(&mut self, contact: ContactView<'_>) {
        StepHook::observe(self, contact);
    }

    fn command(&mut self, contact: ContactView<'_>) -> Option<WorldCommand> {
        StepHook::command(self, contact)
    }
}

pub(in crate::world) struct ContactHookRun<'hook, H> {
    hook: &'hook mut H,
    limits: StepLimits,
    lifecycle: Vec<LifecycleEvent>,
    commands: Vec<WorldCommand>,
}

impl<'hook, H: CollisionDecisionHook> ContactHookRun<'hook, H> {
    pub(in crate::world) fn new(hook: &'hook mut H, limits: StepLimits) -> Self {
        Self {
            hook,
            limits,
            lifecycle: Vec::new(),
            commands: Vec::new(),
        }
    }

    pub(in crate::world) fn should_collide(
        &mut self,
        pair: &FixturePairSnapshot,
    ) -> Result<bool, StepError> {
        let decision = self.hook.should_collide(FixturePairView::new(pair));
        self.push_lifecycle(LifecycleEvent::Filter(CollisionFilterEvent::new(
            *pair, decision,
        )))?;
        Ok(decision == CollisionDirective::Collide)
    }

    pub(in crate::world) fn should_collide_fixture_particle(
        &mut self,
        contact: &ParticleBodyContact,
    ) -> bool {
        self.hook
            .should_collide_fixture_particle(FixtureParticleView { contact })
            == CollisionDirective::Collide
    }

    pub(in crate::world) fn should_collide_particle_pair(
        &mut self,
        contact: &ParticleContact,
    ) -> bool {
        self.hook
            .should_collide_particle_pair(ParticlePairContactView { contact })
            == CollisionDirective::Collide
    }

    pub(in crate::world) fn contact_updated(
        &mut self,
        current: &ManagedContactSnapshot,
        maybe_previous_manifold: Option<&crate::collision::Manifold>,
        allow_pre_solve: bool,
    ) -> Result<PreSolveDirective, StepError> {
        let contact = ContactView { contact: current };
        let directive = if allow_pre_solve {
            self.hook
                .pre_solve(PreSolveView::new(current, maybe_previous_manifold))
        } else {
            PreSolveDirective::Enable
        };
        self.hook.observe(contact);
        if let Some(command) = self.hook.command(contact) {
            check_capacity(self.commands.len(), self.limits.max_commands, "command")?;
            self.commands.push(command);
        }
        self.push_lifecycle(LifecycleEvent::Hook(ContactEvent {
            contact: current.clone(),
            collision: CollisionDirective::Collide,
            maybe_pre_solve: allow_pre_solve.then_some(directive),
        }))?;
        Ok(directive)
    }

    pub(in crate::world) fn record_contact(
        &mut self,
        transition: ContactTransition,
    ) -> Result<(), StepError> {
        self.push_lifecycle(LifecycleEvent::Contact(transition))
    }

    pub(in crate::world) fn record_contact_destruction(
        &mut self,
        transition: ContactTransition,
    ) -> Result<(), StepError> {
        self.push_lifecycle(LifecycleEvent::ContactDestruction(transition))
    }

    pub(in crate::world) fn record_particle_destruction(
        &mut self,
        record: DestructionRecord,
    ) -> Result<(), StepError> {
        self.push_lifecycle(LifecycleEvent::ParticleDestruction(record))
    }

    pub(in crate::world) fn record_destruction(
        &mut self,
        record: DestructionRecord,
    ) -> Result<(), StepError> {
        self.push_lifecycle(LifecycleEvent::Destruction(record))
    }

    pub(in crate::world) fn record_particle_body_contact(
        &mut self,
        effect: ParticleBodyContactEffect,
    ) -> Result<(), StepError> {
        self.push_lifecycle(LifecycleEvent::ParticleBodyContact(effect))
    }

    pub(in crate::world) fn record_particle_contact(
        &mut self,
        effect: ParticleContactEffect,
    ) -> Result<(), StepError> {
        self.push_lifecycle(LifecycleEvent::ParticleContact(effect))
    }

    pub(in crate::world) fn record_discrete_solve(
        &mut self,
        solve: ContactSolve,
    ) -> Result<(), StepError> {
        self.push_lifecycle(LifecycleEvent::Solve(solve))
    }

    pub(in crate::world) fn record_continuous_solve(
        &mut self,
        solve: ContactSolve,
    ) -> Result<(), StepError> {
        self.push_lifecycle(LifecycleEvent::ContinuousSolve(solve))
    }

    pub(in crate::world) fn ensure_lifecycle_capacity(
        &self,
        additional: usize,
    ) -> Result<(), StepError> {
        let required =
            self.lifecycle
                .len()
                .checked_add(additional)
                .ok_or(StepError::LimitExceeded {
                    resource: "event",
                    limit: self.limits.max_events,
                })?;
        if required > self.limits.max_events {
            return Err(StepError::LimitExceeded {
                resource: "event",
                limit: self.limits.max_events,
            });
        }
        Ok(())
    }

    fn push_lifecycle(&mut self, event: LifecycleEvent) -> Result<(), StepError> {
        check_capacity(self.lifecycle.len(), self.limits.max_events, "event")?;
        self.lifecycle.push(event);
        Ok(())
    }

    pub(super) fn finish(self) -> (Vec<LifecycleEvent>, Vec<WorldCommand>) {
        (self.lifecycle, self.commands)
    }
}
