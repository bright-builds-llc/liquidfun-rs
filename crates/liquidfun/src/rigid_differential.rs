//! Owned diagnostics reserved for the unpublished rigid-world differential harness.

use crate::math::Vec2;
use crate::{BodySnapshot, ManagedContactSnapshot};

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
