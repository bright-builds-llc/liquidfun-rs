use crate::particle::ParticleGroupFlags;
use crate::{ParticleFlags, StepConfiguration};

use super::{ParticlePassExecutor, PassGate, PassId, run_particle_solver};

#[derive(Clone)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "independent booleans model each manifest admission family explicitly"
)]
struct TraceExecutor {
    empty: bool,
    paused: bool,
    particle_flags: ParticleFlags,
    group_flags: ParticleGroupFlags,
    lifetime: bool,
    zombie: bool,
    dirty_particle_flags: bool,
    dirty_group_flags: bool,
    needs_depth: bool,
    pending_force: bool,
    trace: Vec<(PassId, Option<u32>)>,
}

impl TraceExecutor {
    fn all() -> Self {
        Self {
            empty: false,
            paused: false,
            particle_flags: ParticleFlags::all(),
            group_flags: ParticleGroupFlags::all(),
            lifetime: true,
            zombie: true,
            dirty_particle_flags: true,
            dirty_group_flags: true,
            needs_depth: true,
            pending_force: true,
            trace: Vec::new(),
        }
    }
}

impl ParticlePassExecutor for TraceExecutor {
    type Error = ();

    fn is_empty(&self) -> bool {
        self.empty
    }

    fn is_paused(&self) -> bool {
        self.paused
    }

    fn admits(&mut self, gate: PassGate) -> bool {
        match gate {
            PassGate::ExpirationLane => self.lifetime,
            PassGate::AggregateParticleFlags(flags) => {
                if flags == ParticleFlags::ZOMBIE {
                    self.zombie
                } else {
                    self.particle_flags.intersects(flags)
                }
            }
            PassGate::DirtyParticleFlags => self.dirty_particle_flags,
            PassGate::DirtyGroupFlags => self.dirty_group_flags,
            PassGate::PauseTerminator | PassGate::Always => true,
            PassGate::NeedsGroupDepth => self.needs_depth,
            PassGate::PendingForce => self.pending_force,
            PassGate::AggregateGroupFlags(flags) => !(self.group_flags & flags).is_empty(),
            PassGate::ExtraDampingAggregateFlags(flags) => self.particle_flags.intersects(flags),
        }
    }

    fn execute(&mut self, pass: PassId, maybe_iteration: Option<u32>) -> Result<(), Self::Error> {
        self.trace.push((pass, maybe_iteration));
        Ok(())
    }
}

fn configuration(iterations: u32) -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3)
        .expect("base configuration is valid")
        .with_particle_iterations(iterations)
        .expect("particle iteration count is valid")
}

#[test]
fn private_pass_trace_maximum_is_five_plus_twenty_six_per_iteration() {
    // Arrange
    let mut executor = TraceExecutor::all();

    // Act
    run_particle_solver(configuration(2), &mut executor).expect("trace execution is infallible");

    // Assert
    assert_eq!(executor.trace.len(), 5 + 26 * 2);
    assert_eq!(executor.trace[0], (PassId::Lifetime, None));
    assert_eq!(executor.trace[4], (PassId::PauseGate, None));
    assert_eq!(executor.trace[5], (PassId::ParticleContacts, Some(0)));
    assert_eq!(executor.trace[30], (PassId::Integrate, Some(0)));
    assert_eq!(executor.trace[31], (PassId::ParticleContacts, Some(1)));
    assert_eq!(executor.trace[56], (PassId::Integrate, Some(1)));
}

#[test]
fn private_pass_trace_one_iteration_matches_manifest_order() {
    // Arrange
    let mut executor = TraceExecutor::all();

    // Act
    run_particle_solver(configuration(1), &mut executor).expect("trace execution is infallible");

    // Assert
    let actual = executor
        .trace
        .iter()
        .map(|(pass, _)| pass.as_str())
        .collect::<Vec<_>>();
    let expected = super::manifest::PASS_GRAPH
        .iter()
        .map(|descriptor| descriptor.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn private_pass_trace_gate_families_omit_only_inactive_passes() {
    // Arrange
    let mut executor = TraceExecutor::all();
    executor.lifetime = false;
    executor.zombie = false;
    executor.dirty_particle_flags = false;
    executor.dirty_group_flags = false;
    executor.needs_depth = false;
    executor.pending_force = false;
    executor.particle_flags = ParticleFlags::empty();
    executor.group_flags = ParticleGroupFlags::empty();

    // Act
    run_particle_solver(configuration(1), &mut executor).expect("trace execution is infallible");

    // Assert
    let actual = executor
        .trace
        .iter()
        .map(|(pass, _)| *pass)
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            PassId::PauseGate,
            PassId::ParticleContacts,
            PassId::BodyContacts,
            PassId::Weight,
            PassId::Gravity,
            PassId::Pressure,
            PassId::Damping,
            PassId::LimitVelocity,
            PassId::Collision,
            PassId::Integrate,
        ]
    );
}

#[test]
fn private_pass_trace_paused_nonempty_system_is_outer_only() {
    // Arrange
    let mut executor = TraceExecutor::all();
    executor.paused = true;

    // Act
    run_particle_solver(configuration(2), &mut executor).expect("trace execution is infallible");

    // Assert
    assert_eq!(executor.trace.len(), 5);
    assert!(
        executor
            .trace
            .iter()
            .all(|(_, iteration)| iteration.is_none())
    );
}

#[test]
fn private_pass_trace_empty_system_has_no_entries() {
    // Arrange
    let mut executor = TraceExecutor::all();
    executor.empty = true;

    // Act
    run_particle_solver(configuration(2), &mut executor).expect("trace execution is infallible");

    // Assert
    assert!(executor.trace.is_empty());
}

#[test]
fn private_pass_trace_refresh_reactive_and_zombie_admission_is_explicit() {
    // Arrange
    let mut executor = TraceExecutor::all();
    executor.particle_flags = ParticleFlags::REACTIVE | ParticleFlags::ZOMBIE;
    executor.group_flags = ParticleGroupFlags::empty();

    // Act
    run_particle_solver(configuration(1), &mut executor).expect("trace execution is infallible");

    // Assert
    assert!(
        executor
            .trace
            .iter()
            .any(|entry| entry.0 == PassId::ZombieCompaction)
    );
    assert!(
        executor
            .trace
            .iter()
            .any(|entry| entry.0 == PassId::RefreshParticleFlags)
    );
    assert!(
        executor
            .trace
            .iter()
            .any(|entry| entry.0 == PassId::RefreshGroupFlags)
    );
    assert!(
        executor
            .trace
            .iter()
            .any(|entry| entry.0 == PassId::ReactiveTopology)
    );
}
