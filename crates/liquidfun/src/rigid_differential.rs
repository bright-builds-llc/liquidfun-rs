//! Owned diagnostics reserved for the unpublished rigid-world differential harness.

use crate::math::Vec2;
use crate::{BodyId, BodySnapshot, FixtureId, ManagedContactSnapshot};

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
