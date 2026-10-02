//! Analytic shape, fixture, and actual inlet emission regressions.

use liquidfun::collision::{CircleShape, PolygonShape};
use liquidfun::math::settings::LINEAR_SLOP;
use liquidfun::math::{Transform, Vec2};

use super::super::SceneId;
use super::geometry::{
    CAP_SAMPLES, FRAME_MAX_X, FRAME_MAX_Y, FRAME_MIN_X, FRAME_MIN_Y, OUTER_CURVES, PARTICLE_RADIUS,
    RETURN_SAMPLES, WALL_HALF_THICKNESS, cubic_point, mirror_y, outline_segments, place_stage,
    valve_geometry, wall_boxes,
};
use crate::session::SessionCore;

#[test]
fn dense_frames_capture_in_both_orientations() {
    for direction in ["1", "-1"] {
        // Arrange
        let mut session = SessionCore::create(SceneId::TeslaValve).expect("valve should construct");

        // Act
        session
            .apply_control("flow-direction", direction)
            .expect("orientation should apply");
        session.capture_frame().expect("dense frame should fit");

        // Assert
        assert_eq!(session.rigid_shape_count(), 300);
        assert_eq!(session.live_particle_count().expect("count"), 0);
    }
}

#[test]
fn thick_wall_boxes_follow_drawn_samples_overlap_joins_and_fit_the_frame() {
    for forward in [true, false] {
        // Arrange
        let geometry = valve_geometry(forward);
        let segments = outline_segments(forward);

        // Act / Assert
        let boxes = wall_boxes(&geometry.outer_paths);
        for (wall, edge) in boxes
            .into_iter()
            .zip(geometry.outer_paths.iter().flat_map(|path| path.windows(2)))
        {
            let polygon = PolygonShape::oriented_box(
                wall.half_length,
                WALL_HALF_THICKNESS,
                wall.center,
                wall.angle,
            )
            .expect("dense short segments should form checked wall boxes");
            for point in [edge[0], (edge[0] + edge[1]) * 0.5, edge[1]] {
                assert!(
                    polygon
                        .test_point(Transform::IDENTITY, point)
                        .expect("finite point"),
                    "wall boxes must cover their full drawn centerline and shared endpoints"
                );
            }
            for corner in polygon.vertices() {
                assert!((FRAME_MIN_X..=FRAME_MAX_X).contains(&corner.x));
                assert!((FRAME_MIN_Y..=FRAME_MAX_Y).contains(&corner.y));
            }
            assert!((edge[1] - edge[0]).length() > LINEAR_SLOP);
            assert!(
                segments
                    .iter()
                    .any(|segment| segment.start == edge[0] && segment.end == edge[1])
            );
        }
        for segment in segments {
            for point in [segment.start, segment.end] {
                assert!((FRAME_MIN_X..=FRAME_MAX_X).contains(&point.x));
                assert!((FRAME_MIN_Y..=FRAME_MAX_Y).contains(&point.y));
            }
        }
    }
}

#[test]
fn cubic_chord_error_is_below_one_millimeter() {
    // Arrange
    let tolerance = 0.001;

    // Act
    let error = maximum_cubic_chord_error();

    // Assert
    assert!(error < tolerance, "curve chord error {error}");
}

pub(super) fn maximum_cubic_chord_error() -> f32 {
    let mut maximum: f32 = 0.0;
    for (curve, samples) in OUTER_CURVES.into_iter().zip([CAP_SAMPLES, RETURN_SAMPLES]) {
        for step in 0..samples {
            let start = place_stage(
                cubic_point(curve, f32::from(step) / f32::from(samples)),
                0,
                true,
            );
            let end = place_stage(
                cubic_point(curve, f32::from(step + 1) / f32::from(samples)),
                0,
                true,
            );
            for fraction in [0.25, 0.5, 0.75] {
                let point = place_stage(
                    cubic_point(curve, (f32::from(step) + fraction) / f32::from(samples)),
                    0,
                    true,
                );
                let error = point_to_segment_distance(point, start, end);
                maximum = maximum.max(error);
            }
        }
    }
    maximum
}

#[test]
fn rounded_splitter_caps_and_tangent_cones_have_solid_interiors() {
    for forward in [true, false] {
        // Arrange
        let geometry = valve_geometry(forward);

        // Act / Assert
        for (stage, island) in geometry.splitters.into_iter().enumerate() {
            let circle =
                CircleShape::new(island.center, island.radius).expect("cap should construct");
            let triangle = PolygonShape::new(&island.triangle).expect("cone should construct");
            let tip = island.triangle[2];
            let sign = if stage.is_multiple_of(2) { 1.0 } else { -1.0 };
            assert!(island.center.x * sign > 0.0);
            for tangent in &island.triangle[..2] {
                let radius = *tangent - island.center;
                assert!((radius.length() - island.radius).abs() < 0.000_001);
                assert!(radius.dot(tip - *tangent).abs() < 0.000_001);
            }
            for row in 0..=40u16 {
                for column in 0..=24u16 {
                    let y = island.center.y - 0.5 + f32::from(row) * 0.025;
                    let x = island.center.x - 0.15 + f32::from(column) * 0.0125;
                    let point = Vec2::new(x, y);
                    if inside_boundary(point, &island.outline) {
                        assert!(
                            circle
                                .test_point(Transform::IDENTITY, point)
                                .expect("finite point")
                                || triangle
                                    .test_point(Transform::IDENTITY, point)
                                    .expect("finite point"),
                            "unfilled splitter interior at {point:?}"
                        );
                    }
                }
            }
            let arc = &island.outline[..island.outline.len() - 1];
            for edge in arc.windows(2) {
                let sagitta = island.radius - ((edge[0] + edge[1]) * 0.5 - island.center).length();
                assert!(sagitta < 0.001);
                assert!((edge[1] - edge[0]).length() > LINEAR_SLOP);
            }
        }
    }
}

#[test]
fn reflection_preserves_island_geometry_and_neck_clearance() {
    // Arrange
    let forward = valve_geometry(true);
    let reverse = valve_geometry(false);

    // Act / Assert
    for (island, mirrored) in forward.splitters.into_iter().zip(reverse.splitters) {
        assert_eq!(mirror_y(island.center), mirrored.center);
        assert_eq!(island.triangle.map(mirror_y), mirrored.triangle);
        assert_eq!(
            island
                .outline
                .iter()
                .copied()
                .map(mirror_y)
                .collect::<Vec<_>>(),
            mirrored.outline
        );
    }
    for orientation in [true, false] {
        let geometry = valve_geometry(orientation);
        let walls: Vec<PolygonShape> = wall_boxes(&geometry.outer_paths)
            .into_iter()
            .map(|wall| {
                PolygonShape::oriented_box(
                    wall.half_length,
                    WALL_HALF_THICKNESS,
                    wall.center,
                    wall.angle,
                )
                .expect("wall should construct")
            })
            .collect();
        for island in geometry.splitters {
            let clearance = geometry
                .outer_paths
                .iter()
                .flat_map(|path| path.windows(2))
                .map(|edge| point_to_segment_distance(island.triangle[2], edge[0], edge[1]))
                .fold(f32::INFINITY, f32::min);
            assert!(clearance > 0.09, "neck clearance {clearance}");
            let physical_clearance = walls
                .iter()
                .map(|wall| {
                    wall.distance_to_point(Transform::IDENTITY, island.triangle[2])
                        .expect("finite tip")
                        .distance()
                })
                .fold(f32::INFINITY, f32::min);
            assert!(
                physical_clearance > 0.06,
                "physical neck clearance {physical_clearance}"
            );
        }
    }
}

#[test]
fn default_source_emits_twenty_four_small_particles_per_frame() {
    // Arrange
    let mut scene = super::build_tesla_valve().expect("valve should construct");

    // Act
    scene
        .hooks
        .on_advance(&mut scene.world, scene.particle_system)
        .expect("source should emit");
    let view = scene
        .world
        .particle_system_view(scene.particle_system)
        .expect("system");

    // Assert
    assert_eq!(scene.particle_radius.to_bits(), 0.005f32.to_bits());
    let definition = scene
        .world
        .particle_system_snapshot(scene.particle_system)
        .expect("system definition")
        .definition();
    assert_eq!(definition.radius().to_bits(), 0.005f32.to_bits());
    assert_eq!(definition.maybe_maximum_count(), Some(16_384));
    assert_eq!(view.positions().len(), 24);
}

#[test]
fn fractional_source_credit_emits_the_selected_thirty_per_second() {
    // Arrange
    let mut scene = super::build_tesla_valve().expect("valve should construct");
    scene
        .hooks
        .apply_control(&mut scene.world, scene.particle_system, "flow-rate", "30")
        .expect("rate should apply");

    // Act
    for _ in 0..2 {
        scene
            .hooks
            .on_advance(&mut scene.world, scene.particle_system)
            .expect("source should emit");
    }

    // Assert
    assert_eq!(
        scene
            .world
            .particle_system_view(scene.particle_system)
            .expect("system")
            .positions()
            .len(),
        1
    );
}

#[test]
fn maximum_rate_uses_distinct_nonoverlapping_real_emission_slots() {
    // Arrange
    let mut scene = super::build_tesla_valve().expect("valve should construct");
    scene
        .hooks
        .apply_control(&mut scene.world, scene.particle_system, "flow-rate", "2880")
        .expect("maximum rate");

    // Act
    scene
        .hooks
        .on_advance(&mut scene.world, scene.particle_system)
        .expect("source should emit");
    let view = scene
        .world
        .particle_system_view(scene.particle_system)
        .expect("system");
    let positions = view.positions();

    // Assert
    assert_eq!(positions.len(), 48);
    for (index, point) in positions.iter().enumerate() {
        assert!(point.x.abs() < 0.17 && point.y >= 2.86 && point.y < 2.88);
        for other in &positions[index + 1..] {
            assert!((*point - *other).length() > 2.0 * PARTICLE_RADIUS + 0.003);
        }
    }
}

#[test]
fn rate_validation_accepts_only_canonical_in_range_steps() {
    // Arrange / Act / Assert
    for rate in ["0", "30", "1440", "2880"] {
        assert!(super::parse_flow_rate(rate).is_some());
    }
    for rate in ["", "01", "-30", "15", "2910", "65536", "1.0", " 30"] {
        assert!(
            super::parse_flow_rate(rate).is_none(),
            "accepted invalid rate {rate}"
        );
    }
}

pub(super) fn point_to_segment_distance(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let edge = end - start;
    let fraction = ((point - start).dot(edge) / edge.length_squared()).clamp(0.0, 1.0);
    (point - (start + edge * fraction)).length()
}

pub(super) fn inside_boundary(point: Vec2, boundary: &[Vec2]) -> bool {
    let mut inside = false;
    for index in 0..boundary.len() {
        let start = boundary[index];
        let end = boundary[(index + 1) % boundary.len()];
        if (start.y > point.y) != (end.y > point.y) {
            let crossing_x = start.x + (point.y - start.y) * (end.x - start.x) / (end.y - start.y);
            if point.x < crossing_x {
                inside = !inside;
            }
        }
    }
    inside
}
