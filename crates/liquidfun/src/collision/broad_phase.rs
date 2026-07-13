//! Broad-phase move buffering, ordered pair generation, and pure filtering.

use crate::collision::{Aabb, RayCastInput};
use crate::math::Vec2;

use super::tree::{DynamicTree, ProxyId, QueryControl, RayCastControl, TreeError};

/// Pure collision-filter data matching the selected upstream defaults and rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FilterData {
    category_bits: u16,
    mask_bits: u16,
    group_index: i16,
}

impl FilterData {
    /// Creates filter data from the complete upstream bit domains.
    #[must_use]
    pub const fn new(category_bits: u16, mask_bits: u16, group_index: i16) -> Self {
        Self {
            category_bits,
            mask_bits,
            group_index,
        }
    }

    /// Returns this proxy's collision category bits.
    #[must_use]
    pub const fn category_bits(self) -> u16 {
        self.category_bits
    }

    /// Returns the categories accepted by this proxy.
    #[must_use]
    pub const fn mask_bits(self) -> u16 {
        self.mask_bits
    }

    /// Returns the signed collision-group override.
    #[must_use]
    pub const fn group_index(self) -> i16 {
        self.group_index
    }

    /// Applies the exact group override and symmetric mask rule.
    #[must_use]
    pub const fn should_collide(self, other: Self) -> bool {
        if self.group_index == other.group_index && self.group_index != 0 {
            return self.group_index > 0;
        }

        (self.mask_bits & other.category_bits) != 0 && (self.category_bits & other.mask_bits) != 0
    }
}

impl Default for FilterData {
    fn default() -> Self {
        Self::new(0x0001, 0xffff, 0)
    }
}

struct BroadProxy<T> {
    payload: T,
    filter: FilterData,
}

#[derive(Clone, Copy)]
struct CandidatePair {
    first: ProxyId,
    second: ProxyId,
    key: (usize, usize),
}

/// Dynamic-tree broad phase with source-ordered move and pair buffers.
pub struct BroadPhase<T> {
    tree: DynamicTree<BroadProxy<T>>,
    move_buffer: Vec<Option<ProxyId>>,
}

impl<T> BroadPhase<T> {
    /// Creates an empty broad phase.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::TreeKeyExhausted`] if no tree identity remains.
    pub fn new() -> Result<Self, TreeError> {
        Ok(Self {
            tree: DynamicTree::new()?,
            move_buffer: Vec::new(),
        })
    }

    /// Creates and buffers one proxy.
    ///
    /// # Errors
    ///
    /// Returns the dynamic tree's checked AABB error.
    pub fn create_proxy(
        &mut self,
        aabb: Aabb,
        payload: T,
        filter: FilterData,
    ) -> Result<ProxyId, TreeError> {
        let proxy = self
            .tree
            .create_proxy(aabb, BroadProxy { payload, filter })?;
        self.buffer_move(proxy);
        Ok(proxy)
    }

    /// Destroys one proxy after tombstoning every buffered occurrence.
    ///
    /// # Errors
    ///
    /// Returns an explicit foreign or stale/destroyed proxy error.
    pub fn destroy_proxy(&mut self, proxy: ProxyId) -> Result<T, TreeError> {
        self.tree.payload(proxy)?;
        for maybe_buffered in &mut self.move_buffer {
            if *maybe_buffered == Some(proxy) {
                *maybe_buffered = None;
            }
        }
        Ok(self.tree.destroy_proxy(proxy)?.payload)
    }

    /// Moves one proxy and buffers it only when tree reinsertion occurs.
    ///
    /// Returns `true` when the proxy was buffered.
    ///
    /// # Errors
    ///
    /// Returns the dynamic tree's identity or checked AABB error.
    pub fn move_proxy(
        &mut self,
        proxy: ProxyId,
        aabb: Aabb,
        displacement: Vec2,
    ) -> Result<bool, TreeError> {
        let moved = self.tree.move_proxy(proxy, aabb, displacement)?;
        if moved {
            self.buffer_move(proxy);
        }
        Ok(moved)
    }

    /// Appends one live proxy to the move buffer, retaining duplicates.
    ///
    /// # Errors
    ///
    /// Returns an explicit foreign or stale/destroyed proxy error.
    pub fn touch_proxy(&mut self, proxy: ProxyId) -> Result<(), TreeError> {
        self.tree.payload(proxy)?;
        self.buffer_move(proxy);
        Ok(())
    }

    /// Replaces filter data and touches the proxy for reconsideration.
    ///
    /// # Errors
    ///
    /// Returns an explicit foreign or stale/destroyed proxy error.
    pub fn set_filter_data(&mut self, proxy: ProxyId, filter: FilterData) -> Result<(), TreeError> {
        self.tree.payload_mut(proxy)?.filter = filter;
        self.buffer_move(proxy);
        Ok(())
    }

    /// Returns the filter data for one live proxy.
    ///
    /// # Errors
    ///
    /// Returns an explicit foreign or stale/destroyed proxy error.
    pub fn filter_data(&self, proxy: ProxyId) -> Result<FilterData, TreeError> {
        Ok(self.tree.payload(proxy)?.filter)
    }

    /// Applies the pure filter rule to two live proxies.
    ///
    /// # Errors
    ///
    /// Returns an explicit foreign or stale/destroyed proxy error.
    pub fn should_collide(&self, first: ProxyId, second: ProxyId) -> Result<bool, TreeError> {
        let first_filter = self.tree.payload(first)?.filter;
        let second_filter = self.tree.payload(second)?.filter;
        Ok(first_filter.should_collide(second_filter))
    }

    /// Returns a shared consumer payload for one live proxy.
    ///
    /// # Errors
    ///
    /// Returns an explicit foreign or stale/destroyed proxy error.
    pub fn payload(&self, proxy: ProxyId) -> Result<&T, TreeError> {
        Ok(&self.tree.payload(proxy)?.payload)
    }

    /// Returns the embedded tree's fat AABB for a live proxy.
    ///
    /// # Errors
    ///
    /// Returns an explicit foreign or stale/destroyed proxy error.
    pub fn fat_aabb(&self, proxy: ProxyId) -> Result<Aabb, TreeError> {
        self.tree.fat_aabb(proxy)
    }

    /// Visits payloads whose broad-phase bounds overlap `aabb`.
    ///
    /// Private tree identities remain inside the broad phase. Traversal keeps
    /// the dynamic tree's LIFO behavior, but callers must not rely on order.
    pub(crate) fn query_aabb<F>(&self, aabb: Aabb, mut visitor: F)
    where
        F: FnMut(&T) -> QueryControl,
    {
        self.tree
            .query(aabb, |_proxy, entry| visitor(&entry.payload));
    }

    /// Visits payloads whose broad-phase bounds intersect a checked ray.
    ///
    /// Private tree identities remain inside the broad phase. Each visitor
    /// receives the tree's narrowed sub-input for exact shape testing.
    pub(crate) fn ray_cast<F>(&self, input: RayCastInput, mut visitor: F) -> Result<(), TreeError>
    where
        F: FnMut(&T, RayCastInput) -> RayCastControl,
    {
        self.tree.ray_cast(input, |_proxy, entry, sub_input| {
            visitor(&entry.payload, sub_input)
        })
    }

    /// Reports all buffered potential pairs in private node-coordinate order.
    ///
    /// Every live move occurrence performs a fat-AABB query. Candidate pairs
    /// are then sorted by `(min_private_slot, max_private_slot)` and adjacent
    /// duplicates are removed before the borrow-scoped callback runs.
    ///
    /// # Errors
    ///
    /// Returns an explicit tree identity error if buffered state is invalid.
    pub fn update_pairs<F>(&mut self, mut callback: F) -> Result<(), TreeError>
    where
        F: FnMut(ProxyId, &T, ProxyId, &T),
    {
        let move_buffer = std::mem::take(&mut self.move_buffer);
        let mut candidates = Vec::new();

        for maybe_query_proxy in move_buffer {
            let Some(query_proxy) = maybe_query_proxy else {
                continue;
            };
            let query_key = self.tree.pair_order_key(query_proxy)?;
            let fat_aabb = self.tree.fat_aabb(query_proxy)?;
            let mut maybe_error = None;
            self.tree.query(fat_aabb, |candidate_proxy, _entry| {
                if candidate_proxy == query_proxy {
                    return QueryControl::Continue;
                }
                let candidate_key = match self.tree.pair_order_key(candidate_proxy) {
                    Ok(key) => key,
                    Err(error) => {
                        maybe_error = Some(error);
                        return QueryControl::Stop;
                    }
                };
                let (first, second, key) = if candidate_key < query_key {
                    (candidate_proxy, query_proxy, (candidate_key, query_key))
                } else {
                    (query_proxy, candidate_proxy, (query_key, candidate_key))
                };
                candidates.push(CandidatePair { first, second, key });
                QueryControl::Continue
            });
            if let Some(error) = maybe_error {
                return Err(error);
            }
        }

        candidates.sort_by_key(|candidate| candidate.key);
        candidates.dedup_by_key(|candidate| candidate.key);
        for candidate in candidates {
            let first = self.tree.payload(candidate.first)?;
            let second = self.tree.payload(candidate.second)?;
            callback(
                candidate.first,
                &first.payload,
                candidate.second,
                &second.payload,
            );
        }
        Ok(())
    }

    /// Returns the number of live broad-phase proxies.
    #[must_use]
    pub const fn proxy_count(&self) -> usize {
        self.tree.proxy_count()
    }

    /// Returns the embedded tree height.
    #[must_use]
    pub fn tree_height(&self) -> i32 {
        self.tree.height()
    }

    /// Returns the embedded tree's maximum child-height difference.
    #[must_use]
    pub fn tree_max_balance(&self) -> i32 {
        self.tree.max_balance()
    }

    /// Returns the embedded tree's total-to-root perimeter ratio.
    #[must_use]
    pub fn tree_area_ratio(&self) -> f32 {
        self.tree.area_ratio()
    }

    fn buffer_move(&mut self, proxy: ProxyId) {
        self.move_buffer.push(Some(proxy));
    }
}
