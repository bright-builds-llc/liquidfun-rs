//! Release-only Fake Ferrofluid outcome tests (D-04, D-07, D-10, D-11, D-21,
//! D-23), driven through the session's pointer, action and control seams:
//! spikes on load, the crown following a dragged magnet, collapse on
//! release, cancel or switch-off, re-rise when switched back on, strength 0
//! staying flat, 60 s held-pointer stability at full strength, liquid staying
//! in the dish through slow full-strength drags to the clamp extremes and a
//! release, gravity 2 and 80 smokes and the README clip timing. Assertions use the binned surface
//! profile with tolerances, never exact trajectories.
//!
//! Debug builds diverge from release, so every test here is ignored in debug.
//! Run them with `cargo test -p liquidfun-wasm --release --lib fake_ferrofluid`.

use liquidfun::math::Vec2;

use super::geometry::{MAGNET_MAX_Y, MAGNET_REST, dish_segments};
use super::test_support::{
    ABOVE_THE_PUDDLE, CROWN_STEP, HoldReport, INSIDE_THE_PUDDLE, INSIDE_THE_PUDDLE_AT_THE_WALL,
    MIN_PEAKS, OFF_CENTER, TARGET_PEAKS, UNDER_THE_DISH, advance, assert_hold_is_bounded,
    escaped_count, first_step_where, out_of_dish_count, profile, run_session_hold, session_lanes,
};
use super::{MAGNET_ACTION, MAGNET_STRENGTH_CONTROL, MAX_STRENGTH_PERCENT};
use crate::ProofFrame;
use crate::scene::SceneId;
use crate::session::SessionCore;

/// Spikes must appear within 1.5 s of the magnet energizing (D-04 floor).
const APPEAR_DEADLINE_STEPS: u32 = 90;
/// Steps the puddle settles into its crown before a drag or a collapse.
const SETTLE_STEPS: u32 = 120;
/// The surface must be flat within 1.5 s of a release or switch-off.
const COLLAPSE_DEADLINE_STEPS: u32 = 90;
/// A 60 s hold at 60 Hz.
const HOLD_STEPS: u32 = 3_600;
/// The gravity stability smoke: 20 s.
const SMOKE_STEPS: u32 = 1_200;
/// Peaks and flatness are sampled every this many steps (0.1 s).
const SAMPLE_EVERY: u32 = 6;
/// How far each drag moves the magnet across the dish, in meters.
const DRAG_DISTANCE: f32 = 1.5;
/// The crown centroid must move at least this fraction of the drag.
const MIN_FOLLOW_FRACTION: f32 = 0.4;
/// Steps the crown gets to follow a drag (1.5 s).
const FOLLOW_STEPS: u32 = 90;
/// Steps between the strength-0 flatness checks (1.5 s and 3 s).
const STRENGTH_ZERO_CHECK_STEPS: u32 = 90;
/// Magnet height inside the puddle for the press-point drag.
const IN_PUDDLE_Y: f32 = 0.3;
/// Magnet height above the puddle for the press-point drag: the clamp's top.
const ABOVE_PUDDLE_Y: f32 = MAGNET_MAX_Y;
/// README clip: samples are 3 steps apart (20 Hz).
const README_SAMPLE_EVERY: u32 = 3;
/// README clip: the crown is checked one sample before the off cue.
const README_CROWN_STEP: u32 = 357;
/// README clip: the `magnet` off cue (sample 120).
const README_OFF_STEP: u32 = 360;
/// README clip: the `magnet` on cue (sample 160).
const README_ON_STEP: u32 = 480;
/// README clip: the last sample before the clip ends.
const README_END_STEP: u32 = 597;
/// Segments the energized marker adds after the dish loops.
const ENERGIZED_MARKER_SEGMENTS: usize = 8;
/// Floats per rigid segment in a frame.
const SEGMENT_FLOATS: usize = 4;
/// Fixed steps per drag-path leg: about 0.01 m per step, a slow drag.
const LEG_STEPS: u16 = 300;
/// Steps the magnet dwells at the end of a drag path before the release.
const DWELL_STEPS: u32 = 120;
/// Steps watched after the release (5 s).
const AFTER_RELEASE_STEPS: u32 = 300;
/// Drag path from rest: up the dish axis to the top of the frame, then
/// across past the right inner wall face to the frame edge. The clamp holds
/// the magnet below `MAGNET_MAX_Y` (WR-02).
const UP_THEN_ACROSS_PATH: [Vec2; 3] = [MAGNET_REST, Vec2::new(0.0, 3.5), Vec2::new(3.2, 3.5)];
/// Drag path from rest: across under the dish to the frame edge, then up
/// past the right inner wall face to the top of the frame.
const ACROSS_THEN_UP_PATH: [Vec2; 3] = [MAGNET_REST, Vec2::new(3.2, -0.35), Vec2::new(3.2, 3.5)];

/// Crown measurements before and after a drag.
#[derive(Debug, Clone, Copy)]
struct DragReport {
    peaks_before: usize,
    maybe_centroid_before: Option<f32>,
    peaks_after: usize,
    maybe_centroid_after: Option<f32>,
}

impl DragReport {
    /// How far the crown centroid moved toward `+x`, the drag direction.
    fn shift(&self) -> Option<f32> {
        Some(self.maybe_centroid_after? - self.maybe_centroid_before?)
    }
}

/// Measurements from a session with the pointer held for a whole run.
#[derive(Debug, Clone, Copy)]
struct HeldRunReport {
    hold: HoldReport,
    built: usize,
    live: usize,
}

/// Containment measurements from a full-strength drag and release.
#[derive(Debug, Clone, Copy)]
struct DragPathReport {
    max_out_of_dish: usize,
    max_escaped: usize,
    built: usize,
    live: usize,
}

/// Measurements from the README clip timing run.
#[derive(Debug, Clone, Copy)]
struct ReadmeReport {
    crown_peaks: usize,
    maybe_flat_at: Option<u32>,
    maybe_rerise_at: Option<u32>,
}

fn new_session() -> SessionCore {
    SessionCore::create(SceneId::FakeFerrofluid).expect("ferrofluid scene")
}

fn press(session: &mut SessionCore, kind: &str, at: Vec2) {
    session
        .apply_pointer(kind, at.x, at.y)
        .expect("pointer events are accepted");
}

fn toggle_magnet(session: &mut SessionCore) {
    session
        .apply_action(MAGNET_ACTION)
        .expect("the magnet action is accepted");
}

fn set_strength(session: &mut SessionCore, percent: u16) {
    let rebuilt = session
        .apply_control(MAGNET_STRENGTH_CONTROL, &percent.to_string())
        .expect("an allowed strength");
    assert!(!rebuilt, "strength is a live control");
}

fn peak_count(session: &SessionCore) -> usize {
    profile(session).peak_count()
}

fn has_crown(session: &SessionCore) -> bool {
    peak_count(session) >= MIN_PEAKS
}

fn is_flat(session: &SessionCore) -> bool {
    profile(session).is_flat()
}

fn frame_segment_count(session: &SessionCore) -> usize {
    let frame = ProofFrame::from(session.capture_frame().expect("frame capture"));
    frame.rigid_segments().len() / SEGMENT_FLOATS
}

/// Records the crown, moves the held magnet `DRAG_DISTANCE` along `+x` and
/// records it again after `FOLLOW_STEPS`.
fn drag_and_measure(session: &mut SessionCore, from: Vec2) -> DragReport {
    let before = profile(session);
    press(session, "move", from + Vec2::new(DRAG_DISTANCE, 0.0));
    advance(session, FOLLOW_STEPS);
    let after = profile(session);
    DragReport {
        peaks_before: before.peak_count(),
        maybe_centroid_before: before.peak_centroid_x(),
        peaks_after: after.peak_count(),
        maybe_centroid_after: after.peak_centroid_x(),
    }
}

/// From load, holds the press at `(0, y)` while the crown forms, then drags.
fn press_point_drag(y: f32) -> DragReport {
    let mut session = new_session();
    let press_point = Vec2::new(0.0, y);
    press(&mut session, "down", press_point);
    advance(&mut session, SETTLE_STEPS);
    drag_and_measure(&mut session, press_point)
}

fn assert_crown_followed(report: &DragReport) {
    assert!(report.peaks_before >= MIN_PEAKS, "precondition: {report:?}");
    assert!(
        report
            .shift()
            .is_some_and(|shift| shift >= MIN_FOLLOW_FRACTION * DRAG_DISTANCE),
        "{report:?}"
    );
    assert!(report.peaks_after >= MIN_PEAKS, "{report:?}");
}

/// From load with the press held at rest, settles the crown, applies
/// `release` and returns the step at which the surface first reads flat.
fn collapse_after(release: impl FnOnce(&mut SessionCore)) -> Option<u32> {
    let mut session = new_session();
    press(&mut session, "down", MAGNET_REST);
    advance(&mut session, SETTLE_STEPS);
    assert!(has_crown(&session), "precondition: crown before release");
    release(&mut session);
    first_step_where(&mut session, COLLAPSE_DEADLINE_STEPS, SAMPLE_EVERY, is_flat)
}

/// A full-strength 60 s hold with the pointer held at `at` from load.
fn held_at_full_strength(at: Vec2) -> HeldRunReport {
    let mut session = new_session();
    set_strength(&mut session, MAX_STRENGTH_PERCENT);
    press(&mut session, "down", at);
    let hold = run_session_hold(&mut session, HOLD_STEPS);
    HeldRunReport {
        hold,
        built: session.particle_count(),
        live: session.live_particle_count().expect("live count"),
    }
}

fn assert_held_run_is_stable(report: &HeldRunReport) {
    assert_hold_is_bounded(&report.hold);
    assert_eq!(report.live, report.built, "no evictions: {report:?}");
}

/// Gravity first (it rebuilds the scene), then full strength and a held
/// press inside the puddle for `SMOKE_STEPS`; returns `(built, live)`.
fn gravity_smoke(magnitude: &str) -> (usize, usize) {
    let mut session = new_session();
    let rebuilt = session
        .apply_control("gravity", magnitude)
        .expect("the gravity slider accepts this magnitude");
    assert!(rebuilt, "gravity rebuilds the session");
    set_strength(&mut session, MAX_STRENGTH_PERCENT);
    press(&mut session, "down", INSIDE_THE_PUDDLE);
    advance(&mut session, SMOKE_STEPS);
    (
        session.particle_count(),
        session.live_particle_count().expect("live count"),
    )
}

/// Advances one fixed step and folds the containment counts into `report`.
fn step_and_watch(session: &mut SessionCore, report: &mut DragPathReport) {
    advance(session, 1);
    let positions = session_lanes(session).0;
    report.max_out_of_dish = report.max_out_of_dish.max(out_of_dish_count(&positions));
    report.max_escaped = report.max_escaped.max(escaped_count(&positions));
}

/// At full strength, presses at `path[0]` while the crown forms, drags the
/// held magnet slowly along every leg of `path`, dwells, releases and
/// watches the liquid for `AFTER_RELEASE_STEPS`.
fn dragged_at_full_strength(path: &[Vec2]) -> DragPathReport {
    let mut session = new_session();
    set_strength(&mut session, MAX_STRENGTH_PERCENT);
    let mut report = DragPathReport {
        max_out_of_dish: 0,
        max_escaped: 0,
        built: session.particle_count(),
        live: 0,
    };
    press(&mut session, "down", path[0]);
    advance(&mut session, SETTLE_STEPS);
    for leg in path.windows(2) {
        for step in 1..=LEG_STEPS {
            let fraction = f32::from(step) / f32::from(LEG_STEPS);
            press(&mut session, "move", leg[0] + (leg[1] - leg[0]) * fraction);
            step_and_watch(&mut session, &mut report);
        }
    }
    for _ in 0..DWELL_STEPS {
        step_and_watch(&mut session, &mut report);
    }
    let end = path[path.len() - 1];
    press(&mut session, "up", end);
    for _ in 0..AFTER_RELEASE_STEPS {
        step_and_watch(&mut session, &mut report);
    }
    report.live = session.live_particle_count().expect("live count");
    report
}

fn assert_drag_stays_in_the_dish(report: &DragPathReport) {
    assert_eq!(report.max_out_of_dish, 0, "{report:?}");
    assert_eq!(report.max_escaped, 0, "{report:?}");
    assert_eq!(report.live, report.built, "no evictions: {report:?}");
}

fn readme_run() -> ReadmeReport {
    let mut session = new_session();
    advance(&mut session, README_CROWN_STEP);
    let crown_peaks = peak_count(&session);
    advance(&mut session, README_OFF_STEP - README_CROWN_STEP);
    toggle_magnet(&mut session);
    let off_window = README_ON_STEP - README_OFF_STEP - README_SAMPLE_EVERY;
    let maybe_flat_at = first_step_where(&mut session, off_window, README_SAMPLE_EVERY, is_flat)
        .map(|step| README_OFF_STEP + step);
    let Some(flat_step) = maybe_flat_at else {
        return ReadmeReport {
            crown_peaks,
            maybe_flat_at,
            maybe_rerise_at: None,
        };
    };
    advance(&mut session, README_ON_STEP - flat_step);
    toggle_magnet(&mut session);
    let maybe_rerise_at = first_step_where(
        &mut session,
        README_END_STEP - README_ON_STEP,
        README_SAMPLE_EVERY,
        has_crown,
    )
    .map(|step| README_ON_STEP + step);
    ReadmeReport {
        crown_peaks,
        maybe_flat_at,
        maybe_rerise_at,
    }
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn load_forms_four_peaks_within_ninety_steps() {
    // Arrange
    let mut session = new_session();

    // Act
    let maybe_step = first_step_where(&mut session, APPEAR_DEADLINE_STEPS, SAMPLE_EVERY, has_crown);

    // Assert
    assert!(
        maybe_step.is_some(),
        "no crown by step {APPEAR_DEADLINE_STEPS}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn default_strength_shows_five_to_nine_spikes() {
    // Arrange
    let mut session = new_session();

    // Act
    advance(&mut session, CROWN_STEP);

    // Assert
    let peaks = peak_count(&session);
    assert!(TARGET_PEAKS.contains(&peaks), "{peaks}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn dragging_the_magnet_off_center_moves_the_crown_toward_it() {
    // Arrange
    let mut session = new_session();
    advance(&mut session, SETTLE_STEPS);
    press(&mut session, "down", MAGNET_REST);

    // Act
    let report = drag_and_measure(&mut session, MAGNET_REST);

    // Assert
    assert_crown_followed(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn dragging_the_magnet_inside_the_puddle_moves_the_crown_toward_it() {
    // Act
    let report = press_point_drag(IN_PUDDLE_Y);

    // Assert
    assert_crown_followed(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn dragging_the_magnet_above_the_puddle_moves_the_crown_toward_it() {
    // Act
    let report = press_point_drag(ABOVE_PUDDLE_Y);

    // Assert
    assert_crown_followed(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn pointer_up_flattens_the_puddle() {
    // Act
    let maybe_step = collapse_after(|session| press(session, "up", MAGNET_REST));

    // Assert
    assert!(
        maybe_step.is_some(),
        "not flat by {COLLAPSE_DEADLINE_STEPS}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn pointer_cancel_flattens_the_puddle() {
    // Act
    let maybe_step = collapse_after(|session| press(session, "cancel", MAGNET_REST));

    // Assert
    assert!(
        maybe_step.is_some(),
        "not flat by {COLLAPSE_DEADLINE_STEPS}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn magnet_action_off_flattens_the_puddle() {
    // Act
    let maybe_step = collapse_after(toggle_magnet);

    // Assert
    assert!(
        maybe_step.is_some(),
        "not flat by {COLLAPSE_DEADLINE_STEPS}"
    );
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn magnet_action_on_reforms_four_peaks_within_ninety_steps() {
    // Arrange
    let mut session = new_session();
    advance(&mut session, SETTLE_STEPS);
    assert!(has_crown(&session), "precondition: crown at rest");
    toggle_magnet(&mut session);
    let maybe_flat = first_step_where(&mut session, COLLAPSE_DEADLINE_STEPS, SAMPLE_EVERY, is_flat);
    assert!(maybe_flat.is_some(), "precondition: flat after off");

    // Act
    toggle_magnet(&mut session);
    let maybe_step = first_step_where(&mut session, APPEAR_DEADLINE_STEPS, SAMPLE_EVERY, has_crown);

    // Assert
    assert!(maybe_step.is_some(), "no crown by {APPEAR_DEADLINE_STEPS}");
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn strength_zero_keeps_the_puddle_flat() {
    // Arrange
    let mut session = new_session();
    set_strength(&mut session, 0);
    let energized_segments = dish_segments().len() + ENERGIZED_MARKER_SEGMENTS;

    // Act
    advance(&mut session, STRENGTH_ZERO_CHECK_STEPS);
    let flat_early = is_flat(&session);
    advance(&mut session, STRENGTH_ZERO_CHECK_STEPS);
    let flat_late = is_flat(&session);

    // Assert
    assert!(flat_early && flat_late, "{flat_early} {flat_late}");
    assert_eq!(frame_segment_count(&session), energized_segments);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn held_under_the_dish_at_full_strength_stays_stable() {
    // Act
    let report = held_at_full_strength(UNDER_THE_DISH);

    // Assert
    assert_held_run_is_stable(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn held_above_the_puddle_at_full_strength_stays_stable() {
    // Act
    let report = held_at_full_strength(ABOVE_THE_PUDDLE);

    // Assert
    assert_held_run_is_stable(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn held_off_center_at_full_strength_stays_stable() {
    // Act
    let report = held_at_full_strength(OFF_CENTER);

    // Assert
    assert_held_run_is_stable(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn held_inside_the_puddle_at_full_strength_stays_stable() {
    // Act
    let report = held_at_full_strength(INSIDE_THE_PUDDLE);

    // Assert
    assert_held_run_is_stable(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn held_inside_the_puddle_at_the_wall_at_full_strength_stays_stable() {
    // Act
    let report = held_at_full_strength(INSIDE_THE_PUDDLE_AT_THE_WALL);

    // Assert
    assert_held_run_is_stable(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn dragging_up_then_past_the_wall_at_full_strength_keeps_the_liquid_in() {
    // Act
    let report = dragged_at_full_strength(&UP_THEN_ACROSS_PATH);

    // Assert
    assert_drag_stays_in_the_dish(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn dragging_past_the_wall_then_up_at_full_strength_keeps_the_liquid_in() {
    // Act
    let report = dragged_at_full_strength(&ACROSS_THEN_UP_PATH);

    // Assert
    assert_drag_stays_in_the_dish(&report);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn gravity_two_smoke_stays_stable() {
    // Act
    let (built, live) = gravity_smoke("2");

    // Assert
    assert_eq!(live, built);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn gravity_eighty_smoke_stays_stable() {
    // Act
    let (built, live) = gravity_smoke("80");

    // Assert
    assert_eq!(live, built);
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only ferrofluid flow (D-23)")]
fn readme_clip_shows_spikes_collapse_and_rerise() {
    // Act
    let report = readme_run();

    // Assert
    assert!(report.crown_peaks >= MIN_PEAKS, "{report:?}");
    assert!(report.maybe_flat_at.is_some(), "{report:?}");
    assert!(report.maybe_rerise_at.is_some(), "{report:?}");
}
