use super::{DestroyedId, DestructionCause, DestructionRecord, ObjectSnapshot};
use crate::world::step::LifecycleEvent;

/// Owned result and source-ordered lifecycle evidence for one direct mutation.
#[derive(Debug, Clone, PartialEq)]
pub struct MutationReport<T> {
    value: T,
    lifecycle: Vec<LifecycleEvent>,
}

impl<T> MutationReport<T> {
    pub(crate) const fn new(value: T, lifecycle: Vec<LifecycleEvent>) -> Self {
        Self { value, lifecycle }
    }

    /// Returns the mutation's owned result.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Returns the authoritative source-ordered lifecycle timeline.
    #[must_use]
    pub fn lifecycle(&self) -> &[LifecycleEvent] {
        &self.lifecycle
    }

    /// Consumes the report and returns its owned result.
    #[must_use]
    pub fn into_value(self) -> T {
        self.value
    }
}

/// Owned destruction records paired with their authoritative lifecycle timeline.
pub type DestructionReport = MutationReport<Vec<DestructionRecord>>;

impl MutationReport<Vec<DestructionRecord>> {
    /// Returns invalidation records in deterministic occurrence order.
    #[must_use]
    pub fn records(&self) -> &[DestructionRecord] {
        &self.value
    }

    /// Returns the number of invalidation records.
    #[must_use]
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// Returns whether the report contains no invalidations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    /// Iterates invalidation records in deterministic occurrence order.
    pub fn iter(&self) -> std::slice::Iter<'_, DestructionRecord> {
        self.value.iter()
    }

    /// Returns a record by occurrence index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&DestructionRecord> {
        self.value.get(index)
    }

    /// Returns the first invalidation record.
    #[must_use]
    pub fn first(&self) -> Option<&DestructionRecord> {
        self.value.first()
    }

    /// Returns the last invalidation record.
    #[must_use]
    pub fn last(&self) -> Option<&DestructionRecord> {
        self.value.last()
    }

    /// Returns the root invalidation record.
    ///
    /// # Panics
    ///
    /// Panics if an internally constructed report violates the invariant that a
    /// successful destruction always records its root invalidation.
    #[must_use]
    pub fn root(&self) -> &DestructionRecord {
        self.value
            .last()
            .expect("a successful destruction report always contains its root")
    }

    /// Returns the exact typed identity invalidated by the root operation.
    #[must_use]
    pub fn destroyed(&self) -> DestroyedId {
        self.root().destroyed()
    }

    /// Returns why the root object was invalidated.
    #[must_use]
    pub fn cause(&self) -> DestructionCause {
        self.root().cause()
    }

    /// Returns semantic root state captured before invalidation.
    #[must_use]
    pub fn snapshot(&self) -> &ObjectSnapshot {
        self.root().snapshot()
    }
}

impl std::ops::Deref for MutationReport<Vec<DestructionRecord>> {
    type Target = DestructionRecord;

    fn deref(&self) -> &Self::Target {
        self.root()
    }
}

impl std::ops::Index<usize> for MutationReport<Vec<DestructionRecord>> {
    type Output = DestructionRecord;

    fn index(&self, index: usize) -> &Self::Output {
        &self.value[index]
    }
}

impl<'report> IntoIterator for &'report MutationReport<Vec<DestructionRecord>> {
    type Item = &'report DestructionRecord;
    type IntoIter = std::slice::Iter<'report, DestructionRecord>;

    fn into_iter(self) -> Self::IntoIter {
        self.value.iter()
    }
}

impl IntoIterator for MutationReport<Vec<DestructionRecord>> {
    type Item = DestructionRecord;
    type IntoIter = std::vec::IntoIter<DestructionRecord>;

    fn into_iter(self) -> Self::IntoIter {
        self.value.into_iter()
    }
}
