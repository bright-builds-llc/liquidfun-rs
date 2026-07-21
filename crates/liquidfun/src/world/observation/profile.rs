//! Nondeterministic wall-clock diagnostics kept outside semantic step evidence.

use std::time::{Duration, Instant};

const MAX_DIAGNOSTIC_STEP_PHASES: usize = 6;

/// Closed diagnostic step phases in measured execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DiagnosticStepPhase {
    /// Contact pair discovery, updates, filtering, and callbacks.
    ContactLifecycle,
    /// Particle lifetime, contact, and constraint preparation.
    ParticleSolve,
    /// Discrete rigid-body and joint constraint solving.
    RigidSolve,
    /// Continuous-collision candidate and TOI processing.
    ContinuousSolve,
    /// Deferred commands applied after releasing the world lock.
    ApplyCommands,
    /// Owned report construction and successful-step bookkeeping.
    Finalize,
}

/// One nondeterministic wall-clock duration for a named diagnostic phase.
///
/// This type deliberately implements neither `PartialEq` nor `Hash`. It has no
/// conversion into a deterministic checkpoint or parity record:
///
/// ```compile_fail
/// use liquidfun::DiagnosticStepProfile;
/// fn compare(left: DiagnosticStepProfile, right: DiagnosticStepProfile) {
///     let _same = left == right;
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DiagnosticStepPhaseTiming {
    phase: DiagnosticStepPhase,
    duration: Duration,
}

impl DiagnosticStepPhaseTiming {
    /// Returns the closed phase name.
    #[must_use]
    pub const fn phase(self) -> DiagnosticStepPhase {
        self.phase
    }

    /// Returns nondeterministic elapsed wall-clock time.
    ///
    /// This value is diagnostic only and must not enter parity comparisons.
    #[must_use]
    pub const fn duration(self) -> Duration {
        self.duration
    }
}

/// A bounded diagnostic-only phase profile from one successful step.
///
/// The profile is intentionally separate from [`crate::StepReport`] and
/// implements neither equality nor hashing. Phase presence is structural and
/// durations are wall-clock observations, never deterministic evidence.
#[derive(Debug, Clone)]
pub struct DiagnosticStepProfile {
    phases: Vec<DiagnosticStepPhaseTiming>,
}

impl DiagnosticStepProfile {
    /// Returns measured phases in execution order.
    #[must_use]
    pub fn phases(&self) -> &[DiagnosticStepPhaseTiming] {
        &self.phases
    }
}

pub(in crate::world) struct DiagnosticStepProfiler {
    maybe_phases: Option<Vec<DiagnosticStepPhaseTiming>>,
}

impl DiagnosticStepProfiler {
    pub(in crate::world) const fn disabled() -> Self {
        Self { maybe_phases: None }
    }

    pub(in crate::world) fn enabled() -> Self {
        Self {
            maybe_phases: Some(Vec::with_capacity(MAX_DIAGNOSTIC_STEP_PHASES)),
        }
    }

    pub(in crate::world) fn start(&self) -> Option<Instant> {
        self.maybe_phases.as_ref().map(|_phases| Instant::now())
    }

    pub(in crate::world) fn record(
        &mut self,
        phase: DiagnosticStepPhase,
        maybe_start: Option<Instant>,
    ) {
        let (Some(phases), Some(start)) = (&mut self.maybe_phases, maybe_start) else {
            return;
        };
        debug_assert!(phases.len() < MAX_DIAGNOSTIC_STEP_PHASES);
        phases.push(DiagnosticStepPhaseTiming {
            phase,
            duration: start.elapsed(),
        });
    }

    pub(in crate::world) fn finish(self) -> DiagnosticStepProfile {
        DiagnosticStepProfile {
            phases: self.maybe_phases.unwrap_or_default(),
        }
    }
}
