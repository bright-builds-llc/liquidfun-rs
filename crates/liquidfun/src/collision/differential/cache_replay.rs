//! Owned semantic cache replay vocabulary and adapter.

use std::error::Error;
use std::fmt;

use crate::math::Transform;
use crate::math::settings::MAX_POLYGON_VERTICES;

use super::super::{
    ChildIndex, CollisionError, DistanceResult, Shape,
    distance::{
        ReplayCacheOutcome, ReplayCacheSeed, ReplayCacheSeedPair, ReplayCacheSeedRejection,
        ReplayCacheSeedReset, ReplayProxyFingerprint, ReplayProxyKind,
        replay_distance_cache as replay_distance_cache_internal, replay_proxy_fingerprint,
    },
};

const MAX_REPLAY_SEED_PAIRS: usize = 4;

/// Closed shape kind carried by one semantic distance-proxy fingerprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceProxyKind {
    /// Circle center proxy.
    Circle,
    /// Two-vertex edge proxy.
    Edge,
    /// Source-ordered polygon proxy.
    Polygon,
    /// Selected two-vertex chain-child proxy.
    Chain,
}

/// Exact bits for one source-ordered semantic proxy vertex.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistanceProxyVertexBits {
    x_bits: u32,
    y_bits: u32,
}

impl DistanceProxyVertexBits {
    /// Creates one exact vertex-bit record.
    #[must_use]
    pub const fn new(x_bits: u32, y_bits: u32) -> Self {
        Self { x_bits, y_bits }
    }

    /// Returns exact x-coordinate bits.
    #[must_use]
    pub const fn x_bits(self) -> u32 {
        self.x_bits
    }

    /// Returns exact y-coordinate bits.
    #[must_use]
    pub const fn y_bits(self) -> u32 {
        self.y_bits
    }
}

/// Bounded owned semantic identity of one selected shape-child proxy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistanceProxyFingerprint {
    kind: DistanceProxyKind,
    child_index: usize,
    radius_bits: u32,
    vertices: Box<[DistanceProxyVertexBits]>,
}

impl DistanceProxyFingerprint {
    /// Creates a bounded semantic proxy fingerprint without engine storage identity.
    ///
    /// # Errors
    ///
    /// Returns [`DistanceCacheSeedError::TooManyFingerprintVertices`] when more
    /// than the pinned polygon-vertex cap is supplied.
    pub fn new(
        kind: DistanceProxyKind,
        child_index: usize,
        radius_bits: u32,
        vertices: Vec<DistanceProxyVertexBits>,
    ) -> Result<Self, DistanceCacheSeedError> {
        if vertices.len() > MAX_POLYGON_VERTICES {
            return Err(DistanceCacheSeedError::TooManyFingerprintVertices);
        }
        Ok(Self {
            kind,
            child_index,
            radius_bits,
            vertices: vertices.into_boxed_slice(),
        })
    }

    /// Returns the closed proxy shape kind.
    #[must_use]
    pub const fn kind(&self) -> DistanceProxyKind {
        self.kind
    }

    /// Returns the checked semantic shape-child coordinate.
    #[must_use]
    pub const fn child_index(&self) -> usize {
        self.child_index
    }

    /// Returns exact radius bits.
    #[must_use]
    pub const fn radius_bits(&self) -> u32 {
        self.radius_bits
    }

    /// Returns source-ordered exact vertex-bit records.
    #[must_use]
    pub fn vertices(&self) -> &[DistanceProxyVertexBits] {
        &self.vertices
    }

    fn into_internal(self) -> ReplayProxyFingerprint {
        ReplayProxyFingerprint {
            kind: match self.kind {
                DistanceProxyKind::Circle => ReplayProxyKind::Circle,
                DistanceProxyKind::Edge => ReplayProxyKind::Edge,
                DistanceProxyKind::Polygon => ReplayProxyKind::Polygon,
                DistanceProxyKind::Chain => ReplayProxyKind::Chain,
            },
            child_index: self.child_index,
            radius_bits: self.radius_bits,
            vertex_bits: self
                .vertices
                .iter()
                .map(|vertex| (vertex.x_bits, vertex.y_bits))
                .collect(),
        }
    }

    fn from_internal(value: &ReplayProxyFingerprint) -> Self {
        Self {
            kind: match value.kind {
                ReplayProxyKind::Circle => DistanceProxyKind::Circle,
                ReplayProxyKind::Edge => DistanceProxyKind::Edge,
                ReplayProxyKind::Polygon => DistanceProxyKind::Polygon,
                ReplayProxyKind::Chain => DistanceProxyKind::Chain,
            },
            child_index: value.child_index,
            radius_bits: value.radius_bits,
            vertices: value
                .vertex_bits
                .iter()
                .map(|(x_bits, y_bits)| DistanceProxyVertexBits::new(*x_bits, *y_bits))
                .collect(),
        }
    }
}

/// One ordered semantic support-index pair in a replay seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistanceCacheSeedPair {
    index_a: usize,
    index_b: usize,
}

impl DistanceCacheSeedPair {
    /// Creates one semantic support-index pair.
    #[must_use]
    pub const fn new(index_a: usize, index_b: usize) -> Self {
        Self { index_a, index_b }
    }

    /// Returns the source-ordered proxy-A vertex coordinate.
    #[must_use]
    pub const fn index_a(self) -> usize {
        self.index_a
    }

    /// Returns the source-ordered proxy-B vertex coordinate.
    #[must_use]
    pub const fn index_b(self) -> usize {
        self.index_b
    }
}

/// Construction failure for a resource-bounded semantic replay record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceCacheSeedError {
    /// A fingerprint exceeded the pinned proxy vertex cap.
    TooManyFingerprintVertices,
    /// A seed exceeded the reviewed malformed-count witness bound.
    TooManySupportPairs,
}

impl fmt::Display for DistanceCacheSeedError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyFingerprintVertices => {
                formatter.write_str("distance proxy fingerprint exceeds the vertex bound")
            }
            Self::TooManySupportPairs => {
                formatter.write_str("distance cache seed exceeds the support-pair bound")
            }
        }
    }
}

impl Error for DistanceCacheSeedError {}

/// Bounded owned semantic cache seed accepted only by the private replay seam.
#[derive(Debug, Clone, PartialEq)]
pub struct DistanceCacheSeed {
    proxy_a: DistanceProxyFingerprint,
    proxy_b: DistanceProxyFingerprint,
    support_pairs: Box<[DistanceCacheSeedPair]>,
    metric: f32,
}

impl DistanceCacheSeed {
    /// Creates a bounded seed, retaining invalid semantic values for typed rejection.
    ///
    /// A four-pair collection is retained so the replay operation can produce
    /// [`DistanceCacheSeedRejection::SupportCountOutOfRange`].
    ///
    /// # Errors
    ///
    /// Returns [`DistanceCacheSeedError::TooManySupportPairs`] above the single
    /// reviewed out-of-range count.
    pub fn new(
        proxy_a: DistanceProxyFingerprint,
        proxy_b: DistanceProxyFingerprint,
        support_pairs: Vec<DistanceCacheSeedPair>,
        metric: f32,
    ) -> Result<Self, DistanceCacheSeedError> {
        if support_pairs.len() > MAX_REPLAY_SEED_PAIRS {
            return Err(DistanceCacheSeedError::TooManySupportPairs);
        }
        Ok(Self {
            proxy_a,
            proxy_b,
            support_pairs: support_pairs.into_boxed_slice(),
            metric,
        })
    }

    /// Returns the proxy-A semantic fingerprint.
    #[must_use]
    pub const fn proxy_a(&self) -> &DistanceProxyFingerprint {
        &self.proxy_a
    }

    /// Returns the proxy-B semantic fingerprint.
    #[must_use]
    pub const fn proxy_b(&self) -> &DistanceProxyFingerprint {
        &self.proxy_b
    }

    /// Returns source-ordered semantic support pairs.
    #[must_use]
    pub fn support_pairs(&self) -> &[DistanceCacheSeedPair] {
        &self.support_pairs
    }

    /// Returns the exact seed metric.
    #[must_use]
    pub const fn metric(&self) -> f32 {
        self.metric
    }

    fn into_internal(self) -> ReplayCacheSeed {
        ReplayCacheSeed {
            proxy_a: self.proxy_a.into_internal(),
            proxy_b: self.proxy_b.into_internal(),
            support_pairs: self
                .support_pairs
                .iter()
                .map(|pair| ReplayCacheSeedPair {
                    index_a: pair.index_a,
                    index_b: pair.index_b,
                })
                .collect(),
            metric: self.metric,
        }
    }
}

/// Fail-closed semantic replay rejection reason in exact validation order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceCacheSeedRejection {
    /// Proxy A does not match the selected shape child.
    ProxyAFingerprintMismatch,
    /// Proxy B does not match the selected shape child.
    ProxyBFingerprintMismatch,
    /// Support count is outside `1..=3`.
    SupportCountOutOfRange,
    /// A proxy-A support coordinate is out of range.
    SupportIndexAOutOfRange,
    /// A proxy-B support coordinate is out of range.
    SupportIndexBOutOfRange,
    /// An ordered support pair duplicates an earlier pair.
    DuplicateSupportPair,
    /// The seed metric is NaN or infinite.
    NonFiniteMetric,
}

/// Source reset reason for a checked multi-point semantic seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceCacheSeedReset {
    /// The current metric lies outside the pinned ratio window.
    MetricRatio,
    /// The current metric is below the pinned epsilon threshold.
    MetricTooSmall,
}

/// Closed result of topology-safe semantic cache replay.
#[derive(Debug, Clone, PartialEq)]
pub enum DistanceCacheReplayOutcome {
    /// The checked seed entered the ordinary distance kernel.
    Used {
        /// Ordinary owned public distance result.
        result: DistanceResult,
    },
    /// The checked multi-point seed source-reset to a cold distance call.
    Reset {
        /// Ordinary owned public distance result from the cold path.
        result: DistanceResult,
        /// Exact source-ordered reset reason.
        reason: DistanceCacheSeedReset,
    },
    /// The seed was rejected before private cache construction.
    Rejected {
        /// First fail-closed semantic rejection reason.
        reason: DistanceCacheSeedRejection,
    },
}

/// Copies the semantic fingerprint of one checked shape-child proxy.
///
/// # Errors
///
/// Returns a typed child-selection error before reading child geometry.
pub fn distance_proxy_fingerprint(
    shape: &Shape,
    child: ChildIndex,
) -> Result<DistanceProxyFingerprint, CollisionError> {
    replay_proxy_fingerprint(shape, child)
        .map(|value| DistanceProxyFingerprint::from_internal(&value))
}

/// Replays one bounded semantic cache seed without exposing cache construction or mutation.
///
/// # Errors
///
/// Returns ordinary typed shape-child, transform, or derived-geometry errors.
#[allow(clippy::too_many_arguments)] // Mirrors the complete checked distance input plus one seed.
pub fn replay_distance_cache(
    shape_a: &Shape,
    child_a: ChildIndex,
    transform_a: Transform,
    shape_b: &Shape,
    child_b: ChildIndex,
    transform_b: Transform,
    use_radii: bool,
    seed: DistanceCacheSeed,
) -> Result<DistanceCacheReplayOutcome, CollisionError> {
    let internal_seed = seed.into_internal();
    let outcome = replay_distance_cache_internal(
        shape_a,
        child_a,
        transform_a,
        shape_b,
        child_b,
        transform_b,
        use_radii,
        &internal_seed,
    )?;
    Ok(match outcome {
        ReplayCacheOutcome::Used(result) => DistanceCacheReplayOutcome::Used { result },
        ReplayCacheOutcome::Reset(result, reason) => DistanceCacheReplayOutcome::Reset {
            result,
            reason: match reason {
                ReplayCacheSeedReset::MetricRatio => DistanceCacheSeedReset::MetricRatio,
                ReplayCacheSeedReset::MetricTooSmall => DistanceCacheSeedReset::MetricTooSmall,
            },
        },
        ReplayCacheOutcome::Rejected(reason) => DistanceCacheReplayOutcome::Rejected {
            reason: match reason {
                ReplayCacheSeedRejection::ProxyAFingerprintMismatch => {
                    DistanceCacheSeedRejection::ProxyAFingerprintMismatch
                }
                ReplayCacheSeedRejection::ProxyBFingerprintMismatch => {
                    DistanceCacheSeedRejection::ProxyBFingerprintMismatch
                }
                ReplayCacheSeedRejection::SupportCountOutOfRange => {
                    DistanceCacheSeedRejection::SupportCountOutOfRange
                }
                ReplayCacheSeedRejection::SupportIndexAOutOfRange => {
                    DistanceCacheSeedRejection::SupportIndexAOutOfRange
                }
                ReplayCacheSeedRejection::SupportIndexBOutOfRange => {
                    DistanceCacheSeedRejection::SupportIndexBOutOfRange
                }
                ReplayCacheSeedRejection::DuplicateSupportPair => {
                    DistanceCacheSeedRejection::DuplicateSupportPair
                }
                ReplayCacheSeedRejection::NonFiniteMetric => {
                    DistanceCacheSeedRejection::NonFiniteMetric
                }
            },
        },
    })
}
