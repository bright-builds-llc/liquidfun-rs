//! Clipping, semantic manifolds, and supported shape-pair dispatch.

mod clipping;

use crate::collision::{Manifold, PointState};

/// Ordered point-state transitions between two semantic manifolds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointStates {
    previous: [PointState; 2],
    current: [PointState; 2],
}

impl PointStates {
    /// Returns old-manifold states in old point order.
    #[must_use]
    pub const fn previous(&self) -> &[PointState; 2] {
        &self.previous
    }

    /// Returns new-manifold states in new point order.
    #[must_use]
    pub const fn current(&self) -> &[PointState; 2] {
        &self.current
    }
}

/// Classifies add, persist, and remove transitions by semantic feature identity.
#[must_use]
pub fn point_states(previous: &Manifold, current: &Manifold) -> PointStates {
    let mut previous_states = [PointState::Null; 2];
    let mut current_states = [PointState::Null; 2];

    for (index, point) in previous.points().iter().enumerate() {
        previous_states[index] = if current
            .points()
            .iter()
            .any(|candidate| candidate.feature_id() == point.feature_id())
        {
            PointState::Persisted
        } else {
            PointState::Removed
        };
    }
    for (index, point) in current.points().iter().enumerate() {
        current_states[index] = if previous
            .points()
            .iter()
            .any(|candidate| candidate.feature_id() == point.feature_id())
        {
            PointState::Persisted
        } else {
            PointState::Added
        };
    }

    PointStates {
        previous: previous_states,
        current: current_states,
    }
}
