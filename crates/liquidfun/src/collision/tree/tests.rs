use super::*;

fn test_aabb(lower_x: f32, upper_x: f32) -> Aabb {
    Aabb::new(Vec2::new(lower_x, -1.0), Vec2::new(upper_x, 1.0))
        .expect("test bounds should be valid")
}

#[test]
fn origin_shift_preserves_tree_topology_and_proxy_identity() {
    // Arrange
    let mut tree = DynamicTree::new().expect("a tree key should remain available");
    let first = tree
        .create_proxy(test_aabb(-4.0, -3.0), "first")
        .expect("finite bounds should create a proxy");
    let removed = tree
        .create_proxy(test_aabb(0.0, 1.0), "removed")
        .expect("finite bounds should create a proxy");
    let second = tree
        .create_proxy(test_aabb(3.0, 4.0), "second")
        .expect("finite bounds should create a proxy");
    assert_eq!(tree.destroy_proxy(removed), Ok("removed"));
    let shift = Vec2::new(11.0, -7.0);
    let first_before = tree.fat_aabb(first).expect("first proxy should be live");
    let second_before = tree.fat_aabb(second).expect("second proxy should be live");
    let root_before = tree.maybe_root;
    let topology_before = tree
        .pool
        .nodes()
        .iter()
        .map(|node| {
            (
                node.generation,
                node.maybe_next,
                node.maybe_parent,
                node.maybe_child1,
                node.maybe_child2,
                node.height,
                node.maybe_payload.is_some(),
            )
        })
        .collect::<Vec<_>>();

    // Act
    tree.shift_origin(shift)
        .expect("finite translated bounds should remain valid");

    // Assert
    let topology_after = tree
        .pool
        .nodes()
        .iter()
        .map(|node| {
            (
                node.generation,
                node.maybe_next,
                node.maybe_parent,
                node.maybe_child1,
                node.maybe_child2,
                node.height,
                node.maybe_payload.is_some(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(tree.maybe_root, root_before);
    assert_eq!(topology_after, topology_before);
    assert_eq!(tree.payload(first), Ok(&"first"));
    assert_eq!(tree.payload(second), Ok(&"second"));
    assert_eq!(
        tree.fat_aabb(first),
        Ok(Aabb::new(
            first_before.lower_bound() - shift,
            first_before.upper_bound() - shift,
        )
        .expect("shifted first bounds should be valid"))
    );
    assert_eq!(
        tree.fat_aabb(second),
        Ok(Aabb::new(
            second_before.lower_bound() - shift,
            second_before.upper_bound() - shift,
        )
        .expect("shifted second bounds should be valid"))
    );
    assert!(tree.validate());
}

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
