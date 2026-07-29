//! Bounded, renderer-neutral semantic diagnostics for rigid differential evidence.

mod reconstruction;
#[cfg(feature = "differential-internals")]
pub use reconstruction::{
    BodyReconstruction, FixtureReconstruction, JointReconstruction, ReconstructionIndex,
    ReconstructionSupport, ReconstructionUnsupported, WorldReconstruction,
    WorldReconstructionError, WorldReconstructionLimits,
};

/// Exact renderer-neutral world counts and dynamic-tree metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldDiagnostics {
    body_count: usize,
    fixture_count: usize,
    joint_count: usize,
    contact_count: usize,
    manifold_point_count: usize,
    proxy_count: usize,
    tree_height: i32,
    tree_balance: i32,
    tree_quality: f32,
}

impl WorldDiagnostics {
    /// Returns the exact live body count.
    #[must_use]
    pub const fn body_count(self) -> usize {
        self.body_count
    }

    /// Returns the exact live fixture count.
    #[must_use]
    pub const fn fixture_count(self) -> usize {
        self.fixture_count
    }

    /// Returns the exact live joint count.
    #[must_use]
    pub const fn joint_count(self) -> usize {
        self.joint_count
    }

    /// Returns the exact private contact-occurrence count.
    #[must_use]
    pub const fn contact_count(self) -> usize {
        self.contact_count
    }

    /// Returns the exact total number of current manifold points.
    #[must_use]
    pub const fn manifold_point_count(self) -> usize {
        self.manifold_point_count
    }

    /// Returns the exact broad-phase proxy count.
    #[must_use]
    pub const fn proxy_count(self) -> usize {
        self.proxy_count
    }

    /// Returns the exact dynamic-tree root height.
    #[must_use]
    pub const fn tree_height(self) -> i32 {
        self.tree_height
    }

    /// Returns the exact maximum dynamic-tree child-height difference.
    #[must_use]
    pub const fn tree_balance(self) -> i32 {
        self.tree_balance
    }

    /// Returns the exact total-to-root dynamic-tree perimeter ratio.
    ///
    /// Later differential policy names how this floating observation compares.
    #[must_use]
    pub const fn tree_quality(self) -> f32 {
        self.tree_quality
    }
}
