//! Shape-child distance, overlap, and reusable cache operations.

use std::fmt;

mod proxy;
mod simplex;

use crate::collision::shape::Shape;
use crate::collision::{ChildIndex, CollisionError};
use crate::math::Vec2;
use crate::math::settings::EPSILON;

use proxy::{DistanceProxy, ProxyIdentity};
use simplex::Simplex;

const MAX_SIMPLEX_VERTICES: usize = 3;
const MAX_GJK_ITERATIONS: usize = 20;

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
#[derive(Clone, PartialEq)]
pub struct DistanceCache {
    entries: [CacheEntry; MAX_SIMPLEX_VERTICES],
    count: usize,
    metric: f32,
    maybe_binding: Option<CacheBinding>,
}

impl fmt::Debug for DistanceCache {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DistanceCache")
            .field("snapshot", &self.snapshot())
            .finish()
    }
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

    const fn metric(&self) -> f32 {
        self.metric
    }

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
#[derive(Clone, PartialEq)]
pub struct DistanceResult {
    point_a: Vec2,
    point_b: Vec2,
    distance: f32,
    iterations: usize,
    cache: DistanceCache,
    diagnostic_trace: GjkDiagnosticTrace,
}

impl fmt::Debug for DistanceResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DistanceResult")
            .field("point_a", &self.point_a)
            .field("point_b", &self.point_b)
            .field("distance", &self.distance)
            .field("iterations", &self.iterations)
            .field("cache", &self.cache)
            .finish_non_exhaustive()
    }
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

fn cache_metric_requires_flush(metric1: f32, metric2: f32) -> bool {
    metric2 < 0.5 * metric1 || 2.0 * metric1 < metric2 || metric2 < EPSILON
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GjkTermination {
    Triangle,
    NearZeroDirection,
    DuplicateSupport,
    IterationLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GjkDiagnosticStep {
    simplex_count: usize,
    support_pair: SupportIndexPair,
    closest_non_decrease: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GjkDiagnosticTrace {
    steps: Vec<GjkDiagnosticStep>,
    termination: GjkTermination,
}

/// Computes closest witnesses through one bounded source-ordered GJK path.
///
/// A supplied cache must have been produced for the same ordered shape-child
/// topology. Incompatible reuse is rejected before any cached index access.
///
/// # Errors
///
/// Returns a typed error for invalid child selection, incompatible cache
/// topology, non-finite transforms, or non-finite derived geometry.
#[allow(clippy::too_many_arguments)] // Mirrors the two complete shape-child inputs.
pub fn distance(
    shape_a: &Shape,
    child_a: ChildIndex,
    transform_a: crate::math::Transform,
    shape_b: &Shape,
    child_b: ChildIndex,
    transform_b: crate::math::Transform,
    use_radii: bool,
    maybe_cache: Option<&DistanceCache>,
) -> Result<DistanceResult, CollisionError> {
    super::shape::validate_transform(transform_a)?;
    super::shape::validate_transform(transform_b)?;
    let proxy_a = DistanceProxy::new(shape_a, child_a)?;
    let proxy_b = DistanceProxy::new(shape_b, child_b)?;
    let mut cache = maybe_cache.cloned().unwrap_or_default();
    let mut simplex = Simplex::read_cache(
        cache.entries(&proxy_a, &proxy_b)?,
        cache.metric(),
        &proxy_a,
        transform_a,
        &proxy_b,
        transform_b,
    );

    let mut iterations = 0;
    let mut previous_distance_squared = f32::MAX;
    let mut diagnostic_steps = Vec::with_capacity(MAX_GJK_ITERATIONS);
    let termination = loop {
        let saved_pairs = simplex.saved_support_pairs();
        simplex.solve();
        if simplex.count() == 3 {
            break GjkTermination::Triangle;
        }

        let closest = simplex.closest_point();
        let distance_squared = closest.length_squared();
        let closest_non_decrease = distance_squared >= previous_distance_squared;
        previous_distance_squared = distance_squared;
        let direction = simplex.search_direction();
        if direction.length_squared() < EPSILON * EPSILON {
            break GjkTermination::NearZeroDirection;
        }

        let support_pair =
            simplex.append_support(&proxy_a, transform_a, &proxy_b, transform_b, direction);
        iterations += 1;
        diagnostic_steps.push(GjkDiagnosticStep {
            simplex_count: simplex.count(),
            support_pair,
            closest_non_decrease,
        });
        if saved_pairs.contains(support_pair) {
            break GjkTermination::DuplicateSupport;
        }
        simplex.accept_support();
        if iterations == MAX_GJK_ITERATIONS {
            break GjkTermination::IterationLimit;
        }
    };

    let (mut point_a, mut point_b) = simplex.witness_points();
    let mut closest_distance = (point_a - point_b).length();
    let (support_pairs, support_count) = simplex.support_pairs();
    cache.write(
        &proxy_a,
        &proxy_b,
        simplex.metric(),
        &support_pairs[..support_count],
    )?;

    if use_radii {
        let radius_a = proxy_a.radius();
        let radius_b = proxy_b.radius();
        if closest_distance > radius_a + radius_b && closest_distance > EPSILON {
            closest_distance -= radius_a + radius_b;
            let mut normal = point_b - point_a;
            normal.normalize();
            point_a += radius_a * normal;
            point_b -= radius_b * normal;
        } else {
            let midpoint = 0.5 * (point_a + point_b);
            point_a = midpoint;
            point_b = midpoint;
            closest_distance = 0.0;
        }
    }

    if !point_a.is_valid()
        || !point_b.is_valid()
        || !closest_distance.is_finite()
        || closest_distance < 0.0
    {
        return Err(CollisionError::NonFiniteValue);
    }
    Ok(DistanceResult {
        point_a,
        point_b,
        distance: closest_distance,
        iterations,
        cache,
        diagnostic_trace: GjkDiagnosticTrace {
            steps: diagnostic_steps,
            termination,
        },
    })
}

/// Tests overlap with radii and the pinned strict `10 * EPSILON` predicate.
///
/// # Errors
///
/// Returns the same checked child, transform, and derived-geometry errors as
/// [`distance`].
#[allow(clippy::too_many_arguments)] // Mirrors two complete shape-child inputs.
pub fn test_overlap(
    shape_a: &Shape,
    child_a: ChildIndex,
    transform_a: crate::math::Transform,
    shape_b: &Shape,
    child_b: ChildIndex,
    transform_b: crate::math::Transform,
) -> Result<bool, CollisionError> {
    let result = distance(
        shape_a,
        child_a,
        transform_a,
        shape_b,
        child_b,
        transform_b,
        true,
        None,
    )?;
    Ok(result.distance < 10.0 * EPSILON)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collision::ChildIndex;
    use crate::collision::shape::{CircleShape, EdgeShape, PolygonShape, Shape};
    use crate::math::Transform;
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

    #[test]
    fn gjk_identical_points_terminate_on_near_zero_direction() {
        // Arrange
        let shape_a = circle(Vec2::ZERO);
        let shape_b = circle(Vec2::ZERO);
        let child = shape_a.child_index(0).expect("child should exist");

        // Act
        let result = distance(
            &shape_a,
            child,
            Transform::IDENTITY,
            &shape_b,
            child,
            Transform::IDENTITY,
            false,
            None,
        )
        .expect("distance should succeed");

        // Assert
        assert_eq!(
            result.diagnostic_trace.termination,
            GjkTermination::NearZeroDirection
        );
        assert_eq!(result.iterations, 0);
    }

    #[test]
    fn gjk_separated_points_terminate_on_duplicate_support() {
        // Arrange
        let shape_a = circle(Vec2::ZERO);
        let shape_b = circle(Vec2::new(4.0, 0.0));
        let child = shape_a.child_index(0).expect("child should exist");

        // Act
        let result = distance(
            &shape_a,
            child,
            Transform::IDENTITY,
            &shape_b,
            child,
            Transform::IDENTITY,
            false,
            None,
        )
        .expect("distance should succeed");

        // Assert
        assert_eq!(
            result.diagnostic_trace.termination,
            GjkTermination::DuplicateSupport
        );
        assert_eq!(result.diagnostic_trace.steps.len(), 1);
    }

    #[test]
    fn gjk_overlapping_polygons_terminate_with_triangle_simplex() {
        // Arrange
        let shape_a: Shape = PolygonShape::box_shape(1.0, 1.0)
            .expect("polygon should be valid")
            .into();
        let shape_b: Shape = PolygonShape::oriented_box(1.0, 1.0, Vec2::new(0.25, 0.1), 0.2)
            .expect("polygon should be valid")
            .into();
        let child = shape_a.child_index(0).expect("child should exist");

        // Act
        let result = distance(
            &shape_a,
            child,
            Transform::IDENTITY,
            &shape_b,
            child,
            Transform::IDENTITY,
            false,
            None,
        )
        .expect("distance should succeed");

        // Assert
        assert_eq!(
            result.diagnostic_trace.termination,
            GjkTermination::Triangle
        );
        assert_eq!(result.cache.snapshot().count(), 3);
    }

    #[test]
    fn gjk_iteration_trace_is_bounded_by_pinned_cap() {
        // Arrange
        let shape_a: Shape = PolygonShape::box_shape(1.0, 2.0)
            .expect("polygon should be valid")
            .into();
        let shape_b: Shape = PolygonShape::oriented_box(1.5, 0.5, Vec2::new(8.0, 3.0), 0.7)
            .expect("polygon should be valid")
            .into();
        let child = shape_a.child_index(0).expect("child should exist");

        // Act
        let result = distance(
            &shape_a,
            child,
            Transform::IDENTITY,
            &shape_b,
            child,
            Transform::IDENTITY,
            false,
            None,
        )
        .expect("distance should succeed");

        // Assert
        assert_eq!(MAX_GJK_ITERATIONS, 20);
        assert!(result.iterations <= MAX_GJK_ITERATIONS);
        assert!(result.diagnostic_trace.steps.len() <= MAX_GJK_ITERATIONS);
    }
}
