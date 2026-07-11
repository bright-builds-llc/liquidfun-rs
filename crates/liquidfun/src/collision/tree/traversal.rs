//! Borrow-scoped AABB and ray traversal.

use crate::collision::{Aabb, RayCastInput};
use crate::math::{Vec2, abs, max, min};

use super::{DynamicTree, ProxyId, TreeError};

/// Continue or stop an AABB query visitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryControl {
    /// Continue visiting candidate leaves.
    Continue,
    /// Stop the query immediately.
    Stop,
}

/// Ignore, terminate, or clip a ray traversal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RayCastControl {
    /// Ignore this candidate and preserve the current ray interval.
    Ignore,
    /// Terminate traversal immediately.
    Terminate,
    /// Clip subsequent traversal to the supplied normalized fraction.
    Clip(f32),
}

impl<T> DynamicTree<T> {
    /// Visits each leaf whose fat AABB overlaps `aabb`.
    ///
    /// Candidate order is intentionally unspecified. The visitor borrows each
    /// payload only for the duration of its call and may stop without allocation.
    ///
    /// # Panics
    ///
    /// Panics only if an internal active leaf has lost its payload.
    pub fn query<F>(&self, aabb: Aabb, mut visitor: F)
    where
        F: FnMut(ProxyId, &T) -> QueryControl,
    {
        let Some(root) = self.maybe_root else {
            return;
        };
        let mut stack = vec![root];

        while let Some(index) = stack.pop() {
            if !self.aabb(index).overlaps(aabb) {
                continue;
            }
            let node = self.pool.node(index);
            if node.is_leaf() {
                let payload = node
                    .maybe_payload
                    .as_ref()
                    .expect("active leaves always own payloads");
                if visitor(self.proxy_id_for_node(index), payload) == QueryControl::Stop {
                    return;
                }
                continue;
            }

            let (child1, child2) = self.children(index);
            stack.push(child1);
            stack.push(child2);
        }
    }

    /// Collects unique overlapping proxy identities in unspecified order.
    #[must_use]
    pub fn query_ids(&self, aabb: Aabb) -> Vec<ProxyId> {
        let mut proxies = Vec::new();
        self.query(aabb, |proxy, _payload| {
            proxies.push(proxy);
            QueryControl::Continue
        });
        proxies
    }

    /// Visits ray candidate leaves with explicit ignore, terminate, and clip controls.
    ///
    /// Candidate order is intentionally unspecified. A clip narrows
    /// later sub-inputs but does not imply that candidates arrive by distance.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::DegenerateRay`] for a zero-length ray,
    /// [`TreeError::AabbOverflow`] for non-finite intermediate geometry, or
    /// [`TreeError::InvalidClipFraction`] when the visitor clips outside the
    /// current inclusive interval.
    ///
    /// # Panics
    ///
    /// Panics only if an internal active leaf has lost its payload.
    pub fn ray_cast<F>(&self, input: RayCastInput, mut visitor: F) -> Result<(), TreeError>
    where
        F: FnMut(ProxyId, &T, RayCastInput) -> RayCastControl,
    {
        let start = input.start();
        let end = input.end();
        let mut direction = end - start;
        let direction_length_squared = direction.length_squared();
        if direction_length_squared == 0.0 {
            return Err(TreeError::DegenerateRay);
        }
        if !direction.is_valid() || !direction_length_squared.is_finite() {
            return Err(TreeError::AabbOverflow);
        }
        direction.normalize();
        let perpendicular = Vec2::scalar_cross(1.0, direction);
        let absolute_perpendicular = Vec2::new(abs(perpendicular.x), abs(perpendicular.y));
        let mut max_fraction = input.max_fraction();
        let mut segment_aabb = ray_segment_aabb(start, end, max_fraction)?;
        let Some(root) = self.maybe_root else {
            return Ok(());
        };
        let mut stack = vec![root];

        while let Some(index) = stack.pop() {
            let node_aabb = self.aabb(index);
            if !node_aabb.overlaps(segment_aabb) {
                continue;
            }

            let center = node_aabb.center();
            let extents = node_aabb.extents();
            let separation =
                abs(perpendicular.dot(start - center)) - absolute_perpendicular.dot(extents);
            if separation > 0.0 {
                continue;
            }

            let node = self.pool.node(index);
            if !node.is_leaf() {
                let (child1, child2) = self.children(index);
                stack.push(child1);
                stack.push(child2);
                continue;
            }

            let sub_input = RayCastInput::new(start, end, max_fraction)
                .map_err(|_error| TreeError::AabbOverflow)?;
            let payload = node
                .maybe_payload
                .as_ref()
                .expect("active leaves always own payloads");
            match visitor(self.proxy_id_for_node(index), payload, sub_input) {
                RayCastControl::Ignore => {}
                RayCastControl::Terminate => return Ok(()),
                RayCastControl::Clip(fraction) => {
                    if !fraction.is_finite() || fraction < 0.0 || fraction > max_fraction {
                        return Err(TreeError::InvalidClipFraction);
                    }
                    max_fraction = fraction;
                    segment_aabb = ray_segment_aabb(start, end, max_fraction)?;
                }
            }
        }

        Ok(())
    }

    /// Collects unique fat-AABB ray candidates in unspecified order.
    ///
    /// # Errors
    ///
    /// Returns the same checked geometry errors as [`Self::ray_cast`].
    pub fn ray_candidate_ids(&self, input: RayCastInput) -> Result<Vec<ProxyId>, TreeError> {
        let mut proxies = Vec::new();
        self.ray_cast(input, |proxy, _payload, _sub_input| {
            proxies.push(proxy);
            RayCastControl::Ignore
        })?;
        Ok(proxies)
    }
}

fn ray_segment_aabb(start: Vec2, end: Vec2, max_fraction: f32) -> Result<Aabb, TreeError> {
    let clipped_end = start + max_fraction * (end - start);
    if !clipped_end.is_valid() {
        return Err(TreeError::AabbOverflow);
    }
    Aabb::new(
        Vec2::new(min(start.x, clipped_end.x), min(start.y, clipped_end.y)),
        Vec2::new(max(start.x, clipped_end.x), max(start.y, clipped_end.y)),
    )
    .map_err(|_error| TreeError::AabbOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_aabb(lower_x: f32, upper_x: f32) -> Aabb {
        Aabb::new(Vec2::new(lower_x, -1.0), Vec2::new(upper_x, 1.0))
            .expect("test bounds should be valid")
    }

    #[test]
    fn diagnostic_traversal_visits_child2_before_child1() {
        // Arrange
        let mut tree = DynamicTree::new().expect("a tree key should remain available");
        tree.create_proxy(test_aabb(0.0, 1.0), "child1")
            .expect("finite bounds should create a proxy");
        tree.create_proxy(test_aabb(2.0, 3.0), "child2")
            .expect("finite bounds should create a proxy");
        let mut visited = Vec::new();

        // Act
        tree.query(test_aabb(-1.0, 4.0), |_proxy, payload| {
            visited.push(*payload);
            QueryControl::Continue
        });

        // Assert
        assert_eq!(visited, ["child2", "child1"]);
    }
}
