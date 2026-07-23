//! Nondeterministic wall-clock diagnostics kept outside semantic step evidence.

use std::time::{Duration, Instant};

const MAX_DIAGNOSTIC_STEP_PHASES: usize = 32;

/// Closed schema identity for the public diagnostic profile structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DiagnosticProfileSchema {
    /// Storage-neutral Phase 12 parent/child profile vocabulary.
    Phase12V1,
}

impl DiagnosticProfileSchema {
    /// Returns the stable profile schema token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Phase12V1 => "phase12-profile-v1",
        }
    }
}

/// Comparable storage-neutral parent phases.
///
/// These names describe semantic execution regions shared by engine
/// implementations. They deliberately expose no private rows, caches, arenas,
/// solver indices, or allocation details.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DiagnosticProfileParent {
    /// Contact discovery, filtering, updates, and callbacks.
    ContactUpdate,
    /// Discrete rigid-body and joint constraint solving.
    RigidSolve,
    /// Continuous-collision candidate and TOI processing.
    ContinuousSolve,
    /// Particle lifecycle and solver-input preparation.
    ParticlePrepare,
    /// Particle constraint and material-pass solving.
    ParticleSolve,
    /// Deferred commands, report assembly, and successful-step bookkeeping.
    Finalize,
}

impl DiagnosticProfileParent {
    /// Every common parent in stable schema order.
    pub const ALL: [Self; 6] = [
        Self::ContactUpdate,
        Self::RigidSolve,
        Self::ContinuousSolve,
        Self::ParticlePrepare,
        Self::ParticleSolve,
        Self::Finalize,
    ];

    /// Returns the stable common-parent token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ContactUpdate => "contact_update",
            Self::RigidSolve => "rigid_solve",
            Self::ContinuousSolve => "continuous_solve",
            Self::ParticlePrepare => "particle_prepare",
            Self::ParticleSolve => "particle_solve",
            Self::Finalize => "finalize",
        }
    }
}

/// Optional Rust-only diagnostic child phases.
///
/// Child names identify semantic algorithm clusters without exposing storage.
/// Each child maps to exactly one common parent. Child durations are optional
/// diagnostics and are never added to or substituted for parent durations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DiagnosticProfileChild {
    /// Rust contact-manager lifecycle cluster.
    RustContactManager,
    /// Rust island construction and solve cluster.
    RustIslandSolve,
    /// Rust continuous TOI selection and solve loop.
    RustToiLoop,
    /// Rust particle lifecycle and contact-preparation cluster.
    RustParticleLifecycle,
    /// Rust closed particle pass graph.
    RustParticlePassGraph,
    /// Rust deferred-command and report-assembly cluster.
    RustReportAssembly,
}

impl DiagnosticProfileChild {
    /// Every optional Rust child in stable schema order.
    pub const ALL: [Self; 6] = [
        Self::RustContactManager,
        Self::RustIslandSolve,
        Self::RustToiLoop,
        Self::RustParticleLifecycle,
        Self::RustParticlePassGraph,
        Self::RustReportAssembly,
    ];

    /// Returns the stable Rust-only child token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RustContactManager => "rust_contact_manager",
            Self::RustIslandSolve => "rust_island_solve",
            Self::RustToiLoop => "rust_toi_loop",
            Self::RustParticleLifecycle => "rust_particle_lifecycle",
            Self::RustParticlePassGraph => "rust_particle_pass_graph",
            Self::RustReportAssembly => "rust_report_assembly",
        }
    }

    /// Returns the storage-neutral parent that owns this child.
    #[must_use]
    pub const fn parent(self) -> DiagnosticProfileParent {
        match self {
            Self::RustContactManager => DiagnosticProfileParent::ContactUpdate,
            Self::RustIslandSolve => DiagnosticProfileParent::RigidSolve,
            Self::RustToiLoop => DiagnosticProfileParent::ContinuousSolve,
            Self::RustParticleLifecycle => DiagnosticProfileParent::ParticlePrepare,
            Self::RustParticlePassGraph => DiagnosticProfileParent::ParticleSolve,
            Self::RustReportAssembly => DiagnosticProfileParent::Finalize,
        }
    }
}

/// Closed diagnostic step phase identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum DiagnosticStepPhase {
    /// Common storage-neutral phase suitable for structural comparison.
    Common(DiagnosticProfileParent),
    /// Optional Rust-only diagnostic child.
    RustChild(DiagnosticProfileChild),
}

impl DiagnosticStepPhase {
    /// Compatibility alias for the Phase 12 `contact_update` parent.
    #[allow(non_upper_case_globals)]
    pub const ContactLifecycle: Self = Self::Common(DiagnosticProfileParent::ContactUpdate);

    /// Compatibility alias for the Phase 12 `particle_solve` parent.
    #[allow(non_upper_case_globals)]
    pub const ParticleSolve: Self = Self::Common(DiagnosticProfileParent::ParticleSolve);

    /// Compatibility alias for the Phase 12 `rigid_solve` parent.
    #[allow(non_upper_case_globals)]
    pub const RigidSolve: Self = Self::Common(DiagnosticProfileParent::RigidSolve);

    /// Compatibility alias for the Phase 12 `continuous_solve` parent.
    #[allow(non_upper_case_globals)]
    pub const ContinuousSolve: Self = Self::Common(DiagnosticProfileParent::ContinuousSolve);

    /// Compatibility alias for the Rust-only deferred-command child.
    #[allow(non_upper_case_globals)]
    pub const ApplyCommands: Self = Self::RustChild(DiagnosticProfileChild::RustReportAssembly);

    /// Compatibility alias for the Phase 12 `finalize` parent.
    #[allow(non_upper_case_globals)]
    pub const Finalize: Self = Self::Common(DiagnosticProfileParent::Finalize);

    /// Returns the phase's common storage-neutral parent.
    #[must_use]
    pub const fn parent(self) -> DiagnosticProfileParent {
        match self {
            Self::Common(parent) => parent,
            Self::RustChild(child) => child.parent(),
        }
    }

    /// Returns this phase's parent only when the phase itself is common.
    #[must_use]
    pub const fn maybe_common_parent(self) -> Option<DiagnosticProfileParent> {
        match self {
            Self::Common(parent) => Some(parent),
            Self::RustChild(_) => None,
        }
    }

    /// Returns the stable schema token for this phase.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Common(parent) => parent.as_str(),
            Self::RustChild(child) => child.as_str(),
        }
    }
}

/// One nondeterministic wall-clock [`Duration`] for a named diagnostic phase.
///
/// This type deliberately implements neither `PartialEq`, `Hash`, nor
/// serialization. It has no conversion into a deterministic checkpoint,
/// semantic report, or parity record:
///
/// ```compile_fail
/// use liquidfun::DiagnosticStepPhaseTiming;
/// fn compare(left: DiagnosticStepPhaseTiming, right: DiagnosticStepPhaseTiming) {
///     let _same = left == right;
/// }
/// ```
///
/// ```compile_fail
/// use std::hash::Hash;
/// use liquidfun::DiagnosticStepPhaseTiming;
/// fn require_hash<T: Hash>() {}
/// require_hash::<DiagnosticStepPhaseTiming>();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DiagnosticStepPhaseTiming {
    phase: DiagnosticStepPhase,
    duration: Duration,
}

impl DiagnosticStepPhaseTiming {
    /// Returns the closed structural phase name.
    #[must_use]
    pub const fn phase(self) -> DiagnosticStepPhase {
        self.phase
    }

    /// Returns nondeterministic elapsed wall-clock time as [`Duration`].
    ///
    /// This value is diagnostic only and must not enter compatibility or
    /// deterministic parity comparisons.
    #[must_use]
    pub const fn duration(self) -> Duration {
        self.duration
    }
}

/// A bounded diagnostic-only phase profile from one successful step.
///
/// The profile is intentionally separate from [`crate::StepReport`] and
/// implements neither equality, hashing, nor serialization. Phase names and
/// parent relationships are structural; durations are wall-clock diagnostics,
/// never D0/D1 physics evidence.
#[derive(Debug, Clone)]
pub struct DiagnosticStepProfile {
    schema: DiagnosticProfileSchema,
    phases: Vec<DiagnosticStepPhaseTiming>,
    complete: bool,
}

impl DiagnosticStepProfile {
    /// Maximum number of phase records accepted by one successful step.
    pub const MAXIMUM_PHASES: usize = MAX_DIAGNOSTIC_STEP_PHASES;

    /// Returns the versioned structural profile schema.
    #[must_use]
    pub const fn schema(&self) -> DiagnosticProfileSchema {
        self.schema
    }

    /// Returns measured phases in execution-boundary order.
    #[must_use]
    pub fn phases(&self) -> &[DiagnosticStepPhaseTiming] {
        &self.phases
    }

    /// Reports whether every attempted phase fit within the reviewed bound.
    ///
    /// A false result invalidates the profile as structural evidence but never
    /// changes the successful semantic [`crate::StepReport`].
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.complete
    }
}

#[derive(Clone, Copy)]
struct DiagnosticPhaseBuffer {
    phases: [Option<DiagnosticStepPhaseTiming>; MAX_DIAGNOSTIC_STEP_PHASES],
    len: usize,
    overflowed: bool,
}

impl DiagnosticPhaseBuffer {
    const fn new() -> Self {
        Self {
            phases: [None; MAX_DIAGNOSTIC_STEP_PHASES],
            len: 0,
            overflowed: false,
        }
    }

    fn push(&mut self, timing: DiagnosticStepPhaseTiming) {
        let Some(slot) = self.phases.get_mut(self.len) else {
            self.overflowed = true;
            return;
        };
        *slot = Some(timing);
        self.len += 1;
    }

    fn into_phases(self) -> Vec<DiagnosticStepPhaseTiming> {
        self.phases.into_iter().take(self.len).flatten().collect()
    }
}

pub(in crate::world) struct DiagnosticStepProfiler {
    maybe_buffer: Option<DiagnosticPhaseBuffer>,
}

impl DiagnosticStepProfiler {
    pub(in crate::world) const fn disabled() -> Self {
        Self { maybe_buffer: None }
    }

    pub(in crate::world) const fn enabled() -> Self {
        Self {
            maybe_buffer: Some(DiagnosticPhaseBuffer::new()),
        }
    }

    pub(in crate::world) fn start(&self) -> Option<Instant> {
        self.maybe_buffer.as_ref().map(|_buffer| Instant::now())
    }

    pub(in crate::world) fn record(
        &mut self,
        phase: DiagnosticStepPhase,
        maybe_start: Option<Instant>,
    ) {
        let (Some(buffer), Some(start)) = (&mut self.maybe_buffer, maybe_start) else {
            return;
        };
        buffer.push(DiagnosticStepPhaseTiming {
            phase,
            duration: start.elapsed(),
        });
    }

    pub(in crate::world) fn finish(self) -> DiagnosticStepProfile {
        let (phases, complete) = match self.maybe_buffer {
            Some(buffer) => {
                let complete = !buffer.overflowed;
                (buffer.into_phases(), complete)
            }
            None => (Vec::new(), true),
        };
        DiagnosticStepProfile {
            schema: DiagnosticProfileSchema::Phase12V1,
            phases,
            complete,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DiagnosticStepProfiler;

    #[test]
    fn disabled_profiler_contains_no_phase_buffer() {
        // Arrange
        let profiler = DiagnosticStepProfiler::disabled();

        // Act
        let maybe_start = profiler.start();

        // Assert
        assert!(maybe_start.is_none());
        assert!(profiler.maybe_buffer.is_none());
    }
}
