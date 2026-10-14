//! Release-only Archimedes Screw outcome tests (D-08, D-17, D-19, D-20):
//! lifting for 60 s at the default and maximum speed, reverse carrying water
//! back down, speed 0 holding still, stability at gravity 2 and 80, and the
//! README clip timing. Assertions use region counts with tolerance, never
//! exact trajectories or step indices.
//!
//! Debug builds diverge from release, so every test here is ignored in debug.
//! Run them with `cargo test -p liquidfun-wasm --release --lib archimedes_screw`.

use std::collections::HashSet;

use liquidfun::ParticleId;

use super::geometry::{INLET_COVER_Y, in_basin, in_reservoir};
use super::test_support::{
    FlowReport, advance, positions, region_counts, run_checked, train_transform,
};
use super::{DEFAULT_SCREW_RPM, MAX_SCREW_RPM, SCREW_DIRECTION_CONTROL, SCREW_SPEED_CONTROL};
use crate::ProofFrame;
use crate::scene::SceneId;
use crate::session::SessionCore;

/// One minute at 60 Hz.
const SIXTY_SECONDS_STEPS: u32 = 3_600;
/// The gravity stability smoke: 20 s (D-20).
const SMOKE_STEPS: u32 = 1_200;
/// The first pocket must reach the basin within 5 s (D-09).
const FIRST_POCKET_DEADLINE_STEPS: u32 = 300;
/// Basin particles that count as the first delivered pocket (floor 10; measured 65).
const FIRST_POCKET_MIN: usize = 30;
/// The README clip ends at step 600 (20 Hz samples, 3 steps each); scooped
/// water must be in the basin a sample before that (D-17).
const README_DEADLINE_STEP: u32 = 597;
/// Reservoir particles at load that must be in the basin by the README deadline.
const README_SCOOPED_MIN: usize = 20;
/// Basin particles gained over a 60 s run (D-19).
const BASIN_GAIN_MIN: usize = 100;
/// Basin particles the forward run must reach before the reverse starts.
const REVERSE_START_BASIN_MIN: usize = 120;
/// Steps between basin checks while filling before the reverse.
const REVERSE_FILL_CHUNK: u32 = 60;
/// Reverse run length: 20 s.
const REVERSE_STEPS: u32 = 1_200;
/// Basin loss a reverse run must show (D-19). Only water above the outlet
/// floor can be scooped back, so the basin keeps most of its water.
const REVERSE_BASIN_DROP_MIN: usize = 20;
/// Reservoir gain a reverse run must show (floor 20; measured 121): the
/// pockets and the scooped basin water all return to it.
const REVERSE_RESERVOIR_RISE_MIN: usize = 60;
/// Steps the stopped screw is watched.
const STILL_STEPS: u32 = 120;
/// Lost-region particles allowed at the end, in percent of the built count (Pitfall 6).
const LOST_PERCENT_MAX: usize = 2;

fn session_at_gravity(magnitude: &str) -> SessionCore {
    let mut session = SessionCore::create(SceneId::ArchimedesScrew).expect("screw scene");
    if magnitude != "10" {
        let rebuilt = session
            .apply_control("gravity", magnitude)
            .expect("the gravity slider accepts this magnitude");
        assert!(rebuilt, "gravity rebuilds the session");
    }
    session
}

fn set_speed(session: &mut SessionCore, rpm: u16) {
    let live = session
        .apply_control(SCREW_SPEED_CONTROL, &rpm.to_string())
        .expect("an allowed speed");
    assert!(!live, "speed is a live control");
}

fn basin_count(session: &SessionCore) -> usize {
    region_counts(&positions(session)).basin
}

fn assert_sealed(report: &FlowReport) {
    assert_eq!(report.max_inside_walls, 0, "{report:?}");
    assert_eq!(report.max_inside_vanes, 0, "{report:?}");
    assert_eq!(report.live, report.built, "no evictions: {report:?}");
}

fn assert_lifted_for_a_minute(report: &FlowReport) {
    assert_sealed(report);
    assert!(
        report.end.basin >= report.start.basin + BASIN_GAIN_MIN,
        "{report:?}"
    );
    assert!(report.min_reservoir_surface >= INLET_COVER_Y, "{report:?}");
    assert!(
        report.end.lost * 100 <= report.built * LOST_PERCENT_MAX,
        "{report:?}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw flow (D-19)")]
fn default_speed_lifts_water_for_sixty_seconds() {
    // Arrange
    let mut session = session_at_gravity("10");

    // Act
    let report = run_checked(&mut session, SIXTY_SECONDS_STEPS);

    // Assert
    assert_lifted_for_a_minute(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw flow (D-19)")]
fn max_speed_lifts_water_for_sixty_seconds() {
    // Arrange
    let mut session = session_at_gravity("10");
    set_speed(&mut session, MAX_SCREW_RPM);

    // Act
    let report = run_checked(&mut session, SIXTY_SECONDS_STEPS);

    // Assert
    assert_lifted_for_a_minute(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw flow (D-19)")]
fn first_pocket_reaches_the_basin_within_five_seconds() {
    // Arrange
    let mut session = session_at_gravity("10");

    // Act
    advance(&mut session, FIRST_POCKET_DEADLINE_STEPS);

    // Assert
    let basin = basin_count(&session);
    assert!(basin >= FIRST_POCKET_MIN, "basin {basin}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw flow (D-19)")]
fn readme_clip_shows_scooped_water_reaching_the_basin() {
    // Arrange
    let mut session = session_at_gravity("10");
    let reservoir_ids: HashSet<ParticleId> = session.read_particles(|world, system| {
        let view = world.particle_system_view(system).expect("water");
        view.particle_ids()
            .iter()
            .zip(view.positions())
            .filter(|(_id, point)| in_reservoir(**point))
            .map(|(id, _point)| *id)
            .collect()
    });

    // Act
    advance(&mut session, README_DEADLINE_STEP);

    // Assert
    let scooped = session.read_particles(|world, system| {
        let view = world.particle_system_view(system).expect("water");
        view.particle_ids()
            .iter()
            .zip(view.positions())
            .filter(|(id, point)| in_basin(**point) && reservoir_ids.contains(*id))
            .count()
    });
    assert!(scooped >= README_SCOOPED_MIN, "scooped {scooped}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw flow (D-19)")]
fn reverse_returns_water_to_the_reservoir() {
    // Arrange
    let mut session = session_at_gravity("10");
    let mut filled = 0;
    while basin_count(&session) < REVERSE_START_BASIN_MIN && filled < SIXTY_SECONDS_STEPS {
        advance(&mut session, REVERSE_FILL_CHUNK);
        filled += REVERSE_FILL_CHUNK;
    }
    assert!(
        basin_count(&session) >= REVERSE_START_BASIN_MIN,
        "the basin fills before the reverse"
    );
    let live = session
        .apply_control(SCREW_DIRECTION_CONTROL, "reverse")
        .expect("reverse is allowed");
    assert!(!live, "direction is a live control");

    // Act
    let report = run_checked(&mut session, REVERSE_STEPS);

    // Assert
    assert_sealed(&report);
    assert!(
        report.end.basin + REVERSE_BASIN_DROP_MIN <= report.start.basin,
        "{report:?}"
    );
    assert!(
        report.end.reservoir >= report.start.reservoir + REVERSE_RESERVOIR_RISE_MIN,
        "{report:?}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw flow (D-19)")]
fn speed_zero_holds_the_vanes_still() {
    // Arrange
    let mut session = session_at_gravity("10");
    set_speed(&mut session, 0);
    let frame_before = ProofFrame::from(session.capture_frame().expect("frame"));
    let train_before = train_transform(&session).position();

    // Act
    advance(&mut session, STILL_STEPS);

    // Assert
    let frame_after = ProofFrame::from(session.capture_frame().expect("frame"));
    let train_after = train_transform(&session).position();
    let bits = |values: &[f32]| {
        values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        bits(&frame_after.rigid_segments()),
        bits(&frame_before.rigid_segments())
    );
    assert_eq!(train_after.x.to_bits(), train_before.x.to_bits());
    assert_eq!(train_after.y.to_bits(), train_before.y.to_bits());
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw flow (D-20)")]
fn gravity_two_stays_sealed() {
    // Arrange
    let mut session = session_at_gravity("2");
    set_speed(&mut session, DEFAULT_SCREW_RPM);

    // Act
    let report = run_checked(&mut session, SMOKE_STEPS);

    // Assert
    assert_sealed(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only screw flow (D-20)")]
fn gravity_eighty_stays_sealed() {
    // Arrange
    let mut session = session_at_gravity("80");
    set_speed(&mut session, DEFAULT_SCREW_RPM);

    // Act
    let report = run_checked(&mut session, SMOKE_STEPS);

    // Assert
    assert_sealed(&report);
}
