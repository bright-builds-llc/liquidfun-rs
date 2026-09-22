//! Pinned `LiquidFun` Rigid Particles test: solid rigid clumps + spinning box.

use liquidfun::collision::{CircleShape, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleGroupDestination, ParticleGroupFlags, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{BodyDef, BodyId, ParticleSystemDef, ParticleSystemId, World};

use super::basin_family::{attach_vertical_wall_basin, create_falling_ball};
use super::{BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.035;
const MAXIMUM_PARTICLE_COUNT: usize = 10240;
const GROUP_RADIUS: f32 = 0.5;
const RED_COLOR: ParticleColor = ParticleColor::new(255, 0, 0, 255);
const GREEN_COLOR: ParticleColor = ParticleColor::new(0, 255, 0, 255);
const BLUE_COLOR: ParticleColor = ParticleColor::new(0, 0, 255, 255);
const RED_CENTER: Vec2 = Vec2::new(0.0, 3.0);
const GREEN_CENTER: Vec2 = Vec2::new(-1.0, 3.0);
const BLUE_BOX_CENTER: Vec2 = Vec2::new(1.0, 4.0);
const BLUE_BOX_HALF_WIDTH: f32 = 1.0;
const BLUE_BOX_HALF_HEIGHT: f32 = 0.5;
const BLUE_BOX_ANGLE: f32 = -0.5;
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);

struct RigidParticlesHooks {
    basin_segments: [RigidSegment; 3],
    ball_body: BodyId,
    ball_radius: f32,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_rigid_particles().map_err(|_error| SessionError::SceneConstruction)
}

fn build_rigid_particles() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let basin_body = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    let basin_segments = attach_vertical_wall_basin(&mut world, basin_body)?;
    let (ball_body, ball_radius) = create_falling_ball(&mut world)?;
    let particle_system = create_rigid_groups(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(RigidParticlesHooks {
            basin_segments,
            ball_body,
            ball_radius,
        }),
    })
}

fn create_rigid_groups(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    create_circle_group(world, system, RED_CENTER, RED_COLOR)?;
    create_circle_group(world, system, GREEN_CENTER, GREEN_COLOR)?;
    create_spinning_box_group(world, system)?;
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
        .with_group_flags(ParticleGroupFlags::RIGID | ParticleGroupFlags::SOLID)
        .with_color(color)
        .with_transform(Transform::from_position_angle(center, 0.0))
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(())
}

fn create_spinning_box_group(
    world: &mut World,
    system: ParticleSystemId,
) -> Result<(), SceneError> {
    let filled = Shape::from(
        PolygonShape::oriented_box(
            BLUE_BOX_HALF_WIDTH,
            BLUE_BOX_HALF_HEIGHT,
            Vec2::ZERO,
            0.0,
        )
        .map_err(|_error| SceneError::Geometry)?,
    );
    let source =
        ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_group_flags(ParticleGroupFlags::RIGID | ParticleGroupFlags::SOLID)
        .with_color(BLUE_COLOR)
        .with_transform(Transform::from_position_angle(BLUE_BOX_CENTER, BLUE_BOX_ANGLE))
        .map_err(|_error| SceneError::Particle)?
        .with_angular_velocity(2.0)
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(())
}

impl SceneHooks for RigidParticlesHooks {
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
    use liquidfun::particle::{ParticleFlags, ParticleGroupFlags};

    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::{SessionCore, SessionError};

    #[test]
    fn create_builds_basin_groups_and_ball() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::RigidParticles)
            .expect("Rigid Particles should construct the pinned basin");
        let frame = capture(&session);

        // Assert
        assert!(
            session.particle_count() > 0,
            "rigid clumps must create at least one particle"
        );
        assert!(
            frame.rigid_circles().len() >= 3,
            "captured frame must export at least one rigid circle (x,y,r)"
        );
    }

    #[test]
    fn constructed_groups_use_rigid_and_solid_without_elastic_particle_flags() {
        // Arrange / Act
        let super::BuiltScene {
            world,
            particle_system,
            ..
        } = super::build(&[]).expect("Rigid Particles should construct");
        let view = world
            .particle_system_view(particle_system)
            .expect("constructed system should stay live");

        let has_elastic = view
            .flags()
            .iter()
            .any(|flags| flags.contains(ParticleFlags::ELASTIC));
        let has_spring = view
            .flags()
            .iter()
            .any(|flags| flags.contains(ParticleFlags::SPRING));

        let mut rigid_solid_group_count = 0usize;
        let mut seen = Vec::new();
        for maybe_group in view.group_ids() {
            let Some(group) = maybe_group else {
                continue;
            };
            if seen.contains(group) {
                continue;
            }
            seen.push(*group);
            let group_view = world
                .particle_group_view(*group)
                .expect("group should stay live");
            let flags = group_view.flags();
            if flags.contains(ParticleGroupFlags::RIGID) && flags.contains(ParticleGroupFlags::SOLID)
            {
                rigid_solid_group_count += 1;
            }
        }

        // Assert
        assert!(
            !has_elastic,
            "Rigid Particles must not carry ParticleFlags::ELASTIC"
        );
        assert!(
            !has_spring,
            "Rigid Particles must not carry ParticleFlags::SPRING"
        );
        assert!(
            rigid_solid_group_count >= 3,
            "all three clumps must carry ParticleGroupFlags::RIGID | SOLID"
        );
    }

    #[test]
    fn blue_box_group_has_pinned_angle_and_spin() {
        // Arrange / Act
        let super::BuiltScene {
            world,
            particle_system,
            ..
        } = super::build(&[]).expect("Rigid Particles should construct");
        let view = world
            .particle_system_view(particle_system)
            .expect("constructed system should stay live");

        let mut seen = Vec::new();
        let mut found_spinning_box = false;
        for maybe_group in view.group_ids() {
            let Some(group) = maybe_group else {
                continue;
            };
            if seen.contains(group) {
                continue;
            }
            seen.push(*group);
            let group_view = world
                .particle_group_view(*group)
                .expect("group should stay live");
            let angle = group_view.angle();
            let angular_velocity = group_view.angular_velocity();
            if (angle - (-0.5)).abs() < 1e-3 && (angular_velocity - 2.0).abs() < 1e-3 {
                found_spinning_box = true;
                break;
            }
        }

        // Assert
        assert!(
            found_spinning_box,
            "blue box group must use transform angle ≈ -0.5 and angular velocity 2.0"
        );
    }

    #[test]
    fn several_advances_do_not_panic() {
        // Arrange
        let mut session = SessionCore::create(SceneId::RigidParticles)
            .expect("Rigid Particles should construct the pinned basin");

        // Act / Assert
        for _ in 0..8 {
            session
                .advance(4)
                .expect("Rigid Particles advance must stay within the catch-up cap");
        }
        assert!(session.particle_count() > 0);
    }

    #[test]
    fn unknown_control_and_action_are_rejected() {
        // Arrange
        let mut session = SessionCore::create(SceneId::RigidParticles)
            .expect("Rigid Particles should construct the pinned basin");
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
        let mut session = SessionCore::create(SceneId::RigidParticles)
            .expect("Rigid Particles should construct the pinned basin");
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
                .expect("Rigid Particles frame capture should succeed"),
        )
    }
}
