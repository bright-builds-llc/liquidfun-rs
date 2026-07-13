//! Borrow-scoped fixture queries over private broad-phase storage.

use std::error::Error;
use std::fmt;

use crate::collision::{Aabb, ChildIndex, QueryControl, RayCastControl, RayCastInput, TreeError};
use crate::math::Vec2;
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

/// A checked normalized fraction for clipping a world ray cast.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayCastFraction(f32);

impl RayCastFraction {
    /// Creates a finite fraction in the inclusive normalized ray interval.
    ///
    /// The world additionally checks a clip against the current narrowed
    /// traversal interval before applying it.
    ///
    /// # Errors
    ///
    /// Returns a field-specific error for a non-finite or out-of-range value.
    pub const fn new(fraction: f32) -> Result<Self, RayCastFractionError> {
        if !fraction.is_finite() {
            return Err(RayCastFractionError::NonFinite);
        }
        if fraction < 0.0 || fraction > 1.0 {
            return Err(RayCastFractionError::OutOfRange);
        }
        Ok(Self(fraction))
    }

    /// Returns the checked normalized fraction.
    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

/// A failure while constructing a checked ray-cast fraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum RayCastFractionError {
    /// The supplied fraction is NaN or infinite.
    NonFinite,
    /// The supplied fraction is outside `0.0..=1.0`.
    OutOfRange,
}

impl fmt::Display for RayCastFractionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite => formatter.write_str("ray-cast fraction must be finite"),
            Self::OutOfRange => {
                formatter.write_str("ray-cast fraction must be in the inclusive interval 0.0..=1.0")
            }
        }
    }
}

impl Error for RayCastFractionError {}

/// Controls how a world ray cast proceeds after one exact shape hit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RayCastDirective {
    /// Ignore this hit and preserve the current ray interval.
    Ignore,
    /// Stop traversal immediately.
    Terminate,
    /// Continue without narrowing the current interval.
    Continue,
    /// Narrow subsequent traversal to a checked normalized fraction.
    Clip(RayCastFraction),
}

/// Owned semantic data for one exact world fixture-child ray hit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldRayHit {
    fixture: FixtureId,
    child_index: ChildIndex,
    point: Vec2,
    normal: Vec2,
    fraction: RayCastFraction,
}

impl WorldRayHit {
    /// Returns the semantic fixture identity.
    #[must_use]
    pub const fn fixture(self) -> FixtureId {
        self.fixture
    }

    /// Returns the checked shape-child coordinate.
    #[must_use]
    pub const fn child_index(self) -> ChildIndex {
        self.child_index
    }

    /// Returns the finite world-space intersection point.
    #[must_use]
    pub const fn point(self) -> Vec2 {
        self.point
    }

    /// Returns the finite outward world-space surface normal.
    #[must_use]
    pub const fn normal(self) -> Vec2 {
        self.normal
    }

    /// Returns the checked normalized intersection fraction.
    #[must_use]
    pub const fn fraction(self) -> RayCastFraction {
        self.fraction
    }
}

/// A failure while traversing or exactly testing a world ray cast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorldRayCastError {
    /// The ray start and end are equal.
    DegenerateRay,
    /// Checked ray or hit arithmetic produced non-finite geometry.
    NonFiniteDerivedGeometry,
    /// A directive attempted to widen the current narrowed ray interval.
    ClipOutsideCurrentInterval,
    /// A live fixture child failed its checked exact-shape ray kernel.
    InvalidFixtureGeometry,
    /// Private broad-phase state violated a world-owned invariant.
    InternalBroadPhaseState,
}

impl fmt::Display for WorldRayCastError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::DegenerateRay => "world ray must have a non-zero direction",
            Self::NonFiniteDerivedGeometry => "world ray arithmetic produced non-finite geometry",
            Self::ClipOutsideCurrentInterval => {
                "ray directive cannot widen the current traversal interval"
            }
            Self::InvalidFixtureGeometry => "fixture child rejected exact ray geometry",
            Self::InternalBroadPhaseState => "world broad-phase invariant failed during ray cast",
        };
        formatter.write_str(message)
    }
}

impl Error for WorldRayCastError {}

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

    /// Casts a checked ray against exact fixture-child geometry.
    ///
    /// Only real shape hits reach `visitor`; broad-phase candidates that miss
    /// exact geometry remain private. Equal-distance and general callback order
    /// are intentionally unspecified, while fixture-child multiplicity is
    /// preserved. The hit value owns semantic fixture, child, point, normal,
    /// and fraction data, and is borrowed only for the visitor call.
    ///
    /// [`RayCastDirective::Ignore`] and [`RayCastDirective::Continue`] preserve
    /// the current interval, [`RayCastDirective::Terminate`] stops traversal,
    /// and [`RayCastDirective::Clip`] narrows later candidates. A clip is
    /// checked against the current inclusive interval before it is applied.
    /// Callback side effects remain application-owned if a later directive is
    /// rejected; they are not rolled back.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a degenerate ray, non-finite derived geometry,
    /// invalid fixture geometry, or a clip outside the current interval.
    pub fn ray_cast<F>(&self, input: RayCastInput, mut visitor: F) -> Result<(), WorldRayCastError>
    where
        F: FnMut(&WorldRayHit) -> RayCastDirective,
    {
        let mut maybe_pending_error = None;
        let traversal = self.broad_phase.ray_cast(input, |proxy, sub_input| {
            let maybe_shape_hit =
                match self.ray_cast_fixture_child(proxy.fixture, proxy.child_index, sub_input) {
                    Ok(maybe_shape_hit) => maybe_shape_hit,
                    Err(_error) => {
                        maybe_pending_error = Some(WorldRayCastError::InvalidFixtureGeometry);
                        return RayCastControl::Terminate;
                    }
                };
            let Some(shape_hit) = maybe_shape_hit else {
                return RayCastControl::Ignore;
            };
            let fraction = match RayCastFraction::new(shape_hit.fraction()) {
                Ok(fraction) => fraction,
                Err(_error) => {
                    maybe_pending_error = Some(WorldRayCastError::InvalidFixtureGeometry);
                    return RayCastControl::Terminate;
                }
            };
            let point = sub_input.start() + fraction.get() * (sub_input.end() - sub_input.start());
            if !point.is_valid() {
                maybe_pending_error = Some(WorldRayCastError::NonFiniteDerivedGeometry);
                return RayCastControl::Terminate;
            }
            let hit = WorldRayHit {
                fixture: proxy.fixture,
                child_index: proxy.child_index,
                point,
                normal: shape_hit.normal(),
                fraction,
            };

            match visitor(&hit) {
                RayCastDirective::Ignore | RayCastDirective::Continue => RayCastControl::Ignore,
                RayCastDirective::Terminate => RayCastControl::Terminate,
                RayCastDirective::Clip(clip) => {
                    if clip.get() > sub_input.max_fraction() {
                        maybe_pending_error = Some(WorldRayCastError::ClipOutsideCurrentInterval);
                        return RayCastControl::Terminate;
                    }
                    RayCastControl::Clip(clip.get())
                }
            }
        });

        if let Some(error) = maybe_pending_error {
            return Err(error);
        }
        traversal.map_err(map_tree_ray_error)
    }
}

fn map_tree_ray_error(error: TreeError) -> WorldRayCastError {
    match error {
        TreeError::DegenerateRay => WorldRayCastError::DegenerateRay,
        TreeError::AabbOverflow => WorldRayCastError::NonFiniteDerivedGeometry,
        TreeError::InvalidClipFraction => WorldRayCastError::ClipOutsideCurrentInterval,
        _ => WorldRayCastError::InternalBroadPhaseState,
    }
}
