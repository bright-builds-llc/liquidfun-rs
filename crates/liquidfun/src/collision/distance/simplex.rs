use crate::math::{Transform, Vec2};

use super::{CacheEntry, DistanceProxy, SupportIndexPair, cache_metric_requires_flush};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct SimplexVertex {
    point_a: Vec2,
    point_b: Vec2,
    difference: Vec2,
    weight: f32,
    index_a: usize,
    index_b: usize,
}

pub(super) struct SavedSupportPairs {
    pairs: [SupportIndexPair; 3],
    count: usize,
}

impl SavedSupportPairs {
    pub(super) fn contains(&self, candidate: SupportIndexPair) -> bool {
        self.pairs[..self.count].contains(&candidate)
    }
}

pub(super) struct Simplex {
    vertices: [SimplexVertex; 3],
    count: usize,
}

impl Simplex {
    pub(super) fn read_cache(
        entries: &[CacheEntry],
        cached_metric: f32,
        proxy_a: &DistanceProxy<'_>,
        transform_a: Transform,
        proxy_b: &DistanceProxy<'_>,
        transform_b: Transform,
    ) -> Self {
        let mut simplex =
            Self::from_cache_entries(entries, proxy_a, transform_a, proxy_b, transform_b);
        if simplex.count > 1 {
            let metric = simplex.metric();
            if cache_metric_requires_flush(cached_metric, metric) {
                simplex.count = 0;
            }
        }
        if simplex.count == 0 {
            let point_a = transform_a.apply(proxy_a.vertex(0));
            let point_b = transform_b.apply(proxy_b.vertex(0));
            simplex.vertices[0] = SimplexVertex {
                point_a,
                point_b,
                difference: point_b - point_a,
                weight: 1.0,
                index_a: 0,
                index_b: 0,
            };
            simplex.count = 1;
        }
        simplex
    }

    #[cfg(feature = "differential-internals")]
    pub(super) fn cache_metric(
        entries: &[CacheEntry],
        proxy_a: &DistanceProxy<'_>,
        transform_a: Transform,
        proxy_b: &DistanceProxy<'_>,
        transform_b: Transform,
    ) -> f32 {
        Self::from_cache_entries(entries, proxy_a, transform_a, proxy_b, transform_b).metric()
    }

    fn from_cache_entries(
        entries: &[CacheEntry],
        proxy_a: &DistanceProxy<'_>,
        transform_a: Transform,
        proxy_b: &DistanceProxy<'_>,
        transform_b: Transform,
    ) -> Self {
        let mut simplex = Self {
            vertices: [SimplexVertex::default(); 3],
            count: entries.len(),
        };
        for (vertex, entry) in simplex.vertices.iter_mut().zip(entries) {
            let local_a = proxy_a.vertex(entry.index_a);
            let local_b = proxy_b.vertex(entry.index_b);
            let point_a = transform_a.apply(local_a);
            let point_b = transform_b.apply(local_b);
            *vertex = SimplexVertex {
                point_a,
                point_b,
                difference: point_b - point_a,
                weight: 0.0,
                index_a: entry.index_a,
                index_b: entry.index_b,
            };
        }
        simplex
    }

    pub(super) const fn count(&self) -> usize {
        self.count
    }

    pub(super) fn solve(&mut self) {
        match self.count {
            1 => {}
            2 => self.solve2(),
            3 => self.solve3(),
            _ => unreachable!("validated simplex count is one through three"),
        }
    }

    pub(super) fn saved_support_pairs(&self) -> SavedSupportPairs {
        let mut pairs = [SupportIndexPair::new(0, 0); 3];
        for (pair, vertex) in pairs.iter_mut().zip(&self.vertices[..self.count]) {
            *pair = SupportIndexPair::new(vertex.index_a, vertex.index_b);
        }
        SavedSupportPairs {
            pairs,
            count: self.count,
        }
    }

    pub(super) fn append_support(
        &mut self,
        proxy_a: &DistanceProxy<'_>,
        transform_a: Transform,
        proxy_b: &DistanceProxy<'_>,
        transform_b: Transform,
        direction: Vec2,
    ) -> SupportIndexPair {
        let index_a = proxy_a.support_index(transform_a.rotation().inverse_apply(-direction));
        let point_a = transform_a.apply(proxy_a.vertex(index_a));
        let index_b = proxy_b.support_index(transform_b.rotation().inverse_apply(direction));
        let point_b = transform_b.apply(proxy_b.vertex(index_b));
        self.vertices[self.count] = SimplexVertex {
            point_a,
            point_b,
            difference: point_b - point_a,
            weight: 0.0,
            index_a,
            index_b,
        };
        SupportIndexPair::new(index_a, index_b)
    }

    pub(super) fn accept_support(&mut self) {
        self.count += 1;
    }

    pub(super) fn closest_point(&self) -> Vec2 {
        match self.count {
            1 => self.vertices[0].difference,
            2 => {
                self.vertices[0].weight * self.vertices[0].difference
                    + self.vertices[1].weight * self.vertices[1].difference
            }
            3 => Vec2::ZERO,
            _ => unreachable!("validated simplex count is one through three"),
        }
    }

    pub(super) fn search_direction(&self) -> Vec2 {
        match self.count {
            1 => -self.vertices[0].difference,
            2 => {
                let edge = self.vertices[1].difference - self.vertices[0].difference;
                let sign = edge.cross(-self.vertices[0].difference);
                if sign > 0.0 {
                    Vec2::scalar_cross(1.0, edge)
                } else {
                    edge.cross_scalar(1.0)
                }
            }
            _ => unreachable!("search direction requires one or two simplex points"),
        }
    }

    pub(super) fn witness_points(&self) -> (Vec2, Vec2) {
        match self.count {
            1 => (self.vertices[0].point_a, self.vertices[0].point_b),
            2 => {
                let point_a = self.vertices[0].weight * self.vertices[0].point_a
                    + self.vertices[1].weight * self.vertices[1].point_a;
                let point_b = self.vertices[0].weight * self.vertices[0].point_b
                    + self.vertices[1].weight * self.vertices[1].point_b;
                (point_a, point_b)
            }
            3 => {
                let point_a = self.vertices[0].weight * self.vertices[0].point_a
                    + self.vertices[1].weight * self.vertices[1].point_a
                    + self.vertices[2].weight * self.vertices[2].point_a;
                (point_a, point_a)
            }
            _ => unreachable!("validated simplex count is one through three"),
        }
    }

    pub(super) fn metric(&self) -> f32 {
        match self.count {
            1 => 0.0,
            2 => (self.vertices[0].difference - self.vertices[1].difference).length(),
            3 => (self.vertices[1].difference - self.vertices[0].difference)
                .cross(self.vertices[2].difference - self.vertices[0].difference),
            _ => unreachable!("validated simplex count is one through three"),
        }
    }

    pub(super) fn support_pairs(&self) -> ([SupportIndexPair; 3], usize) {
        let saved = self.saved_support_pairs();
        (saved.pairs, saved.count)
    }

    fn solve2(&mut self) {
        let first = self.vertices[0].difference;
        let second = self.vertices[1].difference;
        let edge = second - first;
        let second_weight_numerator = -first.dot(edge);
        if second_weight_numerator <= 0.0 {
            self.vertices[0].weight = 1.0;
            self.count = 1;
            return;
        }

        let first_weight_numerator = second.dot(edge);
        if first_weight_numerator <= 0.0 {
            self.vertices[1].weight = 1.0;
            self.count = 1;
            self.vertices[0] = self.vertices[1];
            return;
        }

        let inverse_sum = 1.0 / (first_weight_numerator + second_weight_numerator);
        self.vertices[0].weight = first_weight_numerator * inverse_sum;
        self.vertices[1].weight = second_weight_numerator * inverse_sum;
        self.count = 2;
    }

    fn solve3(&mut self) {
        let first = self.vertices[0].difference;
        let second = self.vertices[1].difference;
        let third = self.vertices[2].difference;

        let edge12 = second - first;
        let first_edge12 = first.dot(edge12);
        let second_edge12 = second.dot(edge12);
        let edge12_first = second_edge12;
        let edge12_second = -first_edge12;

        let edge13 = third - first;
        let first_edge13 = first.dot(edge13);
        let third_edge13 = third.dot(edge13);
        let edge13_first = third_edge13;
        let edge13_third = -first_edge13;

        let edge23 = third - second;
        let second_edge23 = second.dot(edge23);
        let third_edge23 = third.dot(edge23);
        let edge23_second = third_edge23;
        let edge23_third = -second_edge23;

        let triangle_normal = edge12.cross(edge13);
        let triangle_first = triangle_normal * second.cross(third);
        let triangle_second = triangle_normal * third.cross(first);
        let triangle_third = triangle_normal * first.cross(second);

        if edge12_second <= 0.0 && edge13_third <= 0.0 {
            self.vertices[0].weight = 1.0;
            self.count = 1;
            return;
        }

        if edge12_first > 0.0 && edge12_second > 0.0 && triangle_third <= 0.0 {
            let inverse_sum = 1.0 / (edge12_first + edge12_second);
            self.vertices[0].weight = edge12_first * inverse_sum;
            self.vertices[1].weight = edge12_second * inverse_sum;
            self.count = 2;
            return;
        }

        if edge13_first > 0.0 && edge13_third > 0.0 && triangle_second <= 0.0 {
            let inverse_sum = 1.0 / (edge13_first + edge13_third);
            self.vertices[0].weight = edge13_first * inverse_sum;
            self.vertices[2].weight = edge13_third * inverse_sum;
            self.count = 2;
            self.vertices[1] = self.vertices[2];
            return;
        }

        if edge12_first <= 0.0 && edge23_third <= 0.0 {
            self.vertices[1].weight = 1.0;
            self.count = 1;
            self.vertices[0] = self.vertices[1];
            return;
        }

        if edge13_first <= 0.0 && edge23_second <= 0.0 {
            self.vertices[2].weight = 1.0;
            self.count = 1;
            self.vertices[0] = self.vertices[2];
            return;
        }

        if edge23_second > 0.0 && edge23_third > 0.0 && triangle_first <= 0.0 {
            let inverse_sum = 1.0 / (edge23_second + edge23_third);
            self.vertices[1].weight = edge23_second * inverse_sum;
            self.vertices[2].weight = edge23_third * inverse_sum;
            self.count = 2;
            self.vertices[0] = self.vertices[2];
            return;
        }

        let inverse_sum = 1.0 / (triangle_first + triangle_second + triangle_third);
        self.vertices[0].weight = triangle_first * inverse_sum;
        self.vertices[1].weight = triangle_second * inverse_sum;
        self.vertices[2].weight = triangle_third * inverse_sum;
        self.count = 3;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex(difference: Vec2) -> SimplexVertex {
        SimplexVertex {
            point_a: Vec2::ZERO,
            point_b: difference,
            difference,
            weight: 0.0,
            index_a: 0,
            index_b: 0,
        }
    }

    #[test]
    fn solve2_keeps_first_vertex_region() {
        // Arrange
        let mut simplex = Simplex {
            vertices: [
                vertex(Vec2::new(1.0, 0.0)),
                vertex(Vec2::new(2.0, 0.0)),
                SimplexVertex::default(),
            ],
            count: 2,
        };

        // Act
        simplex.solve2();

        // Assert
        assert_eq!(simplex.count, 1);
        assert_eq!(simplex.vertices[0].difference, Vec2::new(1.0, 0.0));
        assert_eq!(simplex.vertices[0].weight.to_bits(), 1.0_f32.to_bits());
    }

    #[test]
    fn one_point_simplex_returns_its_difference() {
        // Arrange
        let simplex = Simplex {
            vertices: [
                vertex(Vec2::new(3.0, -4.0)),
                SimplexVertex::default(),
                SimplexVertex::default(),
            ],
            count: 1,
        };

        // Act
        let closest = simplex.closest_point();

        // Assert
        assert_eq!(closest, Vec2::new(3.0, -4.0));
    }

    #[test]
    fn solve2_computes_edge_barycentric_weights() {
        // Arrange
        let mut simplex = Simplex {
            vertices: [
                vertex(Vec2::new(-1.0, 1.0)),
                vertex(Vec2::new(1.0, 1.0)),
                SimplexVertex::default(),
            ],
            count: 2,
        };

        // Act
        simplex.solve2();

        // Assert
        assert_eq!(simplex.count, 2);
        assert_eq!(simplex.vertices[0].weight.to_bits(), 0.5_f32.to_bits());
        assert_eq!(simplex.vertices[1].weight.to_bits(), 0.5_f32.to_bits());
    }

    #[test]
    fn solve3_contains_origin_in_triangle() {
        // Arrange
        let mut simplex = Simplex {
            vertices: [
                vertex(Vec2::new(-1.0, -1.0)),
                vertex(Vec2::new(1.0, -1.0)),
                vertex(Vec2::new(0.0, 1.0)),
            ],
            count: 3,
        };

        // Act
        simplex.solve3();

        // Assert
        assert_eq!(simplex.count, 3);
        assert_eq!(simplex.closest_point(), Vec2::ZERO);
    }
}
