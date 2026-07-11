//! Initialized shared values used by collision kernels.

use crate::math::{Vec2, max, min};

use super::CollisionError;

/// A finite axis-aligned bounding box corresponding to upstream `b2AABB`.
///
/// Bounds are private and may be created only in coordinate order. No memory
/// layout or raw-parts contract is exposed.
///
/// ```compile_fail
/// use liquidfun::collision::Aabb;
/// use liquidfun::math::Vec2;
///
/// let _aabb = Aabb {
///     lower_bound: Vec2::ZERO,
///     upper_bound: Vec2::new(1.0, 1.0),
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    lower_bound: Vec2,
    upper_bound: Vec2,
}

impl Aabb {
    /// Creates a finite AABB whose lower coordinates do not exceed its upper coordinates.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionError::NonFiniteValue`] for a non-finite coordinate
    /// and [`CollisionError::InvalidBounds`] for reversed bounds.
    #[must_use = "AABB construction can fail for invalid bounds"]
    pub fn new(lower_bound: Vec2, upper_bound: Vec2) -> Result<Self, CollisionError> {
        validate_vec2(lower_bound)?;
        validate_vec2(upper_bound)?;
        if lower_bound.x > upper_bound.x || lower_bound.y > upper_bound.y {
            return Err(CollisionError::InvalidBounds);
        }

        Ok(Self {
            lower_bound,
            upper_bound,
        })
    }

    /// Returns the lower corner.
    #[must_use]
    pub const fn lower_bound(self) -> Vec2 {
        self.lower_bound
    }

    /// Returns the upper corner.
    #[must_use]
    pub const fn upper_bound(self) -> Vec2 {
        self.upper_bound
    }

    /// Returns the source-ordered center.
    #[must_use]
    pub fn center(self) -> Vec2 {
        0.5 * (self.lower_bound + self.upper_bound)
    }

    /// Returns the half-widths along each axis.
    #[must_use]
    pub fn extents(self) -> Vec2 {
        0.5 * (self.upper_bound - self.lower_bound)
    }

    /// Returns the perimeter in meters.
    #[must_use]
    pub fn perimeter(self) -> f32 {
        let width = self.upper_bound.x - self.lower_bound.x;
        let height = self.upper_bound.y - self.lower_bound.y;
        2.0 * (width + height)
    }

    /// Returns the smallest AABB containing both inputs.
    #[must_use]
    pub fn combined(self, other: Self) -> Self {
        Self {
            lower_bound: Vec2::new(
                min(self.lower_bound.x, other.lower_bound.x),
                min(self.lower_bound.y, other.lower_bound.y),
            ),
            upper_bound: Vec2::new(
                max(self.upper_bound.x, other.upper_bound.x),
                max(self.upper_bound.y, other.upper_bound.y),
            ),
        }
    }

    /// Returns whether this AABB contains `other`, including shared boundaries.
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.lower_bound.x <= other.lower_bound.x
            && self.lower_bound.y <= other.lower_bound.y
            && other.upper_bound.x <= self.upper_bound.x
            && other.upper_bound.y <= self.upper_bound.y
    }

    /// Returns whether two AABBs overlap, including shared boundaries.
    #[must_use]
    pub fn overlaps(self, other: Self) -> bool {
        let other_after_self = other.lower_bound - self.upper_bound;
        if other_after_self.x > 0.0 || other_after_self.y > 0.0 {
            return false;
        }

        let self_after_other = self.lower_bound - other.upper_bound;
        self_after_other.x <= 0.0 && self_after_other.y <= 0.0
    }
}

/// Initialized mass properties for one shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MassData {
    mass: f32,
    center: Vec2,
    rotational_inertia: f32,
}

impl MassData {
    /// Creates finite, non-negative shape mass properties.
    ///
    /// # Errors
    ///
    /// Returns a typed error when any value is non-finite or when mass or
    /// rotational inertia is negative.
    #[must_use = "mass-data construction can fail for invalid values"]
    pub fn new(mass: f32, center: Vec2, rotational_inertia: f32) -> Result<Self, CollisionError> {
        validate_scalar(mass)?;
        validate_vec2(center)?;
        validate_scalar(rotational_inertia)?;
        if mass < 0.0 || rotational_inertia < 0.0 {
            return Err(CollisionError::InvalidGeometry);
        }

        Ok(Self {
            mass,
            center,
            rotational_inertia,
        })
    }

    /// Returns the mass in kilograms.
    #[must_use]
    pub const fn mass(self) -> f32 {
        self.mass
    }

    /// Returns the local center of mass in meters.
    #[must_use]
    pub const fn center(self) -> Vec2 {
        self.center
    }

    /// Returns the rotational inertia about the local origin.
    #[must_use]
    pub const fn rotational_inertia(self) -> f32 {
        self.rotational_inertia
    }
}

/// A finite ray segment and its current normalized clipping fraction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayCastInput {
    start: Vec2,
    end: Vec2,
    max_fraction: f32,
}

impl RayCastInput {
    /// Creates a checked ray-cast input.
    ///
    /// The effective segment ends at `start + max_fraction * (end - start)`.
    ///
    /// # Errors
    ///
    /// Returns a typed error for non-finite values or a clipping fraction
    /// outside `0.0..=1.0`.
    #[must_use = "ray input construction can fail for invalid values"]
    pub fn new(start: Vec2, end: Vec2, max_fraction: f32) -> Result<Self, CollisionError> {
        validate_vec2(start)?;
        validate_vec2(end)?;
        validate_fraction(max_fraction)?;
        Ok(Self {
            start,
            end,
            max_fraction,
        })
    }

    /// Returns the ray origin.
    #[must_use]
    pub const fn start(self) -> Vec2 {
        self.start
    }

    /// Returns the unclipped ray endpoint.
    #[must_use]
    pub const fn end(self) -> Vec2 {
        self.end
    }

    /// Returns the current inclusive clipping fraction.
    #[must_use]
    pub const fn max_fraction(self) -> f32 {
        self.max_fraction
    }
}

/// One initialized ray-cast hit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayCastHit {
    normal: Vec2,
    fraction: f32,
}

impl RayCastHit {
    /// Creates a finite hit within the normalized ray interval.
    ///
    /// # Errors
    ///
    /// Returns a typed error for a non-finite normal or invalid fraction.
    #[must_use = "ray-hit construction can fail for invalid values"]
    pub fn new(normal: Vec2, fraction: f32) -> Result<Self, CollisionError> {
        validate_vec2(normal)?;
        validate_fraction(fraction)?;
        Ok(Self { normal, fraction })
    }

    /// Returns the outward surface normal.
    #[must_use]
    pub const fn normal(self) -> Vec2 {
        self.normal
    }

    /// Returns the normalized hit fraction.
    #[must_use]
    pub const fn fraction(self) -> f32 {
        self.fraction
    }
}

/// A checked public coordinate selecting one shape child.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChildIndex(usize);

impl ChildIndex {
    /// Checks that `requested` is smaller than `child_count`.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionError::ChildIndexOutOfRange`] when the child does not exist.
    #[must_use = "child-index construction can fail for an absent child"]
    pub const fn new(requested: usize, child_count: usize) -> Result<Self, CollisionError> {
        if requested >= child_count {
            return Err(CollisionError::ChildIndexOutOfRange {
                requested,
                child_count,
            });
        }
        Ok(Self(requested))
    }

    /// Returns the checked public child coordinate.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// The geometric kind of one contact feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeatureKind {
    /// A shape vertex.
    Vertex,
    /// A shape face.
    Face,
}

/// Portable semantic identity for the two features forming a contact point.
///
/// This value deliberately has no packed integer key or raw constructor.
///
/// ```compile_fail
/// use liquidfun::collision::ContactFeatureId;
///
/// let _feature = ContactFeatureId::from_raw(0_u32);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContactFeatureId {
    index_a: u8,
    index_b: u8,
    kind_a: FeatureKind,
    kind_b: FeatureKind,
}

impl ContactFeatureId {
    /// Creates a semantic feature identity without packing its fields.
    #[must_use]
    pub const fn new(index_a: u8, index_b: u8, kind_a: FeatureKind, kind_b: FeatureKind) -> Self {
        Self {
            index_a,
            index_b,
            kind_a,
            kind_b,
        }
    }

    /// Returns the feature index on shape A.
    #[must_use]
    pub const fn index_a(self) -> u8 {
        self.index_a
    }

    /// Returns the feature index on shape B.
    #[must_use]
    pub const fn index_b(self) -> u8 {
        self.index_b
    }

    /// Returns the feature kind on shape A.
    #[must_use]
    pub const fn kind_a(self) -> FeatureKind {
        self.kind_a
    }

    /// Returns the feature kind on shape B.
    #[must_use]
    pub const fn kind_b(self) -> FeatureKind {
        self.kind_b
    }
}

/// The active geometry form of a contact manifold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifoldKind {
    /// Point-versus-point contact between circles.
    Circles,
    /// Clip points against a face on shape A.
    FaceA,
    /// Clip points against a face on shape B.
    FaceB,
}

/// One geometric manifold point without solver impulses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ManifoldPoint {
    local_point: Vec2,
    feature_id: ContactFeatureId,
}

impl ManifoldPoint {
    /// Creates an initialized local point and semantic feature identity.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionError::NonFiniteValue`] for a non-finite coordinate.
    #[must_use = "manifold-point construction can fail for invalid coordinates"]
    pub fn new(local_point: Vec2, feature_id: ContactFeatureId) -> Result<Self, CollisionError> {
        validate_vec2(local_point)?;
        Ok(Self {
            local_point,
            feature_id,
        })
    }

    /// Returns the shape-local contact point.
    #[must_use]
    pub const fn local_point(self) -> Vec2 {
        self.local_point
    }

    /// Returns the semantic contact feature identity.
    #[must_use]
    pub const fn feature_id(self) -> ContactFeatureId {
        self.feature_id
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ManifoldState {
    Empty,
    Circles {
        local_point: Vec2,
        point: [ManifoldPoint; 1],
    },
    Face {
        kind: ManifoldKind,
        local_normal: Vec2,
        local_point: Vec2,
        points: ActiveManifoldPoints,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum ActiveManifoldPoints {
    One([ManifoldPoint; 1]),
    Two([ManifoldPoint; 2]),
}

impl ActiveManifoldPoints {
    fn from_slice(points: &[ManifoldPoint]) -> Result<Self, CollisionError> {
        match points {
            [point] => Ok(Self::One([*point])),
            [first, second] => Ok(Self::Two([*first, *second])),
            _ => Err(CollisionError::InvalidGeometry),
        }
    }

    fn as_slice(&self) -> &[ManifoldPoint] {
        match self {
            Self::One(points) => points,
            Self::Two(points) => points,
        }
    }
}

/// A fully initialized zero-, one-, or two-point contact manifold.
///
/// Empty manifolds carry no inactive normal, local point, or point payload.
/// Face manifolds require one or two points, matching the pinned
/// `MAX_MANIFOLD_POINTS` capacity. Solver impulses are intentionally absent.
#[derive(Debug, Clone, PartialEq)]
pub struct Manifold {
    state: ManifoldState,
}

impl Manifold {
    /// Creates an empty separated manifold with no inactive payload.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            state: ManifoldState::Empty,
        }
    }

    /// Creates a one-point circle manifold.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionError::NonFiniteValue`] for a non-finite local point.
    #[must_use = "circle-manifold construction can fail for invalid coordinates"]
    pub fn circles(local_point: Vec2, point: ManifoldPoint) -> Result<Self, CollisionError> {
        validate_vec2(local_point)?;
        Ok(Self {
            state: ManifoldState::Circles {
                local_point,
                point: [point],
            },
        })
    }

    /// Creates a one- or two-point manifold against a face on shape A.
    ///
    /// # Errors
    ///
    /// Returns a typed error for non-finite geometry or any point count other
    /// than one or two.
    #[must_use = "face-manifold construction can fail for invalid geometry"]
    pub fn face_a(
        local_normal: Vec2,
        local_point: Vec2,
        points: &[ManifoldPoint],
    ) -> Result<Self, CollisionError> {
        Self::face(ManifoldKind::FaceA, local_normal, local_point, points)
    }

    /// Creates a one- or two-point manifold against a face on shape B.
    ///
    /// # Errors
    ///
    /// Returns a typed error for non-finite geometry or any point count other
    /// than one or two.
    #[must_use = "face-manifold construction can fail for invalid geometry"]
    pub fn face_b(
        local_normal: Vec2,
        local_point: Vec2,
        points: &[ManifoldPoint],
    ) -> Result<Self, CollisionError> {
        Self::face(ManifoldKind::FaceB, local_normal, local_point, points)
    }

    fn face(
        kind: ManifoldKind,
        local_normal: Vec2,
        local_point: Vec2,
        points: &[ManifoldPoint],
    ) -> Result<Self, CollisionError> {
        validate_vec2(local_normal)?;
        validate_vec2(local_point)?;
        let points = ActiveManifoldPoints::from_slice(points)?;
        Ok(Self {
            state: ManifoldState::Face {
                kind,
                local_normal,
                local_point,
                points,
            },
        })
    }

    /// Returns the active manifold kind, or `None` when separated.
    #[must_use]
    pub const fn kind(&self) -> Option<ManifoldKind> {
        match self.state {
            ManifoldState::Empty => None,
            ManifoldState::Circles { .. } => Some(ManifoldKind::Circles),
            ManifoldState::Face { kind, .. } => Some(kind),
        }
    }

    /// Returns the active local normal for a face manifold.
    #[must_use]
    pub const fn local_normal(&self) -> Option<Vec2> {
        match self.state {
            ManifoldState::Face { local_normal, .. } => Some(local_normal),
            ManifoldState::Empty | ManifoldState::Circles { .. } => None,
        }
    }

    /// Returns the active local reference point, or `None` when separated.
    #[must_use]
    pub const fn local_point(&self) -> Option<Vec2> {
        match self.state {
            ManifoldState::Empty => None,
            ManifoldState::Circles { local_point, .. }
            | ManifoldState::Face { local_point, .. } => Some(local_point),
        }
    }

    /// Returns active manifold points in source order.
    #[must_use]
    pub fn points(&self) -> &[ManifoldPoint] {
        match &self.state {
            ManifoldState::Empty => &[],
            ManifoldState::Circles { point, .. } => point,
            ManifoldState::Face { points, .. } => points.as_slice(),
        }
    }
}

/// The state of one semantic contact point across manifold updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointState {
    /// No point exists at this coordinate.
    Null,
    /// The point was added by the new manifold.
    Added,
    /// The feature identity persists across both manifolds.
    Persisted,
    /// The point was removed from the old manifold.
    Removed,
}

/// The exhaustive semantic result of a supported-pair dispatch attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum CollisionOutcome<T> {
    /// The pinned registry contains no kernel for the shape pair.
    Unsupported,
    /// The supported pair is separated.
    Separated,
    /// The supported pair is touching and produced the enclosed value.
    Touching(T),
}

fn validate_scalar(value: f32) -> Result<(), CollisionError> {
    if !value.is_finite() {
        return Err(CollisionError::NonFiniteValue);
    }
    Ok(())
}

fn validate_vec2(value: Vec2) -> Result<(), CollisionError> {
    if !value.is_valid() {
        return Err(CollisionError::NonFiniteValue);
    }
    Ok(())
}

fn validate_fraction(fraction: f32) -> Result<(), CollisionError> {
    validate_scalar(fraction)?;
    if !(0.0..=1.0).contains(&fraction) {
        return Err(CollisionError::FractionOutOfRange);
    }
    Ok(())
}
