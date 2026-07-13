//! Borrow-scoped fixture queries over private broad-phase storage.

use crate::collision::{Aabb, ChildIndex, QueryControl};
use crate::{FixtureId, World};

/// Controls whether an AABB query continues visiting fixture occurrences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryDirective {
    /// Continue visiting overlapping fixture children.
    Continue,
    /// Stop the query immediately.
    Terminate,
}

/// One borrow-scoped fixture-child occurrence from a world AABB query.
///
/// Multi-child fixtures can produce more than one occurrence. No private
/// broad-phase or tree identity is exposed.
#[derive(Debug, PartialEq, Eq)]
pub struct FixtureQueryOccurrence {
    fixture: FixtureId,
    child_index: ChildIndex,
}

impl FixtureQueryOccurrence {
    /// Returns the semantic fixture identity for this occurrence.
    #[must_use]
    pub const fn fixture(&self) -> FixtureId {
        self.fixture
    }

    /// Returns the checked shape-child coordinate for this occurrence.
    #[must_use]
    pub const fn child_index(&self) -> ChildIndex {
        self.child_index
    }
}

impl World {
    /// Visits fixture children whose broad-phase bounds overlap `aabb`.
    ///
    /// The visitor receives semantic fixture and child identities only for the
    /// duration of each call. Query order is intentionally unspecified.
    /// Collision [`crate::collision::FilterData`] is not applied automatically,
    /// and occurrences from the same multi-child fixture are not deduplicated.
    /// Because this method borrows the world immutably for the complete query,
    /// the visitor cannot mutate world objects during traversal.
    pub fn query_aabb<F>(&self, aabb: Aabb, mut visitor: F)
    where
        F: FnMut(&FixtureQueryOccurrence) -> QueryDirective,
    {
        self.broad_phase.query_aabb(aabb, |proxy| {
            let occurrence = FixtureQueryOccurrence {
                fixture: proxy.fixture,
                child_index: proxy.child_index,
            };
            match visitor(&occurrence) {
                QueryDirective::Continue => QueryControl::Continue,
                QueryDirective::Terminate => QueryControl::Stop,
            }
        });
    }
}
