use super::{
    AssertUnwindSafe, BodyId, CollisionDecisionHook, CommandApplication, CommandError,
    ContactHookRun, ContactSolve, ContactSolveFailure, ContactTransition, ContinuousStepKey,
    ContinuousStepKind, DiagnosticProfileParent, DiagnosticStepPhase, DiagnosticStepProfile,
    DiagnosticStepProfiler, FixtureDestructionError, FixtureId, LifecycleEvent, Ordering,
    StepCompletion, StepConfiguration, StepError, StepLifecycleEvent, StepLimits, StepPhase,
    StepReport, StepState, World, WorldCommand, catch_unwind, resume_unwind,
};

struct StepLockGuard {
    state: StepState,
}

struct StepLimitBackup {
    bodies: crate::arena::Arena<crate::world::object::Body, BodyId>,
    fixtures: crate::arena::Arena<crate::world::object::Fixture, FixtureId>,
    joints: crate::arena::Arena<crate::world::joint::JointRecord, crate::JointId>,
    particle_systems:
        crate::arena::Arena<crate::world::object::ParticleSystem, crate::ParticleSystemId>,
    broad_phase: crate::collision::BroadPhase<crate::world::proxy::FixtureProxy>,
    contact_manager: crate::world::contact_manager::ContactManager,
    continuous_step_state: crate::world::continuous::ContinuousStepState,
    configuration: crate::world::config::WorldConfiguration,
}

impl StepLockGuard {
    fn acquire(state: &StepState) -> Result<Self, StepError> {
        if state.inner.locked.swap(true, Ordering::Relaxed) {
            return Err(StepError::Locked);
        }
        Ok(Self {
            state: state.clone(),
        })
    }
}

impl Drop for StepLockGuard {
    fn drop(&mut self) {
        self.state.inner.locked.store(false, Ordering::Relaxed);
    }
}

impl World {
    fn backup_step_limit_state(&self) -> StepLimitBackup {
        StepLimitBackup {
            bodies: self.bodies.clone(),
            fixtures: self.fixtures.clone(),
            joints: self.joints.clone(),
            particle_systems: self.particle_systems.clone(),
            broad_phase: self.broad_phase.clone(),
            contact_manager: self.contact_manager.clone(),
            continuous_step_state: self.continuous_step_state,
            configuration: self.configuration,
        }
    }

    fn restore_step_limit_state(&mut self, backup: StepLimitBackup) {
        self.bodies = backup.bodies;
        self.fixtures = backup.fixtures;
        self.joints = backup.joints;
        self.particle_systems = backup.particle_systems;
        self.broad_phase = backup.broad_phase;
        self.contact_manager = backup.contact_manager;
        self.continuous_step_state = backup.continuous_step_state;
        self.configuration = backup.configuration;
    }

    /// Runs one automatic bounded rigid-world lifecycle and solve.
    ///
    /// Checked configuration is supplied by [`StepConfiguration`]. Discrete
    /// islands are staged and committed transactionally in deterministic source
    /// order. Continuous work remains private and resumable: sub-stepping uses
    /// [`StepCompletion::ContinuousPending`], while budget exhaustion returns
    /// [`StepError::ContinuousWorkLimitExceeded`] with [`crate::ContinuousProgress`].
    /// Ordinary event and command limit failures restore the exact pre-call
    /// rigid-world state so the caller may retry with larger limits.
    /// Accumulated forces clear only after a successful call when automatic
    /// clearing is enabled.
    ///
    /// # Errors
    ///
    /// Returns an error for poisoned or nested stepping, exhausted limits,
    /// unsupported topology, or non-finite solver state.
    pub fn step<H: CollisionDecisionHook>(
        &mut self,
        configuration: StepConfiguration,
        hook: &mut H,
        limits: StepLimits,
    ) -> Result<StepReport, StepError> {
        let mut profiler = DiagnosticStepProfiler::disabled();
        self.step_internal(configuration, hook, limits, &mut profiler)
    }

    /// Runs one automatic step and returns separate nondeterministic phase timings.
    ///
    /// The returned [`DiagnosticStepProfile`] is diagnostic only. It implements
    /// neither equality nor hashing, has no deterministic-checkpoint conversion,
    /// and is never embedded in [`StepReport`]. Calling this method therefore
    /// does not change ordinary step report semantics.
    ///
    /// # Errors
    ///
    /// Returns the same typed failures as [`Self::step`]. No partial profile is
    /// returned for a failed step.
    pub fn step_profiled<H: CollisionDecisionHook>(
        &mut self,
        configuration: StepConfiguration,
        hook: &mut H,
        limits: StepLimits,
    ) -> Result<(StepReport, DiagnosticStepProfile), StepError> {
        let mut profiler = DiagnosticStepProfiler::enabled();
        let report = self.step_internal(configuration, hook, limits, &mut profiler)?;
        Ok((report, profiler.finish()))
    }

    #[allow(
        clippy::too_many_lines,
        reason = "the locked source-ordered step lifecycle is kept visible as one transaction"
    )]
    fn step_internal<H: CollisionDecisionHook>(
        &mut self,
        configuration: StepConfiguration,
        hook: &mut H,
        limits: StepLimits,
        profiler: &mut DiagnosticStepProfiler,
    ) -> Result<StepReport, StepError> {
        if self.step_state.is_poisoned() {
            return Err(StepError::Poisoned);
        }
        let timing = self.prepare_step_timing(configuration);
        let mut phases = Vec::with_capacity(6);
        let continuous_enabled =
            self.is_continuous_physics_enabled() && configuration.time_step() > 0.0;
        let continuous_key = ContinuousStepKey::from_configuration(configuration);
        let step_lock = StepLockGuard::acquire(&self.step_state)?;
        let step_limit_backup = self.backup_step_limit_state();
        let step_kind = if continuous_enabled {
            self.continuous_step_state
                .begin_step(continuous_key, &mut self.contact_manager)
        } else {
            self.continuous_step_state.invalidate();
            ContinuousStepKind::Fresh
        };
        let runs_particle_stages =
            step_kind == ContinuousStepKind::Fresh && configuration.time_step() > 0.0;
        let mut hook_run = ContactHookRun::new(hook, limits);
        let mut contact_transitions = self.contact_manager.drain_transitions();
        let contact_lifecycle_start = profiler.start();
        let locked_result = (|| -> Result<StepCompletion, StepError> {
            let _lock = step_lock;
            let contact_lifecycle_result = (|| {
                for transition in contact_transitions.iter().cloned() {
                    hook_run.record_contact_destruction(transition)?;
                }
                phases.push(StepPhase::FindPairs);
                let hook_result = catch_unwind(AssertUnwindSafe(|| {
                    self.find_pairs_with_hook(&mut hook_run)?;
                    phases.push(StepPhase::UpdateContacts);
                    self.update_contacts_for_step(&mut hook_run)
                }));
                match hook_result {
                    Ok(result) => result,
                    Err(payload) => {
                        self.step_state.poison();
                        resume_unwind(payload);
                    }
                }
            })();
            contact_lifecycle_result?;
            contact_transitions.extend(self.contact_manager.drain_transitions());
            profiler.record(
                DiagnosticStepPhase::Common(DiagnosticProfileParent::ContactUpdate),
                contact_lifecycle_start,
            );
            if runs_particle_stages {
                let maybe_particle_prepare_start = if self.particle_system_order.is_empty() {
                    None
                } else {
                    profiler.start()
                };
                let particle_solve_start = profiler.start();
                let particle_contact_result = catch_unwind(AssertUnwindSafe(|| {
                    self.run_particle_solver(configuration, &mut hook_run)
                }));
                match particle_contact_result {
                    Ok(result) => result?,
                    Err(payload) => {
                        self.step_state.poison();
                        resume_unwind(payload);
                    }
                }
                self.preflight_contact_solver()
                    .map_err(|error| solver_step_error(error, &contact_transitions))?;
                profiler.record(
                    DiagnosticStepPhase::Common(DiagnosticProfileParent::ParticlePrepare),
                    maybe_particle_prepare_start,
                );
                profiler.record(
                    DiagnosticStepPhase::Common(DiagnosticProfileParent::ParticleSolve),
                    particle_solve_start,
                );
            }

            phases.push(StepPhase::Hook);
            if runs_particle_stages {
                let rigid_solve_start = profiler.start();
                phases.push(StepPhase::Solve);
                self.solve_contact_constraints(
                    configuration,
                    timing,
                    limits.maybe_failure_injection,
                    &contact_transitions,
                    &mut hook_run,
                )?;
                profiler.record(
                    DiagnosticStepPhase::Common(DiagnosticProfileParent::RigidSolve),
                    rigid_solve_start,
                );
            }

            let completion = if continuous_enabled {
                let continuous_solve_start = profiler.start();
                let hook_result = catch_unwind(AssertUnwindSafe(|| {
                    self.run_continuous_stage(
                        configuration,
                        continuous_key,
                        limits.max_continuous_work,
                        &contact_transitions,
                        &mut hook_run,
                    )
                }));
                let continuous = match hook_result {
                    Ok(result) => result?,
                    Err(payload) => {
                        self.step_state.poison();
                        resume_unwind(payload);
                    }
                };
                drop(continuous.contact_solves);
                drop(self.contact_manager.drain_transitions());
                profiler.record(
                    DiagnosticStepPhase::Common(DiagnosticProfileParent::ContinuousSolve),
                    continuous_solve_start,
                );
                continuous.completion
            } else {
                StepCompletion::Complete
            };
            Ok(completion)
        })();
        let completion = match locked_result {
            Ok(completion) => completion,
            Err(error @ StepError::LimitExceeded { .. }) => {
                self.restore_step_limit_state(step_limit_backup);
                return Err(error);
            }
            Err(error) => return Err(error),
        };
        let (mut lifecycle, commands) = hook_run.finish();
        phases.push(StepPhase::Unlock);
        let finalize_start = profiler.start();
        let apply_commands_start = profiler.start();
        if !commands.is_empty() {
            phases.push(StepPhase::ApplyCommands);
        }
        let commands_were_present = !commands.is_empty();
        lifecycle.extend(self.apply_commands(commands));
        if commands_were_present {
            profiler.record(DiagnosticStepPhase::ApplyCommands, apply_commands_start);
        }

        let contact_transitions = lifecycle
            .iter()
            .filter_map(|event| match event {
                LifecycleEvent::Contact(transition)
                | LifecycleEvent::ContactDestruction(transition) => Some(transition.clone()),
                _ => None,
            })
            .collect();
        let events = lifecycle
            .iter()
            .filter_map(|event| match event {
                LifecycleEvent::Hook(event) => Some(event.clone()),
                _ => None,
            })
            .collect();
        let contact_solves = lifecycle
            .iter()
            .filter_map(|event| match event {
                LifecycleEvent::Solve(solve) => Some(solve.clone()),
                _ => None,
            })
            .collect();
        let continuous_contact_solves = lifecycle
            .iter()
            .filter_map(|event| match event {
                LifecycleEvent::ContinuousSolve(solve) => Some(solve.clone()),
                _ => None,
            })
            .collect();
        let destructions = lifecycle
            .iter()
            .filter_map(|event| match event {
                LifecycleEvent::JointGoodbye(record)
                | LifecycleEvent::FixtureGoodbye(record)
                | LifecycleEvent::ParticleDestruction(record)
                | LifecycleEvent::Destruction(record) => Some(record.clone()),
                _ => None,
            })
            .collect();
        let command_applications = lifecycle
            .iter()
            .filter_map(|event| match event {
                LifecycleEvent::Command(application) => Some(application.clone()),
                _ => None,
            })
            .collect();

        let completion = self.finish_successful_step(timing, completion);
        let report = StepReport {
            completion,
            time_step_ratio: timing.time_step_ratio(),
            phases,
            events,
            contact_transitions,
            contact_solves,
            continuous_contact_solves,
            lifecycle,
            destructions,
            command_applications,
        };
        profiler.record(
            DiagnosticStepPhase::Common(DiagnosticProfileParent::Finalize),
            finalize_start,
        );
        Ok(report)
    }

    fn find_pairs_with_hook<H: CollisionDecisionHook>(
        &mut self,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        self.find_new_contacts_with_hook(hook_run)
    }

    pub(super) fn finish_successful_step(
        &mut self,
        timing: crate::world::config::StepTiming,
        completion: StepCompletion,
    ) -> StepCompletion {
        self.commit_step_timing(timing);
        if self.is_automatic_force_clearing_enabled() {
            self.clear_force_accumulators();
        }
        completion
    }

    fn update_contacts_for_step<H: CollisionDecisionHook>(
        &mut self,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        self.update_contacts_with_hook(hook_run)
    }

    fn solve_contact_constraints<H: CollisionDecisionHook>(
        &mut self,
        configuration: StepConfiguration,
        timing: crate::world::config::StepTiming,
        maybe_failure_injection: Option<crate::world::island::SolveFailureInjection>,
        contact_transitions: &[ContactTransition],
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<Vec<ContactSolve>, StepError> {
        self.solve_contacts(
            configuration,
            timing,
            maybe_failure_injection,
            contact_transitions,
            hook_run,
        )
    }

    fn apply_commands(&mut self, commands: Vec<WorldCommand>) -> Vec<StepLifecycleEvent> {
        let mut lifecycle = Vec::new();
        for command in commands {
            let result = self.apply_command(command);
            let (owned_result, mutation_lifecycle) = match result {
                Ok(report) => {
                    let mutation_lifecycle = report.lifecycle().to_vec();
                    (Ok(report.into_value()), mutation_lifecycle)
                }
                Err(error) => (Err(error), Vec::new()),
            };
            lifecycle.extend(mutation_lifecycle);
            let application = CommandApplication {
                command,
                result: owned_result,
            };
            lifecycle.push(StepLifecycleEvent::Command(application.clone()));
        }
        lifecycle
    }

    fn apply_command(
        &mut self,
        command: WorldCommand,
    ) -> Result<crate::DestructionReport, CommandError> {
        if self.step_state.is_locked() {
            return Err(CommandError::Locked);
        }
        match command {
            WorldCommand::DestroyBody(body) => {
                self.destroy_body(body).map_err(CommandError::InvalidHandle)
            }
            WorldCommand::DestroyFixture(fixture) => {
                self.destroy_fixture(fixture).map_err(|error| match error {
                    FixtureDestructionError::InvalidHandle(error) => {
                        CommandError::InvalidHandle(error)
                    }
                    FixtureDestructionError::InvalidAggregateMass(error) => {
                        CommandError::InvalidAggregateMass(error)
                    }
                })
            }
        }
    }

    /// Returns whether this world is currently inside its step lock.
    #[must_use]
    pub fn is_locked(&self) -> bool {
        self.step_state.is_locked()
    }

    /// Returns whether a hook panic permanently poisoned coherent operations.
    #[must_use]
    pub fn is_poisoned(&self) -> bool {
        self.step_state.is_poisoned()
    }
}

pub(in crate::world) fn solver_step_error(
    error: ContactSolveFailure,
    contact_transitions: &[ContactTransition],
) -> StepError {
    match error {
        ContactSolveFailure::UnsupportedTopology => StepError::UnsupportedSolverTopology {
            contact_transitions: contact_transitions.to_vec(),
        },
        ContactSolveFailure::NonFinite => StepError::NonFiniteSolverState {
            contact_transitions: contact_transitions.to_vec(),
        },
        ContactSolveFailure::CapacityExceeded { resource, limit } => {
            StepError::LimitExceeded { resource, limit }
        }
        ContactSolveFailure::InvalidProxyBounds => StepError::InvalidSolverProxyBounds {
            contact_transitions: contact_transitions.to_vec(),
        },
    }
}

pub(super) fn check_capacity(
    current: usize,
    limit: usize,
    resource: &'static str,
) -> Result<(), StepError> {
    if current == limit {
        return Err(StepError::LimitExceeded { resource, limit });
    }
    Ok(())
}
