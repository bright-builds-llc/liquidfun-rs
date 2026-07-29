use super::{Aabb, DynamicTree, NodeIndex, PreparedOriginShift, TreeError, Vec2};

impl<T> DynamicTree<T> {
    /// Returns the root height, or zero for an empty tree.
    #[must_use]
    pub fn height(&self) -> i32 {
        self.maybe_root.map_or(0, |root| self.node_height(root))
    }

    /// Returns the largest absolute child-height difference.
    ///
    /// # Panics
    ///
    /// Panics only if an internal active branch has lost a child.
    #[must_use]
    pub fn max_balance(&self) -> i32 {
        self.pool
            .nodes()
            .iter()
            .filter(|node| node.height > 1)
            .map(|node| {
                let child1 = node
                    .maybe_child1
                    .expect("active branches always have a first child");
                let child2 = node
                    .maybe_child2
                    .expect("active branches always have a second child");
                (self.node_height(child2) - self.node_height(child1)).abs()
            })
            .max()
            .unwrap_or(0)
    }

    /// Returns total active-node perimeter divided by root perimeter.
    ///
    /// # Panics
    ///
    /// Panics only if an internal active node has lost its bounds.
    #[must_use]
    pub fn area_ratio(&self) -> f32 {
        let Some(root) = self.maybe_root else {
            return 0.0;
        };
        let root_area = self.aabb(root).perimeter();
        let total_area: f32 = self
            .pool
            .nodes()
            .iter()
            .filter(|node| node.is_active())
            .map(|node| {
                node.maybe_aabb
                    .expect("active nodes always have bounds")
                    .perimeter()
            })
            .sum();
        total_area / root_area
    }

    /// Shifts every active node by subtracting `new_origin`.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::NonFiniteOriginShift`] for a non-finite input and
    /// [`TreeError::AabbOverflow`] if the shifted bounds are non-finite.
    ///
    /// # Panics
    ///
    /// Panics only if an internal active node has lost its bounds.
    pub fn shift_origin(&mut self, new_origin: Vec2) -> Result<(), TreeError> {
        let prepared = self.prepare_origin_shift(new_origin)?;
        self.commit_origin_shift(prepared);
        Ok(())
    }

    pub(in crate::collision) fn prepare_origin_shift(
        &self,
        new_origin: Vec2,
    ) -> Result<PreparedOriginShift, TreeError> {
        if !new_origin.is_valid() {
            return Err(TreeError::NonFiniteOriginShift);
        }

        let shifted_aabbs = self
            .pool
            .nodes()
            .iter()
            .enumerate()
            .filter(|(_index, node)| node.is_active())
            .map(|(index, node)| {
                let aabb = node.maybe_aabb.expect("active nodes always have bounds");
                let shifted = Aabb::new(
                    aabb.lower_bound() - new_origin,
                    aabb.upper_bound() - new_origin,
                )
                .map_err(|_error| TreeError::AabbOverflow)?;
                Ok((NodeIndex(index), shifted))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(PreparedOriginShift { shifted_aabbs })
    }

    pub(in crate::collision) fn commit_origin_shift(&mut self, prepared: PreparedOriginShift) {
        for (index, aabb) in prepared.shifted_aabbs {
            self.pool.node_mut(index).maybe_aabb = Some(aabb);
        }
    }

    /// Checks tree topology, metrics, reachability, and pool accounting.
    #[must_use]
    pub fn validate(&self) -> bool {
        if !self.pool.validate_free_list() {
            return false;
        }
        let Some(root) = self.maybe_root else {
            return self.pool.active_count() == 0 && self.proxy_count == 0;
        };
        if self.pool.node(root).maybe_parent.is_some() {
            return false;
        }

        let mut visited = vec![false; self.pool.nodes().len()];
        let Some((height, _aabb, active_count, leaf_count)) =
            self.validate_subtree(root, None, &mut visited)
        else {
            return false;
        };
        height == self.height()
            && active_count == self.pool.active_count()
            && leaf_count == self.proxy_count
    }

    fn validate_subtree(
        &self,
        index: NodeIndex,
        maybe_parent: Option<NodeIndex>,
        visited: &mut [bool],
    ) -> Option<(i32, Aabb, usize, usize)> {
        let node = self.pool.maybe_node(index)?;
        let marker = visited.get_mut(index.0)?;
        if *marker || !node.is_active() || node.maybe_parent != maybe_parent {
            return None;
        }
        *marker = true;
        let aabb = node.maybe_aabb?;

        if node.is_leaf() {
            if node.maybe_child1.is_some() || node.maybe_child2.is_some() {
                return None;
            }
            return Some((0, aabb, 1, 1));
        }

        let child1 = node.maybe_child1?;
        let child2 = node.maybe_child2?;
        let (height1, aabb1, count1, leaves1) =
            self.validate_subtree(child1, Some(index), visited)?;
        let (height2, aabb2, count2, leaves2) =
            self.validate_subtree(child2, Some(index), visited)?;
        let expected_height = 1 + height1.max(height2);
        let expected_aabb = aabb1.combined(aabb2);
        if node.height != expected_height || aabb != expected_aabb {
            return None;
        }

        Some((
            expected_height,
            expected_aabb,
            count1 + count2 + 1,
            leaves1 + leaves2,
        ))
    }
}
