//! Pinned `LiquidFun` Surface Tension test: tensile color-mixing groups + falling ball.

use liquidfun::collision::{CircleShape, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{BodyDef, BodyId, ParticleSystemDef, ParticleSystemId, World};

use super::basin_family::{attach_vertical_wall_basin, create_falling_ball};
use super::{BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.035;
const PARTICLE_DAMPING: f32 = 0.2;
const MAXIMUM_PARTICLE_COUNT: usize = 10240;
const GROUP_RADIUS: f32 = 0.5;
const GROUP_FLAGS: ParticleFlags = ParticleFlags::from_bits_truncate(
    ParticleFlags::TENSILE.bits() | ParticleFlags::COLOR_MIXING.bits(),
);
const RED_COLOR: ParticleColor = ParticleColor::new(255, 0, 0, 255);
const GREEN_COLOR: ParticleColor = ParticleColor::new(0, 255, 0, 255);
const BLUE_COLOR: ParticleColor = ParticleColor::new(0, 0, 255, 255);
const RED_CENTER: Vec2 = Vec2::new(0.0, 2.0);
const GREEN_CENTER: Vec2 = Vec2::new(-1.0, 2.0);
const BLUE_BOX_VERTICES: [Vec2; 4] = [
    Vec2::new(0.0, 3.0),
    Vec2::new(2.0, 3.0),
    Vec2::new(2.0, 3.5),
    Vec2::new(0.0, 3.5),
];
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);

struct SurfaceTensionHooks {
    basin_segments: [RigidSegment; 3],
    ball_body: BodyId,
    ball_radius: f32,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_surface_tension().map_err(|_error| SessionError::SceneConstruction)
}

fn build_surface_tension() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let basin_body = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    let basin_segments = attach_vertical_wall_basin(&mut world, basin_body)?;
    let (ball_body, ball_radius) = create_falling_ball(&mut world)?;
    let particle_system = create_tensile_color_groups(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(SurfaceTensionHooks {
            basin_segments,
            ball_body,
            ball_radius,
        }),
    })
}

fn create_tensile_color_groups(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_damping(PARTICLE_DAMPING)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    create_circle_group(world, system, RED_CENTER, RED_COLOR)?;
    create_circle_group(world, system, GREEN_CENTER, GREEN_COLOR)?;
    create_box_group(world, system, BLUE_COLOR)?;
    Ok(system)
}

fn create_circle_group(
    world: &mut World,
    system: ParticleSystemId,
    center: Vec2,
    color: ParticleColor,
) -> Result<(), SceneError> {
    let filled = Shape::from(
        CircleShape::new(Vec2::ZERO, GROUP_RADIUS).map_err(|_error| SceneError::Geometry)?,
    );
    let source =
        ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(GROUP_FLAGS)
        .with_color(color)
        .with_transform(Transform::from_position_angle(center, 0.0))
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(())
}

fn create_box_group(
    world: &mut World,
    system: ParticleSystemId,
    color: ParticleColor,
) -> Result<(), SceneError> {
    let filled =
        Shape::from(PolygonShape::new(&BLUE_BOX_VERTICES).map_err(|_error| SceneError::Geometry)?);
    let source =
        ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(GROUP_FLAGS)
        .with_color(color)
        .with_transform(Transform::IDENTITY)
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(())
}

impl SceneHooks for SurfaceTensionHooks {
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
    use liquidfun::ParticleFlags;

    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::{SessionCore, SessionError};

    #[test]
    fn create_builds_basin_groups_and_ball() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::SurfaceTension)
            .expect("Surface Tension should construct the pinned basin");
        let frame = capture(&session);

        // Assert
        assert!(
            session.particle_count() > 0,
            "tensile color-mixing groups must create at least one particle"
        );
        assert!(
            frame.rigid_circles().len() >= 3,
            "captured frame must export at least one rigid circle (x,y,r)"
        );
    }

    #[test]
    fn constructed_group_flags_include_tensile_and_color_mixing() {
        // Arrange / Act
        let super::BuiltScene {
            world,
            particle_system,
            ..
        } = super::build(&[]).expect("Surface Tension should construct");
        let view = world
            .particle_system_view(particle_system)
            .expect("constructed system should stay live");
        let expected = ParticleFlags::TENSILE | ParticleFlags::COLOR_MIXING;

        // Assert
        assert!(
            view.flags().iter().any(|flags| {
                flags.contains(ParticleFlags::TENSILE)
                    && flags.contains(ParticleFlags::COLOR_MIXING)
            }),
            "group particles should carry {expected:?}"
        );
    }

    #[test]
    fn several_advances_keep_particles_alive() {
        // Arrange
        let mut session = SessionCore::create(SceneId::SurfaceTension)
            .expect("Surface Tension should construct the pinned basin");
        let before = session.particle_count();

        // Act
        for _ in 0..8 {
            session
                .advance(4)
                .expect("Surface Tension advance must stay within the catch-up cap");
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
        let mut session = SessionCore::create(SceneId::SurfaceTension)
            .expect("Surface Tension should construct the pinned basin");
        let before_count = session.particle_count();

        // Act
        let control = session.apply_control("stiffness", "high");
        let action = session.apply_action("refill");

        // Assert
        assert_eq!(control, Err(SessionError::UnknownControl));
        assert_eq!(action, Err(SessionError::UnknownControl));
        assert_eq!(session.particle_count(), before_count);
    }

    #[test]
    fn pointer_is_a_noop_without_mutating_particle_count() {
        // Arrange
        let mut session = SessionCore::create(SceneId::SurfaceTension)
            .expect("Surface Tension should construct the pinned basin");
        let before_count = session.particle_count();

        // Act
        session
            .apply_pointer("down", 0.0, 3.0)
            .expect("watch-first pointer must succeed as a no-op");
        session
            .apply_pointer("move", 0.5, 2.0)
            .expect("watch-first pointer must succeed as a no-op");
        session
            .apply_pointer("up", 0.5, 2.0)
            .expect("watch-first pointer must succeed as a no-op");

        // Assert
        assert_eq!(session.particle_count(), before_count);
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("Surface Tension frame capture should succeed"),
        )
    }
}
