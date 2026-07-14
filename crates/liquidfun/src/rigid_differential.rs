//! Owned diagnostics reserved for the unpublished rigid-world differential harness.

use crate::math::Vec2;
use crate::{BodyId, BodySnapshot, FixtureId, ManagedContactSnapshot};

pub use crate::world::{
    BodyReconstruction, FixtureReconstruction, JointReconstruction, ReconstructionIndex,
    ReconstructionSupport, ReconstructionUnsupported, WorldDiagnostics, WorldReconstruction,
    WorldReconstructionError, WorldReconstructionLimits,
};

/// Reviewed private storage bounds for one TOI island diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RigidToiIslandLimits {
    max_bodies: usize,
    max_contacts: usize,
}

impl RigidToiIslandLimits {
    /// Returns the pinned `2 * b2_maxTOIContacts` body and
    /// `b2_maxTOIContacts` contact capacities.
    #[must_use]
    pub const fn reviewed() -> Self {
        Self {
            max_bodies: 2 * crate::math::settings::MAX_TOI_CONTACTS,
            max_contacts: crate::math::settings::MAX_TOI_CONTACTS,
        }
    }

    /// Returns the accepted body capacity.
    #[must_use]
    pub const fn max_bodies(self) -> usize {
        self.max_bodies
    }

    /// Returns the accepted contact capacity.
    #[must_use]
    pub const fn max_contacts(self) -> usize {
        self.max_contacts
    }
}

/// Bounded failure injection used only to prove TOI-event atomicity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidToiFailureInjection {
    /// Reject after every numerical and proxy candidate has been prepared.
    AfterSolve,
}

/// A bounded failure while solving one private TOI event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidToiSolveError {
    /// A reviewed private collection exceeded its finite capacity.
    CapacityExceeded {
        /// Stable semantic resource name.
        resource: &'static str,
        /// Configured finite limit.
        limit: usize,
    },
    /// Private world, graph, collision, or numerical state was not coherent.
    InvalidState,
    /// The requested diagnostic failure was injected after preparation.
    InjectedFailure,
}

/// Owned semantic evidence for one committed private TOI island.
#[derive(Debug, Clone, PartialEq)]
pub struct RigidToiEventDiagnostic {
    body_ids: Vec<BodyId>,
    contact_occurrences: Vec<u64>,
    transient_normal_impulse_sum: f32,
}

impl RigidToiEventDiagnostic {
    pub(crate) const fn new(
        body_ids: Vec<BodyId>,
        contact_occurrences: Vec<u64>,
        transient_normal_impulse_sum: f32,
    ) -> Self {
        Self {
            body_ids,
            contact_occurrences,
            transient_normal_impulse_sum,
        }
    }

    /// Returns bodies in exact TOI-island insertion order.
    #[must_use]
    pub fn body_ids(&self) -> &[BodyId] {
        &self.body_ids
    }

    /// Returns one-based contacts in exact TOI-island insertion order.
    #[must_use]
    pub fn contact_occurrences(&self) -> &[u64] {
        &self.contact_occurrences
    }

    /// Returns the transient solved normal-impulse sum without storing it.
    #[must_use]
    pub const fn transient_normal_impulse_sum(&self) -> f32 {
        self.transient_normal_impulse_sum
    }
}

/// Bounded semantic controls used only to witness private CCD rejection paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidCcdFailureInjection {
    /// Reject the named one-based manager occurrence after tentative refresh.
    RejectCandidate {
        /// One-based semantic manager occurrence.
        occurrence: u64,
    },
    /// Treat the named occurrence as having exhausted the pinned sub-step budget.
    ExhaustSubStepBudget {
        /// One-based semantic manager occurrence.
        occurrence: u64,
    },
}

/// A bounded failure while producing semantic CCD diagnostic evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidCcdScanError {
    /// A reviewed private scan collection exceeded its finite capacity.
    CapacityExceeded {
        /// Stable semantic resource name.
        resource: &'static str,
        /// Configured finite limit.
        limit: usize,
    },
    /// Private world/contact/sweep state was not coherent.
    InvalidState,
}

/// Owned semantic evidence for one accepted private CCD candidate.
#[derive(Debug, Clone, PartialEq)]
pub struct RigidCcdCandidateDiagnostic {
    occurrence: u64,
    alpha: f32,
    contact: ManagedContactSnapshot,
}

impl RigidCcdCandidateDiagnostic {
    pub(crate) const fn new(occurrence: u64, alpha: f32, contact: ManagedContactSnapshot) -> Self {
        Self {
            occurrence,
            alpha,
            contact,
        }
    }

    /// Returns the selected one-based manager occurrence.
    #[must_use]
    pub const fn occurrence(&self) -> u64 {
        self.occurrence
    }

    /// Returns the accepted absolute step fraction.
    #[must_use]
    pub const fn alpha(&self) -> f32 {
        self.alpha
    }

    /// Returns the accepted owned semantic contact snapshot.
    #[must_use]
    pub const fn contact(&self) -> &ManagedContactSnapshot {
        &self.contact
    }
}

/// Bounded failure injection used only to prove transactional world stepping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidStepFailureInjection {
    /// Reject after this many island solutions have been staged.
    LateIsland {
        /// Number of successfully staged island solutions before rejection.
        solved_islands: usize,
    },
    /// Reject the staged broad-phase synchronization for one fixture.
    ProxyBounds {
        /// Fixture whose prepared synchronization is rejected.
        fixture: FixtureId,
    },
}

/// Owned body state needed to compare the bounded Phase 6 solver witness.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RigidBodyDiagnostic {
    snapshot: BodySnapshot,
    linear_velocity: Vec2,
    angular_velocity: f32,
}

impl RigidBodyDiagnostic {
    pub(crate) const fn new(
        snapshot: BodySnapshot,
        linear_velocity: Vec2,
        angular_velocity: f32,
    ) -> Self {
        Self {
            snapshot,
            linear_velocity,
            angular_velocity,
        }
    }

    /// Returns the consumer-visible owned body snapshot.
    #[must_use]
    pub const fn snapshot(self) -> BodySnapshot {
        self.snapshot
    }

    /// Returns the solver's owned linear-velocity result.
    #[must_use]
    pub const fn linear_velocity(self) -> Vec2 {
        self.linear_velocity
    }

    /// Returns the solver's owned angular-velocity result.
    #[must_use]
    pub const fn angular_velocity(self) -> f32 {
        self.angular_velocity
    }
}

/// One owned manager occurrence with a non-storage diagnostic ordinal.
#[derive(Debug, Clone, PartialEq)]
pub struct RigidContactDiagnostic {
    occurrence: u64,
    contact: ManagedContactSnapshot,
}

impl RigidContactDiagnostic {
    pub(crate) const fn new(occurrence: u64, contact: ManagedContactSnapshot) -> Self {
        Self {
            occurrence,
            contact,
        }
    }

    /// Returns the one-based occurrence ordinal used only by differential evidence.
    #[must_use]
    pub const fn occurrence(&self) -> u64 {
        self.occurrence
    }

    /// Returns the bounded owned semantic contact state.
    #[must_use]
    pub const fn contact(&self) -> &ManagedContactSnapshot {
        &self.contact
    }
}

/// Diagnostic classification for a bounded private island-build failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RigidIslandBuildError {
    /// A reviewed scratch collection cannot contain the persistent graph.
    CapacityExceeded {
        /// Stable semantic resource name.
        resource: &'static str,
        /// Configured finite limit.
        limit: usize,
    },
    /// Private persistent graph invariants were not coherent.
    InvalidGraph,
}

/// Owned evidence for one ephemeral source-ordered rigid island.
#[derive(Debug, Clone, PartialEq)]
pub struct RigidIslandDiagnostic {
    body_ids: Vec<BodyId>,
    body_snapshots: Vec<BodySnapshot>,
    contact_occurrences: Vec<u64>,
    position_count: usize,
    velocity_count: usize,
    joint_count: usize,
}

impl RigidIslandDiagnostic {
    pub(crate) fn new(
        body_ids: Vec<BodyId>,
        body_snapshots: Vec<BodySnapshot>,
        contact_occurrences: Vec<u64>,
        position_count: usize,
        velocity_count: usize,
        joint_count: usize,
    ) -> Self {
        Self {
            body_ids,
            body_snapshots,
            contact_occurrences,
            position_count,
            velocity_count,
            joint_count,
        }
    }

    /// Returns body identities in solver-visible island order.
    #[must_use]
    pub fn body_ids(&self) -> &[BodyId] {
        &self.body_ids
    }

    /// Returns candidate body snapshots in matching island order.
    #[must_use]
    pub fn body_snapshots(&self) -> &[BodySnapshot] {
        &self.body_snapshots
    }

    /// Returns one-based semantic occurrences in solver-visible contact order.
    #[must_use]
    pub fn contact_occurrences(&self) -> &[u64] {
        &self.contact_occurrences
    }

    /// Returns the number of position scratch lanes.
    #[must_use]
    pub const fn position_count(&self) -> usize {
        self.position_count
    }

    /// Returns the number of velocity scratch lanes.
    #[must_use]
    pub const fn velocity_count(&self) -> usize {
        self.velocity_count
    }

    /// Returns the reserved Phase 8 joint-lane count.
    #[must_use]
    pub const fn joint_count(&self) -> usize {
        self.joint_count
    }
}
