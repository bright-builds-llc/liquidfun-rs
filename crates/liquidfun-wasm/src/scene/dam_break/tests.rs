use crate::ProofFrame;
use crate::scene::SceneId;
use crate::session::SessionCore;

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_bench_recipe_matches_live_medium_normal_scene() {
    // Arrange
    use crate::dam_break_bench as recipe;

    // Act / Assert
    assert_eq!(
        recipe::RECIPE_PARTICLE_RADIUS.to_bits(),
        super::PARTICLE_RADIUS.to_bits()
    );
    assert_eq!(
        recipe::RECIPE_PARTICLE_SPACING.to_bits(),
        super::PARTICLE_SPACING.to_bits()
    );
    assert_eq!(recipe::RECIPE_PARTICLE_COLUMNS, super::PARTICLE_COLUMNS);
    assert_eq!(recipe::RECIPE_PARTICLE_ROWS, super::PARTICLE_ROWS);
    assert_eq!(recipe::EXPECTED_PARTICLE_COUNT, super::PARTICLE_COUNT);
    assert_eq!(
        recipe::RECIPE_ORIGIN_X.to_bits(),
        super::PARTICLE_ORIGIN.x.to_bits()
    );
    assert_eq!(
        recipe::RECIPE_ORIGIN_Y.to_bits(),
        super::PARTICLE_ORIGIN.y.to_bits()
    );
    assert_eq!(
        recipe::RECIPE_CIRCLE_RADIUS.to_bits(),
        super::DYNAMIC_CIRCLE_RADIUS.to_bits()
    );
    assert_eq!(
        recipe::RECIPE_CIRCLE_X.to_bits(),
        super::DYNAMIC_CIRCLE_POSITION.x.to_bits()
    );
    assert_eq!(
        recipe::RECIPE_CIRCLE_Y.to_bits(),
        super::DYNAMIC_CIRCLE_POSITION.y.to_bits()
    );
    assert_eq!(
        recipe::RECIPE_GRAVITY_Y.to_bits(),
        super::NORMAL_GRAVITY.y.to_bits()
    );
}

#[test]
fn default_create_is_still_medium_normal_basin() {
    // Arrange / Act
    let session = SessionCore::create(SceneId::DamBreak)
        .expect("Dam Break should construct the documented basin");
    let super::BuiltScene {
        world,
        particle_system,
        ..
    } = super::build(&[]).expect("default presets should build Medium/Normal");
    let frame = capture(&session);
    let system = world
        .particle_system_snapshot(particle_system)
        .expect("Dam Break particle system should remain live");

    // Assert
    assert_eq!(session.particle_count(), 1920);
    assert_eq!(world.gravity().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(world.gravity().y.to_bits(), (-10.0_f32).to_bits());
    assert_eq!(system.definition().maximum_count(), Some(10240));
    assert_eq!(
        frame.particle_radii(),
        vec![0.063_245_55; 1920].into_boxed_slice()
    );
}

#[test]
fn water_amount_presets_recreate_with_locked_counts() {
    // Arrange
    let mut session = SessionCore::create(SceneId::DamBreak)
        .expect("Dam Break should construct the documented basin");

    // Act
    let small = session
        .apply_control("water-amount", "small")
        .expect("water-amount=small should recreate");
    let small_count = session.particle_count();
    let large = session
        .apply_control("water-amount", "large")
        .expect("water-amount=large should recreate");
    let large_count = session.particle_count();
    let medium = session
        .apply_control("water-amount", "medium")
        .expect("water-amount=medium should recreate");

    // Assert
    assert!(small, "water-amount must return Recreated");
    assert!(large, "water-amount must return Recreated");
    assert!(medium, "water-amount must return Recreated");
    assert_eq!(small_count, 650);
    assert_eq!(large_count, 2772);
    assert_eq!(session.particle_count(), 1920);
}

#[test]
fn gravity_presets_recreate_with_matching_bits() {
    // Arrange
    let mut session = SessionCore::create(SceneId::DamBreak)
        .expect("Dam Break should construct the documented basin");

    // Act
    let low = session
        .apply_control("gravity", "low")
        .expect("gravity=low should recreate");
    let high = session
        .apply_control("gravity", "high")
        .expect("gravity=high should recreate");
    let normal = session
        .apply_control("gravity", "normal")
        .expect("gravity=normal should recreate");
    let low_scene = super::build(&[("gravity".to_owned(), "low".to_owned())])
        .expect("low gravity preset should construct");
    let high_scene = super::build(&[("gravity".to_owned(), "high".to_owned())])
        .expect("high gravity preset should construct");

    // Assert
    assert!(low, "gravity must return Recreated");
    assert!(high, "gravity must return Recreated");
    assert!(normal, "gravity must return Recreated");
    assert_eq!(low_scene.world.gravity().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(low_scene.world.gravity().y.to_bits(), (-6.0_f32).to_bits());
    assert_eq!(high_scene.world.gravity().x.to_bits(), 0.0_f32.to_bits());
    assert_eq!(
        high_scene.world.gravity().y.to_bits(),
        (-16.0_f32).to_bits()
    );
}

#[test]
fn drop_obstacle_raises_then_falls_without_recreate() {
    // Arrange
    let mut session = SessionCore::create(SceneId::DamBreak)
        .expect("Dam Break should construct the documented basin");
    let before_count = session.particle_count();
    let before = circle_pose(&session);

    // Act
    session
        .apply_action("drop-obstacle")
        .expect("drop-obstacle should apply live");
    let dropped = circle_pose(&session);
    advance_steps(&mut session, 12);
    let fallen = circle_pose(&session);

    // Assert
    assert_eq!(before.0.to_bits(), 2.5_f32.to_bits());
    assert_eq!(before.1.to_bits(), 5.5_f32.to_bits());
    assert_eq!(dropped.0.to_bits(), 2.5_f32.to_bits());
    assert_eq!(dropped.1.to_bits(), 7.2_f32.to_bits());
    assert!(
        fallen.1 < dropped.1,
        "woken obstacle should fall after steps: {} -> {}",
        dropped.1,
        fallen.1
    );
    assert_eq!(session.particle_count(), before_count);
}

#[test]
fn reset_obstacle_restores_documented_bits_without_recreate() {
    // Arrange
    let mut session = SessionCore::create(SceneId::DamBreak)
        .expect("Dam Break should construct the documented basin");
    let before_count = session.particle_count();
    session
        .apply_action("drop-obstacle")
        .expect("drop-obstacle should apply live");
    advance_steps(&mut session, 8);

    // Act
    session
        .apply_action("reset-obstacle")
        .expect("reset-obstacle should apply live");
    let reset = circle_pose(&session);

    // Assert
    assert_eq!(reset.0.to_bits(), 2.5_f32.to_bits());
    assert_eq!(reset.1.to_bits(), 5.5_f32.to_bits());
    assert_eq!(session.particle_count(), before_count);
}

#[test]
fn pointer_drag_moves_circle_and_drop_obstacle_still_teleports() {
    // Arrange
    let mut session = SessionCore::create(SceneId::DamBreak)
        .expect("Dam Break should construct the documented basin");

    // Act
    session
        .apply_pointer("down", 0.0, 4.0)
        .expect("down should start captured drag");
    session
        .apply_pointer("move", 1.0, 5.0)
        .expect("move should relocate the circle");
    let dragged = circle_pose(&session);
    session
        .apply_action("drop-obstacle")
        .expect("drop-obstacle should still teleport");
    let dropped = circle_pose(&session);

    // Assert
    assert!(
        (dragged.0 - 1.0).abs() < 0.05 && (dragged.1 - 5.0).abs() < 0.05,
        "drag should move the existing circle near (1.0, 5.0), got {dragged:?}"
    );
    assert_eq!(dropped.0.to_bits(), 2.5_f32.to_bits());
    assert_eq!(dropped.1.to_bits(), 7.2_f32.to_bits());
}

#[test]
fn pointer_cancel_leaves_pose_and_up_applies_wake() {
    // Arrange
    let mut cancel_session = SessionCore::create(SceneId::DamBreak)
        .expect("Dam Break should construct the documented basin");
    cancel_session
        .apply_pointer("down", 1.0, 5.0)
        .expect("down should start captured drag");
    cancel_session
        .apply_pointer("move", 1.0, 5.0)
        .expect("move should relocate the circle");
    let canceled_pose = circle_pose(&cancel_session);

    let mut up_session = SessionCore::create(SceneId::DamBreak)
        .expect("Dam Break should construct the documented basin");
    up_session
        .apply_pointer("down", 1.0, 5.0)
        .expect("down should start captured drag");
    up_session
        .apply_pointer("move", 1.0, 5.0)
        .expect("move should relocate the circle");

    // Act
    cancel_session
        .apply_pointer("cancel", 1.0, 5.0)
        .expect("cancel should leave the last pose");
    let canceled_after = circle_pose(&cancel_session);
    up_session
        .apply_pointer("up", 1.0, 5.0)
        .expect("up should wake the obstacle");
    let up_release = circle_pose(&up_session);
    advance_steps(&mut up_session, 12);
    let up_fallen = circle_pose(&up_session);

    // Assert
    assert!(
        (canceled_after.0 - canceled_pose.0).abs() < 0.05
            && (canceled_after.1 - canceled_pose.1).abs() < 0.05,
        "cancel must leave the last drag pose, got {canceled_after:?}"
    );
    assert!(
        up_fallen.1 < up_release.1,
        "woken obstacle should fall after up: {} -> {}",
        up_release.1,
        up_fallen.1
    );
}

#[test]
fn pointer_down_clamps_letterboxed_sample_inside_the_basin() {
    // Arrange
    let mut session = SessionCore::create(SceneId::DamBreak)
        .expect("Dam Break should construct the documented basin");

    // Act
    session
        .apply_pointer("down", -8.0, 20.0)
        .expect("letterboxed down should clamp");
    let pose = circle_pose(&session);

    // Assert
    assert!(
        (-5.5..=5.5).contains(&pose.0),
        "clamped x should stay in the basin, got {}",
        pose.0
    );
    assert!(
        (0.75..=7.25).contains(&pose.1),
        "clamped y should stay in the basin, got {}",
        pose.1
    );
    assert!((pose.0 - (-5.5)).abs() < 0.05);
    assert!((pose.1 - 7.25).abs() < 0.05);
}

#[test]
fn unknown_water_gravity_and_obstacle_tokens_fail_closed() {
    // Arrange
    let mut session = SessionCore::create(SceneId::DamBreak)
        .expect("Dam Break should construct the documented basin");

    // Act
    let bad_water = session.apply_control("water-amount", "huge");
    let bad_gravity = session.apply_control("gravity", "zero");
    let unknown_name = session.apply_control("emission-rate", "medium");
    let unknown_action = session.apply_action("poke-jelly");

    // Assert
    assert_eq!(bad_water, Err(crate::session::SessionError::UnknownControl));
    assert_eq!(
        bad_gravity,
        Err(crate::session::SessionError::UnknownControl)
    );
    assert_eq!(
        unknown_name,
        Err(crate::session::SessionError::UnknownControl)
    );
    assert_eq!(
        unknown_action,
        Err(crate::session::SessionError::UnknownControl)
    );
}

/// Headless regression for dam-break wall embedding / velocity blow-ups.
///
/// Before the force-buffer zeroing fix, particles tunneled the left wall by ~step 135.
#[test]
fn dam_break_stays_finite_and_outside_walls_after_settling() {
    use liquidfun::{NoDecisionHook, StepConfiguration, StepLimits};

    // Arrange
    let super::BuiltScene {
        mut world,
        particle_system,
        particle_radius,
        ..
    } = super::build(&[]).expect("default Dam Break should construct");
    let step = StepConfiguration::new(1.0 / 60.0, 8, 3)
        .expect("valid step")
        .with_particle_iterations(2)
        .expect("two particle iterations");
    let limits = StepLimits::default();
    let mut hook = NoDecisionHook;
    let critical = 2.0 * particle_radius * 60.0; // diameter / dt
    let extreme_speed = critical * 8.0;
    let floor_top = 0.0_f32;
    let left_inner = -5.5_f32;
    let right_inner = 5.5_f32;

    // Act / Assert — tunneling previously appeared by ~step 135 with the stacked-force bug.
    // Default 600 steps (~10 s). Override with LIQUIDFUN_DAM_BREAK_STEPS for longer checks.
    let step_limit = std::env::var("LIQUIDFUN_DAM_BREAK_STEPS")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(600);
    for step_index in 0..step_limit {
        world
            .step(step, &mut hook, limits)
            .unwrap_or_else(|error| panic!("World::step failed at {step_index}: {error:?}"));
        let view = world
            .particle_system_view(particle_system)
            .expect("particle system remains live");
        let mut max_speed = 0.0_f32;
        for (index, (position, velocity)) in view
            .positions()
            .iter()
            .copied()
            .zip(view.velocities().iter().copied())
            .enumerate()
        {
            assert!(
                position.x.is_finite()
                    && position.y.is_finite()
                    && velocity.x.is_finite()
                    && velocity.y.is_finite(),
                "non-finite state at step {step_index} particle {index}: p={position:?} v={velocity:?}"
            );
            let speed = velocity.length();
            if speed > max_speed {
                max_speed = speed;
            }
            assert!(
                speed <= extreme_speed,
                "extreme velocity at step {step_index} particle {index}: speed={speed} (critical≈{critical}) p={position:?} v={velocity:?}"
            );
            let deeply_in_floor =
                position.y < floor_top - particle_radius && position.x > left_inner && position.x < right_inner;
            let deeply_in_left =
                position.x < left_inner - particle_radius && position.y > 0.0 && position.y < 8.0;
            let deeply_in_right =
                position.x > right_inner + particle_radius && position.y > 0.0 && position.y < 8.0;
            assert!(
                !(deeply_in_floor || deeply_in_left || deeply_in_right),
                "particle center embedded in wall at step {step_index} particle {index}: p={position:?} v={velocity:?}"
            );
        }
        if step_index % 600 == 599 {
            eprintln!("dam_break progress step={} max_speed={max_speed:.3}", step_index + 1);
        }
    }
}

fn circle_pose(session: &SessionCore) -> (f32, f32) {
    let circles = capture(session).rigid_circles();
    assert!(
        circles.len() >= 3,
        "Dam Break should capture the existing circle obstacle"
    );
    (circles[0], circles[1])
}

fn advance_steps(session: &mut SessionCore, steps: u32) {
    let mut remaining = steps;
    while remaining > 0 {
        let chunk = remaining.min(4);
        session
            .advance(chunk)
            .expect("bounded native steps should succeed");
        remaining -= chunk;
    }
}

fn capture(session: &SessionCore) -> ProofFrame {
    ProofFrame::from(
        session
            .capture_frame()
            .expect("Dam Break should capture a frame"),
    )
}
