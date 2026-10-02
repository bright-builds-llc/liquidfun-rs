//! Live-control, containment, and draining regressions for small particles.

use liquidfun::math::Vec2;

use super::super::SceneId;
use super::geometry::{DRAIN_TOP_Y, inlet_center, inlet_direction, valve_geometry};
use super::tests::inside_boundary;
use crate::session::SessionCore;

#[test]
fn zero_rate_and_direction_changes_preserve_source_selection() {
    // Arrange
    let mut session = SessionCore::create(SceneId::TeslaValve).expect("valve should construct");
    session
        .apply_control("flow-rate", "0")
        .expect("source should stop");

    // Act
    for direction in ["-1", "1"] {
        session
            .apply_control("flow-direction", direction)
            .expect("direction should apply");
        session.advance(4).expect("valve should step");
        assert_eq!(session.live_particle_count().expect("count"), 0);
    }
    session
        .apply_control("flow-rate", "2880")
        .expect("maximum rate");
    session
        .apply_control("flow-direction", "-1")
        .expect("direction should apply");
    session.advance(1).expect("valve should step");

    // Assert
    assert_eq!(session.live_particle_count().expect("count"), 48);
    session
        .apply_control("flow-direction", "1")
        .expect("forward should apply");
    session.advance(1).expect("valve should step");
    assert_eq!(session.live_particle_count().expect("count"), 48);
    assert!(session.apply_control("flow-direction", "0").is_err());
}

#[test]
fn forward_drains_earlier_and_holds_less_water_than_reverse() {
    // Arrange
    let mut forward = SessionCore::create(SceneId::TeslaValve).expect("valve should construct");
    let mut reverse = SessionCore::create(SceneId::TeslaValve).expect("valve should construct");
    reverse
        .apply_control("flow-direction", "-1")
        .expect("reverse should apply");

    // Act
    let (forward_drained, maybe_forward_arrival) = run_contained(&mut forward, true, 180, 24);
    let (reverse_drained, maybe_reverse_arrival) = run_contained(&mut reverse, false, 180, 24);

    // Assert
    let first = maybe_forward_arrival.expect("forward should reach the drain");
    assert!(maybe_reverse_arrival.is_none_or(|reverse_first| first < reverse_first));
    assert!(
        forward_drained > reverse_drained + 300,
        "forward drained {forward_drained}, reverse {reverse_drained}"
    );
    assert!(forward.live_particle_count().expect("count") > 24);
}

#[test]
fn sustained_default_and_maximum_sources_drain_inside_the_conduit() {
    for (rate, emitted_per_step) in [(1440, 24), (2880, 48)] {
        for forward in [true, false] {
            // Arrange
            let mut session =
                SessionCore::create(SceneId::TeslaValve).expect("valve should construct");
            session
                .apply_control("flow-rate", &rate.to_string())
                .expect("rate should apply");
            if !forward {
                session
                    .apply_control("flow-direction", "-1")
                    .expect("reverse should apply");
            }

            // Act
            let (drained, _) = run_contained(&mut session, forward, 240, emitted_per_step);

            // Assert
            if forward {
                assert!(drained > 1_000, "rate {rate} drained {drained}");
            }
            assert!(session.live_particle_count().expect("count") < 16_384);
        }
    }
}

#[test]
fn stopping_source_keeps_draining_the_smooth_valve() {
    // Arrange
    let mut session = SessionCore::create(SceneId::TeslaValve).expect("valve should construct");
    run_contained(&mut session, true, 180, 24);
    let before = session.live_particle_count().expect("count");

    // Act
    session
        .apply_control("flow-rate", "0")
        .expect("source should stop");
    run_contained(&mut session, true, 120, 0);
    let after = session.live_particle_count().expect("count");

    // Assert
    assert!(
        after + 300 < before,
        "stopped source held {before} then {after}"
    );
}

pub(super) fn run_contained(
    session: &mut SessionCore,
    forward: bool,
    steps: u32,
    emitted_per_step: usize,
) -> (usize, Option<u32>) {
    let geometry = valve_geometry(forward);
    let boundary: Vec<Vec2> = geometry.outer_paths[0]
        .iter()
        .copied()
        .chain(geometry.outer_paths[1].iter().copied().rev())
        .collect();
    let starting_count = session.live_particle_count().expect("count");
    let inlet = inlet_center();
    let direction = inlet_direction();
    let mut maybe_arrival = None;
    for elapsed in (4..=steps).step_by(4) {
        session.advance(4).expect("valve should step");
        let points = session.read_particles(|world, system| {
            world
                .particle_system_view(system)
                .expect("system")
                .positions()
                .to_vec()
        });
        for point in points {
            assert!(point.x.is_finite() && point.y.is_finite());
            assert!(point.y >= DRAIN_TOP_Y, "drain left a particle {point:?}");
            if (point - inlet).dot(direction) >= 0.0 {
                assert!(
                    inside_boundary(point, &boundary),
                    "particle escaped at {point:?}, step {elapsed}, forward {forward}, emissions/step {emitted_per_step}"
                );
                for island in &geometry.splitters {
                    assert!(
                        !inside_boundary(point, &island.outline),
                        "particle entered solid island at {point:?}"
                    );
                }
            }
        }
        let emitted = elapsed as usize * emitted_per_step;
        if maybe_arrival.is_none()
            && session.live_particle_count().expect("count") < starting_count + emitted
        {
            maybe_arrival = Some(elapsed);
        }
    }
    let total_emitted = steps as usize * emitted_per_step;
    (
        starting_count + total_emitted - session.live_particle_count().expect("count"),
        maybe_arrival,
    )
}
