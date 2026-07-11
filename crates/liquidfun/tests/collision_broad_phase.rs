//! Public dynamic-tree and broad-phase compatibility tests.

use liquidfun::collision::Aabb;
use liquidfun::collision::tree::{DynamicTree, TreeError};
use liquidfun::math::Vec2;

fn aabb(lower_x: f32, lower_y: f32, upper_x: f32, upper_y: f32) -> Aabb {
    Aabb::new(Vec2::new(lower_x, lower_y), Vec2::new(upper_x, upper_y))
        .expect("test bounds should be valid")
}

#[test]
fn dynamic_tree_rejects_foreign_and_destroyed_proxy_ids() {
    // Arrange
    let mut first = DynamicTree::new().expect("a tree key should remain available");
    let second = DynamicTree::<u32>::new().expect("a second tree key should remain available");
    let proxy = first
        .create_proxy(aabb(0.0, 0.0, 1.0, 1.0), 7_u32)
        .expect("finite bounds should create a proxy");

    // Act
    let destroyed = first
        .destroy_proxy(proxy)
        .expect("a live proxy should be destroyed");

    // Assert
    assert_eq!(destroyed, 7);
    assert_eq!(first.payload(proxy), Err(TreeError::StaleOrDestroyed));
    assert_eq!(second.payload(proxy), Err(TreeError::WrongTree));
}

#[test]
fn dynamic_tree_reuse_never_resurrects_a_destroyed_proxy() {
    // Arrange
    let mut tree = DynamicTree::new().expect("a tree key should remain available");
    let stale = tree
        .create_proxy(aabb(0.0, 0.0, 1.0, 1.0), "first")
        .expect("finite bounds should create a proxy");
    tree.destroy_proxy(stale)
        .expect("a live proxy should be destroyed");

    // Act
    let replacement = tree
        .create_proxy(aabb(2.0, 2.0, 3.0, 3.0), "second")
        .expect("the freed pool coordinate should be reusable");

    // Assert
    assert_ne!(stale, replacement);
    assert_eq!(tree.payload(stale), Err(TreeError::StaleOrDestroyed));
    assert_eq!(tree.payload(replacement), Ok(&"second"));
}

#[test]
fn dynamic_tree_creation_uses_the_pinned_fat_extension() {
    // Arrange
    let mut tree = DynamicTree::new().expect("a tree key should remain available");

    // Act
    let proxy = tree
        .create_proxy(aabb(1.0, 2.0, 3.0, 5.0), ())
        .expect("finite bounds should create a proxy");
    let fat = tree
        .fat_aabb(proxy)
        .expect("a live proxy should have bounds");

    // Assert
    assert_eq!(fat.lower_bound().x.to_bits(), 0.9_f32.to_bits());
    assert_eq!(fat.lower_bound().y.to_bits(), 1.9_f32.to_bits());
    assert_eq!(fat.upper_bound().x.to_bits(), 3.1_f32.to_bits());
    assert_eq!(fat.upper_bound().y.to_bits(), 5.1_f32.to_bits());
}

#[test]
fn dynamic_tree_contained_movement_is_a_no_op() {
    // Arrange
    let mut tree = DynamicTree::new().expect("a tree key should remain available");
    let proxy = tree
        .create_proxy(aabb(0.0, 0.0, 1.0, 1.0), ())
        .expect("finite bounds should create a proxy");
    let before = tree
        .fat_aabb(proxy)
        .expect("a live proxy should have bounds");

    // Act
    let reinserted = tree
        .move_proxy(proxy, aabb(0.05, 0.05, 0.95, 0.95), Vec2::ZERO)
        .expect("a live proxy should move");

    // Assert
    assert!(!reinserted);
    assert_eq!(tree.fat_aabb(proxy), Ok(before));
}

#[test]
fn dynamic_tree_rebuilds_moved_fat_bounds_with_signed_displacement() {
    // Arrange
    let mut tree = DynamicTree::new().expect("a tree key should remain available");
    let proxy = tree
        .create_proxy(aabb(0.0, 0.0, 1.0, 1.0), ())
        .expect("finite bounds should create a proxy");

    // Act
    let reinserted = tree
        .move_proxy(proxy, aabb(3.0, 4.0, 4.0, 5.0), Vec2::new(-0.5, 0.25))
        .expect("a live proxy should move");
    let fat = tree
        .fat_aabb(proxy)
        .expect("a live proxy should have bounds");

    // Assert
    assert!(reinserted);
    assert_eq!(
        fat.lower_bound().x.to_bits(),
        (3.0_f32 - 0.1 - 1.0).to_bits()
    );
    assert_eq!(fat.lower_bound().y.to_bits(), (4.0_f32 - 0.1).to_bits());
    assert_eq!(fat.upper_bound().x.to_bits(), (4.0_f32 + 0.1).to_bits());
    assert_eq!(
        fat.upper_bound().y.to_bits(),
        (5.0_f32 + 0.1 + 0.5).to_bits()
    );
}

#[test]
fn dynamic_tree_payload_mutation_never_exposes_pool_coordinates() {
    // Arrange
    let mut tree = DynamicTree::new().expect("a tree key should remain available");
    let proxy = tree
        .create_proxy(aabb(0.0, 0.0, 1.0, 1.0), String::from("before"))
        .expect("finite bounds should create a proxy");

    // Act
    tree.payload_mut(proxy)
        .expect("a live proxy should expose its payload")
        .push_str("-after");

    // Assert
    assert_eq!(tree.payload(proxy).map(String::as_str), Ok("before-after"));
}

#[test]
fn dynamic_tree_metrics_and_validation_survive_rebalancing_and_origin_shift() {
    // Arrange
    let mut tree = DynamicTree::new().expect("a tree key should remain available");
    let proxies: Vec<_> = (0_u16..32)
        .map(|index| {
            let x = f32::from(index);
            tree.create_proxy(aabb(x, 0.0, x + 0.5, 0.5), index)
                .expect("finite bounds should create a proxy")
        })
        .collect();

    // Act
    for proxy in proxies.iter().step_by(3).copied() {
        tree.destroy_proxy(proxy)
            .expect("selected live proxies should be destroyed");
    }
    tree.shift_origin(Vec2::new(10.0, -4.0))
        .expect("a finite origin shift should succeed");

    // Assert
    assert!(tree.validate());
    assert!(tree.height() > 0);
    assert!(tree.max_balance() <= 1);
    assert!(tree.area_ratio() >= 1.0);
    assert_eq!(tree.proxy_count(), 21);
}
