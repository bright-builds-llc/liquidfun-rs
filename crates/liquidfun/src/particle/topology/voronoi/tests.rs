use proptest::prelude::*;

use crate::math::Vec2;

use super::*;

fn limits() -> VoronoiLimits {
    VoronoiLimits::new(32, 1_024, 4_096, 1_000_000, 2_048)
}

fn generator(x: f32, y: f32, necessary: bool) -> VoronoiGenerator {
    VoronoiGenerator::new(Vec2::new(x, y), necessary)
}

fn node_ordinals(diagram: &VoronoiDiagram) -> Vec<[usize; 3]> {
    diagram
        .nodes()
        .iter()
        .map(|node| node.generator_ordinals())
        .collect()
}

#[test]
fn emits_two_oriented_nodes_in_row_major_order() {
    // Arrange
    let generators = [
        generator(0.0, 0.0, true),
        generator(2.0, 0.0, true),
        generator(0.0, 2.0, true),
        generator(2.0, 2.0, true),
    ];

    // Act
    let diagram = VoronoiDiagram::generate(&generators, 1.0, 0.0, limits())
        .expect("bounded square should generate");

    // Assert
    assert_eq!(node_ordinals(&diagram), vec![[0, 1, 2], [1, 3, 2]]);
}

#[test]
fn equal_distance_cells_retain_the_incumbent_generator() {
    // Arrange
    let generators = [
        generator(0.0, 0.0, true),
        generator(2.0, 0.0, true),
        generator(0.0, 2.0, true),
        generator(2.0, 2.0, true),
    ];

    // Act
    let diagram = VoronoiDiagram::generate(&generators, 1.0, 0.0, limits())
        .expect("bounded square should generate");

    // Assert
    assert_eq!(diagram.owner_ordinal(1, 1), Some(0));
    assert_eq!(diagram.owner_ordinal(2, 2), Some(3));
}

#[test]
fn duplicate_positions_retain_the_first_dense_generator() {
    // Arrange
    let generators = [
        generator(0.0, 0.0, true),
        generator(0.0, 0.0, true),
        generator(2.0, 0.0, true),
        generator(0.0, 2.0, true),
    ];

    // Act
    let diagram = VoronoiDiagram::generate(&generators, 1.0, 0.0, limits())
        .expect("bounded duplicate should generate");

    // Assert
    assert_eq!(node_ordinals(&diagram), vec![[0, 2, 3]]);
    assert!(
        diagram
            .nodes()
            .iter()
            .all(|node| !node.generator_ordinals().contains(&1))
    );
}

#[test]
fn neighbor_iteration_is_left_down_right_up() {
    // Arrange
    let center = GridCell::new(1, 1, 3);

    // Act
    let neighbors = neighbor_cells(center, 3, 3);

    // Assert
    assert_eq!(
        neighbors.map(|maybe_cell| maybe_cell.map(|cell| cell.index)),
        [Some(3), Some(1), Some(5), Some(7)]
    );
}

#[test]
fn nodes_without_a_necessary_generator_are_filtered() {
    // Arrange
    let generators = [
        generator(0.0, 0.0, true),
        generator(2.0, 0.0, false),
        generator(0.0, 2.0, false),
        generator(2.0, 2.0, false),
    ];

    // Act
    let diagram = VoronoiDiagram::generate(&generators, 1.0, 2.0, limits())
        .expect("bounded square should generate");

    // Assert
    assert_eq!(node_ordinals(&diagram), vec![[0, 1, 2]]);
}

#[test]
fn fewer_than_three_generators_emit_no_nodes() {
    // Arrange
    let cases = [
        Vec::new(),
        vec![generator(0.0, 0.0, true)],
        vec![generator(0.0, 0.0, true), generator(2.0, 0.0, true)],
    ];

    // Act
    let results: Vec<_> = cases
        .iter()
        .map(|generators| {
            VoronoiDiagram::generate(generators, 1.0, 0.0, limits())
                .expect("small bounded case should generate")
        })
        .collect();

    // Assert
    assert!(results.iter().all(|diagram| diagram.nodes().is_empty()));
}

#[test]
fn no_necessary_generators_produce_an_empty_diagram() {
    // Arrange
    let generators = [
        generator(0.0, 0.0, false),
        generator(2.0, 0.0, false),
        generator(0.0, 2.0, false),
    ];

    // Act
    let diagram = VoronoiDiagram::generate(&generators, 1.0, 0.0, limits())
        .expect("unnecessary generators should be safely ignored");

    // Assert
    assert!(diagram.nodes().is_empty());
    assert_eq!(diagram.dimensions(), (0, 0));
}

#[test]
fn invalid_radius_is_rejected_before_generation() {
    // Arrange
    let generators = [generator(0.0, 0.0, true)];

    // Act
    let zero = VoronoiDiagram::generate(&generators, 0.0, 0.0, limits());
    let negative = VoronoiDiagram::generate(&generators, -1.0, 0.0, limits());
    let non_finite = VoronoiDiagram::generate(&generators, f32::INFINITY, 0.0, limits());

    // Assert
    assert_eq!(zero, Err(VoronoiError::NonPositiveRadius));
    assert_eq!(negative, Err(VoronoiError::NonPositiveRadius));
    assert_eq!(non_finite, Err(VoronoiError::NonFiniteRadius));
}

#[test]
fn non_finite_generator_is_rejected_with_its_dense_ordinal() {
    // Arrange
    let generators = [generator(0.0, 0.0, true), generator(f32::NAN, 1.0, false)];

    // Act
    let result = VoronoiDiagram::generate(&generators, 1.0, 0.0, limits());

    // Assert
    assert_eq!(result, Err(VoronoiError::NonFiniteGenerator { ordinal: 1 }));
}

#[test]
fn extreme_finite_bounds_fail_before_grid_allocation() {
    // Arrange
    let generators = [generator(-1.0e30, 0.0, true), generator(1.0e30, 0.0, true)];

    // Act
    let result = VoronoiDiagram::generate(&generators, 1.0, 0.0, limits());

    // Assert
    assert_eq!(result, Err(VoronoiError::AxisCountOutOfRange));
}

#[test]
fn checked_grid_and_queue_limits_fail_before_allocation() {
    // Arrange
    let generators = [generator(0.0, 0.0, true), generator(8.0, 8.0, true)];
    let grid_limited = VoronoiLimits::new(8, 64, 256, 100_000, 128);
    let queue_limited = VoronoiLimits::new(8, 81, 323, 100_000, 128);

    // Act
    let grid_result = VoronoiDiagram::generate(&generators, 1.0, 0.0, grid_limited);
    let queue_result = VoronoiDiagram::generate(&generators, 1.0, 0.0, queue_limited);

    // Assert
    assert_eq!(
        grid_result,
        Err(VoronoiError::GridLimitExceeded {
            required: 81,
            limit: 64,
        })
    );
    assert_eq!(
        queue_result,
        Err(VoronoiError::QueueLimitExceeded {
            required: 324,
            limit: 323,
        })
    );
}

proptest! {
    #[test]
    fn repeated_runs_are_structurally_identical(
        points in prop::collection::vec((-4_i8..=4, -4_i8..=4), 1..=8),
        radius in 1_u8..=3,
    ) {
        // Arrange
        let generators: Vec<_> = points
            .iter()
            .enumerate()
            .map(|(ordinal, &(x, y))| {
                generator(f32::from(x), f32::from(y), ordinal % 3 == 0)
            })
            .collect();
        let radius = f32::from(radius);

        // Act
        let first = VoronoiDiagram::generate(&generators, radius, radius, limits())
            .expect("bounded generated case should succeed");
        let second = VoronoiDiagram::generate(&generators, radius, radius, limits())
            .expect("bounded replay should succeed");

        // Assert
        prop_assert_eq!(first, second);
    }

    #[test]
    fn emitted_nodes_have_distinct_bounded_necessary_endpoints(
        points in prop::collection::vec((-4_i8..=4, -4_i8..=4), 1..=8),
    ) {
        // Arrange
        let generators: Vec<_> = points
            .iter()
            .enumerate()
            .map(|(ordinal, &(x, y))| {
                generator(f32::from(x), f32::from(y), ordinal % 3 == 0)
            })
            .collect();

        // Act
        let diagram = VoronoiDiagram::generate(&generators, 1.0, 1.0, limits())
            .expect("bounded generated case should succeed");

        // Assert
        for node in diagram.nodes() {
            let [a, b, c] = node.generator_ordinals();
            prop_assert!(a < generators.len());
            prop_assert!(b < generators.len());
            prop_assert!(c < generators.len());
            prop_assert!(a != b && a != c && b != c);
            prop_assert!(
                generators[a].necessary()
                    || generators[b].necessary()
                    || generators[c].necessary()
            );
        }
    }
}
