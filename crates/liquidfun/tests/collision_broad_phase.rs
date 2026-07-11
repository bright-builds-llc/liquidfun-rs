//! Public dynamic-tree and broad-phase compatibility tests.

use liquidfun::collision::Aabb;
use liquidfun::collision::RayCastInput;
use liquidfun::collision::broad_phase::{BroadPhase, FilterData};
use liquidfun::collision::tree::{DynamicTree, QueryControl, RayCastControl, TreeError};
use liquidfun::math::Vec2;
use std::collections::HashSet;

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

#[test]
fn dynamic_tree_query_visitor_can_stop_without_allocating() {
    // Arrange
    let mut tree = DynamicTree::new().expect("a tree key should remain available");
    for value in 0_u8..4 {
        tree.create_proxy(aabb(0.0, 0.0, 1.0, 1.0), value)
            .expect("finite bounds should create a proxy");
    }
    let mut visited = 0;

    // Act
    tree.query(aabb(-1.0, -1.0, 2.0, 2.0), |_proxy, _payload| {
        visited += 1;
        QueryControl::Stop
    });

    // Assert
    assert_eq!(visited, 1);
}

#[test]
fn dynamic_tree_query_collection_is_a_unique_unspecified_order_set() {
    // Arrange
    let mut tree = DynamicTree::new().expect("a tree key should remain available");
    let first = tree
        .create_proxy(aabb(0.0, 0.0, 1.0, 1.0), 1_u8)
        .expect("finite bounds should create a proxy");
    let second = tree
        .create_proxy(aabb(0.5, 0.5, 1.5, 1.5), 2_u8)
        .expect("finite bounds should create a proxy");
    let outside = tree
        .create_proxy(aabb(5.0, 5.0, 6.0, 6.0), 3_u8)
        .expect("finite bounds should create a proxy");
    tree.move_proxy(outside, aabb(6.0, 6.0, 7.0, 7.0), Vec2::new(1.0, 1.0))
        .expect("a live proxy should move");

    // Act
    let actual: HashSet<_> = tree
        .query_ids(aabb(-0.2, -0.2, 2.0, 2.0))
        .into_iter()
        .collect();

    // Assert
    assert_eq!(actual, HashSet::from([first, second]));
}

#[test]
fn dynamic_tree_ray_visitor_can_ignore_and_terminate() {
    // Arrange
    let mut tree = DynamicTree::new().expect("a tree key should remain available");
    tree.create_proxy(aabb(0.0, -0.5, 1.0, 0.5), "first")
        .expect("finite bounds should create a proxy");
    tree.create_proxy(aabb(2.0, -0.5, 3.0, 0.5), "second")
        .expect("finite bounds should create a proxy");
    let input = RayCastInput::new(Vec2::new(-1.0, 0.0), Vec2::new(4.0, 0.0), 1.0)
        .expect("test ray should be valid");
    let mut visits = 0;

    // Act
    tree.ray_cast(input, |_proxy, _payload, _sub_input| {
        visits += 1;
        if visits == 1 {
            RayCastControl::Ignore
        } else {
            RayCastControl::Terminate
        }
    })
    .expect("a valid visitor should complete");

    // Assert
    assert_eq!(visits, 2);
}

#[test]
fn dynamic_tree_ray_clip_narrows_subsequent_inputs() {
    // Arrange
    let mut tree = DynamicTree::new().expect("a tree key should remain available");
    for value in 0_u8..4 {
        let x = f32::from(value);
        tree.create_proxy(aabb(x, -0.25, x + 0.5, 0.25), value)
            .expect("finite bounds should create a proxy");
    }
    let input = RayCastInput::new(Vec2::new(-1.0, 0.0), Vec2::new(5.0, 0.0), 1.0)
        .expect("test ray should be valid");
    let mut fractions = Vec::new();

    // Act
    tree.ray_cast(input, |_proxy, _payload, sub_input| {
        fractions.push(sub_input.max_fraction());
        if fractions.len() == 1 {
            RayCastControl::Clip(0.5)
        } else {
            RayCastControl::Ignore
        }
    })
    .expect("a valid clip should complete");

    // Assert
    assert_eq!(fractions.first().copied(), Some(1.0));
    assert!(
        fractions
            .iter()
            .skip(1)
            .all(|fraction| fraction.to_bits() == 0.5_f32.to_bits())
    );
}

#[test]
fn dynamic_tree_invalid_ray_clip_preserves_tree_state() {
    // Arrange
    let mut tree = DynamicTree::new().expect("a tree key should remain available");
    let proxy = tree
        .create_proxy(aabb(0.0, -0.5, 1.0, 0.5), 7_u8)
        .expect("finite bounds should create a proxy");
    let input = RayCastInput::new(Vec2::new(-1.0, 0.0), Vec2::new(2.0, 0.0), 0.75)
        .expect("test ray should be valid");

    // Act
    let result = tree.ray_cast(input, |_proxy, _payload, _sub_input| {
        RayCastControl::Clip(f32::NAN)
    });

    // Assert
    assert_eq!(result, Err(TreeError::InvalidClipFraction));
    assert!(tree.validate());
    assert_eq!(tree.payload(proxy), Ok(&7));
}

#[test]
fn dynamic_tree_out_of_interval_ray_clip_is_rejected() {
    // Arrange
    let mut tree = DynamicTree::new().expect("a tree key should remain available");
    tree.create_proxy(aabb(0.0, -0.5, 1.0, 0.5), 7_u8)
        .expect("finite bounds should create a proxy");
    let input = RayCastInput::new(Vec2::new(-1.0, 0.0), Vec2::new(2.0, 0.0), 0.5)
        .expect("test ray should be valid");

    // Act
    let result = tree.ray_cast(input, |_proxy, _payload, _sub_input| {
        RayCastControl::Clip(0.75)
    });

    // Assert
    assert_eq!(result, Err(TreeError::InvalidClipFraction));
}

#[test]
fn dynamic_tree_ray_collection_is_a_unique_unspecified_order_set() {
    // Arrange
    let mut tree = DynamicTree::new().expect("a tree key should remain available");
    let first = tree
        .create_proxy(aabb(0.0, -0.5, 1.0, 0.5), 1_u8)
        .expect("finite bounds should create a proxy");
    let second = tree
        .create_proxy(aabb(2.0, -0.5, 3.0, 0.5), 2_u8)
        .expect("finite bounds should create a proxy");
    tree.create_proxy(aabb(2.0, 4.0, 3.0, 5.0), 3_u8)
        .expect("finite bounds should create a proxy");
    let input = RayCastInput::new(Vec2::new(-1.0, 0.0), Vec2::new(4.0, 0.0), 1.0)
        .expect("test ray should be valid");

    // Act
    let actual: HashSet<_> = tree
        .ray_candidate_ids(input)
        .expect("a valid ray should collect candidates")
        .into_iter()
        .collect();

    // Assert
    assert_eq!(actual, HashSet::from([first, second]));
}

#[test]
fn broad_phase_filter_defaults_and_symmetric_masks_match_upstream() {
    // Arrange
    let defaults = FilterData::default();
    let category_two = FilterData::new(0x0002, 0x0001, 0);
    let rejects_two = FilterData::new(0x0001, 0x0001, 0);

    // Act
    let default_pair = defaults.should_collide(category_two);
    let rejected_pair = rejects_two.should_collide(category_two);

    // Assert
    assert_eq!(defaults.category_bits(), 0x0001);
    assert_eq!(defaults.mask_bits(), 0xffff);
    assert_eq!(defaults.group_index(), 0);
    assert!(default_pair);
    assert!(!rejected_pair);
}

#[test]
fn broad_phase_equal_nonzero_groups_override_masks_by_sign() {
    // Arrange
    let positive_a = FilterData::new(0x0001, 0x0000, 4);
    let positive_b = FilterData::new(0x0002, 0x0000, 4);
    let negative_a = FilterData::new(0x0001, 0xffff, -4);
    let negative_b = FilterData::new(0x0002, 0xffff, -4);

    // Act
    let positive = positive_a.should_collide(positive_b);
    let negative = negative_a.should_collide(negative_b);

    // Assert
    assert!(positive);
    assert!(!negative);
}

#[test]
fn broad_phase_creation_reports_each_overlapping_pair_once_in_private_slot_order() {
    // Arrange
    let mut broad_phase = BroadPhase::new().expect("a tree key should remain available");
    broad_phase
        .create_proxy(aabb(0.0, 0.0, 2.0, 2.0), "a", FilterData::default())
        .expect("finite bounds should create a proxy");
    broad_phase
        .create_proxy(aabb(0.5, 0.5, 2.5, 2.5), "b", FilterData::default())
        .expect("finite bounds should create a proxy");
    broad_phase
        .create_proxy(aabb(1.0, 1.0, 3.0, 3.0), "c", FilterData::default())
        .expect("finite bounds should create a proxy");
    let mut pairs = Vec::new();

    // Act
    broad_phase
        .update_pairs(|_proxy_a, payload_a, _proxy_b, payload_b| {
            pairs.push((*payload_a, *payload_b));
        })
        .expect("live move entries should update");

    // Assert
    assert_eq!(pairs, [("a", "b"), ("a", "c"), ("b", "c")]);
}

#[test]
fn broad_phase_duplicate_touches_are_deduplicated_after_sorting() {
    // Arrange
    let mut broad_phase = BroadPhase::new().expect("a tree key should remain available");
    let first = broad_phase
        .create_proxy(aabb(0.0, 0.0, 1.0, 1.0), 1_u8, FilterData::default())
        .expect("finite bounds should create a proxy");
    broad_phase
        .create_proxy(aabb(0.5, 0.5, 1.5, 1.5), 2_u8, FilterData::default())
        .expect("finite bounds should create a proxy");
    broad_phase
        .update_pairs(|_, _, _, _| {})
        .expect("initial pairs should drain");
    broad_phase
        .touch_proxy(first)
        .expect("a live proxy should touch");
    broad_phase
        .touch_proxy(first)
        .expect("a live proxy should touch");
    let mut pair_count = 0;

    // Act
    broad_phase
        .update_pairs(|_, _, _, _| pair_count += 1)
        .expect("duplicate move entries should update");

    // Assert
    assert_eq!(pair_count, 1);
}

#[test]
fn broad_phase_destroy_tombstones_every_buffered_occurrence() {
    // Arrange
    let mut broad_phase = BroadPhase::new().expect("a tree key should remain available");
    let first = broad_phase
        .create_proxy(aabb(0.0, 0.0, 1.0, 1.0), 1_u8, FilterData::default())
        .expect("finite bounds should create a proxy");
    broad_phase
        .create_proxy(aabb(0.5, 0.5, 1.5, 1.5), 2_u8, FilterData::default())
        .expect("finite bounds should create a proxy");
    broad_phase
        .touch_proxy(first)
        .expect("a live proxy should touch");
    broad_phase
        .touch_proxy(first)
        .expect("a live proxy should touch");

    // Act
    let removed = broad_phase
        .destroy_proxy(first)
        .expect("a live proxy should be destroyed");
    let mut pair_count = 0;
    broad_phase
        .update_pairs(|_, _, _, _| pair_count += 1)
        .expect("tombstoned entries should be skipped");

    // Assert
    assert_eq!(removed, 1);
    assert_eq!(pair_count, 0);
}

#[test]
fn broad_phase_contained_move_does_not_buffer_until_touched() {
    // Arrange
    let mut broad_phase = BroadPhase::new().expect("a tree key should remain available");
    let first = broad_phase
        .create_proxy(aabb(0.0, 0.0, 1.0, 1.0), 1_u8, FilterData::default())
        .expect("finite bounds should create a proxy");
    broad_phase
        .create_proxy(aabb(0.5, 0.5, 1.5, 1.5), 2_u8, FilterData::default())
        .expect("finite bounds should create a proxy");
    broad_phase
        .update_pairs(|_, _, _, _| {})
        .expect("initial pairs should drain");
    broad_phase
        .move_proxy(first, aabb(0.05, 0.05, 0.95, 0.95), Vec2::ZERO)
        .expect("a contained move should succeed");
    let mut before_touch = 0;
    broad_phase
        .update_pairs(|_, _, _, _| before_touch += 1)
        .expect("an empty move buffer should update");

    // Act
    broad_phase
        .touch_proxy(first)
        .expect("a live proxy should touch");
    let mut after_touch = 0;
    broad_phase
        .update_pairs(|_, _, _, _| after_touch += 1)
        .expect("a touched proxy should update");

    // Assert
    assert_eq!(before_touch, 0);
    assert_eq!(after_touch, 1);
}

#[test]
fn broad_phase_refilter_touches_proxy_for_newly_eligible_pair() {
    // Arrange
    let mut broad_phase = BroadPhase::new().expect("a tree key should remain available");
    let first = broad_phase
        .create_proxy(
            aabb(0.0, 0.0, 1.0, 1.0),
            1_u8,
            FilterData::new(0x0001, 0x0000, 0),
        )
        .expect("finite bounds should create a proxy");
    let second = broad_phase
        .create_proxy(
            aabb(0.5, 0.5, 1.5, 1.5),
            2_u8,
            FilterData::new(0x0002, 0x0000, 0),
        )
        .expect("finite bounds should create a proxy");
    broad_phase
        .update_pairs(|_, _, _, _| {})
        .expect("initial candidates should drain");
    assert_eq!(broad_phase.should_collide(first, second), Ok(false));

    // Act
    broad_phase
        .set_filter_data(first, FilterData::new(0x0001, 0x0002, 0))
        .expect("a live proxy should refilter");
    broad_phase
        .set_filter_data(second, FilterData::new(0x0002, 0x0001, 0))
        .expect("a live proxy should refilter");
    let mut reconsidered = Vec::new();
    broad_phase
        .update_pairs(|proxy_a, _, proxy_b, _| {
            reconsidered.push((proxy_a, proxy_b));
        })
        .expect("refiltered proxies should be reconsidered");

    // Assert
    assert_eq!(reconsidered.len(), 1);
    assert_eq!(
        broad_phase.should_collide(reconsidered[0].0, reconsidered[0].1),
        Ok(true)
    );
}
