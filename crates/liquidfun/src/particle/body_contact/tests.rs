//! Independent legacy witnesses for spatial fixture-particle selection.

use crate::collision::{ChainShape, CircleShape, EdgeShape, PolygonShape};
use crate::identity::{HandleIdentity, Identity, ParticleSystemId, WorldKey};
use crate::particle::contact_scan::fill_stored_contacts;
use crate::particle::storage::{ParticleInput, ParticleStorage};

use super::*;

pub(super) fn storage(positions: &[Vec2], diameter: f32) -> ParticleStorage {
    let world = WorldKey::fresh().expect("test world");
    let system = ParticleSystemId::from_identity(Identity::new(world, 0, 0));
    let capacity = positions.len().max(8);
    let mut storage = ParticleStorage::new(world, system, 0, capacity, capacity).expect("storage");
    for (index, position) in positions.iter().copied().enumerate() {
        let mut flags = ParticleFlags::WATER
            | ParticleFlags::FIXTURE_CONTACT_FILTER
            | ParticleFlags::FIXTURE_CONTACT_LISTENER;
        if index.is_multiple_of(3) {
            flags |= ParticleFlags::WALL;
        }
        storage
            .create(ParticleInput {
                position,
                velocity: Vec2::ZERO,
                flags,
                maybe_group: None,
                maybe_color: None,
                maybe_user_association: None,
                maybe_expiration_time: None,
            })
            .expect("particle");
    }
    refresh(&mut storage, diameter);
    storage
}

fn refresh(storage: &mut ParticleStorage, diameter: f32) {
    let mut proxies = storage.take_contact_proxies();
    let mut contacts = storage.take_particle_contacts();
    fill_stored_contacts(
        storage.positions(),
        storage.flags(),
        storage.particle_ids(),
        diameter,
        &mut proxies,
        &mut contacts,
        &mut |_| true,
    )
    .expect("contact proxies");
    storage.install_contact_proxies(proxies);
    storage.install_particle_contacts(contacts);
}

fn sources(storage: &ParticleStorage) -> Vec<FixtureContactSource> {
    let world = storage.system().identity().world();
    let shapes = [
        Shape::from(CircleShape::new(Vec2::ZERO, 0.4).expect("circle")),
        Shape::from(PolygonShape::oriented_box(0.3, 0.2, Vec2::ZERO, 0.0).expect("box")),
        Shape::from(EdgeShape::new(Vec2::new(-0.5, 0.0), Vec2::new(0.5, 0.0)).expect("edge")),
        Shape::from(
            ChainShape::open(
                &[Vec2::new(-0.5, 0.0), Vec2::ZERO, Vec2::new(0.5, 0.2)],
                None,
                None,
            )
            .expect("chain"),
        ),
    ];
    shapes
        .into_iter()
        .enumerate()
        .map(|(index, shape)| FixtureContactSource {
            body: BodyId::from_identity(Identity::new(world, 1, 0)),
            fixture: FixtureId::from_identity(Identity::new(world, index + 2, 0)),
            shape,
            transform: Transform::IDENTITY,
            center: Vec2::new(0.15, 0.1),
            inverse_mass: 0.25,
            inverse_inertia: 0.2,
        })
        .collect()
}

#[test]
fn current_spatial_candidates_skip_far_rows_and_keep_source_row_order() {
    // Arrange
    let storage = storage(
        &[
            Vec2::new(10.0, 1.0),
            Vec2::new(0.2, 0.1),
            Vec2::new(-10.0, -1.0),
            Vec2::new(-0.2, -0.1),
        ],
        0.1,
    );
    let view = ParticleSystemView::new(&storage);
    let bounds = Aabb::new(Vec2::new(-0.5, -0.5), Vec2::new(0.5, 0.5)).expect("bounds");
    let mut rows = Vec::new();

    // Act
    let indexed = collect_candidate_rows(
        view.maybe_current_contact_proxies(0.1)
            .expect("current proxies"),
        bounds,
        0.1,
        &mut rows,
        &mut Vec::new(),
    );

    // Assert
    assert!(indexed, "current proxies should enable spatial selection");
    assert_eq!(rows, [1, 3]);
}

#[test]
fn all_shape_children_preserve_exact_contacts_effects_and_stateful_filter_order() {
    // Arrange
    let storage = storage(
        &[
            Vec2::new(0.2, 0.05),
            Vec2::new(-0.2, -0.02),
            Vec2::new(10.0, 0.0),
            Vec2::new(0.42, 0.05),
            Vec2::new(0.0, 0.19),
        ],
        0.1,
    );
    let view = ParticleSystemView::new(&storage);
    let sources = sources(&storage);
    let previous = legacy(&view, &sources, &[], 0.1, 2.0, false, |_| true).contacts;

    // Act / Assert
    for strict in [false, true] {
        let mut expected_trace = Vec::new();
        let expected = legacy(&view, &sources, &previous, 0.1, 2.0, strict, |contact| {
            expected_trace.push(*contact);
            expected_trace.len().is_multiple_of(2)
        });
        let mut actual_trace = Vec::new();
        let actual = generate(&view, &sources, &previous, 0.1, 2.0, strict, |contact| {
            actual_trace.push(*contact);
            actual_trace.len().is_multiple_of(2)
        });
        assert_eq!(actual_trace, expected_trace);
        assert_eq!(actual, expected);
    }
}

#[test]
fn missing_and_stale_indices_preserve_the_full_scan() {
    // Arrange
    let mut storage = storage(&[Vec2::new(0.2, 0.05), Vec2::new(-0.2, 0.0)], 0.1);
    let sources = sources(&storage);

    // Act / Assert
    for mode in 0..2 {
        if mode == 0 {
            storage.clear_contact_scan();
        }
        if mode == 1 {
            refresh(&mut storage, 0.1);
            let particle = storage.particle_ids()[0];
            storage
                .commit_kinematic_edit(particle, Vec2::new(2.0, 0.05), Vec2::ZERO)
                .expect("position edit");
        }
        let view = ParticleSystemView::new(&storage);
        assert!(view.maybe_current_contact_proxies(0.1).is_none());
        let actual = generate(&view, &sources, &[], 0.1, 2.0, false, |_| true);
        let expected = legacy(&view, &sources, &[], 0.1, 2.0, false, |_| true);
        assert_eq!(actual, expected);
    }
}

#[test]
fn overflowing_fixture_query_falls_back_before_any_filter_effects() {
    // Arrange
    let storage = storage(&[Vec2::new(204.7, 0.05)], 0.1);
    let view = ParticleSystemView::new(&storage);
    let mut source = sources(&storage).remove(0);
    source.shape = Shape::from(CircleShape::new(Vec2::new(204.6, 0.0), 0.4).expect("circle"));
    let bounds = expanded_fixture_aabb(&source, source.shape.child_index(0).expect("child"), 0.1)
        .expect("finite bounds");
    let mut rows = vec![7, 13];

    // Act
    let indexed = collect_candidate_rows(
        view.maybe_current_contact_proxies(0.1)
            .expect("valid current particle tags"),
        bounds,
        0.1,
        &mut rows,
        &mut Vec::new(),
    );
    let mut expected_trace = Vec::new();
    let expected = legacy(
        &view,
        std::slice::from_ref(&source),
        &[],
        0.1,
        2.0,
        false,
        |contact| {
            expected_trace.push(*contact);
            true
        },
    );
    let mut actual_trace = Vec::new();
    let actual = generate(
        &view,
        std::slice::from_ref(&source),
        &[],
        0.1,
        2.0,
        false,
        |contact| {
            actual_trace.push(*contact);
            true
        },
    );

    // Assert
    assert!(!indexed);
    assert!(rows.is_empty());
    assert!(!actual_trace.is_empty());
    assert_eq!(actual_trace, expected_trace);
    assert_eq!(actual, expected);
}

#[test]
fn strict_aabb_edges_and_tiny_position_edits_match_legacy_selection() {
    // Arrange
    let mut storage = storage(&[Vec2::new(0.2, 0.05), Vec2::new(-0.2, -0.05)], 0.1);
    let source = sources(&storage).remove(0);
    let bounds = expanded_fixture_aabb(&source, source.shape.child_index(0).expect("child"), 0.1)
        .expect("bounds");
    let first = storage.particle_ids()[0];
    storage
        .commit_kinematic_edit(first, Vec2::new(0.200_001, 0.05), Vec2::ZERO)
        .expect("same-cell edit");

    // Act / Assert
    let view = ParticleSystemView::new(&storage);
    assert!(view.maybe_current_contact_proxies(0.1).is_some());
    assert_eq!(
        generate(
            &view,
            std::slice::from_ref(&source),
            &[],
            0.1,
            2.0,
            false,
            |_| true
        ),
        legacy(
            &view,
            std::slice::from_ref(&source),
            &[],
            0.1,
            2.0,
            false,
            |_| true
        )
    );
    for x in [
        bounds.lower_bound().x,
        bounds.upper_bound().x,
        f32::from_bits(bounds.upper_bound().x.to_bits() - 1),
        f32::from_bits(bounds.upper_bound().x.to_bits() + 1),
    ] {
        storage
            .commit_kinematic_edit(first, Vec2::new(x, 0.0), Vec2::ZERO)
            .expect("edge edit");
        refresh(&mut storage, 0.1);
        let view = ParticleSystemView::new(&storage);
        assert_eq!(
            generate(
                &view,
                std::slice::from_ref(&source),
                &[],
                0.1,
                2.0,
                false,
                |_| true
            ),
            legacy(
                &view,
                std::slice::from_ref(&source),
                &[],
                0.1,
                2.0,
                false,
                |_| true
            )
        );
    }
}

// Preserve the pre-index loop as an independent witness, including arithmetic
// grouping and callback timing. Only unchanged strict/listener rules are shared.
#[test]
fn clear_contact_scan_empties_current_proxies_and_keeps_the_order() {
    // Arrange
    let mut storage = storage(&[Vec2::new(0.2, 0.05), Vec2::new(-0.2, 0.0)], 0.1);
    let installed = storage.contact_proxies().to_vec();

    // Act
    storage.clear_contact_scan();

    // Assert
    assert!(!installed.is_empty());
    assert!(storage.contact_proxies().is_empty());
    assert_eq!(storage.take_contact_proxies(), installed);
}

#[test]
fn storage_equality_ignores_retained_proxy_order() {
    // Arrange
    let storage = storage(&[Vec2::new(0.2, 0.05), Vec2::new(-0.2, 0.0)], 0.1);
    let mut retained = storage.clone();
    let mut plain = storage.clone();

    // Act
    retained.clear_contact_scan();
    let _ = plain.take_contact_proxies();

    // Assert
    assert!(retained.contact_proxies().is_empty() && plain.contact_proxies().is_empty());
    assert!(!retained.clone().take_contact_proxies().is_empty());
    assert!(plain.clone().take_contact_proxies().is_empty());
    assert!(retained == plain);
}

pub(super) fn legacy(
    view: &ParticleSystemView<'_>,
    sources: &[FixtureContactSource],
    previous: &[ParticleBodyContact],
    diameter: f32,
    density: f32,
    strict: bool,
    mut filter: impl FnMut(&ParticleBodyContact) -> bool,
) -> ParticleBodyContactUpdate {
    let inverse_diameter = 1.0 / diameter;
    let inverse_stride = inverse_diameter * (1.0 / settings::PARTICLE_STRIDE);
    let particle_inverse_mass = (1.0 / density) * inverse_stride * inverse_stride;
    let mut contacts = Vec::new();
    for source in sources {
        for child in 0..source.shape.child_count() {
            let child = ChildIndex::new(child, source.shape.child_count()).expect("child");
            let maybe_aabb = expanded_fixture_aabb(source, child, diameter);
            for (row, particle) in view.particle_ids().iter().copied().enumerate() {
                let position = view.positions()[row];
                if let Some(aabb) = maybe_aabb
                    && !aabb_contains_point(aabb, position)
                {
                    continue;
                }
                let distance = source
                    .shape
                    .distance_to_point(source.transform, position, child)
                    .expect("shape");
                if distance.distance() >= diameter {
                    continue;
                }
                let flags = view.flags()[row];
                let normal = -distance.normal();
                let offset = position - source.center;
                let normal_lever = offset.cross(distance.normal());
                let particle_term = if flags.contains(ParticleFlags::WALL) {
                    0.0
                } else {
                    particle_inverse_mass
                };
                let inverse_effective_mass = particle_term
                    + source.inverse_mass
                    + source.inverse_inertia * normal_lever * normal_lever;
                let contact = ParticleBodyContact {
                    particle,
                    body: source.body,
                    fixture: source.fixture,
                    weight: 1.0 - distance.distance() * inverse_diameter,
                    normal,
                    mass: if inverse_effective_mass > 0.0 {
                        1.0 / inverse_effective_mass
                    } else {
                        0.0
                    },
                };
                if flags.contains(ParticleFlags::FIXTURE_CONTACT_FILTER) && !filter(&contact) {
                    continue;
                }
                contacts.push(contact);
            }
        }
    }
    if strict {
        contacts.sort_by(|left, right| {
            let order = particle_row(view, left.particle).cmp(&particle_row(view, right.particle));
            if order == Ordering::Equal {
                right
                    .weight
                    .partial_cmp(&left.weight)
                    .unwrap_or(Ordering::Equal)
            } else {
                order
            }
        });
        prune_strict_contacts(view, sources, diameter, &mut contacts);
    }
    let effects = if fixture_contact_listeners_active(view) {
        listener_effects(view, previous, &contacts)
    } else {
        Vec::new()
    };
    ParticleBodyContactUpdate { contacts, effects }
}

/// `sort_unstable` plus `dedup` over the query's rows, the order before A5.
fn reference_rows(proxies: &[ContactProxy], bounds: Aabb, diameter: f32) -> Vec<usize> {
    let mut rows = Vec::new();
    visit_sorted_tag_indices_in_aabb(
        proxies.len(),
        |index| proxies[index].tag,
        diameter,
        bounds,
        |index| rows.push(proxies[index].row),
    )
    .expect("reference query stays in the tag domain");
    rows.sort_unstable();
    rows.dedup();
    rows
}

/// A 25-column grid whose rows are a fixed permutation of its cells, so row
/// order differs from tag order and several particles share a tag cell.
fn scrambled_grid(count: u16) -> Vec<Vec2> {
    (0..count)
        .map(|index| {
            let cell = (u32::from(index) * 7_919) % u32::from(count);
            let column = u16::try_from(cell % 25).expect("column fits");
            let row = u16::try_from(cell / 25).expect("row fits");
            Vec2::new(
                f32::from(column) * 0.07 - 0.875,
                f32::from(row) * 0.07 - 0.7,
            )
        })
        .collect()
}

#[test]
fn candidate_rows_match_sorted_dedup() {
    // Arrange
    let storage = storage(&scrambled_grid(500), 0.1);
    let view = ParticleSystemView::new(&storage);
    let proxies = view
        .maybe_current_contact_proxies(0.1)
        .expect("current proxies");
    let cases = [
        Aabb::new(Vec2::new(-0.05, -0.05), Vec2::new(0.05, 0.05)).expect("small"),
        Aabb::new(Vec2::new(-0.3, -0.2), Vec2::new(0.3, 0.2)).expect("middle"),
        Aabb::new(Vec2::new(-0.9, -0.1), Vec2::new(0.9, 0.0)).expect("wide"),
        Aabb::new(Vec2::new(0.4, -0.8), Vec2::new(0.5, 0.8)).expect("tall"),
        Aabb::new(Vec2::new(-2.0, -2.0), Vec2::new(2.0, 2.0)).expect("everything"),
        Aabb::new(Vec2::new(5.0, 5.0), Vec2::new(6.0, 6.0)).expect("nothing"),
    ];
    let expected: Vec<Vec<usize>> = cases
        .iter()
        .map(|&bounds| reference_rows(proxies, bounds, 0.1))
        .collect();
    let mut rows = Vec::new();
    let mut row_marks = Vec::new();

    // Act
    let actual: Vec<Vec<usize>> = cases
        .iter()
        .map(|&bounds| {
            assert!(collect_candidate_rows(
                proxies,
                bounds,
                0.1,
                &mut rows,
                &mut row_marks
            ));
            rows.clone()
        })
        .collect();

    // Assert
    assert!(
        expected.iter().any(|rows| rows.len() >= 32),
        "some cases should take the bitset walk"
    );
    assert!(
        expected
            .iter()
            .any(|rows| !rows.is_empty() && rows.len() < 32),
        "some cases should take the small sort"
    );
    assert_eq!(actual, expected);
}

#[test]
fn candidate_rows_small_query_uses_same_order() {
    // Arrange
    let storage = storage(
        &[
            Vec2::new(0.3, 0.0),
            Vec2::new(5.0, 5.0),
            Vec2::new(-0.3, 0.0),
            Vec2::new(0.0, 0.2),
        ],
        0.1,
    );
    let view = ParticleSystemView::new(&storage);
    let bounds = Aabb::new(Vec2::new(-0.5, -0.5), Vec2::new(0.5, 0.5)).expect("bounds");
    let mut rows = Vec::new();

    // Act
    let indexed = collect_candidate_rows(
        view.maybe_current_contact_proxies(0.1)
            .expect("current proxies"),
        bounds,
        0.1,
        &mut rows,
        &mut Vec::new(),
    );

    // Assert
    assert!(indexed);
    assert_eq!(rows, [0, 2, 3]);
}

#[test]
fn candidate_rows_failed_query_returns_false_and_empty() {
    // Arrange
    let storage = storage(&scrambled_grid(64), 0.1);
    let view = ParticleSystemView::new(&storage);
    let bounds = Aabb::new(Vec2::new(0.0, 0.0), Vec2::new(1.0e6, 1.0)).expect("bounds");
    let mut rows = vec![7, 13];
    let mut row_marks = Vec::new();

    // Act
    let indexed = collect_candidate_rows(
        view.maybe_current_contact_proxies(0.1)
            .expect("current proxies"),
        bounds,
        0.1,
        &mut rows,
        &mut row_marks,
    );

    // Assert
    assert!(!indexed);
    assert!(rows.is_empty());
}
