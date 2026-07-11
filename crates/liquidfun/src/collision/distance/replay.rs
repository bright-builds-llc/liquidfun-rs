use super::proxy::{DistanceProxy, ProxyKind};
use super::simplex::Simplex;
use super::{
    DistanceCache, DistanceResult, DistanceTopologyIdentity, MAX_SIMPLEX_VERTICES,
    SupportIndexPair, distance,
};
use crate::collision::shape::Shape;
use crate::collision::{ChildIndex, CollisionError};
use crate::math::Transform;
use crate::math::settings::EPSILON;

#[cfg(feature = "differential-internals")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReplayProxyKind {
    Circle,
    Edge,
    Polygon,
    Chain,
}

#[cfg(feature = "differential-internals")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReplayProxyFingerprint {
    pub(crate) kind: ReplayProxyKind,
    pub(crate) child_index: usize,
    pub(crate) radius_bits: u32,
    pub(crate) vertex_bits: Box<[(u32, u32)]>,
}

#[cfg(feature = "differential-internals")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReplayCacheSeedPair {
    pub(crate) index_a: usize,
    pub(crate) index_b: usize,
}

#[cfg(feature = "differential-internals")]
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ReplayCacheSeed {
    pub(crate) proxy_a: ReplayProxyFingerprint,
    pub(crate) proxy_b: ReplayProxyFingerprint,
    pub(crate) support_pairs: Box<[ReplayCacheSeedPair]>,
    pub(crate) metric: f32,
}

#[cfg(feature = "differential-internals")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReplayCacheSeedRejection {
    ProxyAFingerprintMismatch,
    ProxyBFingerprintMismatch,
    SupportCountOutOfRange,
    SupportIndexAOutOfRange,
    SupportIndexBOutOfRange,
    DuplicateSupportPair,
    NonFiniteMetric,
}

#[cfg(feature = "differential-internals")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReplayCacheSeedReset {
    MetricRatio,
    MetricTooSmall,
}

#[cfg(feature = "differential-internals")]
pub(crate) enum ReplayCacheOutcome {
    Used(DistanceResult),
    Reset(DistanceResult, ReplayCacheSeedReset),
    Rejected(ReplayCacheSeedRejection),
}

#[cfg(feature = "differential-internals")]
pub(crate) fn replay_proxy_fingerprint(
    shape: &Shape,
    child: ChildIndex,
) -> Result<ReplayProxyFingerprint, CollisionError> {
    let proxy = DistanceProxy::new(shape, child)?;
    Ok(replay_fingerprint_from_identity(proxy.identity()))
}

#[cfg(feature = "differential-internals")]
#[allow(clippy::too_many_arguments)] // Mirrors the checked public distance input plus one seed.
pub(crate) fn replay_distance_cache(
    shape_a: &Shape,
    child_a: ChildIndex,
    transform_a: Transform,
    shape_b: &Shape,
    child_b: ChildIndex,
    transform_b: Transform,
    use_radii: bool,
    seed: &ReplayCacheSeed,
) -> Result<ReplayCacheOutcome, CollisionError> {
    crate::collision::shape::validate_transform(transform_a)?;
    crate::collision::shape::validate_transform(transform_b)?;
    let proxy_a = DistanceProxy::new(shape_a, child_a)?;
    let proxy_b = DistanceProxy::new(shape_b, child_b)?;
    if seed.proxy_a != replay_fingerprint_from_identity(proxy_a.identity()) {
        return Ok(ReplayCacheOutcome::Rejected(
            ReplayCacheSeedRejection::ProxyAFingerprintMismatch,
        ));
    }
    if seed.proxy_b != replay_fingerprint_from_identity(proxy_b.identity()) {
        return Ok(ReplayCacheOutcome::Rejected(
            ReplayCacheSeedRejection::ProxyBFingerprintMismatch,
        ));
    }
    if !(1..=MAX_SIMPLEX_VERTICES).contains(&seed.support_pairs.len()) {
        return Ok(ReplayCacheOutcome::Rejected(
            ReplayCacheSeedRejection::SupportCountOutOfRange,
        ));
    }

    let mut support_pairs = Vec::with_capacity(seed.support_pairs.len());
    for (position, pair) in seed.support_pairs.iter().copied().enumerate() {
        if pair.index_a >= proxy_a.vertex_count() {
            return Ok(ReplayCacheOutcome::Rejected(
                ReplayCacheSeedRejection::SupportIndexAOutOfRange,
            ));
        }
        if pair.index_b >= proxy_b.vertex_count() {
            return Ok(ReplayCacheOutcome::Rejected(
                ReplayCacheSeedRejection::SupportIndexBOutOfRange,
            ));
        }
        if seed.support_pairs[..position].contains(&pair) {
            return Ok(ReplayCacheOutcome::Rejected(
                ReplayCacheSeedRejection::DuplicateSupportPair,
            ));
        }
        support_pairs.push(SupportIndexPair::new(pair.index_a, pair.index_b));
    }
    if !seed.metric.is_finite() {
        return Ok(ReplayCacheOutcome::Rejected(
            ReplayCacheSeedRejection::NonFiniteMetric,
        ));
    }

    let mut cache = DistanceCache::empty();
    cache.write(&proxy_a, &proxy_b, seed.metric, &support_pairs)?;
    if support_pairs.len() == 1 {
        let result = distance(
            shape_a,
            child_a,
            transform_a,
            shape_b,
            child_b,
            transform_b,
            use_radii,
            Some(&cache),
        )?;
        return Ok(ReplayCacheOutcome::Used(result));
    }

    let metric2 = Simplex::cache_metric(
        cache.entries(&proxy_a, &proxy_b)?,
        &proxy_a,
        transform_a,
        &proxy_b,
        transform_b,
    );
    let maybe_reset = if metric2 < 0.5 * seed.metric || 2.0 * seed.metric < metric2 {
        Some(ReplayCacheSeedReset::MetricRatio)
    } else if metric2 < EPSILON {
        Some(ReplayCacheSeedReset::MetricTooSmall)
    } else {
        None
    };
    let Some(reset) = maybe_reset else {
        let result = distance(
            shape_a,
            child_a,
            transform_a,
            shape_b,
            child_b,
            transform_b,
            use_radii,
            Some(&cache),
        )?;
        return Ok(ReplayCacheOutcome::Used(result));
    };
    let result = distance(
        shape_a,
        child_a,
        transform_a,
        shape_b,
        child_b,
        transform_b,
        use_radii,
        None,
    )?;
    Ok(ReplayCacheOutcome::Reset(result, reset))
}

#[cfg(feature = "differential-internals")]
fn replay_fingerprint_from_identity(identity: DistanceTopologyIdentity) -> ReplayProxyFingerprint {
    ReplayProxyFingerprint {
        kind: match identity.kind {
            ProxyKind::Circle => ReplayProxyKind::Circle,
            ProxyKind::Edge => ReplayProxyKind::Edge,
            ProxyKind::Polygon => ReplayProxyKind::Polygon,
            ProxyKind::Chain => ReplayProxyKind::Chain,
        },
        child_index: identity.child_index,
        radius_bits: identity.radius_bits,
        vertex_bits: identity.vertex_bits.into_boxed_slice(),
    }
}
