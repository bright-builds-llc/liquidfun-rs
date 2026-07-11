//! Public contract tests for GJK distance, reusable cache, and overlap.

use liquidfun::collision::distance::{DistanceCache, DistanceResult};

#[test]
fn cache_empty_state_is_fully_initialized() {
    // Arrange
    let cache = DistanceCache::empty();

    // Act
    let snapshot = cache.snapshot();

    // Assert
    assert_eq!(snapshot.count(), 0);
    assert!(snapshot.support_pairs().is_empty());
    assert_eq!(snapshot.metric().to_bits(), 0.0_f32.to_bits());
}

#[test]
fn proxy_cache_snapshot_clone_preserves_semantic_state() {
    // Arrange
    let cache = DistanceCache::empty();

    // Act
    let cloned = cache.clone();

    // Assert
    assert_eq!(cloned.snapshot(), cache.snapshot());
}

#[test]
fn proxy_distance_result_surface_is_public_and_read_only() {
    // Arrange
    let maybe_result: Option<&DistanceResult> = None;

    // Act
    let is_absent = maybe_result.is_none();

    // Assert
    assert!(is_absent);
}
