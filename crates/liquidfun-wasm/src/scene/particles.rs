//! Pinned LiquidFun Particles test: open basin, water circle, dynamic ball.

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, ParticleSystemDef, ParticleSystemId, World,
};

use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
    attach_basin_fixture,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.035;
const MAXIMUM_PARTICLE_COUNT: usize = 10240;
const WATER_COLOR: ParticleColor = ParticleColor::new(255, 0, 0, 255);
const WATER_CENTER: Vec2 = Vec2::new(0.0, 3.0);
const WATER_GROUP_RADIUS: f32 = 2.0;
const DYNAMIC_BALL_RADIUS: f32 = 0.5;
const DYNAMIC_BALL_POSITION: Vec2 = Vec2::new(0.0, 8.0);
const DYNAMIC_BALL_DENSITY: f32 = 0.5;
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);

struct ParticlesHooks {
    basin_segments: [RigidSegment; 3],
    ball_body: BodyId,
    ball_radius: f32,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_particles().map_err(|_error| SessionError::SceneConstruction)
}

fn build_particles() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let basin_body = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_basin_fixture(
        &mut world,
        basin_body,
        &[
            Vec2::new(-4.0, -1.0),
            Vec2::new(4.0, -1.0),
            Vec2::new(4.0, 0.0),
            Vec2::new(-4.0, 0.0),
        ],
    )?;
    attach_basin_fixture(
        &mut world,
        basin_body,
        &[
            Vec2::new(-4.0, -0.1),
            Vec2::new(-2.0, -0.1),
            Vec2::new(-2.0, 2.0),
            Vec2::new(-4.0, 3.0),
        ],
    )?;
    attach_basin_fixture(
        &mut world,
        basin_body,
        &[
            Vec2::new(2.0, -0.1),
            Vec2::new(4.0, -0.1),
            Vec2::new(4.0, 3.0),
            Vec2::new(2.0, 2.0),
        ],
    )?;

    let ball_body = create_dynamic_ball(&mut world)?;
    let particle_system = create_water_group(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(ParticlesHooks {
            basin_segments: [
                RigidSegment {
                    start: Vec2::new(-4.0, 0.0),
                    end: Vec2::new(4.0, 0.0),
                },
                RigidSegment {
                    start: Vec2::new(-2.0, 2.0),
                    end: Vec2::new(-4.0, 3.0),
                },
                RigidSegment {
                    start: Vec2::new(2.0, 2.0),
                    end: Vec2::new(4.0, 3.0),
                },
            ],
            ball_body,
            ball_radius: DYNAMIC_BALL_RADIUS,
        }),
    })
}

fn create_dynamic_ball(world: &mut World) -> Result<BodyId, SceneError> {
    let body_definition = BodyDef::new(BodyType::Dynamic, DYNAMIC_BALL_POSITION, 0.0, true)
        .map_err(|_error| SceneError::Body)?;
    let body = world
        .create_body(&body_definition)
        .map_err(|_error| SceneError::Body)?;
    let circle =
        CircleShape::new(Vec2::ZERO, DYNAMIC_BALL_RADIUS).map_err(|_error| SceneError::Geometry)?;
    let fixture_definition = FixtureDef::new(
        Shape::from(circle),
        DYNAMIC_BALL_DENSITY,
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

fn create_water_group(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    let filled = Shape::from(
        CircleShape::new(Vec2::ZERO, WATER_GROUP_RADIUS).map_err(|_error| SceneError::Geometry)?,
    );
    let source =
        ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(ParticleFlags::WATER)
        .with_color(WATER_COLOR)
        .with_transform(Transform::from_position_angle(WATER_CENTER, 0.0))
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(system)
}

impl SceneHooks for ParticlesHooks {
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
        _name: &str,
        _value: &str,
    ) -> Result<ControlEffect, SessionError> {
        Err(SessionError::UnknownControl)
    }

    fn apply_action(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        _name: &str,
    ) -> Result<(), SessionError> {
        Err(SessionError::UnknownControl)
    }

    fn apply_pointer(
        &mut self,
        _world: &mut World,
        _system: ParticleSystemId,
        _kind: PointerKind,
        _world_x: f32,
        _world_y: f32,
    ) -> Result<(), SessionError> {
        Ok(())
    }

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(self.basin_segments.to_vec())
    }

    fn collect_circles(&self, world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        let position = world
            .body_snapshot(self.ball_body)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .position();
        Ok(vec![(position, self.ball_radius)])
    }
}

#[cfg(test)]
mod tests {
    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::{SessionCore, SessionError};

    #[test]
    fn create_builds_open_basin_water_and_ball() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::Particles)
            .expect("Particles should construct the pinned open basin");
        let frame = capture(&session);

        // Assert
        assert!(
            session.particle_count() > 0,
            "water group must create at least one particle"
        );
        assert!(
            session.rigid_shape_count() >= 4,
            "three basin segments plus one dynamic ball"
        );
        assert!(
            frame.rigid_circles().len() >= 3,
            "captured frame must export at least one rigid circle (x,y,r)"
        );
    }

    #[test]
    fn eight_advances_keep_particles_alive() {
        // Arrange
        let mut session = SessionCore::create(SceneId::Particles)
            .expect("Particles should construct the pinned open basin");
        let before = session.particle_count();

        // Act
        for _ in 0..8 {
            session
                .advance(4)
                .expect("Particles advance must stay within the catch-up cap");
        }

        // Assert
        assert!(before > 0);
        assert!(
            session.particle_count() > 0,
            "particle count must not drop to zero after eight capped advances"
        );
    }

    #[test]
    fn unknown_control_and_action_are_rejected() {
        // Arrange
        let mut session = SessionCore::create(SceneId::Particles)
            .expect("Particles should construct the pinned open basin");
        let before_count = session.particle_count();

        // Act
        let control = session.apply_control("water-amount", "medium");
        let action = session.apply_action("drop-obstacle");

        // Assert
        assert_eq!(control, Err(SessionError::UnknownControl));
        assert_eq!(action, Err(SessionError::UnknownControl));
        assert_eq!(session.particle_count(), before_count);
    }

    #[test]
    fn pointer_is_a_noop_without_mutating_particle_count() {
        // Arrange
        let mut session = SessionCore::create(SceneId::Particles)
            .expect("Particles should construct the pinned open basin");
        let before_count = session.particle_count();

        // Act
        session
            .apply_pointer("down", 0.0, 4.0)
            .expect("watch-first pointer must succeed as a no-op");
        session
            .apply_pointer("move", 1.0, 3.0)
            .expect("watch-first pointer must succeed as a no-op");
        session
            .apply_pointer("up", 1.0, 3.0)
            .expect("watch-first pointer must succeed as a no-op");

        // Assert
        assert_eq!(session.particle_count(), before_count);
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("Particles frame capture should succeed"),
        )
    }
}
