use std::cmp::Ordering;

/// A value with an explicit stable semantic key and deterministic tie-breaker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalValue<K, T> {
    stable_key: K,
    tie_breaker: T,
}

impl<K, T> CanonicalValue<K, T> {
    /// Creates one value eligible for explicitly unordered comparison.
    #[must_use]
    pub const fn new(stable_key: K, tie_breaker: T) -> Self {
        Self {
            stable_key,
            tie_breaker,
        }
    }
}

impl<K: Ord, T: Ord> Ord for CanonicalValue<K, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.stable_key
            .cmp(&other.stable_key)
            .then_with(|| self.tie_breaker.cmp(&other.tie_breaker))
    }
}

impl<K: Ord, T: Ord> PartialOrd for CanonicalValue<K, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Typed collection semantics; only `Set` and `Multiset` canonicalize.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticCollection<K, T> {
    /// Solver-significant order is authoritative.
    Ordered(Vec<CanonicalValue<K, T>>),
    /// Unique semantic values compare independent of order.
    Set(Vec<CanonicalValue<K, T>>),
    /// Semantic values compare independent of order while retaining multiplicity.
    Multiset(Vec<CanonicalValue<K, T>>),
}

/// Compares only collections carrying the same explicit typed semantics.
#[must_use]
pub fn collections_match<K, T>(
    expected: &SemanticCollection<K, T>,
    actual: &SemanticCollection<K, T>,
) -> bool
where
    K: Clone + Ord,
    T: Clone + Ord,
{
    match (expected, actual) {
        (SemanticCollection::Ordered(expected), SemanticCollection::Ordered(actual)) => {
            expected == actual
        }
        (SemanticCollection::Set(expected), SemanticCollection::Set(actual)) => {
            canonical_set(expected) == canonical_set(actual)
        }
        (SemanticCollection::Multiset(expected), SemanticCollection::Multiset(actual)) => {
            canonical_multiset(expected) == canonical_multiset(actual)
        }
        _ => false,
    }
}

fn canonical_set<K: Clone + Ord, T: Clone + Ord>(
    values: &[CanonicalValue<K, T>],
) -> Vec<CanonicalValue<K, T>> {
    let mut canonical = canonical_multiset(values);
    canonical.dedup();
    canonical
}

fn canonical_multiset<K: Clone + Ord, T: Clone + Ord>(
    values: &[CanonicalValue<K, T>],
) -> Vec<CanonicalValue<K, T>> {
    let mut canonical = values.to_vec();
    canonical.sort();
    canonical
}
