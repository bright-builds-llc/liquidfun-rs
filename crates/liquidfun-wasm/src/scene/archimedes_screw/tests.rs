//! Debug-safe Archimedes Screw tests: geometry invariants (D-03..D-08, D-12).

use std::f32::consts::TAU;

use liquidfun::collision::PolygonShape;
use liquidfun::math::{Transform, Vec2};

use super::geometry::{
    AXIS, CHUTE_INNER_X, CHUTE_TOP_Y, FRAME_MAX_X, FRAME_MAX_Y, FRAME_MIN_X, FRAME_MIN_Y,
    HOOD_BOTTOM_Y, HOOD_LEFT_X, HOUSING_LENGTH, INLET_LENGTH, INLET_SEAL_Y, NORMAL, OUTLET_LENGTH,
    PARTICLE_DIAMETER, PARTICLE_RADIUS, PITCH, SLAB_THICKNESS, TUBE_GAP, TUBE_LENGTH, TUBE_ORIGIN,
    VANE_COS, VANE_COUNT, VANE_EMBED, VANE_HALF_SPAN, VANE_OVERHANG, VANE_TAN, VANE_THICKNESS,
    WEIR_LIP_Y, bottom_housing, distance_to_boundary, floor_slab, in_basin, in_reservoir,
    quad_contains, top_housing, tube_point, vane_quads, visible_vane_outline, wall_quads,
    water_positions,
};

const LITERAL_TOLERANCE: f32 = 1e-6;
const FRAME_MARGIN: f32 = 0.1;
const MIN_SPAWN_COUNT: usize = 1_000;
const MAX_SPAWN_COUNT: usize = 3_000;

/// World point at vane coordinate `q` and height `t`.
fn vane_point(q: f32, t: f32) -> Vec2 {
    tube_point(q + (t - TUBE_GAP / 2.0) * VANE_TAN, t)
}

fn polygon(quad: &[Vec2; 4]) -> PolygonShape {
    PolygonShape::new(quad).expect("geometry quads are convex")
}

fn vane_at_phase(index: u8, phase: f32) -> [Vec2; 4] {
    vane_quads()[usize::from(index)].map(|corner| corner + TUBE_ORIGIN + AXIS * phase)
}

#[test]
fn tube_axis_literals_match_a_thirty_degree_tilt() {
    // Arrange
    let angle = TAU / 12.0;

    // Act
    let errors = [
        AXIS.x - angle.cos(),
        AXIS.y - angle.sin(),
        NORMAL.x + angle.sin(),
        NORMAL.y - angle.cos(),
    ];

    // Assert
    assert!(
        errors.iter().all(|error| error.abs() < LITERAL_TOLERANCE),
        "{errors:?}"
    );
}

#[test]
fn vane_slant_literals_match_twenty_degrees() {
    // Arrange
    let angle = TAU / 18.0;

    // Act
    let errors = [VANE_TAN - angle.tan(), VANE_COS - angle.cos()];

    // Assert
    assert!(
        errors.iter().all(|error| error.abs() < LITERAL_TOLERANCE),
        "{errors:?}"
    );
}

#[test]
fn the_vane_added_by_a_forward_snap_lies_inside_the_bottom_housing() {
    // Arrange
    let housing = polygon(&bottom_housing());

    // Act
    let vane = vane_at_phase(0, 0.0);

    // Assert
    for corner in vane {
        assert!(
            housing
                .test_point(Transform::IDENTITY, corner)
                .expect("finite corner"),
            "vane 0 corner {corner:?} is outside the bottom housing"
        );
    }
}

#[test]
fn the_vane_removed_by_a_forward_snap_lies_inside_the_top_housing() {
    // Arrange
    let housing = polygon(&top_housing());

    // Act
    let vane = vane_at_phase(VANE_COUNT - 1, PITCH - 1e-4);

    // Assert
    for corner in vane {
        assert!(
            housing
                .test_point(Transform::IDENTITY, corner)
                .expect("finite corner"),
            "last vane corner {corner:?} is outside the top housing"
        );
    }
}

#[test]
fn openings_are_longer_than_a_pitch_plus_the_vane_overhang() {
    // Arrange
    let minimum = PITCH + VANE_OVERHANG;

    // Act
    let lengths = [INLET_LENGTH, OUTLET_LENGTH];

    // Assert
    assert!(
        lengths.iter().all(|length| *length >= minimum),
        "{lengths:?} < {minimum}"
    );
}

#[test]
fn openings_run_up_to_the_housing_faces() {
    // Arrange
    let slab = floor_slab();
    let middle_t = -SLAB_THICKNESS / 2.0;
    let samples: Vec<f32> = (0..=140_u8).map(|step| f32::from(step) * 0.01).collect();

    // Act
    let floor_in_openings: Vec<f32> = samples
        .iter()
        .flat_map(|offset| [*offset, TUBE_LENGTH - *offset])
        .filter(|q| {
            let inside_inlet = *q < INLET_LENGTH;
            let inside_outlet = *q > TUBE_LENGTH - OUTLET_LENGTH;
            (inside_inlet || inside_outlet) && quad_contains(&slab, vane_point(*q, middle_t))
        })
        .collect();

    // Assert
    assert!(floor_in_openings.is_empty(), "{floor_in_openings:?}");
}

#[test]
fn vanes_are_thick_and_embedded_within_the_slabs() {
    // Arrange / Act
    let thick_enough = VANE_THICKNESS >= 3.0 * PARTICLE_DIAMETER;
    let embed_fits = VANE_EMBED <= SLAB_THICKNESS && VANE_EMBED >= 0.5 * PARTICLE_DIAMETER;

    // Assert
    assert!(thick_enough, "vane thickness {VANE_THICKNESS}");
    assert!(embed_fits, "vane embed {VANE_EMBED}");
}

#[test]
fn tube_gap_is_six_to_eight_particle_diameters() {
    // Arrange / Act
    let diameters = TUBE_GAP / PARTICLE_DIAMETER;

    // Assert
    assert!((6.0..=8.0).contains(&diameters), "{diameters} diameters");
}

#[test]
fn housings_hold_a_whole_vane_with_margin() {
    // Arrange / Act
    let vane_span = 2.0 * VANE_HALF_SPAN;

    // Assert
    assert!(HOUSING_LENGTH > vane_span, "{vane_span}");
}

#[test]
fn weir_lip_submerges_the_outlet_below_the_top_housing() {
    // Arrange
    let outlet_floor_y = vane_point(TUBE_LENGTH - OUTLET_LENGTH, 0.0).y;
    let housing_low_y = top_housing()
        .iter()
        .map(|corner| corner.y)
        .fold(f32::INFINITY, f32::min);

    // Act
    let between = WEIR_LIP_Y > outlet_floor_y && WEIR_LIP_Y < housing_low_y;

    // Assert
    assert!(
        between,
        "lip {WEIR_LIP_Y}, outlet floor {outlet_floor_y}, housing {housing_low_y}"
    );
}

#[test]
fn the_hood_closes_the_gap_between_the_top_housing_and_the_chute() {
    // Arrange
    let walls = wall_quads();
    let hood = walls[walls.len() - 1];
    let hood_inner_corner = Vec2::new(HOOD_LEFT_X, HOOD_BOTTOM_Y);
    let chute_inner_top = Vec2::new(CHUTE_INNER_X, CHUTE_TOP_Y);

    // Act
    let buried_in_housing = quad_contains(&top_housing(), hood_inner_corner);
    let chute_top_buried = quad_contains(&hood, chute_inner_top);

    // Assert
    assert!(buried_in_housing, "{hood_inner_corner:?}");
    assert!(chute_top_buried, "{chute_inner_top:?}");
}

#[test]
fn every_wall_vertex_lies_inside_the_frame() {
    // Arrange
    let quads = wall_quads();

    // Act
    let outside: Vec<Vec2> = quads
        .iter()
        .flatten()
        .copied()
        .filter(|corner| {
            corner.x < FRAME_MIN_X + FRAME_MARGIN
                || corner.x > FRAME_MAX_X - FRAME_MARGIN
                || corner.y < FRAME_MIN_Y + FRAME_MARGIN
                || corner.y > FRAME_MAX_Y - FRAME_MARGIN
        })
        .collect();

    // Assert
    assert!(outside.is_empty(), "{outside:?}");
}

#[test]
fn water_count_stays_inside_the_particle_budget() {
    // Arrange / Act
    let count = water_positions().len();

    // Assert
    assert!(
        (MIN_SPAWN_COUNT..=MAX_SPAWN_COUNT).contains(&count),
        "{count} particles"
    );
}

#[test]
fn water_spawns_clear_of_every_wall_and_vane() {
    // Arrange
    let mut quads: Vec<[Vec2; 4]> = wall_quads().to_vec();
    quads.extend((0..VANE_COUNT).map(|index| vane_at_phase(index, 0.0)));

    // Act
    let overlapping: Vec<Vec2> = water_positions()
        .into_iter()
        .filter(|point| {
            quads.iter().any(|quad| {
                quad_contains(quad, *point) || distance_to_boundary(quad, *point) < PARTICLE_RADIUS
            })
        })
        .collect();

    // Assert
    assert!(overlapping.is_empty(), "{overlapping:?}");
}

#[test]
fn the_basin_starts_empty() {
    // Arrange / Act
    let in_basin_count = water_positions()
        .into_iter()
        .filter(|point| in_basin(*point))
        .count();

    // Assert
    assert_eq!(in_basin_count, 0);
}

#[test]
fn the_reservoir_fills_to_at_least_the_inlet_seal() {
    // Arrange / Act
    let top = water_positions()
        .into_iter()
        .filter(|point| in_reservoir(*point))
        .map(|point| point.y)
        .fold(f32::NEG_INFINITY, f32::max);

    // Assert
    assert!(top >= INLET_SEAL_Y, "reservoir top {top} < {INLET_SEAL_Y}");
}

#[test]
fn a_vane_inside_a_housing_has_no_outline() {
    // Arrange
    let q_center = -HOUSING_LENGTH / 2.0;

    // Act
    let maybe_outline = visible_vane_outline(q_center);

    // Assert
    assert!(maybe_outline.is_none());
}

#[test]
fn a_vane_cut_by_the_top_housing_face_draws_only_its_outside_part() {
    // Arrange
    let housing = top_housing();

    // Act
    let outline = visible_vane_outline(TUBE_LENGTH).expect("part of the vane is in the tube");

    // Assert
    assert!(outline.len() >= 3, "{outline:?}");
    let inside: Vec<&Vec2> = outline
        .iter()
        .filter(|corner| {
            quad_contains(&housing, **corner) && distance_to_boundary(&housing, **corner) > 1e-4
        })
        .collect();
    assert!(inside.is_empty(), "{inside:?}");
}

#[test]
fn a_vane_cut_by_the_bottom_housing_face_draws_only_its_outside_part() {
    // Arrange
    let housing = bottom_housing();

    // Act
    let outline = visible_vane_outline(0.0).expect("part of the vane is in the tube");

    // Assert
    assert!(outline.len() >= 3, "{outline:?}");
    let inside: Vec<&Vec2> = outline
        .iter()
        .filter(|corner| {
            quad_contains(&housing, **corner) && distance_to_boundary(&housing, **corner) > 1e-4
        })
        .collect();
    assert!(inside.is_empty(), "{inside:?}");
}

#[test]
fn a_mid_tube_vane_draws_its_full_outline() {
    // Arrange
    let q_center = TUBE_LENGTH / 2.0;

    // Act
    let outline = visible_vane_outline(q_center).expect("the vane is in the tube");

    // Assert
    assert_eq!(outline.len(), 4);
    let expected = [
        vane_point(q_center - VANE_HALF_SPAN, -VANE_EMBED),
        vane_point(q_center + VANE_HALF_SPAN, -VANE_EMBED),
        vane_point(q_center + VANE_HALF_SPAN, TUBE_GAP + VANE_EMBED),
        vane_point(q_center - VANE_HALF_SPAN, TUBE_GAP + VANE_EMBED),
    ];
    for (drawn, wanted) in outline.iter().zip(expected) {
        assert!((*drawn - wanted).length() < 1e-5, "{drawn:?} vs {wanted:?}");
    }
}
