use crate::collision::Aabb;

const INITIAL_NODE_CAPACITY: usize = 16;
const FREE_HEIGHT: i32 = -1;
const RETIRED_HEIGHT: i32 = -2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) struct NodeIndex(pub(super) usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Allocation {
    pub(super) index: NodeIndex,
    pub(super) generation: u64,
}

pub(super) struct Node<T> {
    pub(super) generation: u64,
    pub(super) maybe_next: Option<NodeIndex>,
    pub(super) maybe_parent: Option<NodeIndex>,
    pub(super) maybe_child1: Option<NodeIndex>,
    pub(super) maybe_child2: Option<NodeIndex>,
    pub(super) maybe_aabb: Option<Aabb>,
    pub(super) maybe_payload: Option<T>,
    pub(super) height: i32,
}

impl<T> Node<T> {
    fn free(generation: u64, maybe_next: Option<NodeIndex>) -> Self {
        Self {
            generation,
            maybe_next,
            maybe_parent: None,
            maybe_child1: None,
            maybe_child2: None,
            maybe_aabb: None,
            maybe_payload: None,
            height: FREE_HEIGHT,
        }
    }

    pub(super) fn is_active(&self) -> bool {
        self.height >= 0
    }

    pub(super) fn is_leaf(&self) -> bool {
        self.height == 0 && self.maybe_payload.is_some()
    }
}

pub(super) struct NodePool<T> {
    nodes: Vec<Node<T>>,
    maybe_free_head: Option<NodeIndex>,
    active_count: usize,
    retired_count: usize,
}

impl<T> NodePool<T> {
    pub(super) fn new() -> Self {
        let mut nodes = Vec::with_capacity(INITIAL_NODE_CAPACITY);
        for index in 0..INITIAL_NODE_CAPACITY {
            let maybe_next = (index + 1 < INITIAL_NODE_CAPACITY).then_some(NodeIndex(index + 1));
            nodes.push(Node::free(0, maybe_next));
        }

        Self {
            nodes,
            maybe_free_head: Some(NodeIndex(0)),
            active_count: 0,
            retired_count: 0,
        }
    }

    pub(super) fn allocate(&mut self) -> Allocation {
        if self.maybe_free_head.is_none() {
            self.grow();
        }

        let index = self
            .maybe_free_head
            .expect("growing the node pool always creates a free node");
        let maybe_next = self.node(index).maybe_next;
        self.maybe_free_head = maybe_next;
        let generation = {
            let node = self.node_mut(index);
            node.maybe_next = None;
            node.maybe_parent = None;
            node.maybe_child1 = None;
            node.maybe_child2 = None;
            node.maybe_aabb = None;
            node.maybe_payload = None;
            node.height = 0;
            node.generation
        };
        self.active_count += 1;

        Allocation { index, generation }
    }

    fn grow(&mut self) {
        let old_capacity = self.nodes.len();
        let new_capacity = old_capacity
            .checked_mul(2)
            .expect("the process cannot address a doubled node pool");
        self.nodes.reserve_exact(old_capacity);
        for index in old_capacity..new_capacity {
            let maybe_next = (index + 1 < new_capacity).then_some(NodeIndex(index + 1));
            self.nodes.push(Node::free(0, maybe_next));
        }
        self.maybe_free_head = Some(NodeIndex(old_capacity));
    }

    pub(super) fn free(&mut self, index: NodeIndex) {
        let maybe_next_generation = self.node(index).generation.checked_add(1);
        self.active_count -= 1;

        let Some(next_generation) = maybe_next_generation else {
            let node = self.node_mut(index);
            node.maybe_next = None;
            node.maybe_parent = None;
            node.maybe_child1 = None;
            node.maybe_child2 = None;
            node.maybe_aabb = None;
            node.maybe_payload = None;
            node.height = RETIRED_HEIGHT;
            self.retired_count += 1;
            return;
        };

        let previous_head = self.maybe_free_head;
        *self.node_mut(index) = Node::free(next_generation, previous_head);
        self.maybe_free_head = Some(index);
    }

    pub(super) fn set_leaf(&mut self, allocation: Allocation, aabb: Aabb, payload: T) {
        let node = self.node_mut(allocation.index);
        node.maybe_aabb = Some(aabb);
        node.maybe_payload = Some(payload);
        node.height = 0;
    }

    pub(super) fn set_branch(
        &mut self,
        allocation: Allocation,
        maybe_parent: Option<NodeIndex>,
        child1: NodeIndex,
        child2: NodeIndex,
        aabb: Aabb,
        height: i32,
    ) {
        let node = self.node_mut(allocation.index);
        node.maybe_parent = maybe_parent;
        node.maybe_child1 = Some(child1);
        node.maybe_child2 = Some(child2);
        node.maybe_aabb = Some(aabb);
        node.maybe_payload = None;
        node.height = height;
    }

    pub(super) fn take_payload(&mut self, index: NodeIndex) -> T {
        self.node_mut(index)
            .maybe_payload
            .take()
            .expect("a validated leaf always owns a payload")
    }

    pub(super) fn node(&self, index: NodeIndex) -> &Node<T> {
        self.nodes
            .get(index.0)
            .expect("tree node coordinates always refer to the node pool")
    }

    pub(super) fn node_mut(&mut self, index: NodeIndex) -> &mut Node<T> {
        self.nodes
            .get_mut(index.0)
            .expect("tree node coordinates always refer to the node pool")
    }

    pub(super) fn maybe_node(&self, index: NodeIndex) -> Option<&Node<T>> {
        self.nodes.get(index.0)
    }

    pub(super) fn nodes(&self) -> &[Node<T>] {
        &self.nodes
    }

    pub(super) fn active_count(&self) -> usize {
        self.active_count
    }

    pub(super) fn validate_free_list(&self) -> bool {
        let mut free_count = 0;
        let mut maybe_index = self.maybe_free_head;
        while let Some(index) = maybe_index {
            let node = self.node(index);
            if node.height != FREE_HEIGHT {
                return false;
            }
            free_count += 1;
            if free_count > self.nodes.len() {
                return false;
            }
            maybe_index = node.maybe_next;
        }

        self.active_count + free_count + self.retired_count == self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freed_nodes_are_reused_in_lifo_order() {
        // Arrange
        let mut pool = NodePool::<()>::new();
        let first = pool.allocate();
        let second = pool.allocate();
        pool.free(first.index);
        pool.free(second.index);

        // Act
        let replacement = pool.allocate();

        // Assert
        assert_eq!(replacement.index, second.index);
        assert_eq!(replacement.generation, second.generation + 1);
    }

    #[test]
    fn empty_free_list_doubles_the_initial_capacity() {
        // Arrange
        let mut pool = NodePool::<()>::new();
        let initial: Vec<_> = (0..INITIAL_NODE_CAPACITY)
            .map(|_| pool.allocate())
            .collect();

        // Act
        let grown = pool.allocate();

        // Assert
        assert_eq!(initial[0].index, NodeIndex(0));
        assert_eq!(grown.index, NodeIndex(INITIAL_NODE_CAPACITY));
        assert_eq!(pool.nodes.len(), INITIAL_NODE_CAPACITY * 2);
    }

    #[test]
    fn maximum_generation_retires_instead_of_wrapping() {
        // Arrange
        let mut pool = NodePool::<()>::new();
        let allocation = pool.allocate();
        pool.node_mut(allocation.index).generation = u64::MAX;

        // Act
        pool.free(allocation.index);

        // Assert
        assert_eq!(pool.node(allocation.index).height, RETIRED_HEIGHT);
        assert!(pool.validate_free_list());
    }
}
