//! Pinned `LiquidFun` Soup Stirrer: soup basin plus paddle on a prismatic rail.

use std::f32::consts::TAU;

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::{
    BodyDef, BodyId, BodyType, FixtureDef, JointDef, JointId, ParticleSystemId, PrismaticJointDef,
    WakePolicy, World,
};

use super::soup_family::{self, SoupFamilyBuilt};
use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
};
use crate::session::SessionError;

/// Pinned `SetDamping(1.0)` from `testSoupStirrer.js`.
/// `soup_family` already builds with `ParticleSystemDef` default damping `1.0`.
const PARTICLE_DAMPING: f32 = 1.0;
const PADDLE_CENTER: Vec2 = Vec2::new(0.0, 0.7);
const PADDLE_RADIUS: f32 = 0.4;
const PADDLE_DENSITY: f32 = 1.0;
const FORCE_MAGNITUDE: f32 = 10.0;
const FORCE_OSCILLATION_PER_SECOND: f32 = 0.2;
const FORCE_OSCILLATION_PERIOD: f32 = 1.0 / FORCE_OSCILLATION_PER_SECOND;
const MAX_STIR_SPEED: f32 = 2.0;
const STEP_DT: f32 = 1.0 / 60.0;
const TOGGLE_PADDLE_RAIL: &str = "toggle-paddle-rail";

struct SoupStirrerHooks {
    basin_segments: [RigidSegment; 3],
    circle_body: BodyId,
    circle_radius: f32,
    box_bodies: [BodyId; 2],
    box_local_corners: [[Vec2; 4]; 2],
    edge_bodies: [BodyId; 3],
    edge_locals: [(Vec2, Vec2); 3],
    ground: BodyId,
    paddle: BodyId,
    paddle_radius: f32,
    maybe_joint: Option<JointId>,
    oscillation_offset: f32,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_soup_stirrer().map_err(|_error| SessionError::SceneConstruction)
}

fn build_soup_stirrer() -> Result<BuiltScene, SceneError> {
    let SoupFamilyBuilt {
        mut world,
        ground,
        particle_system,
        particle_radius,
        basin_segments,
        circle_body,
        circle_radius,
        box_bodies,
        box_local_corners,
        edge_bodies,
        edge_locals,
    } = soup_family::build_soup_family()?;

    // Explicit pin matching testSoupStirrer.js SetDamping(1.0).
    let _pinned_damping = PARTICLE_DAMPING;
    debug_assert!(
        (world
            .particle_system_snapshot(particle_system)
            .map_err(|_error| SceneError::ParticleSystem)?
            .definition()
            .damping()
            - PARTICLE_DAMPING)
            .abs()
            < f32::EPSILON
    );

    let paddle = create_paddle(&mut world)?;
    carve_under_paddle(&mut world, particle_system, paddle)?;
    let joint = create_paddle_rail(&mut world, ground, paddle)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius,
        hooks: Box::new(SoupStirrerHooks {
            basin_segments,
            circle_body,
            circle_radius,
            box_bodies,
            box_local_corners,
            edge_bodies,
            edge_locals,
            ground,
            paddle,
            paddle_radius: PADDLE_RADIUS,
            maybe_joint: Some(joint),
            oscillation_offset: 0.0,
        }),
    })
}

fn create_paddle(world: &mut World) -> Result<BodyId, SceneError> {
    let body_definition = BodyDef::new(BodyType::Dynamic, Vec2::ZERO, 0.0, true)
        .map_err(|_error| SceneError::Body)?;
    let body = world
        .create_body(&body_definition)
        .map_err(|_error| SceneError::Body)?;
    let circle = CircleShape::new(PADDLE_CENTER, PADDLE_RADIUS).map_err(|_error| SceneError::Geometry)?;
    let fixture_definition = FixtureDef::new(
        Shape::from(circle),
        PADDLE_DENSITY,
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

fn carve_under_paddle(
    world: &mut World,
    system: ParticleSystemId,
    paddle: BodyId,
) -> Result<(), SceneError> {
    let transform = world
        .body_snapshot(paddle)
        .map_err(|_error| SceneError::Body)?
        .transform();
    let shape = Shape::from(
        CircleShape::new(PADDLE_CENTER, PADDLE_RADIUS).map_err(|_error| SceneError::Geometry)?,
    );
    world
        .destroy_particles_in_shape(system, &shape, transform)
        .map_err(|_error| SceneError::Particle)?;
    Ok(())
}

fn create_paddle_rail(
    world: &mut World,
    ground: BodyId,
    paddle: BodyId,
) -> Result<JointId, SceneError> {
    let paddle_position = world
        .body_snapshot(paddle)
        .map_err(|_error| SceneError::Body)?
        .position();
    let definition = PrismaticJointDef::new(ground, paddle)
        .map_err(|_error| SceneError::Body)?
        .with_collide_connected(true)
        .with_frame(
            paddle_position,
            Vec2::ZERO,
            Vec2::new(1.0, 0.0),
            0.0,
        )
        .map_err(|_error| SceneError::Body)?;
    world
        .create_joint(JointDef::from(definition))
        .map_err(|_error| SceneError::Body)
}

fn in_soup(position: Vec2) -> bool {
    position.y > -1.0 && position.y < 2.0 && position.x > -3.0 && position.x < 3.0
}

impl SoupStirrerHooks {
    fn toggle_paddle_rail(&mut self, world: &mut World) -> Result<(), SessionError> {
        if let Some(joint) = self.maybe_joint.take() {
            world
                .destroy_joint(joint)
                .map_err(|_error| SessionError::SceneConstruction)?;
            return Ok(());
        }
        let joint = create_paddle_rail(world, self.ground, self.paddle)
            .map_err(|_error| SessionError::SceneConstruction)?;
        self.maybe_joint = Some(joint);
        Ok(())
    }
}

impl SceneHooks for SoupStirrerHooks {
    fn on_advance(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        self.oscillation_offset += STEP_DT;
        if self.oscillation_offset > FORCE_OSCILLATION_PERIOD {
            self.oscillation_offset -= FORCE_OSCILLATION_PERIOD;
        }

        let Some(_joint) = self.maybe_joint else {
            return Ok(());
        };

        let snapshot = world
            .body_snapshot(self.paddle)
            .map_err(|_error| SessionError::StepFailed)?;
        let position = snapshot.position();
        let speed = snapshot.linear_velocity().length();
        if !(in_soup(position) && speed < MAX_STIR_SPEED) {
            return Ok(());
        }

        let force_angle = self.oscillation_offset * FORCE_OSCILLATION_PER_SECOND * TAU;
        let force = Vec2::new(force_angle.sin(), force_angle.cos()) * FORCE_MAGNITUDE;
        world
            .apply_body_force_to_center(self.paddle, force, WakePolicy::Wake)
            .map_err(|_error| SessionError::StepFailed)?;
        Ok(())
    }

    fn apply_control(
        &mut self,
        world: &mut World,
        _system: ParticleSystemId,
        name: &str,
        _value: &str,
    ) -> Result<ControlEffect, SessionError> {
        if name != TOGGLE_PADDLE_RAIL {
            return Err(SessionError::UnknownControl);
        }
        self.toggle_paddle_rail(world)?;
        Ok(ControlEffect::Live)
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
        world: &mut World,
        _system: ParticleSystemId,
        kind: PointerKind,
        world_x: f32,
        world_y: f32,
    ) -> Result<(), SessionError> {
        if kind != PointerKind::Up {
            return Ok(());
        }
        if !in_soup(Vec2::new(world_x, world_y)) {
            return Ok(());
        }
        self.toggle_paddle_rail(world)
    }

    fn collect_segments(&self, world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        let mut segments = self.basin_segments.to_vec();
        for (body, corners) in self.box_bodies.iter().zip(self.box_local_corners.iter()) {
            let transform = world
                .body_snapshot(*body)
                .map_err(|_error| SessionError::FrameCaptureFailed)?
                .transform();
            let world_corners: [Vec2; 4] = [
                transform.apply(corners[0]),
                transform.apply(corners[1]),
                transform.apply(corners[2]),
                transform.apply(corners[3]),
            ];
            for index in 0..4 {
                segments.push(RigidSegment {
                    start: world_corners[index],
                    end: world_corners[(index + 1) % 4],
                });
            }
        }
        for (body, (start, end)) in self.edge_bodies.iter().zip(self.edge_locals.iter()) {
            let transform = world
                .body_snapshot(*body)
                .map_err(|_error| SessionError::FrameCaptureFailed)?
                .transform();
            segments.push(RigidSegment {
                start: transform.apply(*start),
                end: transform.apply(*end),
            });
        }
        Ok(segments)
    }

    fn collect_circles(&self, world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        let soup_circle = {
            let transform = world
                .body_snapshot(self.circle_body)
                .map_err(|_error| SessionError::FrameCaptureFailed)?
                .transform();
            (transform.apply(Vec2::new(0.0, 0.5)), self.circle_radius)
        };
        let paddle_circle = {
            let transform = world
                .body_snapshot(self.paddle)
                .map_err(|_error| SessionError::FrameCaptureFailed)?
                .transform();
            (transform.apply(PADDLE_CENTER), self.paddle_radius)
        };
        Ok(vec![soup_circle, paddle_circle])
    }
}

#[cfg(test)]
mod tests {
    use crate::ProofFrame;
    use crate::scene::{ControlEffect, PointerKind, SceneId};
    use crate::session::{SessionCore, SessionError};

    #[test]
    fn create_builds_soup_with_paddle_circle() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::SoupStirrer)
            .expect("Soup Stirrer should construct over soup_family");
        let frame = capture(&session);

        // Assert
        assert!(
            session.particle_count() > 0,
            "water group must create at least one particle"
        );
        assert!(
            frame.rigid_circles().len() >= 2,
            "captured frame must export soup solids plus the paddle circle"
        );
    }

    #[test]
    fn toggle_paddle_rail_twice_returns_live() {
        // Arrange
        let super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = super::build(&[]).expect("Soup Stirrer should construct over soup_family");
        assert_eq!(
            world.joint_count(),
            1,
            "rail must start attached after construction"
        );

        // Act
        let first = hooks
            .apply_control(&mut world, particle_system, "toggle-paddle-rail", "")
            .expect("first toggle must detach the rail");
        let joints_after_detach = world.joint_count();
        let second = hooks
            .apply_control(&mut world, particle_system, "toggle-paddle-rail", "")
            .expect("second toggle must restore the rail");
        let joints_after_reattach = world.joint_count();

        // Assert
        assert!(matches!(first, ControlEffect::Live));
        assert!(matches!(second, ControlEffect::Live));
        assert_eq!(joints_after_detach, 0);
        assert_eq!(joints_after_reattach, 1);
    }

    #[test]
    fn pointer_up_inside_soup_toggles_rail_outside_is_noop() {
        // Arrange
        let super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = super::build(&[]).expect("Soup Stirrer should construct over soup_family");
        assert_eq!(world.joint_count(), 1);

        // Act — outside AABB must not toggle
        hooks
            .apply_pointer(
                &mut world,
                particle_system,
                PointerKind::Up,
                10.0,
                10.0,
            )
            .expect("out-of-soup pointer up must succeed as a no-op");
        assert_eq!(
            world.joint_count(),
            1,
            "pointer outside InSoup AABB must leave the rail attached"
        );

        // Act — inside AABB toggles (detaches)
        hooks
            .apply_pointer(&mut world, particle_system, PointerKind::Up, 0.0, 0.5)
            .expect("InSoup pointer up must toggle the rail");
        assert_eq!(world.joint_count(), 0);

        // Act — outside again must not reattach
        hooks
            .apply_pointer(
                &mut world,
                particle_system,
                PointerKind::Up,
                10.0,
                10.0,
            )
            .expect("out-of-soup pointer up must succeed as a no-op");
        assert_eq!(
            world.joint_count(),
            0,
            "pointer outside InSoup AABB must leave rail state unchanged"
        );
    }

    #[test]
    fn remount_starts_with_rail_attached_so_first_toggle_detaches() {
        // Arrange — detach, then remount via SessionCore::create
        let mut session = SessionCore::create(SceneId::SoupStirrer)
            .expect("Soup Stirrer should construct over soup_family");
        session
            .apply_control("toggle-paddle-rail", "")
            .expect("detach before remount");

        // Act
        session = SessionCore::create(SceneId::SoupStirrer)
            .expect("remount must reconstruct Soup Stirrer");
        let first = session
            .apply_control("toggle-paddle-rail", "")
            .expect("first toggle after remount must detach");

        // Assert — remount starts attached so the first toggle is a live detach
        assert!(!first, "toggle is a live control, not a recreate");
    }

    #[test]
    fn unknown_control_is_rejected() {
        // Arrange
        let mut session = SessionCore::create(SceneId::SoupStirrer)
            .expect("Soup Stirrer should construct over soup_family");

        // Act
        let control = session.apply_control("stiffness", "high");
        let action = session.apply_action("refill");

        // Assert
        assert_eq!(control, Err(SessionError::UnknownControl));
        assert_eq!(action, Err(SessionError::UnknownControl));
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("Soup Stirrer frame capture should succeed"),
        )
    }
}
