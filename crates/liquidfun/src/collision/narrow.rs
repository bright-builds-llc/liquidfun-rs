//! Clipping, semantic manifolds, and supported shape-pair dispatch.

mod circle;
mod clipping;
mod polygon;

use crate::collision::shape::{CircleShape, PolygonShape};
use crate::collision::{CollisionError, Manifold, ManifoldKind, PointState};
use crate::math::{Transform, Vec2};

/// One active world-space contact point and its signed separation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldManifoldPoint {
    point: Vec2,
    separation: f32,
}

impl WorldManifoldPoint {
    /// Returns the world-space midpoint between the two collision skins.
    #[must_use]
    pub const fn point(self) -> Vec2 {
        self.point
    }

    /// Returns the signed skin-to-skin separation along the manifold normal.
    #[must_use]
    pub const fn separation(self) -> f32 {
        self.separation
    }
}

/// An active world-space manifold whose normal points from shape A to shape B.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldManifold {
    normal: Vec2,
    points: Vec<WorldManifoldPoint>,
}

impl WorldManifold {
    /// Returns the world normal pointing from shape A toward shape B.
    #[must_use]
    pub const fn normal(&self) -> Vec2 {
        self.normal
    }

    /// Returns active world points in manifold order.
    #[must_use]
    pub fn points(&self) -> &[WorldManifoldPoint] {
        &self.points
    }
}

/// Generates a circle-circle manifold in A/B orientation.
///
/// # Errors
///
/// Returns [`CollisionError::NonFiniteValue`] when either transform is invalid.
pub fn collide_circles(
    circle_a: &CircleShape,
    transform_a: Transform,
    circle_b: &CircleShape,
    transform_b: Transform,
) -> Result<Option<Manifold>, CollisionError> {
    validate_transform(transform_a)?;
    validate_transform(transform_b)?;
    circle::collide_circles(circle_a, transform_a, circle_b, transform_b)
}

/// Generates a polygon-circle manifold with the polygon as shape A.
///
/// # Errors
///
/// Returns [`CollisionError::NonFiniteValue`] when either transform is invalid.
pub fn collide_polygon_circle(
    polygon_a: &PolygonShape,
    transform_a: Transform,
    circle_b: &CircleShape,
    transform_b: Transform,
) -> Result<Option<Manifold>, CollisionError> {
    validate_transform(transform_a)?;
    validate_transform(transform_b)?;
    circle::collide_polygon_circle(polygon_a, transform_a, circle_b, transform_b)
}

/// Generates a polygon-polygon manifold in A/B orientation.
///
/// # Errors
///
/// Returns [`CollisionError::NonFiniteValue`] when either transform is invalid.
pub fn collide_polygons(
    polygon_a: &PolygonShape,
    transform_a: Transform,
    polygon_b: &PolygonShape,
    transform_b: Transform,
) -> Result<Option<Manifold>, CollisionError> {
    validate_transform(transform_a)?;
    validate_transform(transform_b)?;
    polygon::collide_polygons(polygon_a, transform_a, polygon_b, transform_b)
}

/// Converts an active local manifold to world coordinates.
///
/// # Errors
///
/// Returns a typed error for an invalid transform, radius, or derived value.
pub fn world_manifold(
    manifold: &Manifold,
    transform_a: Transform,
    radius_a: f32,
    transform_b: Transform,
    radius_b: f32,
) -> Result<Option<WorldManifold>, CollisionError> {
    validate_transform(transform_a)?;
    validate_transform(transform_b)?;
    validate_radius(radius_a)?;
    validate_radius(radius_b)?;
    let Some(kind) = manifold.kind() else {
        return Ok(None);
    };

    let (normal, points) = match kind {
        ManifoldKind::Circles => {
            world_circles(manifold, transform_a, radius_a, transform_b, radius_b)
        }
        ManifoldKind::FaceA => world_face_a(manifold, transform_a, radius_a, transform_b, radius_b),
        ManifoldKind::FaceB => world_face_b(manifold, transform_a, radius_a, transform_b, radius_b),
    };
    if !normal.is_valid()
        || points
            .iter()
            .any(|point| !point.point.is_valid() || !point.separation.is_finite())
    {
        return Err(CollisionError::NonFiniteValue);
    }
    Ok(Some(WorldManifold { normal, points }))
}

/// Ordered point-state transitions between two semantic manifolds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointStates {
    previous: [PointState; 2],
    current: [PointState; 2],
}

impl PointStates {
    /// Returns old-manifold states in old point order.
    #[must_use]
    pub const fn previous(&self) -> &[PointState; 2] {
        &self.previous
    }

    /// Returns new-manifold states in new point order.
    #[must_use]
    pub const fn current(&self) -> &[PointState; 2] {
        &self.current
    }
}

/// Classifies add, persist, and remove transitions by semantic feature identity.
#[must_use]
pub fn point_states(previous: &Manifold, current: &Manifold) -> PointStates {
    let mut previous_states = [PointState::Null; 2];
    let mut current_states = [PointState::Null; 2];

    for (index, point) in previous.points().iter().enumerate() {
        previous_states[index] = if current
            .points()
            .iter()
            .any(|candidate| candidate.feature_id() == point.feature_id())
        {
            PointState::Persisted
        } else {
            PointState::Removed
        };
    }
    for (index, point) in current.points().iter().enumerate() {
        current_states[index] = if previous
            .points()
            .iter()
            .any(|candidate| candidate.feature_id() == point.feature_id())
        {
            PointState::Persisted
        } else {
            PointState::Added
        };
    }

    PointStates {
        previous: previous_states,
        current: current_states,
    }
}

fn world_circles(
    manifold: &Manifold,
    transform_a: Transform,
    radius_a: f32,
    transform_b: Transform,
    radius_b: f32,
) -> (Vec2, Vec<WorldManifoldPoint>) {
    let point_a = transform_a.apply(
        manifold
            .local_point()
            .expect("active circle manifold has a local point"),
    );
    let point_b = transform_b.apply(manifold.points()[0].local_point());
    let mut normal = Vec2::new(1.0, 0.0);
    if (point_a - point_b).length_squared()
        > crate::math::settings::EPSILON * crate::math::settings::EPSILON
    {
        normal = point_b - point_a;
        normal.normalize();
    }
    let center_a = point_a + radius_a * normal;
    let center_b = point_b - radius_b * normal;
    (
        normal,
        vec![WorldManifoldPoint {
            point: 0.5 * (center_a + center_b),
            separation: (center_b - center_a).dot(normal),
        }],
    )
}

fn world_face_a(
    manifold: &Manifold,
    transform_a: Transform,
    radius_a: f32,
    transform_b: Transform,
    radius_b: f32,
) -> (Vec2, Vec<WorldManifoldPoint>) {
    let normal = transform_a.rotation().apply(
        manifold
            .local_normal()
            .expect("active face manifold has a local normal"),
    );
    let plane_point = transform_a.apply(
        manifold
            .local_point()
            .expect("active face manifold has a local point"),
    );
    let points = manifold
        .points()
        .iter()
        .map(|point| {
            let clip_point = transform_b.apply(point.local_point());
            let center_a =
                clip_point + (radius_a - (clip_point - plane_point).dot(normal)) * normal;
            let center_b = clip_point - radius_b * normal;
            WorldManifoldPoint {
                point: 0.5 * (center_a + center_b),
                separation: (center_b - center_a).dot(normal),
            }
        })
        .collect();
    (normal, points)
}

fn world_face_b(
    manifold: &Manifold,
    transform_a: Transform,
    radius_a: f32,
    transform_b: Transform,
    radius_b: f32,
) -> (Vec2, Vec<WorldManifoldPoint>) {
    let normal = transform_b.rotation().apply(
        manifold
            .local_normal()
            .expect("active face manifold has a local normal"),
    );
    let plane_point = transform_b.apply(
        manifold
            .local_point()
            .expect("active face manifold has a local point"),
    );
    let points = manifold
        .points()
        .iter()
        .map(|point| {
            let clip_point = transform_a.apply(point.local_point());
            let center_b =
                clip_point + (radius_b - (clip_point - plane_point).dot(normal)) * normal;
            let center_a = clip_point - radius_a * normal;
            WorldManifoldPoint {
                point: 0.5 * (center_a + center_b),
                separation: (center_a - center_b).dot(normal),
            }
        })
        .collect();
    (-normal, points)
}

fn validate_transform(transform: Transform) -> Result<(), CollisionError> {
    if !transform.position().is_valid()
        || !transform.rotation().sine().is_finite()
        || !transform.rotation().cosine().is_finite()
    {
        return Err(CollisionError::NonFiniteValue);
    }
    Ok(())
}

fn validate_radius(radius: f32) -> Result<(), CollisionError> {
    if !radius.is_finite() {
        return Err(CollisionError::NonFiniteValue);
    }
    if radius < 0.0 {
        return Err(CollisionError::InvalidGeometry);
    }
    Ok(())
}
