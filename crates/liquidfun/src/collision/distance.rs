//! Shape-child distance, overlap, and reusable cache operations.

#[allow(dead_code)] // Task 2 consumes the completed proxy from production GJK.
mod proxy;

use crate::collision::CollisionError;
use crate::math::Vec2;
use crate::math::settings::EPSILON;

use proxy::{DistanceProxy, ProxyIdentity};

const MAX_SIMPLEX_VERTICES: usize = 3;

/// One semantic ordered support-index pair stored by a distance cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupportIndexPair {
    index_a: usize,
    index_b: usize,
}

impl SupportIndexPair {
    pub(super) const fn new(index_a: usize, index_b: usize) -> Self {
        Self { index_a, index_b }
    }

    /// Returns the source-ordered support index on shape A.
    #[must_use]
    pub const fn index_a(self) -> usize {
        self.index_a
    }

    /// Returns the source-ordered support index on shape B.
    #[must_use]
    pub const fn index_b(self) -> usize {
        self.index_b
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CacheEntry {
    index_a: usize,
    index_b: usize,
}

impl From<CacheEntry> for SupportIndexPair {
    fn from(entry: CacheEntry) -> Self {
        Self::new(entry.index_a, entry.index_b)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CacheBinding {
    proxy_a: ProxyIdentity,
    proxy_b: ProxyIdentity,
}

/// Initialized reusable state for source-ordered GJK distance calls.
///
/// The cache exposes only a semantic snapshot. Its bounded entries and private
/// topology binding cannot be constructed or mutated by callers.
///
/// ```compile_fail
/// use liquidfun::collision::distance::DistanceCache;
///
/// let _cache = DistanceCache {
///     entries: Vec::new(),
///     metric: 0.0,
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DistanceCache {
    entries: [CacheEntry; MAX_SIMPLEX_VERTICES],
    count: usize,
    metric: f32,
    maybe_binding: Option<CacheBinding>,
}

impl DistanceCache {
    /// Creates valid cold cache state for a first distance call.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            entries: [CacheEntry {
                index_a: 0,
                index_b: 0,
            }; MAX_SIMPLEX_VERTICES],
            count: 0,
            metric: 0.0,
            maybe_binding: None,
        }
    }

    /// Returns an owned semantic view without private topology identity.
    #[must_use]
    pub fn snapshot(&self) -> DistanceCacheSnapshot {
        let support_pairs = self.entries[..self.count]
            .iter()
            .copied()
            .map(SupportIndexPair::from)
            .collect();
        DistanceCacheSnapshot {
            support_pairs,
            metric: self.metric,
        }
    }

    #[allow(dead_code)] // Task 2 reads bounded entries into the simplex.
    fn entries(
        &self,
        proxy_a: &DistanceProxy<'_>,
        proxy_b: &DistanceProxy<'_>,
    ) -> Result<&[CacheEntry], CollisionError> {
        self.validate_binding(proxy_a, proxy_b)?;
        for entry in &self.entries[..self.count] {
            if entry.index_a >= proxy_a.vertex_count() || entry.index_b >= proxy_b.vertex_count() {
                return Err(CollisionError::IncompatibleDistanceCache);
            }
        }
        Ok(&self.entries[..self.count])
    }

    #[allow(dead_code)] // Task 2 applies the pinned metric flush window.
    const fn metric(&self) -> f32 {
        self.metric
    }

    #[allow(dead_code)] // Task 2 writes the solved simplex back to the cache.
    fn write(
        &mut self,
        proxy_a: &DistanceProxy<'_>,
        proxy_b: &DistanceProxy<'_>,
        metric: f32,
        support_pairs: &[SupportIndexPair],
    ) -> Result<(), CollisionError> {
        if support_pairs.len() > MAX_SIMPLEX_VERTICES || !metric.is_finite() {
            return Err(CollisionError::IncompatibleDistanceCache);
        }
        for pair in support_pairs {
            if pair.index_a >= proxy_a.vertex_count() || pair.index_b >= proxy_b.vertex_count() {
                return Err(CollisionError::IncompatibleDistanceCache);
            }
        }

        let mut entries = [CacheEntry::default(); MAX_SIMPLEX_VERTICES];
        for (entry, pair) in entries.iter_mut().zip(support_pairs) {
            *entry = CacheEntry {
                index_a: pair.index_a,
                index_b: pair.index_b,
            };
        }
        self.entries = entries;
        self.count = support_pairs.len();
        self.metric = metric;
        self.maybe_binding = Some(CacheBinding {
            proxy_a: proxy_a.identity(),
            proxy_b: proxy_b.identity(),
        });
        Ok(())
    }

    fn validate_binding(
        &self,
        proxy_a: &DistanceProxy<'_>,
        proxy_b: &DistanceProxy<'_>,
    ) -> Result<(), CollisionError> {
        let Some(binding) = &self.maybe_binding else {
            if self.count == 0 {
                return Ok(());
            }
            return Err(CollisionError::IncompatibleDistanceCache);
        };
        if binding.proxy_a != proxy_a.identity() || binding.proxy_b != proxy_b.identity() {
            return Err(CollisionError::IncompatibleDistanceCache);
        }
        Ok(())
    }
}

impl Default for DistanceCache {
    fn default() -> Self {
        Self::empty()
    }
}

/// An owned read-only semantic cache snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct DistanceCacheSnapshot {
    support_pairs: Vec<SupportIndexPair>,
    metric: f32,
}

impl DistanceCacheSnapshot {
    /// Returns the number of active cached simplex points.
    #[must_use]
    pub fn count(&self) -> usize {
        self.support_pairs.len()
    }

    /// Returns ordered semantic support-index pairs.
    #[must_use]
    pub fn support_pairs(&self) -> &[SupportIndexPair] {
        &self.support_pairs
    }

    /// Returns the cached segment length or triangle area metric.
    #[must_use]
    pub const fn metric(&self) -> f32 {
        self.metric
    }
}

/// The initialized result of one source-ordered GJK distance call.
#[derive(Debug, Clone, PartialEq)]
pub struct DistanceResult {
    point_a: Vec2,
    point_b: Vec2,
    distance: f32,
    iterations: usize,
    cache: DistanceCache,
}

impl DistanceResult {
    /// Returns the closest point on shape A.
    #[must_use]
    pub const fn point_a(&self) -> Vec2 {
        self.point_a
    }

    /// Returns the closest point on shape B.
    #[must_use]
    pub const fn point_b(&self) -> Vec2 {
        self.point_b
    }

    /// Returns the non-negative closest distance.
    #[must_use]
    pub const fn distance(&self) -> f32 {
        self.distance
    }

    /// Returns the number of support-point calls.
    #[must_use]
    pub const fn iterations(&self) -> usize {
        self.iterations
    }

    /// Returns the topology-bound cache produced by this call.
    #[must_use]
    pub const fn cache(&self) -> &DistanceCache {
        &self.cache
    }
}

#[allow(dead_code)] // Task 2 applies this branch after reconstructing a simplex.
fn cache_metric_requires_flush(metric1: f32, metric2: f32) -> bool {
    metric2 < 0.5 * metric1 || 2.0 * metric1 < metric2 || metric2 < EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collision::ChildIndex;
    use crate::collision::shape::{CircleShape, EdgeShape, PolygonShape, Shape};
    use crate::math::Vec2;

    fn circle(center: Vec2) -> Shape {
        CircleShape::new(center, 1.0)
            .expect("circle should be valid")
            .into()
    }

    #[test]
    fn cache_compatible_reuse_preserves_ordered_pairs() {
        // Arrange
        let shape_a = circle(Vec2::ZERO);
        let shape_b = EdgeShape::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0))
            .expect("edge should be valid")
            .into();
        let child = ChildIndex::new(0, 1).expect("child should exist");
        let proxy_a = DistanceProxy::new(&shape_a, child).expect("proxy should be valid");
        let proxy_b = DistanceProxy::new(&shape_b, child).expect("proxy should be valid");
        let pairs = [SupportIndexPair::new(0, 1)];
        let mut cache = DistanceCache::empty();
        cache
            .write(&proxy_a, &proxy_b, 0.0, &pairs)
            .expect("cache write should be valid");

        // Act
        let entries = cache
            .entries(&proxy_a, &proxy_b)
            .expect("same topology should be compatible");

        // Assert
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].index_a, 0);
        assert_eq!(entries[0].index_b, 1);
    }

    #[test]
    fn cache_rejects_cross_topology_reuse_before_indexing() {
        // Arrange
        let shape_a = circle(Vec2::ZERO);
        let shape_b = circle(Vec2::new(2.0, 0.0));
        let polygon: Shape = PolygonShape::box_shape(1.0, 1.0)
            .expect("polygon should be valid")
            .into();
        let child = ChildIndex::new(0, 1).expect("child should exist");
        let proxy_a = DistanceProxy::new(&shape_a, child).expect("proxy should be valid");
        let proxy_b = DistanceProxy::new(&shape_b, child).expect("proxy should be valid");
        let polygon_proxy = DistanceProxy::new(&polygon, child).expect("proxy should be valid");
        let mut cache = DistanceCache::empty();
        cache
            .write(&proxy_a, &proxy_b, 0.0, &[SupportIndexPair::new(0, 0)])
            .expect("cache write should be valid");

        // Act
        let result = cache.entries(&polygon_proxy, &proxy_b);

        // Assert
        assert_eq!(result, Err(CollisionError::IncompatibleDistanceCache));
    }

    #[test]
    fn cache_ratio_boundaries_are_inclusive() {
        // Arrange
        let metric = 4.0;

        // Act
        let half_flushes = cache_metric_requires_flush(metric, 2.0);
        let double_flushes = cache_metric_requires_flush(metric, 8.0);
        let below_half_flushes =
            cache_metric_requires_flush(metric, f32::from_bits(2.0_f32.to_bits() - 1));
        let above_double_flushes =
            cache_metric_requires_flush(metric, f32::from_bits(8.0_f32.to_bits() + 1));

        // Assert
        assert!(!half_flushes);
        assert!(!double_flushes);
        assert!(below_half_flushes);
        assert!(above_double_flushes);
    }

    #[test]
    fn cache_epsilon_flush_is_strict() {
        // Arrange
        let below = f32::from_bits(EPSILON.to_bits() - 1);

        // Act
        let below_flushes = cache_metric_requires_flush(EPSILON, below);
        let equal_flushes = cache_metric_requires_flush(EPSILON, EPSILON);

        // Assert
        assert!(below_flushes);
        assert!(!equal_flushes);
    }
}
