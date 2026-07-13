//! Consumer-level coverage for the checked standalone rope.

use liquidfun::math::Vec2;
use liquidfun::rope::{Rope, RopeDef, RopeError, RopeIterations};

fn rope_definition(
    vertices: Vec<Vec2>,
    masses: Vec<f32>,
    gravity: Vec2,
    damping: f32,
    stretching_stiffness: f32,
    bending_stiffness: f32,
) -> RopeDef {
    RopeDef::new(
        vertices,
        masses,
        gravity,
        damping,
        stretching_stiffness,
        bending_stiffness,
    )
    .expect("test rope definition should be valid")
}

fn vertex_bits(vertices: &[Vec2]) -> Vec<(u32, u32)> {
    vertices
        .iter()
        .map(|vertex| (vertex.x.to_bits(), vertex.y.to_bits()))
        .collect()
}

#[test]
fn fixed_and_free_vertices_follow_source_integration() {
    // Arrange
    let definition = rope_definition(
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(2.0, 0.0),
        ],
        vec![0.0, 1.0, 1.0],
        Vec2::new(0.0, -10.0),
        0.0,
        0.0,
        0.0,
    );
    let mut rope = Rope::new(definition).expect("rest state should be finite");

    // Act
    rope.step(
        0.1,
        RopeIterations::new(0).expect("zero iterations are source-supported"),
    )
    .expect("step should remain finite");

    // Assert
    assert_eq!(rope.vertices()[0], Vec2::new(0.0, 0.0));
    assert_eq!(rope.vertices()[1], Vec2::new(1.0, -0.1));
    assert_eq!(rope.vertices()[2], Vec2::new(2.0, -0.1));
}

#[test]
fn gravity_is_damped_before_position_integration() {
    // Arrange
    let definition = rope_definition(
        vec![Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(2.0, 0.0)],
        vec![0.0, 1.0, 1.0],
        Vec2::new(0.0, -10.0),
        2.0,
        0.0,
        0.0,
    );
    let mut rope = Rope::new(definition).expect("rest state should be finite");

    // Act
    rope.step(
        0.1,
        RopeIterations::new(0).expect("zero iterations are source-supported"),
    )
    .expect("step should remain finite");

    // Assert
    let expected_y = -0.1 * (-0.2_f32).exp();
    assert_eq!(rope.vertices()[1].y.to_bits(), expected_y.to_bits());
}

#[test]
fn zero_timestep_is_a_bit_identical_no_op() {
    // Arrange
    let definition = rope_definition(
        vec![
            Vec2::new(-1.0, 0.5),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 0.25),
        ],
        vec![0.0, 1.0, 2.0],
        Vec2::new(0.0, -9.8),
        0.1,
        0.9,
        0.1,
    );
    let mut rope = Rope::new(definition).expect("rest state should be finite");
    let before = vertex_bits(rope.vertices());

    // Act
    rope.step(
        0.0,
        RopeIterations::new(8).expect("ordinary iteration count should fit"),
    )
    .expect("zero step should succeed");

    // Assert
    assert_eq!(vertex_bits(rope.vertices()), before);
}

#[test]
fn each_iteration_solves_stretch_then_bend_then_stretch() {
    // Arrange
    let definition = rope_definition(
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(3.0, 1.0),
        ],
        vec![0.0, 1.0, 1.0, 1.0],
        Vec2::new(0.0, -10.0),
        0.25,
        0.9,
        0.1,
    );
    let mut rope = Rope::new(definition).expect("rest state should be finite");

    // Act
    rope.step(
        0.1,
        RopeIterations::new(1).expect("one iteration should fit"),
    )
    .expect("step should remain finite");

    // Assert
    // Oracle: pinned C++ b2Rope at 7f20402173fd143a3988c921bc384459c6a858f2,
    // initialized from the same four vertices/masses and stepped once at 0.1s.
    assert_eq!(
        vertex_bits(rope.vertices()),
        vec![
            (0x0000_0000, 0x0000_0000),
            (0x3f7e_c0ee, 0xbdbc_cdfa),
            (0x4000_0bfa, 0x3f66_a5fe),
            (0x4040_08a1, 0x3f66_f86b),
        ]
    );
}

#[test]
fn zero_one_and_many_iterations_are_observably_distinct() {
    // Arrange
    let definition = rope_definition(
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(0.75, 0.25),
            Vec2::new(1.5, -0.25),
            Vec2::new(2.25, 0.0),
        ],
        vec![0.0, 1.0, 1.0, 1.0],
        Vec2::new(0.0, -9.8),
        0.1,
        0.9,
        0.1,
    );
    let mut zero = Rope::new(definition.clone()).expect("rest state should be finite");
    let mut one = Rope::new(definition.clone()).expect("rest state should be finite");
    let mut many = Rope::new(definition).expect("rest state should be finite");

    // Act
    zero.step(
        0.2,
        RopeIterations::new(0).expect("zero iterations should fit"),
    )
    .expect("zero-iteration step should remain finite");
    one.step(
        0.2,
        RopeIterations::new(1).expect("one iteration should fit"),
    )
    .expect("one-iteration step should remain finite");
    many.step(
        0.2,
        RopeIterations::new(8).expect("eight iterations should fit"),
    )
    .expect("multi-iteration step should remain finite");

    // Assert
    assert_ne!(vertex_bits(zero.vertices()), vertex_bits(one.vertices()));
    assert_ne!(vertex_bits(one.vertices()), vertex_bits(many.vertices()));
}

#[test]
fn angle_wrapping_takes_the_short_path_across_pi() {
    // Arrange
    let definition = rope_definition(
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 0.000_001),
        ],
        vec![0.0, 1.0, 1.0],
        Vec2::ZERO,
        0.0,
        0.0,
        0.5,
    );
    let mut rope = Rope::new(definition).expect("rest state should be finite");
    rope.set_angle(-std::f32::consts::PI + 0.000_001)
        .expect("finite target angle should be accepted");

    // Act
    rope.step(
        0.1,
        RopeIterations::new(1).expect("one iteration should fit"),
    )
    .expect("wrapped solve should remain finite");

    // Assert
    assert!(rope.vertices()[1].length() < 1.1);
    assert!(rope.vertices()[2].length() < 1.1);
}

#[test]
fn definition_rejects_mismatched_and_too_short_lanes() {
    // Arrange
    let mismatched_vertices = vec![Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(2.0, 0.0)];
    let too_short_vertices = vec![Vec2::ZERO, Vec2::new(1.0, 0.0)];

    // Act
    let mismatched = RopeDef::new(
        mismatched_vertices,
        vec![1.0, 1.0],
        Vec2::ZERO,
        0.1,
        0.9,
        0.1,
    );
    let too_short = RopeDef::new(
        too_short_vertices,
        vec![1.0, 1.0],
        Vec2::ZERO,
        0.1,
        0.9,
        0.1,
    );

    // Assert
    assert_eq!(
        mismatched,
        Err(RopeError::VertexMassLengthMismatch {
            vertices: 3,
            masses: 2,
        })
    );
    assert_eq!(
        too_short,
        Err(RopeError::TooFewVertices {
            count: 2,
            minimum: 3,
        })
    );
}

#[test]
fn definition_rejects_non_finite_negative_and_derived_overflow() {
    // Arrange
    let valid_vertices = vec![Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(2.0, 0.0)];
    let overflowing_vertices = vec![
        Vec2::new(-f32::MAX, 0.0),
        Vec2::new(f32::MAX, 0.0),
        Vec2::new(0.0, 0.0),
    ];

    // Act
    let non_finite = RopeDef::new(
        valid_vertices.clone(),
        vec![1.0, f32::NAN, 1.0],
        Vec2::ZERO,
        0.1,
        0.9,
        0.1,
    );
    let negative = RopeDef::new(
        valid_vertices,
        vec![1.0, -1.0, 1.0],
        Vec2::ZERO,
        0.1,
        0.9,
        0.1,
    );
    let overflow = Rope::new(rope_definition(
        overflowing_vertices,
        vec![1.0, 1.0, 1.0],
        Vec2::ZERO,
        0.1,
        0.9,
        0.1,
    ));

    // Assert
    assert_eq!(non_finite, Err(RopeError::NonFiniteMass { index: 1 }));
    assert_eq!(negative, Err(RopeError::NegativeMass { index: 1 }));
    assert_eq!(overflow, Err(RopeError::NonFiniteDerivedState { index: 0 }));
}

#[test]
fn iteration_counts_are_bounded_and_include_zero() {
    // Arrange
    let maximum = RopeIterations::MAX;

    // Act
    let zero = RopeIterations::new(0);
    let at_maximum = RopeIterations::new(maximum);
    let above_maximum = RopeIterations::new(maximum + 1);

    // Assert
    assert_eq!(zero.map(RopeIterations::get), Ok(0));
    assert_eq!(at_maximum.map(RopeIterations::get), Ok(maximum));
    assert_eq!(
        above_maximum,
        Err(RopeError::IterationCountOutOfRange {
            count: maximum + 1,
            maximum,
        })
    );
}

#[test]
fn invalid_step_and_angle_leave_vertices_bit_identical() {
    // Arrange
    let definition = rope_definition(
        vec![Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(2.0, 0.0)],
        vec![0.0, 1.0, 1.0],
        Vec2::new(0.0, -10.0),
        0.1,
        0.9,
        0.1,
    );
    let mut rope = Rope::new(definition).expect("rest state should be finite");
    let before = vertex_bits(rope.vertices());

    // Act
    let time_step = rope.step(
        f32::NAN,
        RopeIterations::new(1).expect("one iteration should fit"),
    );
    let angle = rope.set_angle(f32::INFINITY);

    // Assert
    assert_eq!(time_step, Err(RopeError::NonFiniteTimeStep));
    assert_eq!(angle, Err(RopeError::NonFiniteAngle));
    assert_eq!(vertex_bits(rope.vertices()), before);
}

#[test]
fn derived_step_overflow_leaves_vertices_bit_identical() {
    // Arrange
    let definition = rope_definition(
        vec![Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(2.0, 0.0)],
        vec![0.0, f32::MIN_POSITIVE, 1.0],
        Vec2::new(f32::MAX, 0.0),
        0.0,
        0.9,
        0.1,
    );
    let mut rope = Rope::new(definition).expect("rest state should be finite");
    let before = vertex_bits(rope.vertices());

    // Act
    let result = rope.step(
        f32::MAX,
        RopeIterations::new(1).expect("one iteration should fit"),
    );

    // Assert
    assert_eq!(result, Err(RopeError::NonFiniteDerivedState { index: 1 }));
    assert_eq!(vertex_bits(rope.vertices()), before);
}
