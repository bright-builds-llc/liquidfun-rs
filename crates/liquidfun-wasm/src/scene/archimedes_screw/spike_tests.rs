//! Release-only D-02 wrap and leak spike for the vane train (candidate A),
//! kept as regression guards. The pass rules were written before measuring:
//!
//! - P1: no particle center more than half a radius inside a wall or vane.
//! - P2: no non-finite or escaped particle, and no session eviction.
//! - P3: snap steps never reach a higher particle speed than other steps.
//! - P4: every fully sealed pocket holds at least 15 particles (forward, default speed).
//! - P5: the basin gains at least 50 particles (forward, default speed).
//!
//! Debug builds diverge from release, so every test here is ignored in debug.
//! Run them with `cargo test -p liquidfun-wasm --release --lib archimedes_screw`.

use liquidfun::{NoDecisionHook, StepLimits};

use super::test_support::{
    advance, basin_count, escaped, leak_counts, max_speed, sealed_pocket_counts, step_configuration,
};
use super::{DEFAULT_SCREW_RPM, build_parts};
use crate::scene::{SceneHooks, SceneId};
use crate::session::SessionCore;

/// Steps the scene runs before any count starts.
const SETTLE_STEPS: u32 = 60;
/// Measured steps after the settle: 20 s at 60 Hz.
const RUN_STEPS: u32 = 1_200;
/// Leak and escape samples are taken every this many steps.
const SAMPLE_EVERY: u32 = 4;
/// Measured steps at which every sealed pocket is counted (P4).
const POCKET_CHECKPOINTS: [u32; 4] = [300, 600, 900, 1_200];
/// Fewest particles a sealed pocket may hold (P4).
const MIN_POCKET_PARTICLES: usize = 15;
/// Fewest particles the basin must hold at the end of the run (P5).
const MIN_BASIN_PARTICLES: usize = 50;
/// Twice the default speed.
const DOUBLE_RPM: u16 = 2 * DEFAULT_SCREW_RPM;

/// Measurements from one manual spike run.
#[derive(Debug)]
struct SpikeReport {
    max_inside_walls: usize,
    max_inside_vanes: usize,
    max_escaped: usize,
    snap_steps: u32,
    max_snap_speed: f32,
    max_steady_speed: f32,
    /// Sealed pocket counts at each `POCKET_CHECKPOINTS` step.
    pocket_counts: Vec<Vec<usize>>,
    final_basin: usize,
}

/// Builds the scene, runs it the way the session does (`on_advance`, then
/// `world.step`) for the settle plus `steps`, and measures every pass rule.
fn run_spike(rpm: u16, reverse: bool, steps: u32) -> SpikeReport {
    let mut parts = build_parts().expect("screw parts");
    parts.hooks.rpm = rpm;
    parts.hooks.reverse = reverse;
    let step = step_configuration();
    let system = parts.particle_system;
    let mut report = SpikeReport {
        max_inside_walls: 0,
        max_inside_vanes: 0,
        max_escaped: 0,
        snap_steps: 0,
        max_snap_speed: 0.0,
        max_steady_speed: 0.0,
        pocket_counts: Vec::new(),
        final_basin: 0,
    };

    for index in 0..SETTLE_STEPS + steps {
        let previous = parts.hooks.phase;
        parts
            .hooks
            .on_advance(&mut parts.world, system)
            .expect("the vane train advances");
        let next = parts.hooks.phase;
        let snapped = if reverse {
            next > previous
        } else {
            next < previous
        };
        parts
            .world
            .step(step, &mut NoDecisionHook, StepLimits::default())
            .expect("the screw world steps");
        let Some(measured) = index.checked_sub(SETTLE_STEPS).map(|offset| offset + 1) else {
            continue;
        };

        let view = parts.world.particle_system_view(system).expect("water");
        let speed = max_speed(view.velocities());
        if snapped {
            report.snap_steps += 1;
            report.max_snap_speed = report.max_snap_speed.max(speed);
        } else {
            report.max_steady_speed = report.max_steady_speed.max(speed);
        }
        if measured % SAMPLE_EVERY == 0 {
            let train = parts
                .world
                .body_snapshot(parts.hooks.train)
                .expect("live train")
                .transform();
            let leaks = leak_counts(view.positions(), train);
            report.max_inside_walls = report.max_inside_walls.max(leaks.inside_walls);
            report.max_inside_vanes = report.max_inside_vanes.max(leaks.inside_vanes);
            report.max_escaped = report.max_escaped.max(escaped(view.positions()));
        }
        if POCKET_CHECKPOINTS.contains(&measured) {
            report
                .pocket_counts
                .push(sealed_pocket_counts(view.positions(), parts.hooks.phase));
        }
        if measured == steps {
            report.final_basin = basin_count(view.positions());
        }
    }
    report
}

fn assert_no_leaks_or_escapes(report: &SpikeReport) {
    assert_eq!(report.max_inside_walls, 0, "{report:?}");
    assert_eq!(report.max_inside_vanes, 0, "{report:?}");
    assert_eq!(report.max_escaped, 0, "{report:?}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw spike (D-02)")]
fn forward_default_has_no_leaks_or_escapes() {
    // Arrange / Act
    let report = run_spike(DEFAULT_SCREW_RPM, false, RUN_STEPS);

    // Assert
    assert_no_leaks_or_escapes(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw spike (D-02)")]
fn forward_double_speed_has_no_leaks_or_escapes() {
    // Arrange / Act
    let report = run_spike(DOUBLE_RPM, false, RUN_STEPS);

    // Assert
    assert_no_leaks_or_escapes(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw spike (D-02)")]
fn reverse_default_has_no_leaks_or_escapes() {
    // Arrange / Act
    let report = run_spike(DEFAULT_SCREW_RPM, true, RUN_STEPS);

    // Assert
    assert_no_leaks_or_escapes(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw spike (D-02)")]
fn reverse_double_speed_has_no_leaks_or_escapes() {
    // Arrange / Act
    let report = run_spike(DOUBLE_RPM, true, RUN_STEPS);

    // Assert
    assert_no_leaks_or_escapes(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw spike (D-02)")]
fn snap_steps_add_no_speed_burst() {
    // Arrange
    let runs = [
        (DEFAULT_SCREW_RPM, false),
        (DOUBLE_RPM, false),
        (DEFAULT_SCREW_RPM, true),
        (DOUBLE_RPM, true),
    ];

    // Act
    let reports: Vec<SpikeReport> = runs
        .iter()
        .map(|(rpm, reverse)| run_spike(*rpm, *reverse, RUN_STEPS))
        .collect();

    // Assert
    for report in &reports {
        assert!(report.snap_steps > 0, "{report:?}");
        assert!(
            report.max_snap_speed <= report.max_steady_speed,
            "{report:?}"
        );
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw spike (D-02)")]
fn sealed_pockets_hold_water_at_default_speed() {
    // Arrange / Act
    let report = run_spike(DEFAULT_SCREW_RPM, false, RUN_STEPS);

    // Assert
    assert_eq!(
        report.pocket_counts.len(),
        POCKET_CHECKPOINTS.len(),
        "{report:?}"
    );
    for counts in &report.pocket_counts {
        assert!(!counts.is_empty(), "{report:?}");
        assert!(
            counts.iter().all(|count| *count >= MIN_POCKET_PARTICLES),
            "{report:?}"
        );
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw spike (D-02)")]
fn basin_gains_water_at_default_speed() {
    // Arrange / Act
    let report = run_spike(DEFAULT_SCREW_RPM, false, RUN_STEPS);

    // Assert
    assert!(report.final_basin >= MIN_BASIN_PARTICLES, "{report:?}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw spike (D-02)")]
fn session_run_evicts_nothing() {
    // Arrange
    let mut session = SessionCore::create(SceneId::ArchimedesScrew).expect("screw scene");
    let built = session.particle_count();

    // Act
    advance(&mut session, SETTLE_STEPS + RUN_STEPS);

    // Assert
    assert_eq!(session.live_particle_count().expect("live count"), built);
}
