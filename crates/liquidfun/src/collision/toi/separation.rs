use crate::collision::distance::SupportIndexPair;
use crate::collision::shape::Shape;
use crate::collision::{ChildIndex, CollisionError};
use crate::math::{Sweep, Transform, Vec2};

enum ProxyVertices<'a> {
    Borrowed(&'a [Vec2]),
    Inline { values: [Vec2; 2], count: usize },
}

pub(super) struct ToiProxy<'a> {
    vertices: ProxyVertices<'a>,
    radius: f32,
}

impl<'a> ToiProxy<'a> {
    pub(super) fn new(shape: &'a Shape, child: ChildIndex) -> Result<Self, CollisionError> {
        ChildIndex::new(child.get(), shape.child_count())?;
        match shape {
            Shape::Circle(circle) => Ok(Self::inline(
                [circle.center(), Vec2::ZERO],
                1,
                circle.radius(),
            )),
            Shape::Edge(edge) => Ok(Self::inline([edge.start(), edge.end()], 2, edge.radius())),
            Shape::Polygon(polygon) => Ok(Self {
                vertices: ProxyVertices::Borrowed(polygon.vertices()),
                radius: polygon.radius(),
            }),
            Shape::Chain(chain) => {
                let edge = chain.child_edge(child)?;
                Ok(Self::inline([edge.start(), edge.end()], 2, chain.radius()))
            }
        }
    }

    const fn inline(values: [Vec2; 2], count: usize, radius: f32) -> Self {
        Self {
            vertices: ProxyVertices::Inline { values, count },
            radius,
        }
    }

    pub(super) fn radius(&self) -> f32 {
        self.radius
    }

    fn vertex(&self, index: usize) -> Result<Vec2, CollisionError> {
        self.vertices()
            .get(index)
            .copied()
            .ok_or(CollisionError::IncompatibleShapeProxy)
    }

    fn support_index(&self, direction: Vec2) -> usize {
        let vertices = self.vertices();
        let mut best_index = 0;
        let mut best_value = vertices[0].dot(direction);
        for (index, vertex) in vertices.iter().enumerate().skip(1) {
            let value = vertex.dot(direction);
            if value > best_value {
                best_index = index;
                best_value = value;
            }
        }
        best_index
    }

    fn vertices(&self) -> &[Vec2] {
        match &self.vertices {
            ProxyVertices::Borrowed(vertices) => vertices,
            ProxyVertices::Inline { values, count } => &values[..*count],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SeparationKind {
    Points,
    FaceA,
    FaceB,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SeparationIndices {
    Points { index_a: usize, index_b: usize },
    FaceA { index_b: usize },
    FaceB { index_a: usize },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct MinimumSeparation {
    pub(super) indices: SeparationIndices,
    pub(super) value: f32,
}

pub(super) struct SeparationFunction<'a, 'shape> {
    proxy_a: &'a ToiProxy<'shape>,
    proxy_b: &'a ToiProxy<'shape>,
    sweep_a: Sweep,
    sweep_b: Sweep,
    kind: SeparationKind,
    local_point: Vec2,
    axis: Vec2,
}

impl<'a, 'shape> SeparationFunction<'a, 'shape> {
    pub(super) fn initialize(
        support_pairs: &[SupportIndexPair],
        proxy_a: &'a ToiProxy<'shape>,
        sweep_a: Sweep,
        proxy_b: &'a ToiProxy<'shape>,
        sweep_b: Sweep,
        time: f32,
    ) -> Result<(Self, f32), CollisionError> {
        if support_pairs.is_empty() || support_pairs.len() >= 3 {
            return Err(CollisionError::IncompatibleDistanceCache);
        }
        let transform_a = transform_at(sweep_a, time)?;
        let transform_b = transform_at(sweep_b, time)?;
        if support_pairs.len() == 1 {
            return Self::initialize_points(
                support_pairs[0],
                proxy_a,
                sweep_a,
                transform_a,
                proxy_b,
                sweep_b,
                transform_b,
            );
        }
        if support_pairs[0].index_a() == support_pairs[1].index_a() {
            return Self::initialize_face_b(
                support_pairs,
                proxy_a,
                sweep_a,
                transform_a,
                proxy_b,
                sweep_b,
                transform_b,
            );
        }
        Self::initialize_face_a(
            support_pairs,
            proxy_a,
            sweep_a,
            transform_a,
            proxy_b,
            sweep_b,
            transform_b,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn initialize_points(
        pair: SupportIndexPair,
        proxy_a: &'a ToiProxy<'shape>,
        sweep_a: Sweep,
        transform_a: Transform,
        proxy_b: &'a ToiProxy<'shape>,
        sweep_b: Sweep,
        transform_b: Transform,
    ) -> Result<(Self, f32), CollisionError> {
        let point_a = transform_a.apply(proxy_a.vertex(pair.index_a())?);
        let point_b = transform_b.apply(proxy_b.vertex(pair.index_b())?);
        let mut axis = point_b - point_a;
        let separation = axis.normalize();
        validate_derived(axis, separation)?;
        Ok((
            Self {
                proxy_a,
                proxy_b,
                sweep_a,
                sweep_b,
                kind: SeparationKind::Points,
                local_point: Vec2::ZERO,
                axis,
            },
            separation,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn initialize_face_b(
        pairs: &[SupportIndexPair],
        proxy_a: &'a ToiProxy<'shape>,
        sweep_a: Sweep,
        transform_a: Transform,
        proxy_b: &'a ToiProxy<'shape>,
        sweep_b: Sweep,
        transform_b: Transform,
    ) -> Result<(Self, f32), CollisionError> {
        let local_b1 = proxy_b.vertex(pairs[0].index_b())?;
        let local_b2 = proxy_b.vertex(pairs[1].index_b())?;
        let mut axis = (local_b2 - local_b1).cross_scalar(1.0);
        if axis.normalize() == 0.0 {
            return Err(CollisionError::InvalidGeometry);
        }
        let normal = transform_b.rotation().apply(axis);
        let local_point = 0.5 * (local_b1 + local_b2);
        let point_b = transform_b.apply(local_point);
        let point_a = transform_a.apply(proxy_a.vertex(pairs[0].index_a())?);
        let mut separation = (point_a - point_b).dot(normal);
        if separation < 0.0 {
            axis = -axis;
            separation = -separation;
        }
        validate_derived(axis, separation)?;
        Ok((
            Self {
                proxy_a,
                proxy_b,
                sweep_a,
                sweep_b,
                kind: SeparationKind::FaceB,
                local_point,
                axis,
            },
            separation,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn initialize_face_a(
        pairs: &[SupportIndexPair],
        proxy_a: &'a ToiProxy<'shape>,
        sweep_a: Sweep,
        transform_a: Transform,
        proxy_b: &'a ToiProxy<'shape>,
        sweep_b: Sweep,
        transform_b: Transform,
    ) -> Result<(Self, f32), CollisionError> {
        let local_a1 = proxy_a.vertex(pairs[0].index_a())?;
        let local_a2 = proxy_a.vertex(pairs[1].index_a())?;
        let mut axis = (local_a2 - local_a1).cross_scalar(1.0);
        if axis.normalize() == 0.0 {
            return Err(CollisionError::InvalidGeometry);
        }
        let normal = transform_a.rotation().apply(axis);
        let local_point = 0.5 * (local_a1 + local_a2);
        let point_a = transform_a.apply(local_point);
        let point_b = transform_b.apply(proxy_b.vertex(pairs[0].index_b())?);
        let mut separation = (point_b - point_a).dot(normal);
        if separation < 0.0 {
            axis = -axis;
            separation = -separation;
        }
        validate_derived(axis, separation)?;
        Ok((
            Self {
                proxy_a,
                proxy_b,
                sweep_a,
                sweep_b,
                kind: SeparationKind::FaceA,
                local_point,
                axis,
            },
            separation,
        ))
    }

    pub(super) const fn kind(&self) -> SeparationKind {
        self.kind
    }

    pub(super) fn find_minimum(&self, time: f32) -> Result<MinimumSeparation, CollisionError> {
        let transform_a = transform_at(self.sweep_a, time)?;
        let transform_b = transform_at(self.sweep_b, time)?;
        let minimum = match self.kind {
            SeparationKind::Points => {
                let axis_a = transform_a.rotation().inverse_apply(self.axis);
                let axis_b = transform_b.rotation().inverse_apply(-self.axis);
                let index_a = self.proxy_a.support_index(axis_a);
                let index_b = self.proxy_b.support_index(axis_b);
                let point_a = transform_a.apply(self.proxy_a.vertex(index_a)?);
                let point_b = transform_b.apply(self.proxy_b.vertex(index_b)?);
                MinimumSeparation {
                    indices: SeparationIndices::Points { index_a, index_b },
                    value: (point_b - point_a).dot(self.axis),
                }
            }
            SeparationKind::FaceA => {
                let normal = transform_a.rotation().apply(self.axis);
                let point_a = transform_a.apply(self.local_point);
                let axis_b = transform_b.rotation().inverse_apply(-normal);
                let index_b = self.proxy_b.support_index(axis_b);
                let point_b = transform_b.apply(self.proxy_b.vertex(index_b)?);
                MinimumSeparation {
                    indices: SeparationIndices::FaceA { index_b },
                    value: (point_b - point_a).dot(normal),
                }
            }
            SeparationKind::FaceB => {
                let normal = transform_b.rotation().apply(self.axis);
                let point_b = transform_b.apply(self.local_point);
                let axis_a = transform_a.rotation().inverse_apply(-normal);
                let index_a = self.proxy_a.support_index(axis_a);
                let point_a = transform_a.apply(self.proxy_a.vertex(index_a)?);
                MinimumSeparation {
                    indices: SeparationIndices::FaceB { index_a },
                    value: (point_a - point_b).dot(normal),
                }
            }
        };
        validate_separation(minimum.value)?;
        Ok(minimum)
    }

    pub(super) fn evaluate(
        &self,
        indices: SeparationIndices,
        time: f32,
    ) -> Result<f32, CollisionError> {
        let transform_a = transform_at(self.sweep_a, time)?;
        let transform_b = transform_at(self.sweep_b, time)?;
        let separation = match (self.kind, indices) {
            (SeparationKind::Points, SeparationIndices::Points { index_a, index_b }) => {
                let point_a = transform_a.apply(self.proxy_a.vertex(index_a)?);
                let point_b = transform_b.apply(self.proxy_b.vertex(index_b)?);
                (point_b - point_a).dot(self.axis)
            }
            (SeparationKind::FaceA, SeparationIndices::FaceA { index_b }) => {
                let normal = transform_a.rotation().apply(self.axis);
                let point_a = transform_a.apply(self.local_point);
                let point_b = transform_b.apply(self.proxy_b.vertex(index_b)?);
                (point_b - point_a).dot(normal)
            }
            (SeparationKind::FaceB, SeparationIndices::FaceB { index_a }) => {
                let normal = transform_b.rotation().apply(self.axis);
                let point_b = transform_b.apply(self.local_point);
                let point_a = transform_a.apply(self.proxy_a.vertex(index_a)?);
                (point_a - point_b).dot(normal)
            }
            _ => return Err(CollisionError::IncompatibleShapeProxy),
        };
        validate_separation(separation)?;
        Ok(separation)
    }
}

fn transform_at(sweep: Sweep, time: f32) -> Result<Transform, CollisionError> {
    sweep
        .transform_at(time)
        .map_err(|_error| CollisionError::NonFiniteValue)
}

fn validate_derived(axis: Vec2, separation: f32) -> Result<(), CollisionError> {
    if !axis.is_valid() {
        return Err(CollisionError::NonFiniteValue);
    }
    validate_separation(separation)
}

fn validate_separation(separation: f32) -> Result<(), CollisionError> {
    if !separation.is_finite() {
        return Err(CollisionError::NonFiniteValue);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collision::shape::{CircleShape, EdgeShape};

    fn circle(radius: f32) -> Shape {
        CircleShape::new(Vec2::ZERO, radius)
            .expect("circle should be valid")
            .into()
    }

    fn edge() -> Shape {
        EdgeShape::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0))
            .expect("edge should be valid")
            .into()
    }

    fn sweep(center: Vec2) -> Sweep {
        Sweep::new(Vec2::ZERO, center, center, 0.0, 0.0, 0.0).expect("sweep should be valid")
    }

    #[test]
    fn separation_points_uses_one_cached_pair() {
        // Arrange
        let shape_a = circle(1.0);
        let shape_b = circle(1.0);
        let child = shape_a.child_index(0).expect("child should exist");
        let proxy_a = ToiProxy::new(&shape_a, child).expect("proxy should be valid");
        let proxy_b = ToiProxy::new(&shape_b, child).expect("proxy should be valid");
        let pairs = [SupportIndexPair::new(0, 0)];

        // Act
        let (function, separation) = SeparationFunction::initialize(
            &pairs,
            &proxy_a,
            sweep(Vec2::ZERO),
            &proxy_b,
            sweep(Vec2::new(4.0, 0.0)),
            0.0,
        )
        .expect("points separation should initialize");

        // Assert
        assert_eq!(function.kind(), SeparationKind::Points);
        assert_eq!(separation.to_bits(), 4.0_f32.to_bits());
    }

    #[test]
    fn separation_face_a_uses_two_vertices_on_a() {
        // Arrange
        let shape_a = edge();
        let shape_b = circle(1.0);
        let child = shape_a.child_index(0).expect("child should exist");
        let proxy_a = ToiProxy::new(&shape_a, child).expect("proxy should be valid");
        let proxy_b = ToiProxy::new(&shape_b, child).expect("proxy should be valid");
        let pairs = [SupportIndexPair::new(0, 0), SupportIndexPair::new(1, 0)];

        // Act
        let (function, separation) = SeparationFunction::initialize(
            &pairs,
            &proxy_a,
            sweep(Vec2::ZERO),
            &proxy_b,
            sweep(Vec2::new(0.0, 3.0)),
            0.0,
        )
        .expect("face A separation should initialize");

        // Assert
        assert_eq!(function.kind(), SeparationKind::FaceA);
        assert_eq!(separation.to_bits(), 3.0_f32.to_bits());
    }

    #[test]
    fn separation_face_b_uses_two_vertices_on_b() {
        // Arrange
        let shape_a = circle(1.0);
        let shape_b = edge();
        let child = shape_a.child_index(0).expect("child should exist");
        let proxy_a = ToiProxy::new(&shape_a, child).expect("proxy should be valid");
        let proxy_b = ToiProxy::new(&shape_b, child).expect("proxy should be valid");
        let pairs = [SupportIndexPair::new(0, 0), SupportIndexPair::new(0, 1)];

        // Act
        let (function, separation) = SeparationFunction::initialize(
            &pairs,
            &proxy_a,
            sweep(Vec2::new(0.0, 3.0)),
            &proxy_b,
            sweep(Vec2::ZERO),
            0.0,
        )
        .expect("face B separation should initialize");

        // Assert
        assert_eq!(function.kind(), SeparationKind::FaceB);
        assert_eq!(separation.to_bits(), 3.0_f32.to_bits());
    }

    #[test]
    fn separation_support_tie_retains_first_vertex() {
        // Arrange
        let shape = edge();
        let child = shape.child_index(0).expect("child should exist");
        let proxy = ToiProxy::new(&shape, child).expect("proxy should be valid");

        // Act
        let support = proxy.support_index(Vec2::new(0.0, 1.0));

        // Assert
        assert_eq!(support, 0);
    }
}
