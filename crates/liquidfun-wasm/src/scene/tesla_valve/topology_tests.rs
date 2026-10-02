//! Actual return-to-next-wall topology and revised port integration.
use super::geometry::{
    BORDER_HALF_WIDTH, CircularArc, DRAIN_CENTER, DRAIN_TOP_Y, inlet_center, inlet_direction,
    mirror_y, outlet_center, valve_geometry,
};
use liquidfun::math::Vec2;
use std::f32::consts::TAU;

#[test]
fn every_loop_return_is_the_next_main_wall_including_the_fourth() {
    for forward in [true, false] {
        // Arrange
        let geometry = valve_geometry(forward);
        let angle = TAU / 18.0;
        let amplitude = 0.5 * 0.85 * angle.tan();

        // Act / Assert
        for (stage, arc) in geometry.bypass_arcs.into_iter().enumerate() {
            let side = if stage.is_multiple_of(2) { 1.0 } else { -1.0 };
            let next = Vec2::new(side * amplitude, [3.0, 2.15, 1.3, 0.45][stage]);
            let mut direction = Vec2::new(-side * angle.sin(), -angle.cos());
            let mut normal = Vec2::new(-direction.y, direction.x);
            let next = if forward {
                next
            } else {
                direction.y = -direction.y;
                normal.y = -normal.y;
                mirror_y(next)
            };
            let outer = CircularArc {
                radius: 0.23,
                ..arc
            }
            .point(1.0);
            let inner = CircularArc {
                radius: 0.11,
                ..arc
            }
            .point(1.0);
            let radial = (arc.point(1.0) - arc.center) * (1.0 / arc.radius);
            let tangent = Vec2::new(-radial.y, radial.x) * arc.sweep.signum();
            assert!(
                tangent.dot(direction) > 0.99999,
                "stage {stage} return must point along next main leg"
            );
            assert!(
                ((outer - next).dot(normal) - side * 0.06).abs() < 0.000_001,
                "outer return must occupy next border"
            );
            assert!(
                ((inner - next).dot(normal) + side * 0.06).abs() < 0.000_001,
                "opposite return must occupy parallel border"
            );
            let path = &geometry.outer_paths[usize::from(side < 0.0)];
            let index = path
                .iter()
                .position(|point| (*point - outer).length() < 0.000_001)
                .expect("actual outer return point");
            let continuation = if forward {
                path[index + 1] - path[index]
            } else {
                path[index - 1] - path[index]
            };
            assert!(
                continuation.dot(direction) / continuation.length() > 0.99999,
                "stage {stage} has an old-leg stub or merge turn after the return"
            );
        }
    }
}

#[test]
fn vertical_ports_are_symmetric_and_both_twenty_degree_adapters_are_tangent() {
    // Arrange
    let inlet = inlet_center();
    let outlet = outlet_center();

    // Act / Assert
    assert!((inlet - Vec2::new(-0.093_123_72, 4.272_043)).length() < 0.000_003);
    assert!((outlet - Vec2::new(-0.093_123_72, 0.027_957_2)).length() < 0.000_003);
    assert!((mirror_y(inlet) - outlet).length() < 0.000_003);
    assert_eq!(inlet_direction(), Vec2::new(0.0, -1.0));
    for forward in [true, false] {
        let geometry = valve_geometry(forward);
        for arc in geometry.port_arcs {
            assert!((arc.sweep.abs() - TAU / 18.0).abs() < 0.000_001);
            for radius in [
                arc.radius - BORDER_HALF_WIDTH,
                arc.radius + BORDER_HALF_WIDTH,
            ] {
                let border = CircularArc { radius, ..arc };
                let path = geometry
                    .outer_paths
                    .iter()
                    .find(|path| {
                        path.iter()
                            .any(|point| (*point - border.point(0.0)).length() < 0.000_001)
                    })
                    .expect("actual port border");
                for fraction in [0.0, 1.0] {
                    let point = border.point(fraction);
                    let index = path
                        .iter()
                        .position(|candidate| (*candidate - point).length() < 0.000_001)
                        .expect("port endpoint");
                    let radial = (point - arc.center) * (1.0 / radius);
                    let direction = Vec2::new(-radial.y, radial.x)
                        * arc.sweep.signum()
                        * if forward { 1.0 } else { -1.0 };
                    let incoming = if forward {
                        fraction < 0.5
                    } else {
                        fraction > 0.5
                    };
                    let edge = if incoming {
                        path[index] - path[index - 1]
                    } else {
                        path[index + 1] - path[index]
                    };
                    assert!(
                        edge.dot(direction) / edge.length() > 0.99999,
                        "port adapter introduces a kink"
                    );
                }
            }
        }
    }
}

#[test]
fn drain_is_five_centimeters_above_outlet_and_below_all_bypasses() {
    // Arrange
    let outlet = outlet_center();
    // Act / Assert
    assert!((DRAIN_TOP_Y - 0.08).abs() < 0.000_001);
    assert!((DRAIN_CENTER.y + 0.14).abs() < 0.000_001);
    assert!((0.05..0.055).contains(&(DRAIN_TOP_Y - outlet.y)));
    for forward in [true, false] {
        let geometry = valve_geometry(forward);
        for arc in geometry.bypass_arcs {
            let outer = CircularArc {
                radius: 0.23,
                ..arc
            };
            assert!(
                outer
                    .points()
                    .iter()
                    .all(|point| point.y - 0.03 > DRAIN_TOP_Y)
            );
        }
    }
}
