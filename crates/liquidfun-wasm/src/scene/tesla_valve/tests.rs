//! Circular width, solid fill, physical wall, and source regressions.
use super::super::SceneId;
use super::geometry::{
    BORDER_HALF_WIDTH, BYPASS_RADIUS, CLEAR_WIDTH, CircularArc, FRAME_MAX_X, FRAME_MAX_Y,
    FRAME_MIN_X, FRAME_MIN_Y, PARTICLE_RADIUS, SOURCE_SPACING, TRUNK_RADIUS, WALL_HALF_THICKNESS,
    inlet_center, inlet_direction, island_wall_path, mirror_y, outline_segments, valve_geometry,
    wall_boxes,
};
use crate::session::SessionCore;
use liquidfun::collision::{CircleShape, PolygonShape};
use liquidfun::math::{Transform, Vec2};
use std::f32::consts::TAU;

fn tangent(arc: CircularArc, fraction: f32) -> Vec2 {
    let radial = (arc.point(fraction) - arc.center) * (1.0 / arc.radius);
    Vec2::new(-radial.y, radial.x) * arc.sweep.signum()
}

#[test]
fn bypasses_are_world_space_semicircles_with_constant_radial_width() {
    for forward in [true, false] {
        // Arrange
        let geometry = valve_geometry(forward);
        // Act / Assert
        for (arc, island) in geometry.bypass_arcs.into_iter().zip(geometry.splitters) {
            assert!((arc.radius - BYPASS_RADIUS).abs() < 0.000_001);
            assert!((arc.sweep.abs() - TAU * 0.5).abs() < 0.000_001);
            assert!((island.radius - 0.055).abs() < 0.000_001);
            let outer = CircularArc {
                radius: arc.radius + BORDER_HALF_WIDTH,
                ..arc
            };
            let inner = CircularArc {
                radius: arc.radius - BORDER_HALF_WIDTH,
                ..arc
            };
            for fraction in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let outer_point = outer.point(fraction);
                let inner_point = inner.point(fraction);
                assert!(((outer_point - arc.center).length() - 0.285).abs() < 0.000_001);
                assert!(((inner_point - arc.center).length() - 0.055).abs() < 0.000_001);
                assert!(
                    ((outer_point - inner_point).length() - 2.0 * BORDER_HALF_WIDTH).abs()
                        < 0.000_001
                );
                assert!(
                    ((outer_point - inner_point).length()
                        - 2.0 * WALL_HALF_THICKNESS
                        - CLEAR_WIDTH)
                        .abs()
                        < 0.000_001
                );
            }
            let upper = island.stem[1] - island.stem[0];
            let lower = island.stem[3] - island.stem[2];
            assert!(upper.dot(tangent(arc, 0.0)) / upper.length() > 0.999_99);
            assert!(lower.dot(tangent(arc, 1.0)) / lower.length() > 0.999_99);
        }
    }
}

#[test]
fn trunk_fillets_preserve_circle_radius_width_and_tangent_joins() {
    for forward in [true, false] {
        // Arrange
        let geometry = valve_geometry(forward);
        // Act / Assert
        for arc in geometry.trunk_arcs {
            assert!((arc.radius - TRUNK_RADIUS).abs() < 0.000_001);
            assert!((arc.sweep.abs() - TAU / 9.0).abs() < 0.000_001);
            for radius in [
                TRUNK_RADIUS - BORDER_HALF_WIDTH,
                TRUNK_RADIUS + BORDER_HALF_WIDTH,
            ] {
                let border = CircularArc { radius, ..arc };
                for path in &geometry.outer_paths {
                    if let Some(index) = path
                        .iter()
                        .position(|point| (*point - border.point(0.0)).length() < 0.000_001)
                    {
                        let direction = if forward {
                            tangent(border, 0.0)
                        } else {
                            -tangent(border, 0.0)
                        };
                        let edge = if forward {
                            path[index] - path[index - 1]
                        } else {
                            path[index + 1] - path[index]
                        };
                        assert!(edge.dot(direction) / edge.length() > 0.999_9);
                    }
                }
                for fraction in [0.0, 0.5, 1.0] {
                    assert!(
                        ((border.point(fraction) - arc.center).length() - radius).abs() < 0.000_001
                    );
                }
            }
        }
    }
}

#[test]
fn dense_frames_and_every_wall_fixture_fit_coordinated_bounds() {
    for forward in [true, false] {
        // Arrange
        let geometry = valve_geometry(forward);
        let mut paths = geometry.outer_paths.to_vec();
        paths.extend(geometry.splitters.iter().map(island_wall_path));
        let mut session = SessionCore::create(SceneId::TeslaValve).expect("valve should construct");
        // Act
        session
            .apply_control("flow-direction", if forward { "1" } else { "-1" })
            .expect("direction");
        session.capture_frame().expect("dense capture");
        // Assert
        assert!((400..=450).contains(&session.rigid_shape_count()));
        for (wall, edge) in wall_boxes(&paths)
            .into_iter()
            .zip(paths.iter().flat_map(|path| path.windows(2)))
        {
            let shape = PolygonShape::oriented_box(
                wall.half_length,
                WALL_HALF_THICKNESS,
                wall.center,
                wall.angle,
            )
            .expect("checked wall box");
            for point in [edge[0], (edge[0] + edge[1]) * 0.5, edge[1]] {
                assert!(
                    shape
                        .test_point(Transform::IDENTITY, point)
                        .expect("finite point")
                );
            }
            for point in shape.vertices() {
                assert!(
                    (FRAME_MIN_X..=FRAME_MAX_X).contains(&point.x),
                    "x {} left frame",
                    point.x
                );
                assert!(
                    (FRAME_MIN_Y..=FRAME_MAX_Y).contains(&point.y),
                    "y {} left frame",
                    point.y
                );
            }
        }
    }
}

#[test]
fn islands_have_filled_semicircles_and_convex_tangent_stems() {
    for forward in [true, false] {
        // Arrange
        let geometry = valve_geometry(forward);
        // Act / Assert
        for island in geometry.splitters {
            let circle = CircleShape::new(island.center, island.radius).expect("circle");
            let stem = PolygonShape::new(&island.stem).expect("convex stem");
            for row in 0..=40u16 {
                for column in 0..=40u16 {
                    let point = island.center
                        + Vec2::new(
                            f32::from(column) * 0.0125 - 0.25,
                            f32::from(row) * 0.0125 - 0.25,
                        );
                    if inside_boundary(point, &island.outline) {
                        assert!(
                            circle
                                .test_point(Transform::IDENTITY, point)
                                .expect("point")
                                || stem.test_point(Transform::IDENTITY, point).expect("point"),
                            "island hole at {point:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn channel_union_has_no_crossed_boundaries_and_contains_every_closed_island() {
    for forward in [true, false] {
        // Arrange
        let geometry = valve_geometry(forward);
        let boundary: Vec<Vec2> = geometry.outer_paths[0]
            .iter()
            .copied()
            .chain(geometry.outer_paths[1].iter().copied().rev())
            .collect();

        // Act / Assert
        for first in 0..boundary.len() {
            for second in first + 2..boundary.len() {
                if first == 0 && second + 1 == boundary.len() {
                    continue;
                }
                let a = boundary[first];
                let b = boundary[(first + 1) % boundary.len()];
                let c = boundary[second];
                let d = boundary[(second + 1) % boundary.len()];
                let crosses = (b - a).cross(c - a) * (b - a).cross(d - a) < -0.000_000_000_001
                    && (d - c).cross(a - c) * (d - c).cross(b - c) < -0.000_000_000_001;
                assert!(!crosses, "boundary edges {first}/{second} cross");
            }
        }
        for island in geometry.splitters {
            assert!(
                island
                    .outline
                    .iter()
                    .all(|point| inside_boundary(*point, &boundary)),
                "island lies outside the conduit envelope"
            );
        }
    }
}

#[test]
fn arc_chord_error_stays_below_one_millimeter() {
    // Arrange
    let geometry = valve_geometry(true);
    // Act / Assert
    for island in &geometry.splitters {
        for edge in island.outline[..island.outline.len() - 2].windows(2) {
            let sagitta = island.radius - ((edge[0] + edge[1]) * 0.5 - island.center).length();
            assert!(
                sagitta < 0.001,
                "actual inner semicircle chord error {sagitta}"
            );
        }
    }
    for arc in geometry.bypass_arcs.into_iter().chain(geometry.trunk_arcs) {
        for radius in [
            arc.radius - BORDER_HALF_WIDTH,
            arc.radius + BORDER_HALF_WIDTH,
        ] {
            let border = CircularArc { radius, ..arc };
            for edge in border.points().windows(2) {
                let sagitta = radius - ((edge[0] + edge[1]) * 0.5 - arc.center).length();
                assert!(sagitta < 0.001, "chord error {sagitta}");
            }
        }
    }
}

#[test]
fn physical_bend_faces_keep_the_shared_clear_width() {
    for forward in [true, false] {
        // Arrange
        let geometry = valve_geometry(forward);
        let mut paths = geometry.outer_paths.to_vec();
        paths.extend(geometry.splitters.iter().map(island_wall_path));
        let walls: Vec<PolygonShape> = wall_boxes(&paths)
            .into_iter()
            .map(|wall| {
                PolygonShape::oriented_box(
                    wall.half_length,
                    WALL_HALF_THICKNESS,
                    wall.center,
                    wall.angle,
                )
                .expect("wall")
            })
            .collect();

        // Act / Assert
        for arc in geometry.bypass_arcs {
            for fraction in [0.25, 0.5, 0.75] {
                let point = arc.point(fraction);
                let (wall_index, clearance) = walls
                    .iter()
                    .enumerate()
                    .map(|(index, wall)| {
                        assert!(!wall.test_point(Transform::IDENTITY, point).expect("point"));
                        let vertices = wall.vertices();
                        let distance = (0..vertices.len())
                            .map(|edge| {
                                point_to_segment_distance(
                                    point,
                                    vertices[edge],
                                    vertices[(edge + 1) % vertices.len()],
                                )
                            })
                            .fold(f32::INFINITY, f32::min);
                        (index, distance)
                    })
                    .min_by(|first, second| first.1.total_cmp(&second.1))
                    .expect("walls exist");
                assert!(
                    (clearance - CLEAR_WIDTH * 0.5).abs() < 0.0015,
                    "bend center {:?}, fraction {fraction} has {clearance} m physical clearance; wall {wall_index} {:?}",
                    arc.center,
                    walls[wall_index].vertices()
                );
            }
        }
    }
}

#[test]
fn reflection_preserves_solid_geometry_and_ports() {
    // Arrange
    let forward = valve_geometry(true);
    let reverse = valve_geometry(false);
    // Act / Assert
    for (island, reflected) in forward.splitters.into_iter().zip(reverse.splitters) {
        assert_eq!(mirror_y(island.center), reflected.center);
        assert_eq!(island.stem.map(mirror_y), reflected.stem);
        assert_eq!(
            island.outline.into_iter().map(mirror_y).collect::<Vec<_>>(),
            reflected.outline
        );
    }
    for path in reverse.outer_paths {
        assert!((path[0].y - 4.03794).abs() < 0.05);
        assert!((path[path.len() - 1].y - 0.26206).abs() < 0.05);
    }
    assert_eq!(outline_segments(true).len(), outline_segments(false).len());
}

#[test]
fn maximum_source_slots_fit_inlet_and_emit_aligned_velocity() {
    // Arrange
    let mut scene = super::build_tesla_valve().expect("construction");
    scene
        .hooks
        .apply_control(&mut scene.world, scene.particle_system, "flow-rate", "2880")
        .expect("rate");
    // Act
    scene
        .hooks
        .on_advance(&mut scene.world, scene.particle_system)
        .expect("emission");
    let view = scene
        .world
        .particle_system_view(scene.particle_system)
        .expect("system");
    let direction = inlet_direction();
    let normal = Vec2::new(-direction.y, direction.x);
    // Assert
    assert_eq!(view.positions().len(), 48);
    for (index, point) in view.positions().iter().enumerate() {
        let offset = *point - inlet_center();
        assert!(offset.dot(direction) > PARTICLE_RADIUS);
        assert!(offset.dot(normal).abs() + PARTICLE_RADIUS < CLEAR_WIDTH * 0.5);
        for other in &view.positions()[index + 1..] {
            assert!((*point - *other).length() > SOURCE_SPACING - 0.000_001);
        }
    }
    for velocity in view.velocities() {
        assert!((velocity.length() - 2.0).abs() < 0.000_001);
        assert!(velocity.dot(direction) > 1.999_99);
    }
}

#[test]
fn default_particle_configuration_and_credit_remain_unchanged() {
    // Arrange
    let mut scene = super::build_tesla_valve().expect("construction");
    // Act
    scene
        .hooks
        .on_advance(&mut scene.world, scene.particle_system)
        .expect("source");
    let definition = scene
        .world
        .particle_system_snapshot(scene.particle_system)
        .expect("system")
        .definition();
    // Assert
    assert_eq!(definition.radius().to_bits(), 0.005f32.to_bits());
    assert_eq!(definition.maybe_maximum_count(), Some(16_384));
    assert_eq!(
        scene
            .world
            .particle_system_view(scene.particle_system)
            .expect("system")
            .positions()
            .len(),
        24
    );
}

#[test]
fn rate_validation_accepts_only_canonical_in_range_steps() {
    for rate in ["0", "30", "1440", "2880"] {
        assert!(super::parse_flow_rate(rate).is_some());
    }
    for rate in ["", "01", "-30", "15", "2910", "65536", "1.0", " 30"] {
        assert!(super::parse_flow_rate(rate).is_none());
    }
}

pub(super) fn inside_boundary(point: Vec2, boundary: &[Vec2]) -> bool {
    let mut inside = false;
    for index in 0..boundary.len() {
        let start = boundary[index];
        let end = boundary[(index + 1) % boundary.len()];
        if (start.y > point.y) != (end.y > point.y)
            && point.x < start.x + (point.y - start.y) * (end.x - start.x) / (end.y - start.y)
        {
            inside = !inside;
        }
    }
    inside
}

fn point_to_segment_distance(point: Vec2, start: Vec2, end: Vec2) -> f32 {
    let edge = end - start;
    let fraction = ((point - start).dot(edge) / edge.length_squared()).clamp(0.0, 1.0);
    (point - (start + edge * fraction)).length()
}
