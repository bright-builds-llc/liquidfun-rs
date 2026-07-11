use crate::collision::{Aabb, CollisionError, MassData, RayCastHit, RayCastInput};
use crate::math::settings::{EPSILON, LINEAR_SLOP, MAX_POLYGON_VERTICES, POLYGON_RADIUS};
use crate::math::{Rotation, Transform, Vec2};

use super::{
    PointDistance, validate_density, validate_query, validate_scalar, validate_transform,
    validate_vec2,
};

/// An immutable owned convex polygon with source-ordered hull state.
#[derive(Debug, Clone, PartialEq)]
pub struct PolygonShape {
    vertices: Vec<Vec2>,
    normals: Vec<Vec2>,
    centroid: Vec2,
}

impl PolygonShape {
    /// Builds the pinned gift-wrapped hull from at most eight finite points.
    ///
    /// The source's squared-distance comparison against `0.5 * LINEAR_SLOP`
    /// is intentionally preserved. Inputs that source code handled through
    /// assertions or substitute geometry return a typed error here.
    ///
    /// # Errors
    ///
    /// Returns a typed error for non-finite, excessive, welded, collinear, or
    /// otherwise degenerate geometry.
    pub fn new(points: &[Vec2]) -> Result<Self, CollisionError> {
        if points.len() < 3 || points.len() > MAX_POLYGON_VERTICES {
            return Err(CollisionError::InvalidGeometry);
        }
        for point in points {
            validate_vec2(*point)?;
        }

        let mut welded = Vec::with_capacity(points.len());
        for point in points {
            let is_unique = welded
                .iter()
                .all(|existing: &Vec2| (*point - *existing).length_squared() >= 0.5 * LINEAR_SLOP);
            if is_unique {
                welded.push(*point);
            }
        }
        if welded.len() < 3 {
            return Err(CollisionError::InvalidGeometry);
        }

        let start = rightmost_lowest(&welded);
        let hull = gift_wrap(&welded, start)?;
        Self::from_vertices(hull)
    }

    /// Builds an axis-aligned box from positive finite half-extents.
    ///
    /// # Errors
    ///
    /// Returns a typed error for non-finite or non-positive half-extents.
    pub fn box_shape(half_width: f32, half_height: f32) -> Result<Self, CollisionError> {
        validate_half_extents(half_width, half_height)?;
        Self::from_box(half_width, half_height, Vec2::ZERO, Rotation::IDENTITY)
    }

    /// Builds a local oriented box with the pinned initial ordering.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid half-extents, center, angle, or
    /// transformed geometry.
    pub fn oriented_box(
        half_width: f32,
        half_height: f32,
        center: Vec2,
        angle: f32,
    ) -> Result<Self, CollisionError> {
        validate_half_extents(half_width, half_height)?;
        validate_vec2(center)?;
        validate_scalar(angle)?;
        Self::from_box(half_width, half_height, center, Rotation::from_angle(angle))
    }

    fn from_box(
        half_width: f32,
        half_height: f32,
        center: Vec2,
        rotation: Rotation,
    ) -> Result<Self, CollisionError> {
        let local_vertices = [
            Vec2::new(-half_width, -half_height),
            Vec2::new(half_width, -half_height),
            Vec2::new(half_width, half_height),
            Vec2::new(-half_width, half_height),
        ];
        let local_normals = [
            Vec2::new(0.0, -1.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(-1.0, 0.0),
        ];
        let vertices = local_vertices
            .into_iter()
            .map(|point| center + rotation.apply(point))
            .collect();
        let normals = local_normals
            .into_iter()
            .map(|normal| rotation.apply(normal))
            .collect();
        Self::from_derived(vertices, normals, center)
    }

    fn from_vertices(vertices: Vec<Vec2>) -> Result<Self, CollisionError> {
        if vertices.len() < 3 {
            return Err(CollisionError::InvalidGeometry);
        }
        let mut normals = Vec::with_capacity(vertices.len());
        for index in 0..vertices.len() {
            let next = (index + 1) % vertices.len();
            let edge = vertices[next] - vertices[index];
            if edge.length_squared() <= EPSILON * EPSILON {
                return Err(CollisionError::InvalidGeometry);
            }
            let mut normal = edge.cross_scalar(1.0);
            if normal.normalize() == 0.0 || !normal.is_valid() {
                return Err(CollisionError::InvalidGeometry);
            }
            normals.push(normal);
        }
        let centroid = compute_centroid(&vertices)?;
        Self::from_derived(vertices, normals, centroid)
    }

    fn from_derived(
        vertices: Vec<Vec2>,
        normals: Vec<Vec2>,
        centroid: Vec2,
    ) -> Result<Self, CollisionError> {
        if vertices.len() < 3
            || vertices.len() > MAX_POLYGON_VERTICES
            || vertices.len() != normals.len()
            || !centroid.is_valid()
            || vertices.iter().any(|point| !point.is_valid())
            || normals.iter().any(|normal| !normal.is_valid())
        {
            return Err(CollisionError::InvalidGeometry);
        }
        let polygon = Self {
            vertices,
            normals,
            centroid,
        };
        if !polygon.validate() {
            return Err(CollisionError::InvalidGeometry);
        }
        Ok(polygon)
    }

    /// Returns the source-ordered hull points.
    #[must_use]
    pub fn vertices(&self) -> &[Vec2] {
        &self.vertices
    }

    /// Returns source-ordered outward unit normals.
    #[must_use]
    pub fn normals(&self) -> &[Vec2] {
        &self.normals
    }

    /// Returns the number of hull points.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Returns the local centroid.
    #[must_use]
    pub const fn centroid(&self) -> Vec2 {
        self.centroid
    }

    /// Returns the pinned polygon collision skin radius.
    #[must_use]
    pub const fn radius(&self) -> f32 {
        POLYGON_RADIUS
    }

    /// Returns the single child count.
    #[must_use]
    pub const fn child_count(&self) -> usize {
        1
    }

    /// Performs the pinned exhaustive convexity check.
    #[must_use]
    pub fn validate(&self) -> bool {
        for index in 0..self.vertices.len() {
            let next = (index + 1) % self.vertices.len();
            let point = self.vertices[index];
            let edge = self.vertices[next] - point;
            for other in 0..self.vertices.len() {
                if other == index || other == next {
                    continue;
                }
                if edge.cross(self.vertices[other] - point) < 0.0 {
                    return false;
                }
            }
        }
        true
    }

    /// Tests a finite point against every source-ordered half-space.
    ///
    /// # Errors
    ///
    /// Returns [`CollisionError::NonFiniteValue`] for invalid query geometry.
    pub fn test_point(&self, transform: Transform, point: Vec2) -> Result<bool, CollisionError> {
        validate_query(transform, point)?;
        let local_point = transform
            .rotation()
            .inverse_apply(point - transform.position());
        for (normal, vertex) in self.normals.iter().zip(&self.vertices) {
            if normal.dot(local_point - *vertex) > 0.0 {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Computes source-ordered signed distance to a finite world point.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid query geometry or derived data.
    pub fn distance_to_point(
        &self,
        transform: Transform,
        point: Vec2,
    ) -> Result<PointDistance, CollisionError> {
        validate_query(transform, point)?;
        let local_point = transform
            .rotation()
            .inverse_apply(point - transform.position());
        let mut maximum_distance = -f32::MAX;
        let mut maximum_normal = local_point;
        for (normal, vertex) in self.normals.iter().zip(&self.vertices) {
            let distance = normal.dot(local_point - *vertex);
            if distance > maximum_distance {
                maximum_distance = distance;
                maximum_normal = *normal;
            }
        }
        if maximum_distance <= 0.0 {
            return PointDistance::new(
                maximum_distance,
                transform.rotation().apply(maximum_normal),
            );
        }

        let mut minimum = maximum_normal;
        let mut minimum_squared = maximum_distance * maximum_distance;
        for vertex in &self.vertices {
            let offset = local_point - *vertex;
            let squared = offset.length_squared();
            if minimum_squared > squared {
                minimum = offset;
                minimum_squared = squared;
            }
        }
        let distance = minimum_squared.sqrt();
        let mut normal = transform.rotation().apply(minimum);
        normal.normalize();
        PointDistance::new(distance, normal)
    }

    /// Casts a clipped ray against all polygon half-spaces.
    ///
    /// A ray beginning inside has no entering face and therefore no hit.
    ///
    /// # Errors
    ///
    /// Returns a typed error for an invalid transform or derived hit.
    pub fn ray_cast(
        &self,
        input: RayCastInput,
        transform: Transform,
    ) -> Result<Option<RayCastHit>, CollisionError> {
        validate_transform(transform)?;
        let first = transform
            .rotation()
            .inverse_apply(input.start() - transform.position());
        let second = transform
            .rotation()
            .inverse_apply(input.end() - transform.position());
        let direction = second - first;
        let mut lower = 0.0;
        let mut upper = input.max_fraction();
        let mut maybe_index = None;
        for (index, (normal, vertex)) in self.normals.iter().zip(&self.vertices).enumerate() {
            let numerator = normal.dot(*vertex - first);
            let denominator = normal.dot(direction);
            if denominator == 0.0 {
                if numerator < 0.0 {
                    return Ok(None);
                }
            } else if denominator < 0.0 && numerator < lower * denominator {
                lower = numerator / denominator;
                maybe_index = Some(index);
            } else if denominator > 0.0 && numerator < upper * denominator {
                upper = numerator / denominator;
            }
            if upper < lower {
                return Ok(None);
            }
        }
        let Some(index) = maybe_index else {
            return Ok(None);
        };
        let normal = transform.rotation().apply(self.normals[index]);
        Ok(Some(RayCastHit::new(normal, lower)?))
    }

    /// Computes the transformed hull bounds expanded by the polygon radius.
    ///
    /// # Errors
    ///
    /// Returns a typed error if the transform or derived bounds are invalid.
    pub fn compute_aabb(&self, transform: Transform) -> Result<Aabb, CollisionError> {
        validate_transform(transform)?;
        let first = transform.apply(self.vertices[0]);
        let mut lower = first;
        let mut upper = first;
        for vertex in &self.vertices[1..] {
            let point = transform.apply(*vertex);
            lower = Vec2::new(lower.x.min(point.x), lower.y.min(point.y));
            upper = Vec2::new(upper.x.max(point.x), upper.y.max(point.y));
        }
        let radius = Vec2::new(POLYGON_RADIUS, POLYGON_RADIUS);
        Aabb::new(lower - radius, upper + radius)
    }

    /// Computes pinned polygon mass, centroid, and origin inertia.
    ///
    /// # Errors
    ///
    /// Returns a typed error for invalid density or non-finite derived data.
    #[allow(clippy::cast_precision_loss)] // The checked hull count is at most eight.
    pub fn compute_mass(&self, density: f32) -> Result<MassData, CollisionError> {
        validate_density(density)?;
        let mut center = Vec2::ZERO;
        let mut area = 0.0;
        let mut inertia = 0.0;
        let mut reference = Vec2::ZERO;
        for vertex in &self.vertices {
            reference += *vertex;
        }
        reference *= 1.0 / self.vertices.len() as f32;
        let inverse_three = 1.0 / 3.0;
        for index in 0..self.vertices.len() {
            let first = self.vertices[index] - reference;
            let second = self.vertices[(index + 1) % self.vertices.len()] - reference;
            let cross = first.cross(second);
            let triangle_area = 0.5 * cross;
            area += triangle_area;
            center += triangle_area * inverse_three * (first + second);
            let integral_x = first.x * first.x + second.x * first.x + second.x * second.x;
            let integral_y = first.y * first.y + second.y * first.y + second.y * second.y;
            inertia += (0.25 * inverse_three * cross) * (integral_x + integral_y);
        }
        if area <= EPSILON || !area.is_finite() {
            return Err(CollisionError::InvalidGeometry);
        }
        let mass = density * area;
        center *= 1.0 / area;
        let mass_center = center + reference;
        let mut origin_inertia = density * inertia;
        origin_inertia += mass * (mass_center.dot(mass_center) - center.dot(center));
        MassData::new(mass, mass_center, origin_inertia)
    }
}

fn validate_half_extents(half_width: f32, half_height: f32) -> Result<(), CollisionError> {
    validate_scalar(half_width)?;
    validate_scalar(half_height)?;
    if half_width <= 0.0 || half_height <= 0.0 {
        return Err(CollisionError::InvalidGeometry);
    }
    Ok(())
}

#[allow(clippy::float_cmp)] // Exact equality is the pinned start-point tie branch.
fn rightmost_lowest(points: &[Vec2]) -> usize {
    let mut selected = 0;
    let mut selected_x = points[0].x;
    for index in 1..points.len() {
        let point = points[index];
        if point.x > selected_x || (point.x == selected_x && point.y < points[selected].y) {
            selected = index;
            selected_x = point.x;
        }
    }
    selected
}

fn gift_wrap(points: &[Vec2], start: usize) -> Result<Vec<Vec2>, CollisionError> {
    let mut hull = Vec::with_capacity(points.len());
    let mut current = start;
    loop {
        if hull.len() >= points.len() {
            return Err(CollisionError::InvalidGeometry);
        }
        hull.push(points[current]);
        let mut endpoint = 0;
        for candidate in 1..points.len() {
            if endpoint == current {
                endpoint = candidate;
                continue;
            }
            let ray = points[endpoint] - points[current];
            let offset = points[candidate] - points[current];
            let cross = ray.cross(offset);
            if cross < 0.0 || (cross == 0.0 && offset.length_squared() > ray.length_squared()) {
                endpoint = candidate;
            }
        }
        current = endpoint;
        if endpoint == start {
            break;
        }
    }
    if hull.len() < 3 {
        return Err(CollisionError::InvalidGeometry);
    }
    Ok(hull)
}

fn compute_centroid(vertices: &[Vec2]) -> Result<Vec2, CollisionError> {
    let mut centroid = Vec2::ZERO;
    let mut area = 0.0;
    let reference = Vec2::ZERO;
    let inverse_three = 1.0 / 3.0;
    for index in 0..vertices.len() {
        let first = reference;
        let second = vertices[index];
        let third = vertices[(index + 1) % vertices.len()];
        let first_edge = second - first;
        let second_edge = third - first;
        let cross = first_edge.cross(second_edge);
        let triangle_area = 0.5 * cross;
        area += triangle_area;
        centroid += triangle_area * inverse_three * (first + second + third);
    }
    if area <= EPSILON || !area.is_finite() {
        return Err(CollisionError::InvalidGeometry);
    }
    centroid *= 1.0 / area;
    validate_vec2(centroid)?;
    Ok(centroid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hull_collinear_tie_selects_farthest_point() {
        // Arrange
        let points = [
            Vec2::new(1.0, -1.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
            Vec2::new(-1.0, -1.0),
        ];

        // Act
        let polygon = PolygonShape::new(&points).expect("hull should be valid");

        // Assert
        assert_eq!(polygon.vertex_count(), 4);
        assert!(!polygon.vertices().contains(&Vec2::new(1.0, 0.0)));
    }
}
