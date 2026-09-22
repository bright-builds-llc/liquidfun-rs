//! Pinned `LiquidFun` Soup test: basin broth with floating solids (watch-first).

use liquidfun::math::Vec2;
use liquidfun::{BodyId, ParticleSystemId, World};

use super::soup_family::{self, SoupFamilyBuilt};
use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
};
use crate::session::SessionError;

struct SoupHooks {
    basin_segments: [RigidSegment; 3],
    circle_body: BodyId,
    circle_radius: f32,
    box_bodies: [BodyId; 2],
    box_local_corners: [[Vec2; 4]; 2],
    edge_bodies: [BodyId; 3],
    edge_locals: [(Vec2, Vec2); 3],
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_soup().map_err(|_error| SessionError::SceneConstruction)
}

fn build_soup() -> Result<BuiltScene, SceneError> {
    let SoupFamilyBuilt {
        world,
        ground: _ground,
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

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius,
        hooks: Box::new(SoupHooks {
            basin_segments,
            circle_body,
            circle_radius,
            box_bodies,
            box_local_corners,
            edge_bodies,
            edge_locals,
        }),
    })
}

impl SceneHooks for SoupHooks {
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
        let transform = world
            .body_snapshot(self.circle_body)
            .map_err(|_error| SessionError::FrameCaptureFailed)?
            .transform();
        let center = transform.apply(Vec2::new(0.0, 0.5));
        Ok(vec![(center, self.circle_radius)])
    }
}

#[cfg(test)]
mod tests {
    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::{SessionCore, SessionError};

    #[test]
    fn create_builds_basin_liquid_and_floating_solids() {
        // Arrange / Act
        let session =
            SessionCore::create(SceneId::Soup).expect("Soup should construct the pinned basin");
        let frame = capture(&session);

        // Assert
        assert!(
            session.particle_count() > 0,
            "water group must create at least one particle"
        );
        assert!(
            frame.rigid_circles().len() >= 3,
            "captured frame must export at least one rigid circle (x,y,r)"
        );
    }

    #[test]
    fn several_advances_keep_particles_alive() {
        // Arrange
        let mut session =
            SessionCore::create(SceneId::Soup).expect("Soup should construct the pinned basin");
        let before = session.particle_count();

        // Act
        for _ in 0..8 {
            session
                .advance(4)
                .expect("Soup advance must stay within the catch-up cap");
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
        let mut session =
            SessionCore::create(SceneId::Soup).expect("Soup should construct the pinned basin");
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
        let mut session =
            SessionCore::create(SceneId::Soup).expect("Soup should construct the pinned basin");
        let before_count = session.particle_count();

        // Act
        session
            .apply_pointer("down", 0.0, 1.0)
            .expect("watch-first pointer must succeed as a no-op");
        session
            .apply_pointer("move", 0.5, 0.5)
            .expect("watch-first pointer must succeed as a no-op");
        session
            .apply_pointer("up", 0.5, 0.5)
            .expect("watch-first pointer must succeed as a no-op");

        // Assert
        assert_eq!(session.particle_count(), before_count);
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("Soup frame capture should succeed"),
        )
    }
}
