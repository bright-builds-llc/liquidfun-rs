use crate::collision::shape::Shape;
use crate::collision::{ChildIndex, CollisionError};
use crate::math::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProxyKind {
    Circle,
    Edge,
    Polygon,
    Chain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProxyIdentity {
    pub(super) kind: ProxyKind,
    pub(super) child_index: usize,
    pub(super) radius_bits: u32,
    pub(super) vertex_bits: Vec<(u32, u32)>,
}

enum ProxyVertices<'a> {
    Borrowed(&'a [Vec2]),
    Inline { values: [Vec2; 2], count: usize },
}

pub(super) struct DistanceProxy<'a> {
    vertices: ProxyVertices<'a>,
    radius: f32,
    kind: ProxyKind,
    child_index: usize,
}

impl<'a> DistanceProxy<'a> {
    pub(super) fn new(shape: &'a Shape, child_index: ChildIndex) -> Result<Self, CollisionError> {
        ChildIndex::new(child_index.get(), shape.child_count())?;
        let child_coordinate = child_index.get();
        match shape {
            Shape::Circle(circle) => Ok(Self::inline(
                [circle.center(), Vec2::ZERO],
                1,
                circle.radius(),
                ProxyKind::Circle,
                child_coordinate,
            )),
            Shape::Edge(edge) => Ok(Self::inline(
                [edge.start(), edge.end()],
                2,
                edge.radius(),
                ProxyKind::Edge,
                child_coordinate,
            )),
            Shape::Polygon(polygon) => Ok(Self {
                vertices: ProxyVertices::Borrowed(polygon.vertices()),
                radius: polygon.radius(),
                kind: ProxyKind::Polygon,
                child_index: child_coordinate,
            }),
            Shape::Chain(chain) => {
                let edge = chain.child_edge(child_index)?;
                Ok(Self::inline(
                    [edge.start(), edge.end()],
                    2,
                    chain.radius(),
                    ProxyKind::Chain,
                    child_coordinate,
                ))
            }
        }
    }

    fn inline(
        values: [Vec2; 2],
        count: usize,
        radius: f32,
        kind: ProxyKind,
        child_index: usize,
    ) -> Self {
        Self {
            vertices: ProxyVertices::Inline { values, count },
            radius,
            kind,
            child_index,
        }
    }

    pub(super) fn vertex_count(&self) -> usize {
        self.vertices().len()
    }

    pub(super) fn vertex(&self, index: usize) -> Vec2 {
        self.vertices()[index]
    }

    pub(super) fn radius(&self) -> f32 {
        self.radius
    }

    pub(super) fn support_index(&self, direction: Vec2) -> usize {
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

    pub(super) fn identity(&self) -> ProxyIdentity {
        ProxyIdentity {
            kind: self.kind,
            child_index: self.child_index,
            radius_bits: self.radius.to_bits(),
            vertex_bits: self
                .vertices()
                .iter()
                .map(|vertex| (vertex.x.to_bits(), vertex.y.to_bits()))
                .collect(),
        }
    }

    fn vertices(&self) -> &[Vec2] {
        match &self.vertices {
            ProxyVertices::Borrowed(vertices) => vertices,
            ProxyVertices::Inline { values, count } => &values[..*count],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collision::shape::{ChainShape, CircleShape, EdgeShape, PolygonShape};

    #[test]
    fn proxy_maps_every_concrete_shape_kind() {
        // Arrange
        let child = ChildIndex::new(0, 1).expect("child should exist");
        let circle: Shape = CircleShape::new(Vec2::new(2.0, 3.0), 4.0)
            .expect("circle should be valid")
            .into();
        let edge: Shape = EdgeShape::new(Vec2::ZERO, Vec2::new(2.0, 0.0))
            .expect("edge should be valid")
            .into();
        let polygon: Shape = PolygonShape::box_shape(1.0, 2.0)
            .expect("polygon should be valid")
            .into();
        let chain: Shape = ChainShape::open(&[Vec2::ZERO, Vec2::new(1.0, 0.0)], None, None)
            .expect("chain should be valid")
            .into();

        // Act
        let proxies = [&circle, &edge, &polygon, &chain]
            .map(|shape| DistanceProxy::new(shape, child).expect("proxy should be valid"));

        // Assert
        assert_eq!(proxies[0].vertex_count(), 1);
        assert_eq!(proxies[0].vertex(0), Vec2::new(2.0, 3.0));
        assert_eq!(proxies[1].vertex_count(), 2);
        assert_eq!(proxies[2].vertex_count(), 4);
        assert_eq!(proxies[3].vertex_count(), 2);
    }

    #[test]
    fn proxy_support_tie_retains_first_vertex() {
        // Arrange
        let edge: Shape = EdgeShape::new(Vec2::new(-1.0, 0.0), Vec2::new(1.0, 0.0))
            .expect("edge should be valid")
            .into();
        let child = ChildIndex::new(0, 1).expect("child should exist");
        let proxy = DistanceProxy::new(&edge, child).expect("proxy should be valid");

        // Act
        let support = proxy.support_index(Vec2::new(0.0, 1.0));

        // Assert
        assert_eq!(support, 0);
    }

    #[test]
    fn proxy_selects_each_checked_chain_child() {
        // Arrange
        let chain: Shape = ChainShape::closed(&[
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(1.0, 1.0),
        ])
        .expect("chain should be valid")
        .into();

        // Act
        let proxies: Vec<DistanceProxy<'_>> = (0..chain.child_count())
            .map(|index| {
                let child = chain.child_index(index).expect("child should exist");
                DistanceProxy::new(&chain, child).expect("proxy should be valid")
            })
            .collect();

        // Assert
        assert_eq!(proxies[0].vertex(0), Vec2::new(0.0, 0.0));
        assert_eq!(proxies[0].vertex(1), Vec2::new(2.0, 0.0));
        assert_eq!(proxies[1].vertex(0), Vec2::new(2.0, 0.0));
        assert_eq!(proxies[1].vertex(1), Vec2::new(1.0, 1.0));
        assert_eq!(proxies[2].vertex(0), Vec2::new(1.0, 1.0));
        assert_eq!(proxies[2].vertex(1), Vec2::new(0.0, 0.0));
    }

    #[test]
    fn proxy_rejects_child_from_larger_topology() {
        // Arrange
        let edge: Shape = EdgeShape::new(Vec2::ZERO, Vec2::new(1.0, 0.0))
            .expect("edge should be valid")
            .into();
        let foreign_child = ChildIndex::new(1, 2).expect("foreign child should exist");

        // Act
        let result = DistanceProxy::new(&edge, foreign_child);

        // Assert
        assert!(matches!(
            result,
            Err(CollisionError::ChildIndexOutOfRange {
                requested: 1,
                child_count: 1,
            })
        ));
    }
}
