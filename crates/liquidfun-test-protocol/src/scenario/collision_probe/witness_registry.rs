use serde::{Deserialize, Serialize};

use super::types::CollisionProbeOperation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollisionWitnessFamily {
    ShapeUnaryQuery,
    ShapeAcceptedCircle,
    ShapeAcceptedEdge,
    ShapeAcceptedPolygon,
    ShapeAcceptedChain,
    ShapeAcceptedEdgeGhosts,
    ShapeAcceptedPolygonWeldHull,
    ShapeAcceptedChainTopology,
    ShapeRejectedCircle,
    ShapeRejectedEdge,
    ShapeRejectedPolygon,
    ShapeRejectedChain,
    ShapeRejectedEdgeGhost,
    ShapeRejectedPolygonWeldHull,
    ShapeRejectedChainAdjacentClosing,
    DistanceCold,
    DistanceWarmUsed,
    DistanceWarmSinglePointUsed,
    DistanceCacheReset,
    DistanceCacheRejected,
    DistanceCacheRejectedPrecedence,
    DistanceCacheResetPrecedence,
    DistanceSimplexOne,
    DistanceSimplexTwo,
    DistanceSimplexThree,
    DistanceSupportTie,
    DistanceNearZeroTermination,
    DistanceDuplicateSupportTermination,
    DistanceIterationLimitTermination,
    OverlapBelowThreshold,
    OverlapAtThreshold,
    OverlapAboveThreshold,
    ClipInsideInside,
    ClipOutsideOutside,
    ClipCrossingForward,
    ClipCrossingReverse,
    ClipOnPlane,
    PairCircleCircle,
    PairPolygonCircle,
    PairPolygonPolygon,
    PairEdgeCircle,
    PairEdgePolygon,
    PairChainCircle,
    PairChainPolygon,
    PairCirclePolygonReversed,
    PairCircleEdgeReversed,
    PairPolygonEdgeReversed,
    PairCircleChainReversed,
    PairPolygonChainReversed,
    EdgeIsolated,
    EdgeConvexAdjacency,
    EdgeConcaveAdjacency,
    EdgeFront,
    EdgeBack,
    EdgeEndpointOwnership,
    FeatureAdded,
    FeaturePersisted,
    FeatureRemoved,
    TreeInsertionTie,
    TreeRotationTie,
    TreeLifecycleReuse,
    TreeQueryContinueStop,
    TreeRayIgnoreTerminateClip,
    TreeMetrics,
    BroadPhaseDuplicateMoveTouch,
    BroadPhasePairOrderDedup,
    BroadPhaseFilterGroupsMasks,
    BroadPhaseRefilter,
    ToiOverlapped,
    ToiTouching,
    ToiSeparated,
    ToiTranslation,
    ToiRotation,
    ToiTangent,
    ToiSupportTie,
    ToiCapFailed,
    ToiLargeAngle,
    ToiEdgeChainChildren,
}

impl CollisionWitnessFamily {
    pub const REQUIRED: [Self; 78] = [
        Self::ShapeUnaryQuery,
        Self::ShapeAcceptedCircle,
        Self::ShapeAcceptedEdge,
        Self::ShapeAcceptedPolygon,
        Self::ShapeAcceptedChain,
        Self::ShapeAcceptedEdgeGhosts,
        Self::ShapeAcceptedPolygonWeldHull,
        Self::ShapeAcceptedChainTopology,
        Self::ShapeRejectedCircle,
        Self::ShapeRejectedEdge,
        Self::ShapeRejectedPolygon,
        Self::ShapeRejectedChain,
        Self::ShapeRejectedEdgeGhost,
        Self::ShapeRejectedPolygonWeldHull,
        Self::ShapeRejectedChainAdjacentClosing,
        Self::DistanceCold,
        Self::DistanceWarmUsed,
        Self::DistanceWarmSinglePointUsed,
        Self::DistanceCacheReset,
        Self::DistanceCacheRejected,
        Self::DistanceCacheRejectedPrecedence,
        Self::DistanceCacheResetPrecedence,
        Self::DistanceSimplexOne,
        Self::DistanceSimplexTwo,
        Self::DistanceSimplexThree,
        Self::DistanceSupportTie,
        Self::DistanceNearZeroTermination,
        Self::DistanceDuplicateSupportTermination,
        Self::DistanceIterationLimitTermination,
        Self::OverlapBelowThreshold,
        Self::OverlapAtThreshold,
        Self::OverlapAboveThreshold,
        Self::ClipInsideInside,
        Self::ClipOutsideOutside,
        Self::ClipCrossingForward,
        Self::ClipCrossingReverse,
        Self::ClipOnPlane,
        Self::PairCircleCircle,
        Self::PairPolygonCircle,
        Self::PairPolygonPolygon,
        Self::PairEdgeCircle,
        Self::PairEdgePolygon,
        Self::PairChainCircle,
        Self::PairChainPolygon,
        Self::PairCirclePolygonReversed,
        Self::PairCircleEdgeReversed,
        Self::PairPolygonEdgeReversed,
        Self::PairCircleChainReversed,
        Self::PairPolygonChainReversed,
        Self::EdgeIsolated,
        Self::EdgeConvexAdjacency,
        Self::EdgeConcaveAdjacency,
        Self::EdgeFront,
        Self::EdgeBack,
        Self::EdgeEndpointOwnership,
        Self::FeatureAdded,
        Self::FeaturePersisted,
        Self::FeatureRemoved,
        Self::TreeInsertionTie,
        Self::TreeRotationTie,
        Self::TreeLifecycleReuse,
        Self::TreeQueryContinueStop,
        Self::TreeRayIgnoreTerminateClip,
        Self::TreeMetrics,
        Self::BroadPhaseDuplicateMoveTouch,
        Self::BroadPhasePairOrderDedup,
        Self::BroadPhaseFilterGroupsMasks,
        Self::BroadPhaseRefilter,
        Self::ToiOverlapped,
        Self::ToiTouching,
        Self::ToiSeparated,
        Self::ToiTranslation,
        Self::ToiRotation,
        Self::ToiTangent,
        Self::ToiSupportTie,
        Self::ToiCapFailed,
        Self::ToiLargeAngle,
        Self::ToiEdgeChainChildren,
    ];

    #[must_use]
    pub const fn expected_operation(self) -> CollisionProbeOperation {
        match self {
            Self::ShapeUnaryQuery => CollisionProbeOperation::ShapeUnaryQuery,
            Self::ShapeAcceptedCircle
            | Self::ShapeAcceptedEdge
            | Self::ShapeAcceptedPolygon
            | Self::ShapeAcceptedChain
            | Self::ShapeAcceptedEdgeGhosts
            | Self::ShapeAcceptedPolygonWeldHull
            | Self::ShapeAcceptedChainTopology
            | Self::ShapeRejectedCircle
            | Self::ShapeRejectedEdge
            | Self::ShapeRejectedPolygon
            | Self::ShapeRejectedChain
            | Self::ShapeRejectedEdgeGhost
            | Self::ShapeRejectedPolygonWeldHull
            | Self::ShapeRejectedChainAdjacentClosing => CollisionProbeOperation::ShapeConstruction,
            Self::DistanceCold
            | Self::DistanceWarmUsed
            | Self::DistanceWarmSinglePointUsed
            | Self::DistanceCacheReset
            | Self::DistanceCacheRejected
            | Self::DistanceCacheRejectedPrecedence
            | Self::DistanceCacheResetPrecedence
            | Self::DistanceSimplexOne
            | Self::DistanceSimplexTwo
            | Self::DistanceSimplexThree
            | Self::DistanceSupportTie
            | Self::DistanceNearZeroTermination
            | Self::DistanceDuplicateSupportTermination
            | Self::DistanceIterationLimitTermination => CollisionProbeOperation::Distance,
            Self::OverlapBelowThreshold
            | Self::OverlapAtThreshold
            | Self::OverlapAboveThreshold => CollisionProbeOperation::Overlap,
            Self::ClipInsideInside
            | Self::ClipOutsideOutside
            | Self::ClipCrossingForward
            | Self::ClipCrossingReverse
            | Self::ClipOnPlane => CollisionProbeOperation::Clip,
            Self::PairCircleCircle
            | Self::PairPolygonCircle
            | Self::PairPolygonPolygon
            | Self::PairEdgeCircle
            | Self::PairEdgePolygon
            | Self::PairChainCircle
            | Self::PairChainPolygon
            | Self::PairCirclePolygonReversed
            | Self::PairCircleEdgeReversed
            | Self::PairPolygonEdgeReversed
            | Self::PairCircleChainReversed
            | Self::PairPolygonChainReversed => CollisionProbeOperation::PairDispatch,
            Self::EdgeIsolated
            | Self::EdgeConvexAdjacency
            | Self::EdgeConcaveAdjacency
            | Self::EdgeFront
            | Self::EdgeBack
            | Self::EdgeEndpointOwnership => CollisionProbeOperation::Manifold,
            Self::FeatureAdded | Self::FeaturePersisted | Self::FeatureRemoved => {
                CollisionProbeOperation::FeatureTransition
            }
            Self::TreeInsertionTie | Self::TreeRotationTie | Self::TreeLifecycleReuse => {
                CollisionProbeOperation::TreeLifecycle
            }
            Self::TreeQueryContinueStop => CollisionProbeOperation::TreeQuery,
            Self::TreeRayIgnoreTerminateClip => CollisionProbeOperation::TreeRay,
            Self::TreeMetrics => CollisionProbeOperation::TreeMetrics,
            Self::BroadPhaseDuplicateMoveTouch => CollisionProbeOperation::BroadPhaseMoveTouch,
            Self::BroadPhasePairOrderDedup => CollisionProbeOperation::BroadPhasePairs,
            Self::BroadPhaseFilterGroupsMasks => CollisionProbeOperation::BroadPhaseFilter,
            Self::BroadPhaseRefilter => CollisionProbeOperation::BroadPhaseRefilter,
            Self::ToiOverlapped
            | Self::ToiTouching
            | Self::ToiSeparated
            | Self::ToiTranslation
            | Self::ToiRotation
            | Self::ToiTangent
            | Self::ToiSupportTie
            | Self::ToiCapFailed
            | Self::ToiLargeAngle
            | Self::ToiEdgeChainChildren => CollisionProbeOperation::TimeOfImpact,
        }
    }

    #[must_use]
    pub const fn expects_rejection(self) -> bool {
        matches!(
            self,
            Self::ShapeRejectedCircle
                | Self::ShapeRejectedEdge
                | Self::ShapeRejectedPolygon
                | Self::ShapeRejectedChain
                | Self::ShapeRejectedEdgeGhost
                | Self::ShapeRejectedPolygonWeldHull
                | Self::ShapeRejectedChainAdjacentClosing
        )
    }
}
