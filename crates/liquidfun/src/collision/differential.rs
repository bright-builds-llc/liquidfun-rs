//! Owned diagnostic vocabulary for the private cross-language collision harness.
//!
//! This module is development-only. Its records intentionally carry semantic
//! identities and bounded copies rather than engine storage or mutation access.

mod cache_replay;

pub use cache_replay::{
    DistanceCacheReplayOutcome, DistanceCacheSeed, DistanceCacheSeedError, DistanceCacheSeedPair,
    DistanceCacheSeedRejection, DistanceCacheSeedReset, DistanceProxyFingerprint,
    DistanceProxyKind, DistanceProxyVertexBits, distance_proxy_fingerprint, replay_distance_cache,
};

use crate::math::Vec2;

use super::{
    CollisionError, ContactFeatureId, DistanceCache, FeatureKind, ManifoldKind, PairOrientation,
    PointState, PointStates, SupportIndexPair, TimeOfImpactState,
};

const MAX_GJK_STEPS: usize = 20;

/// Source-ordered GJK termination classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GjkTermination {
    /// A three-point simplex contains the origin.
    Triangle,
    /// The next search direction is below the pinned epsilon threshold.
    NearZeroDirection,
    /// The next support pair duplicates a saved simplex pair.
    DuplicateSupport,
    /// The fixed twenty-support-call cap was reached.
    IterationLimit,
}

/// Owned bounded GJK cache and termination evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct DistanceDiagnosticRecord {
    support_pairs: Box<[SupportIndexPair]>,
    metric: f32,
    iterations: usize,
    termination: GjkTermination,
}

impl DistanceDiagnosticRecord {
    /// Returns source-ordered support pairs.
    #[must_use]
    pub fn support_pairs(&self) -> &[SupportIndexPair] {
        &self.support_pairs
    }

    /// Returns the cache metric.
    #[must_use]
    pub const fn metric(&self) -> f32 {
        self.metric
    }

    /// Returns the bounded support-call count.
    #[must_use]
    pub const fn iterations(&self) -> usize {
        self.iterations
    }

    /// Returns the semantic termination classification.
    #[must_use]
    pub const fn termination(&self) -> GjkTermination {
        self.termination
    }
}

/// Copies bounded semantic cache evidence into an owned diagnostic record.
#[must_use]
pub fn distance_diagnostic(cache: &DistanceCache, iterations: usize) -> DistanceDiagnosticRecord {
    let snapshot = cache.snapshot();
    let termination = if snapshot.count() == 3 {
        GjkTermination::Triangle
    } else if iterations == 0 {
        GjkTermination::NearZeroDirection
    } else if iterations >= MAX_GJK_STEPS {
        GjkTermination::IterationLimit
    } else {
        GjkTermination::DuplicateSupport
    };
    DistanceDiagnosticRecord {
        support_pairs: snapshot.support_pairs().into(),
        metric: snapshot.metric(),
        iterations: iterations.min(MAX_GJK_STEPS),
        termination,
    }
}

/// Semantic contact feature copied into diagnostic evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiagnosticFeature(ContactFeatureId);

impl DiagnosticFeature {
    /// Copies a four-field semantic feature identity.
    #[must_use]
    pub const fn new(feature: ContactFeatureId) -> Self {
        Self(feature)
    }

    /// Returns the semantic feature identity.
    #[must_use]
    pub const fn feature(self) -> ContactFeatureId {
        self.0
    }
}

/// One owned clipping input with exactly two source-ordered points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipDiagnosticInput {
    points: [(Vec2, DiagnosticFeature); 2],
    normal: Vec2,
    offset: f32,
    vertex_index_a: u8,
}

impl ClipDiagnosticInput {
    /// Creates one finite bounded clipping input.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionError::NonFiniteValue`] for non-finite geometry.
    pub fn new(
        points: [(Vec2, DiagnosticFeature); 2],
        normal: Vec2,
        offset: f32,
        vertex_index_a: u8,
    ) -> Result<Self, CollisionError> {
        if points.iter().any(|(point, _feature)| !point.is_valid())
            || !normal.is_valid()
            || !offset.is_finite()
        {
            return Err(CollisionError::NonFiniteValue);
        }
        Ok(Self {
            points,
            normal,
            offset,
            vertex_index_a,
        })
    }
}

/// One source-ordered owned clipping result point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClipDiagnosticPoint {
    point: Vec2,
    feature: DiagnosticFeature,
}

impl ClipDiagnosticPoint {
    /// Returns the clipped point.
    #[must_use]
    pub const fn point(self) -> Vec2 {
        self.point
    }

    /// Returns its semantic feature identity.
    #[must_use]
    pub const fn feature(self) -> DiagnosticFeature {
        self.feature
    }
}

/// Fixed-capacity source-ordered clipping evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct ClipDiagnosticRecord {
    points: Box<[ClipDiagnosticPoint]>,
}

impl ClipDiagnosticRecord {
    /// Returns at most two points in source order.
    #[must_use]
    pub fn points(&self) -> &[ClipDiagnosticPoint] {
        &self.points
    }
}

/// Executes the private probe's bounded semantic copy of the source clip step.
#[must_use]
pub fn clip_segment_diagnostic(input: ClipDiagnosticInput) -> ClipDiagnosticRecord {
    let distance0 = input.normal.dot(input.points[0].0) - input.offset;
    let distance1 = input.normal.dot(input.points[1].0) - input.offset;
    let mut points = Vec::with_capacity(2);
    if distance0 <= 0.0 {
        points.push(ClipDiagnosticPoint {
            point: input.points[0].0,
            feature: input.points[0].1,
        });
    }
    if distance1 <= 0.0 {
        points.push(ClipDiagnosticPoint {
            point: input.points[1].0,
            feature: input.points[1].1,
        });
    }
    if distance0 * distance1 < 0.0 {
        let interpolation = distance0 / (distance0 - distance1);
        points.push(ClipDiagnosticPoint {
            point: input.points[0].0 + interpolation * (input.points[1].0 - input.points[0].0),
            feature: DiagnosticFeature::new(ContactFeatureId::new(
                input.vertex_index_a,
                input.points[0].1.feature().index_b(),
                FeatureKind::Vertex,
                FeatureKind::Face,
            )),
        });
    }
    ClipDiagnosticRecord {
        points: points.into_boxed_slice(),
    }
}

/// Exact pair orientation, manifold branch, and point-state evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairDiagnosticRecord {
    orientation: PairOrientation,
    maybe_manifold_kind: Option<ManifoldKind>,
    previous_states: [PointState; 2],
    current_states: [PointState; 2],
}

impl PairDiagnosticRecord {
    /// Returns the canonical pair orientation.
    #[must_use]
    pub const fn orientation(&self) -> PairOrientation {
        self.orientation
    }

    /// Returns the active manifold branch, when touching.
    #[must_use]
    pub const fn maybe_manifold_kind(&self) -> Option<ManifoldKind> {
        self.maybe_manifold_kind
    }

    /// Returns old-manifold states in source order.
    #[must_use]
    pub const fn previous_states(&self) -> &[PointState; 2] {
        &self.previous_states
    }

    /// Returns new-manifold states in source order.
    #[must_use]
    pub const fn current_states(&self) -> &[PointState; 2] {
        &self.current_states
    }
}

/// Copies pair and optional transition evidence into one owned record.
#[must_use]
pub fn pair_diagnostic(
    orientation: PairOrientation,
    maybe_manifold_kind: Option<ManifoldKind>,
    maybe_states: Option<PointStates>,
) -> PairDiagnosticRecord {
    let (previous_states, current_states) = maybe_states
        .map_or(([PointState::Null; 2], [PointState::Null; 2]), |states| {
            (*states.previous(), *states.current())
        });
    PairDiagnosticRecord {
        orientation,
        maybe_manifold_kind,
        previous_states,
        current_states,
    }
}

/// Stable semantic payload pair in source diagnostic order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SemanticPairDiagnostic {
    first_payload_id: u32,
    second_payload_id: u32,
}

impl SemanticPairDiagnostic {
    /// Creates one semantic payload pair.
    #[must_use]
    pub const fn new(first_payload_id: u32, second_payload_id: u32) -> Self {
        Self {
            first_payload_id,
            second_payload_id,
        }
    }

    /// Returns the first payload identity.
    #[must_use]
    pub const fn first_payload_id(self) -> u32 {
        self.first_payload_id
    }

    /// Returns the second payload identity.
    #[must_use]
    pub const fn second_payload_id(self) -> u32 {
        self.second_payload_id
    }
}

/// Closed TOI termination classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeOfImpactTermination {
    /// Initial core overlap.
    Overlapped,
    /// Target separation reached.
    Touching,
    /// Separation persisted through the interval.
    Separated,
    /// A fixed algorithm cap or progress guard failed.
    Failed,
}

/// Owned public TOI outcome plus semantic termination.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeOfImpactDiagnosticRecord {
    state: TimeOfImpactState,
    time: f32,
    termination: TimeOfImpactTermination,
}

impl TimeOfImpactDiagnosticRecord {
    /// Returns the closed public state.
    #[must_use]
    pub const fn state(self) -> TimeOfImpactState {
        self.state
    }

    /// Returns the exact finite time.
    #[must_use]
    pub const fn time(self) -> f32 {
        self.time
    }

    /// Returns the semantic termination family.
    #[must_use]
    pub const fn termination(self) -> TimeOfImpactTermination {
        self.termination
    }
}

/// Copies a checked public TOI outcome into diagnostic vocabulary.
#[must_use]
pub const fn time_of_impact_diagnostic(
    state: TimeOfImpactState,
    time: f32,
) -> TimeOfImpactDiagnosticRecord {
    let termination = match state {
        TimeOfImpactState::Overlapped => TimeOfImpactTermination::Overlapped,
        TimeOfImpactState::Touching => TimeOfImpactTermination::Touching,
        TimeOfImpactState::Separated => TimeOfImpactTermination::Separated,
        TimeOfImpactState::Failed => TimeOfImpactTermination::Failed,
    };
    TimeOfImpactDiagnosticRecord {
        state,
        time,
        termination,
    }
}
