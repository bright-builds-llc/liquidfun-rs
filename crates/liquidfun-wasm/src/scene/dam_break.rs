//! Named Dam Break scene using the documented Medium / Normal defaults.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, ParticleColor, ParticleDef, ParticleFlags,
    ParticleSystemDef, ParticleSystemId, WakePolicy, World,
};

use super::{
    attach_basin_fixture, BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError,
    SceneHooks,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.2;
const PARTICLE_COLUMNS: u8 = 16;
const PARTICLE_ROWS: u8 = 12;
/// Medium water amount remains the documented 16 * 12 basin.
const PARTICLE_COUNT: usize = 16 * 12;
const SMALL_PARTICLE_COLUMNS: u8 = 8;
const SMALL_PARTICLE_ROWS: u8 = 8;
const LARGE_PARTICLE_COLUMNS: u8 = 20;
const LARGE_PARTICLE_ROWS: u8 = 14;
const PARTICLE_SPACING: f32 = 0.32;
const PARTICLE_ORIGIN: Vec2 = Vec2::new(-4.7, 0.4);
const PARTICLE_COLOR: ParticleColor = ParticleColor::new(57, 211, 199, 255);
const DYNAMIC_CIRCLE_RADIUS: f32 = 0.75;
const DYNAMIC_CIRCLE_POSITION: Vec2 = Vec2::new(2.5, 5.5);
const DROP_CIRCLE_POSITION: Vec2 = Vec2::new(2.5, 7.2);
const DROP_WAKE_IMPULSE: Vec2 = Vec2::new(0.0, -0.1);
const MAXIMUM_PARTICLE_COUNT: usize = 512;
const LOW_GRAVITY: Vec2 = Vec2::new(0.0, -6.0);
const NORMAL_GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const HIGH_GRAVITY: Vec2 = Vec2::new(0.0, -16.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WaterAmount {
    Small,
    Medium,
    Large,
}

impl WaterAmount {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "small" => Some(Self::Small),
            "medium" => Some(Self::Medium),
            "large" => Some(Self::Large),
            _ => None,
        }
    }

    fn grid(self) -> (u8, u8) {
        match self {
            Self::Small => (SMALL_PARTICLE_COLUMNS, SMALL_PARTICLE_ROWS),
            Self::Medium => (PARTICLE_COLUMNS, PARTICLE_ROWS),
            Self::Large => (LARGE_PARTICLE_COLUMNS, LARGE_PARTICLE_ROWS),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum GravityPreset {
    Low,
    Normal,
    High,
}

impl GravityPreset {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "low" => Some(Self::Low),
            "normal" => Some(Self::Normal),
            "high" => Some(Self::High),
            _ => None,
        }
    }

    fn vector(self) -> Vec2 {
        match self {
            Self::Low => LOW_GRAVITY,
            Self::Normal => NORMAL_GRAVITY,
            Self::High => HIGH_GRAVITY,
        }
    }
}

struct DamBreakHooks {
    basin_segments: [RigidSegment; 3],
    circle_body: BodyId,
    circle_radius: f32,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let water = preset_value(presets, "water-amount")
        .map(WaterAmount::parse)
        .map_or(Ok(WaterAmount::Medium), |maybe_water| {
            maybe_water.ok_or(SessionError::UnknownControl)
        })?;
    let gravity = preset_value(presets, "gravity")
        .map(GravityPreset::parse)
        .map_or(Ok(GravityPreset::Normal), |maybe_gravity| {
            maybe_gravity.ok_or(SessionError::UnknownControl)
        })?;
    debug_assert_eq!(PARTICLE_COUNT, usize::from(PARTICLE_COLUMNS) * usize::from(PARTICLE_ROWS));
    build_basin(water, gravity).map_err(|_error| SessionError::SceneConstruction)
}

fn preset_value<'a>(presets: &'a [(String, String)], name: &str) -> Option<&'a str> {
    presets
        .iter()
        .rev()
        .find(|(key, _value)| key == name)
        .map(|(_key, value)| value.as_str())
}

fn build_basin(water: WaterAmount, gravity: GravityPreset) -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(gravity.vector())
        .map_err(|_error| SceneError::Gravity)?;

    let basin_body = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_basin_fixture(
        &mut world,
        basin_body,
        &[
            Vec2::new(-6.0, -1.0),
            Vec2::new(6.0, -1.0),
            Vec2::new(6.0, 0.0),
            Vec2::new(-6.0, 0.0),
        ],
    )?;
    attach_basin_fixture(
        &mut world,
        basin_body,
        &[
            Vec2::new(-6.0, 0.0),
            Vec2::new(-5.5, 0.0),
            Vec2::new(-5.5, 8.0),
            Vec2::new(-6.0, 8.0),
        ],
    )?;
    attach_basin_fixture(
        &mut world,
        basin_body,
        &[
            Vec2::new(5.5, 0.0),
            Vec2::new(6.0, 0.0),
            Vec2::new(6.0, 8.0),
            Vec2::new(5.5, 8.0),
        ],
    )?;

    let circle_body = create_dynamic_circle(&mut world)?;
    let particle_system = create_particles(&mut world, water)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(DamBreakHooks {
            basin_segments: [
                RigidSegment {
                    start: Vec2::new(-5.5, 0.0),
                    end: Vec2::new(5.5, 0.0),
                },
                RigidSegment {
                    start: Vec2::new(-5.5, 0.0),
                    end: Vec2::new(-5.5, 8.0),
                },
                RigidSegment {
                    start: Vec2::new(5.5, 0.0),
                    end: Vec2::new(5.5, 8.0),
                },
            ],
            circle_body,
            circle_radius: DYNAMIC_CIRCLE_RADIUS,
        }),
    })
}

fn create_dynamic_circle(world: &mut World) -> Result<BodyId, SceneError> {
    let body_definition = BodyDef::new(BodyType::Dynamic, DYNAMIC_CIRCLE_POSITION, 0.0, true)
        .map_err(|_error| SceneError::Body)?;
    let body = world
        .create_body(&body_definition)
        .map_err(|_error| SceneError::Body)?;
    let circle = CircleShape::new(Vec2::ZERO, DYNAMIC_CIRCLE_RADIUS)
        .map_err(|_error| SceneError::Geometry)?;
    let fixture_definition = FixtureDef::new(
        Shape::from(circle),
        1.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(body, &fixture_definition)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(body)
}

fn create_particles(world: &mut World, water: WaterAmount) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let (columns, rows) = water.grid();

    for row in 0..rows {
        for column in 0..columns {
            let position = Vec2::new(
                PARTICLE_ORIGIN.x + f32::from(column) * PARTICLE_SPACING,
                PARTICLE_ORIGIN.y + f32::from(row) * PARTICLE_SPACING,
            );
            let definition = ParticleDef::default()
                .with_flags(ParticleFlags::WATER)
                .with_color(PARTICLE_COLOR)
                .with_position(position)
                .map_err(|_error| SceneError::Particle)?;
            let receipt = world
                .create_particle_with_def(system, None, &definition)
                .map_err(|_error| SceneError::Particle)?;
            if !receipt.destruction_occurrences().is_empty() {
                return Err(SceneError::Particle);
            }
        }
    }

    Ok(system)
}

fn drop_obstacle(world: &mut World, circle_body: BodyId) -> Result<(), SessionError> {
    world
        .set_body_transform(circle_body, DROP_CIRCLE_POSITION, 0.0)
        .map_err(|_error| SessionError::SceneConstruction)?;
    world
        .apply_body_linear_impulse_to_center(circle_body, DROP_WAKE_IMPULSE, WakePolicy::Wake)
        .map_err(|_error| SessionError::SceneConstruction)?;
    Ok(())
}

fn reset_obstacle(world: &mut World, circle_body: BodyId) -> Result<(), SessionError> {
    world
        .set_body_transform(circle_body, DYNAMIC_CIRCLE_POSITION, 0.0)
        .map_err(|_error| SessionError::SceneConstruction)?;
    Ok(())
}

impl SceneHooks for DamBreakHooks {
    fn on_advance(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        Ok(())
    }

    fn apply_control(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        name: &str,
        value: &str,
    ) -> Result<ControlEffect, SessionError> {
        match name {
            "water-amount" => {
                let Some(_water) = WaterAmount::parse(value) else {
                    return Err(SessionError::UnknownControl);
                };
                Ok(ControlEffect::Recreated)
            }
            "gravity" => {
                let Some(_gravity) = GravityPreset::parse(value) else {
                    return Err(SessionError::UnknownControl);
                };
                Ok(ControlEffect::Recreated)
            }
            _ => Err(SessionError::UnknownControl),
        }
    }

    fn apply_action(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
        name: &str,
    ) -> Result<(), SessionError> {
        match name {
            "drop-obstacle" => drop_obstacle(world, self.circle_body),
            "reset-obstacle" => reset_obstacle(world, self.circle_body),
            _ => Err(SessionError::UnknownControl),
        }
    }

    fn apply_pointer(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        kind: PointerKind,
        _world_x: f32,
        _world_y: f32,
    ) -> Result<(), SessionError> {
        match kind {
            PointerKind::Down | PointerKind::Move | PointerKind::Up | PointerKind::Cancel => {
                Ok(())
            }
        }
    }

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(self.basin_segments.to_vec())
    }

    fn collect_circles(&self, world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        let position = world
            .body_snapshot(self.circle_body)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .position();
        Ok(vec![(position, self.circle_radius)])
    }
}

#[cfg(test)]
mod tests {
    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::SessionCore;

    #[test]
    fn default_create_is_still_medium_normal_basin() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::DamBreak)
            .expect("Dam Break should construct the documented basin");
        let super::BuiltScene { world, .. } =
            super::build(&[]).expect("default presets should build Medium/Normal");

        // Assert
        assert_eq!(session.particle_count(), 192);
        assert_eq!(world.gravity().x.to_bits(), 0.0_f32.to_bits());
        assert_eq!(world.gravity().y.to_bits(), (-10.0_f32).to_bits());
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
        assert_eq!(small_count, 64);
        assert_eq!(large_count, 280);
        assert_eq!(session.particle_count(), 192);
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
        assert_eq!(
            low_scene.world.gravity().y.to_bits(),
            (-6.0_f32).to_bits()
        );
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
}
