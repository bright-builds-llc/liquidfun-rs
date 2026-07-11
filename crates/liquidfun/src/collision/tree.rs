//! Dynamic AABB-tree proxy storage, queries, ray casts, and metrics.
//!
//! The public proxy identity is tree-scoped and generational. Private node
//! coordinates retain the selected upstream allocation and ordering behavior
//! without becoming a consumer contract.

mod pool;
mod traversal;

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::collision::Aabb;
use crate::math::Vec2;
use crate::math::settings::{AABB_EXTENSION, AABB_MULTIPLIER};

use pool::{NodeIndex, NodePool};

pub use traversal::{QueryControl, RayCastControl};

static NEXT_TREE_KEY: AtomicU64 = AtomicU64::new(1);

/// Opaque identity for one leaf in a specific [`DynamicTree`].
///
/// Equality and hashing cover the complete tree, private pool coordinate, and
/// generation. No raw constructor, stable serialization, or ordering contract
/// is exposed.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProxyId {
    tree_key: u64,
    node: NodeIndex,
    generation: u64,
}

impl fmt::Debug for ProxyId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ProxyId(<opaque>)")
    }
}

/// Failure at a dynamic-tree identity or geometry boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TreeError {
    /// The process-unique tree-key space has been exhausted.
    TreeKeyExhausted,
    /// A proxy belongs to another tree.
    WrongTree,
    /// A proxy was destroyed, reused, or never represented a live leaf.
    StaleOrDestroyed,
    /// Fattening, prediction, or shifting overflowed finite AABB coordinates.
    AabbOverflow,
    /// An origin shift was not finite.
    NonFiniteOriginShift,
    /// A ray has no direction.
    DegenerateRay,
    /// A visitor supplied a non-finite or out-of-interval ray clip fraction.
    InvalidClipFraction,
}

impl fmt::Display for TreeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::TreeKeyExhausted => "dynamic-tree identity space is exhausted",
            Self::WrongTree => "proxy belongs to another dynamic tree",
            Self::StaleOrDestroyed => "proxy is stale, destroyed, or not a leaf",
            Self::AabbOverflow => "dynamic-tree AABB arithmetic overflowed",
            Self::NonFiniteOriginShift => "dynamic-tree origin shift must be finite",
            Self::DegenerateRay => "dynamic-tree ray must have a non-zero direction",
            Self::InvalidClipFraction => "ray visitor returned an invalid clip fraction",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for TreeError {}

/// Generic source-ordered dynamic AABB tree.
pub struct DynamicTree<T> {
    tree_key: u64,
    maybe_root: Option<NodeIndex>,
    pool: NodePool<T>,
    proxy_count: usize,
}

impl<T> DynamicTree<T> {
    /// Creates an empty tree with a process-unique identity scope.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::TreeKeyExhausted`] if every tree identity has
    /// already been issued in this process.
    pub fn new() -> Result<Self, TreeError> {
        let tree_key = NEXT_TREE_KEY
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_current| TreeError::TreeKeyExhausted)?;

        Ok(Self {
            tree_key,
            maybe_root: None,
            pool: NodePool::new(),
            proxy_count: 0,
        })
    }

    /// Creates a leaf with the pinned AABB extension and returns its opaque identity.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::AabbOverflow`] if extending the finite bounds
    /// produces a non-finite coordinate.
    pub fn create_proxy(&mut self, aabb: Aabb, payload: T) -> Result<ProxyId, TreeError> {
        let fat_aabb = fatten(aabb)?;
        let allocation = self.pool.allocate();
        self.pool.set_leaf(allocation, fat_aabb, payload);
        self.insert_leaf(allocation.index);
        self.proxy_count += 1;

        Ok(ProxyId {
            tree_key: self.tree_key,
            node: allocation.index,
            generation: allocation.generation,
        })
    }

    /// Destroys a live leaf and returns its payload.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::WrongTree`] for a foreign identity and
    /// [`TreeError::StaleOrDestroyed`] for a non-live generation.
    pub fn destroy_proxy(&mut self, proxy: ProxyId) -> Result<T, TreeError> {
        let leaf = self.validate_proxy(proxy)?;
        self.remove_leaf(leaf);
        let payload = self.pool.take_payload(leaf);
        self.pool.free(leaf);
        self.proxy_count -= 1;
        Ok(payload)
    }

    /// Moves a tight proxy AABB, reinserting only when it escapes its fat AABB.
    ///
    /// Returns `true` when reinsertion occurred.
    ///
    /// # Errors
    ///
    /// Returns an identity error for a non-live proxy or
    /// [`TreeError::AabbOverflow`] when predicted bounds become non-finite.
    pub fn move_proxy(
        &mut self,
        proxy: ProxyId,
        tight_aabb: Aabb,
        displacement: Vec2,
    ) -> Result<bool, TreeError> {
        let leaf = self.validate_proxy(proxy)?;
        if self.aabb(leaf).contains(tight_aabb) {
            return Ok(false);
        }

        let predicted = predicted_fat_aabb(tight_aabb, displacement)?;
        self.remove_leaf(leaf);
        self.pool.node_mut(leaf).maybe_aabb = Some(predicted);
        self.insert_leaf(leaf);
        Ok(true)
    }

    /// Returns a shared payload reference for a live proxy.
    ///
    /// # Errors
    ///
    /// Returns an explicit foreign or stale/destroyed identity error.
    ///
    /// # Panics
    ///
    /// Panics only if an internal validated leaf has lost its payload.
    pub fn payload(&self, proxy: ProxyId) -> Result<&T, TreeError> {
        let leaf = self.validate_proxy(proxy)?;
        Ok(self
            .pool
            .node(leaf)
            .maybe_payload
            .as_ref()
            .expect("validated leaves always own payloads"))
    }

    /// Returns a mutable payload reference for a live proxy.
    ///
    /// # Errors
    ///
    /// Returns an explicit foreign or stale/destroyed identity error.
    ///
    /// # Panics
    ///
    /// Panics only if an internal validated leaf has lost its payload.
    pub fn payload_mut(&mut self, proxy: ProxyId) -> Result<&mut T, TreeError> {
        let leaf = self.validate_proxy(proxy)?;
        Ok(self
            .pool
            .node_mut(leaf)
            .maybe_payload
            .as_mut()
            .expect("validated leaves always own payloads"))
    }

    /// Returns the fat AABB for a live proxy.
    ///
    /// # Errors
    ///
    /// Returns an explicit foreign or stale/destroyed identity error.
    pub fn fat_aabb(&self, proxy: ProxyId) -> Result<Aabb, TreeError> {
        let leaf = self.validate_proxy(proxy)?;
        Ok(self.aabb(leaf))
    }

    /// Returns the number of live leaf proxies.
    #[must_use]
    pub const fn proxy_count(&self) -> usize {
        self.proxy_count
    }

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
        if !new_origin.is_valid() {
            return Err(TreeError::NonFiniteOriginShift);
        }

        let shifted: Result<Vec<_>, _> = self
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
            .collect();

        for (index, aabb) in shifted? {
            self.pool.node_mut(index).maybe_aabb = Some(aabb);
        }
        Ok(())
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

    fn validate_proxy(&self, proxy: ProxyId) -> Result<NodeIndex, TreeError> {
        if proxy.tree_key != self.tree_key {
            return Err(TreeError::WrongTree);
        }
        let Some(node) = self.pool.maybe_node(proxy.node) else {
            return Err(TreeError::StaleOrDestroyed);
        };
        if node.generation != proxy.generation || !node.is_leaf() {
            return Err(TreeError::StaleOrDestroyed);
        }
        Ok(proxy.node)
    }

    pub(crate) fn pair_order_key(&self, proxy: ProxyId) -> Result<usize, TreeError> {
        Ok(self.validate_proxy(proxy)?.0)
    }

    fn proxy_id_for_node(&self, node: NodeIndex) -> ProxyId {
        ProxyId {
            tree_key: self.tree_key,
            node,
            generation: self.pool.node(node).generation,
        }
    }

    fn aabb(&self, index: NodeIndex) -> Aabb {
        self.pool
            .node(index)
            .maybe_aabb
            .expect("active tree nodes always have bounds")
    }

    fn children(&self, index: NodeIndex) -> (NodeIndex, NodeIndex) {
        let node = self.pool.node(index);
        (
            node.maybe_child1
                .expect("branch nodes always have a first child"),
            node.maybe_child2
                .expect("branch nodes always have a second child"),
        )
    }

    fn node_height(&self, index: NodeIndex) -> i32 {
        self.pool.node(index).height
    }

    fn insert_leaf(&mut self, leaf: NodeIndex) {
        let Some(mut index) = self.maybe_root else {
            self.maybe_root = Some(leaf);
            self.pool.node_mut(leaf).maybe_parent = None;
            return;
        };

        let leaf_aabb = self.aabb(leaf);
        while !self.pool.node(index).is_leaf() {
            let (child1, child2) = self.children(index);
            let area = self.aabb(index).perimeter();
            let combined_area = self.aabb(index).combined(leaf_aabb).perimeter();
            let cost = 2.0 * combined_area;
            let inheritance_cost = 2.0 * (combined_area - area);
            let cost1 = self.descend_cost(child1, leaf_aabb, inheritance_cost);
            let cost2 = self.descend_cost(child2, leaf_aabb, inheritance_cost);

            if cost < cost1 && cost < cost2 {
                break;
            }
            index = descend_child(child1, child2, cost1, cost2);
        }

        let sibling = index;
        let maybe_old_parent = self.pool.node(sibling).maybe_parent;
        let allocation = self.pool.allocate();
        let new_parent = allocation.index;
        let height = self.node_height(sibling) + 1;
        self.pool.set_branch(
            allocation,
            maybe_old_parent,
            sibling,
            leaf,
            leaf_aabb.combined(self.aabb(sibling)),
            height,
        );

        if let Some(old_parent) = maybe_old_parent {
            self.replace_child(old_parent, sibling, new_parent);
        } else {
            self.maybe_root = Some(new_parent);
        }
        self.pool.node_mut(sibling).maybe_parent = Some(new_parent);
        self.pool.node_mut(leaf).maybe_parent = Some(new_parent);

        let mut maybe_index = Some(new_parent);
        while let Some(current) = maybe_index {
            let balanced = self.balance(current);
            self.recompute(balanced);
            maybe_index = self.pool.node(balanced).maybe_parent;
        }
    }

    fn descend_cost(&self, child: NodeIndex, leaf_aabb: Aabb, inheritance_cost: f32) -> f32 {
        let combined = leaf_aabb.combined(self.aabb(child));
        if self.pool.node(child).is_leaf() {
            return combined.perimeter() + inheritance_cost;
        }
        combined.perimeter() - self.aabb(child).perimeter() + inheritance_cost
    }

    fn remove_leaf(&mut self, leaf: NodeIndex) {
        if self.maybe_root == Some(leaf) {
            self.maybe_root = None;
            self.pool.node_mut(leaf).maybe_parent = None;
            return;
        }

        let parent = self
            .pool
            .node(leaf)
            .maybe_parent
            .expect("a non-root leaf always has a parent");
        let maybe_grand_parent = self.pool.node(parent).maybe_parent;
        let (child1, child2) = self.children(parent);
        let sibling = if child1 == leaf { child2 } else { child1 };

        if let Some(grand_parent) = maybe_grand_parent {
            self.replace_child(grand_parent, parent, sibling);
            self.pool.node_mut(sibling).maybe_parent = Some(grand_parent);
            self.pool.free(parent);

            let mut maybe_index = Some(grand_parent);
            while let Some(current) = maybe_index {
                let balanced = self.balance(current);
                self.recompute(balanced);
                maybe_index = self.pool.node(balanced).maybe_parent;
            }
        } else {
            self.maybe_root = Some(sibling);
            self.pool.node_mut(sibling).maybe_parent = None;
            self.pool.free(parent);
        }
        self.pool.node_mut(leaf).maybe_parent = None;
    }

    fn replace_child(&mut self, parent: NodeIndex, old: NodeIndex, new: NodeIndex) {
        let parent_node = self.pool.node_mut(parent);
        if parent_node.maybe_child1 == Some(old) {
            parent_node.maybe_child1 = Some(new);
            return;
        }
        debug_assert_eq!(parent_node.maybe_child2, Some(old));
        parent_node.maybe_child2 = Some(new);
    }

    fn recompute(&mut self, index: NodeIndex) {
        if self.pool.node(index).is_leaf() {
            return;
        }
        let (child1, child2) = self.children(index);
        let height = 1 + self.node_height(child1).max(self.node_height(child2));
        let aabb = self.aabb(child1).combined(self.aabb(child2));
        let node = self.pool.node_mut(index);
        node.height = height;
        node.maybe_aabb = Some(aabb);
    }

    fn balance(&mut self, index_a: NodeIndex) -> NodeIndex {
        if self.pool.node(index_a).is_leaf() || self.node_height(index_a) < 2 {
            return index_a;
        }

        let (index_b, index_c) = self.children(index_a);
        let balance = self.node_height(index_c) - self.node_height(index_b);
        if balance > 1 {
            return self.rotate_c_up(index_a, index_b, index_c);
        }
        if balance < -1 {
            return self.rotate_b_up(index_a, index_b, index_c);
        }
        index_a
    }

    fn rotate_c_up(
        &mut self,
        index_a: NodeIndex,
        index_b: NodeIndex,
        index_c: NodeIndex,
    ) -> NodeIndex {
        let (index_f, index_g) = self.children(index_c);
        let maybe_old_parent = self.pool.node(index_a).maybe_parent;

        self.pool.node_mut(index_c).maybe_child1 = Some(index_a);
        self.pool.node_mut(index_c).maybe_parent = maybe_old_parent;
        self.pool.node_mut(index_a).maybe_parent = Some(index_c);
        if let Some(old_parent) = maybe_old_parent {
            self.replace_child(old_parent, index_a, index_c);
        } else {
            self.maybe_root = Some(index_c);
        }

        if first_height_wins(self.node_height(index_f), self.node_height(index_g)) {
            self.pool.node_mut(index_c).maybe_child2 = Some(index_f);
            self.pool.node_mut(index_a).maybe_child2 = Some(index_g);
            self.pool.node_mut(index_g).maybe_parent = Some(index_a);
        } else {
            self.pool.node_mut(index_c).maybe_child2 = Some(index_g);
            self.pool.node_mut(index_a).maybe_child2 = Some(index_f);
            self.pool.node_mut(index_f).maybe_parent = Some(index_a);
        }

        debug_assert_eq!(self.pool.node(index_a).maybe_child1, Some(index_b));
        self.recompute(index_a);
        self.recompute(index_c);
        index_c
    }

    fn rotate_b_up(
        &mut self,
        index_a: NodeIndex,
        index_b: NodeIndex,
        index_c: NodeIndex,
    ) -> NodeIndex {
        let (index_d, index_e) = self.children(index_b);
        let maybe_old_parent = self.pool.node(index_a).maybe_parent;

        self.pool.node_mut(index_b).maybe_child1 = Some(index_a);
        self.pool.node_mut(index_b).maybe_parent = maybe_old_parent;
        self.pool.node_mut(index_a).maybe_parent = Some(index_b);
        if let Some(old_parent) = maybe_old_parent {
            self.replace_child(old_parent, index_a, index_b);
        } else {
            self.maybe_root = Some(index_b);
        }

        if first_height_wins(self.node_height(index_d), self.node_height(index_e)) {
            self.pool.node_mut(index_b).maybe_child2 = Some(index_d);
            self.pool.node_mut(index_a).maybe_child1 = Some(index_e);
            self.pool.node_mut(index_e).maybe_parent = Some(index_a);
        } else {
            self.pool.node_mut(index_b).maybe_child2 = Some(index_e);
            self.pool.node_mut(index_a).maybe_child1 = Some(index_d);
            self.pool.node_mut(index_d).maybe_parent = Some(index_a);
        }

        debug_assert_eq!(self.pool.node(index_a).maybe_child2, Some(index_c));
        self.recompute(index_a);
        self.recompute(index_b);
        index_b
    }
}

fn fatten(aabb: Aabb) -> Result<Aabb, TreeError> {
    let extension = Vec2::new(AABB_EXTENSION, AABB_EXTENSION);
    Aabb::new(
        aabb.lower_bound() - extension,
        aabb.upper_bound() + extension,
    )
    .map_err(|_error| TreeError::AabbOverflow)
}

fn predicted_fat_aabb(aabb: Aabb, displacement: Vec2) -> Result<Aabb, TreeError> {
    if !displacement.is_valid() {
        return Err(TreeError::AabbOverflow);
    }
    let fat = fatten(aabb)?;
    let predicted = AABB_MULTIPLIER * displacement;
    let mut lower = fat.lower_bound();
    let mut upper = fat.upper_bound();

    if predicted.x < 0.0 {
        lower.x += predicted.x;
    } else {
        upper.x += predicted.x;
    }
    if predicted.y < 0.0 {
        lower.y += predicted.y;
    } else {
        upper.y += predicted.y;
    }

    Aabb::new(lower, upper).map_err(|_error| TreeError::AabbOverflow)
}

fn descend_child(child1: NodeIndex, child2: NodeIndex, cost1: f32, cost2: f32) -> NodeIndex {
    if cost1 < cost2 { child1 } else { child2 }
}

fn first_height_wins(height1: i32, height2: i32) -> bool {
    height1 > height2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_insertion_cost_descends_to_child2() {
        // Arrange
        let child1 = NodeIndex(4);
        let child2 = NodeIndex(9);

        // Act
        let selected = descend_child(child1, child2, 3.0, 3.0);

        // Assert
        assert_eq!(selected, child2);
    }

    #[test]
    fn equal_rotation_heights_choose_grandchild2() {
        // Arrange
        let first_height = 2;
        let second_height = 2;

        // Act
        let first_selected = first_height_wins(first_height, second_height);

        // Assert
        assert!(!first_selected);
    }
}
