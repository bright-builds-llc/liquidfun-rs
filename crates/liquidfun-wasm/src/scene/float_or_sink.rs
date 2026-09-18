//! Native Float or Sink pool with live cork, wood, and stone density presets.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, ParticleColor, ParticleDef, ParticleFlags,
    ParticleSystemDef, ParticleSystemId, WakePolicy, World,
};

use super::{
    BuiltScene, ControlEffect, RigidSegment, SceneError, SceneHooks, attach_basin_fixture,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.2;
const PARTICLE_COLUMNS: u8 = 15;
const PARTICLE_ROWS: u8 = 12;
const PARTICLE_SPACING: f32 = 0.32;
const PARTICLE_ORIGIN: Vec2 = Vec2::new(-2.24, 0.35);
const PARTICLE_COLOR: ParticleColor = ParticleColor::new(57, 211, 199, 255);
const MAXIMUM_PARTICLE_COUNT: usize = 384;
const DROP_POSITION: Vec2 = Vec2::new(0.0, 6.0);
const DROP_RADIUS: f32 = 0.5;
const MAX_DROPPED_BODIES: usize = 4;
const CORK_DENSITY: f32 = 0.3;
const WOOD_DENSITY: f32 = 0.6;
const STONE_DENSITY: f32 = 2.0;
const WAKE_IMPULSE: Vec2 = Vec2::new(0.0, -0.05);

#[derive(Debug, Clone, Copy, PartialEq)]
enum BodyPreset {
    Cork,
    Wood,
    Stone,
}

impl BodyPreset {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "cork" => Some(Self::Cork),
            "wood" => Some(Self::Wood),
            "stone" => Some(Self::Stone),
            _ => None,
        }
    }

    fn density(self) -> f32 {
        match self {
            Self::Cork => CORK_DENSITY,
            Self::Wood => WOOD_DENSITY,
            Self::Stone => STONE_DENSITY,
        }
    }
}

struct FloatOrSinkHooks {
    basin_segments: [RigidSegment; 3],
    body_preset: BodyPreset,
    dropped: Vec<(BodyId, f32)>,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let _ = presets;
    build_pool().map_err(|_error| SessionError::SceneConstruction)
}

fn build_pool() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(Vec2::new(0.0, -10.0))
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

    let particle_system = create_pool_particles(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(FloatOrSinkHooks {
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
            body_preset: BodyPreset::Wood,
            dropped: Vec::new(),
        }),
    })
}

fn create_pool_particles(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_density(1.0)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    for row in 0..PARTICLE_ROWS {
        for column in 0..PARTICLE_COLUMNS {
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

fn drop_body(world: &mut World, hooks: &mut FloatOrSinkHooks) -> Result<(), SessionError> {
    if hooks.dropped.len() >= MAX_DROPPED_BODIES {
        return Ok(());
    }

    let body_definition = BodyDef::new(BodyType::Dynamic, DROP_POSITION, 0.0, true)
        .map_err(|_error| SessionError::SceneConstruction)?;
    let body = world
        .create_body(&body_definition)
        .map_err(|_error| SessionError::SceneConstruction)?;
    let circle = CircleShape::new(Vec2::ZERO, DROP_RADIUS)
        .map_err(|_error| SessionError::SceneConstruction)?;
    let fixture_definition = FixtureDef::new(
        Shape::from(circle),
        hooks.body_preset.density(),
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .map_err(|_error| SessionError::SceneConstruction)?;
    world
        .create_fixture(body, &fixture_definition)
        .map_err(|_error| SessionError::SceneConstruction)?;
    world
        .apply_body_linear_impulse_to_center(body, WAKE_IMPULSE, WakePolicy::Wake)
        .map_err(|_error| SessionError::SceneConstruction)?;
    hooks.dropped.push((body, DROP_RADIUS));
    Ok(())
}

impl SceneHooks for FloatOrSinkHooks {
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
        if name != "body" {
            return Err(SessionError::UnknownControl);
        }
        let Some(preset) = BodyPreset::parse(value) else {
            return Err(SessionError::UnknownControl);
        };
        self.body_preset = preset;
        Ok(ControlEffect::Live)
    }

    fn apply_action(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
        name: &str,
    ) -> Result<(), SessionError> {
        if name != "drop-body" {
            return Err(SessionError::UnknownControl);
        }
        drop_body(world, self)
    }

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(self.basin_segments.to_vec())
    }

    fn collect_circles(&self, world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        let mut circles = Vec::with_capacity(self.dropped.len());
        for (body, radius) in &self.dropped {
            let position = world
                .body_snapshot(*body)
                .map_err(|_error| SessionError::FrameCaptureFailed)?
                .position();
            circles.push((position, *radius));
        }
        Ok(circles)
    }
}

#[cfg(test)]
mod tests {
    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::{SessionCore, SessionError};

    const Y_SEPARATION_STEPS: u32 = 120;

    #[test]
    fn create_float_or_sink_builds_a_pool_without_dropped_circles() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::FloatOrSink)
            .expect("Float or Sink should construct a native pool");
        let frame = capture(&session);

        // Assert
        assert!((120..=220).contains(&session.particle_count()));
        assert!(session.particle_count() <= 384);
        assert!(frame.particle_count() >= 1);
        assert!(frame.rigid_segments().len() >= 4);
        assert!(frame.rigid_circles().is_empty());
    }

    #[test]
    fn particle_count_stays_at_or_below_the_frame_budget() {
        // Arrange
        let mut session = SessionCore::create(SceneId::FloatOrSink)
            .expect("Float or Sink should construct a native pool");

        // Act
        advance_steps(&mut session, Y_SEPARATION_STEPS);
        let frame = capture(&session);

        // Assert
        assert!(session.particle_count() <= 384);
        assert!(frame.particle_count() <= 384);
    }

    #[test]
    fn cork_finishes_above_stone_after_the_same_native_steps() {
        // Arrange / Act
        let cork_y = dropped_body_y_after_steps("cork", Y_SEPARATION_STEPS);
        let stone_y = dropped_body_y_after_steps("stone", Y_SEPARATION_STEPS);

        // Assert
        assert!(
            cork_y > stone_y,
            "cork y {cork_y} should stay above stone y {stone_y} from native coupling"
        );
        assert!(
            cork_y > 1.0,
            "cork y {cork_y} should remain above the basin floor after contacting the pool"
        );
    }

    fn dropped_body_y_after_steps(preset: &str, steps: u32) -> f32 {
        let mut session = SessionCore::create(SceneId::FloatOrSink)
            .expect("Float or Sink should construct a native pool");
        let recreated = session
            .apply_control("body", preset)
            .expect("body preset should apply live");
        assert!(!recreated, "body preset must not recreate the pool");
        session
            .apply_action("drop-body")
            .expect("drop-body should create one dynamic fixture");
        advance_steps(&mut session, steps);
        let circles = capture(&session).rigid_circles();
        assert_eq!(circles.len(), 3, "one dropped circle reports x, y, radius");
        circles[1]
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
                .expect("Float or Sink should capture a frame"),
        )
    }

    #[test]
    fn unknown_control_names_still_fail_closed() {
        // Arrange
        let mut session = SessionCore::create(SceneId::FloatOrSink)
            .expect("Float or Sink should construct a native pool");

        // Act
        let result = session.apply_control("nope", "x");

        // Assert
        assert_eq!(result, Err(SessionError::UnknownControl));
    }
}
