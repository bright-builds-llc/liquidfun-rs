//! Periodic hydraulic fountain: a piston squeezes one water group through a throat.
//!
//! The construction in this file is a compile stub until the piston, walls, and
//! water group land. Tests lock the layout and the half-period motor schedule.

use liquidfun::math::Vec2;
use liquidfun::{ParticleSystemDef, ParticleSystemId, World};

use super::{BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneHooks};
use crate::session::SessionError;

struct HydraulicFountainHooks;

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }

    let mut world = World::new().map_err(|_error| SessionError::SceneConstruction)?;
    world
        .set_gravity(Vec2::new(0.0, -10.0))
        .map_err(|_error| SessionError::SceneConstruction)?;
    let system_definition = ParticleSystemDef::default()
        .with_radius(0.025)
        .map_err(|_error| SessionError::SceneConstruction)?;
    let particle_system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SessionError::SceneConstruction)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: 0.025,
        hooks: Box::new(HydraulicFountainHooks),
    })
}

impl SceneHooks for HydraulicFountainHooks {
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
        Ok(Vec::new())
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use liquidfun::{BodyType, JointDef, JointKind, WorldObservationLimits};

    use super::build;
    use crate::scene::{SceneId, build_scene};
    use crate::session::{SessionCore, SessionError};

    const FOUNTAIN_SIDE_X: f32 = 0.04;
    const ADVANCE_SPEED: f32 = 0.6;

    #[test]
    fn fresh_build_places_water_only_in_the_piston_chamber() {
        // Arrange
        let session = SessionCore::create(SceneId::HydraulicFountain)
            .expect("hydraulic fountain should construct");

        // Act
        let positions = session.read_particles(|world, system| {
            world
                .particle_system_view(system)
                .expect("the water system should be live")
                .positions()
                .to_vec()
        });

        // Assert
        assert!(
            !positions.is_empty(),
            "the piston chamber should start with water"
        );
        assert!(
            positions.iter().all(|position| position.x < 0.0),
            "every water particle starts on the piston side of the throat"
        );
        assert!(
            positions
                .iter()
                .all(|position| position.x <= FOUNTAIN_SIDE_X),
            "the fountain side starts empty"
        );
    }

    #[test]
    fn piston_body_is_dynamic() {
        // Arrange
        let session = SessionCore::create(SceneId::HydraulicFountain)
            .expect("hydraulic fountain should construct");

        // Act
        let dynamic_count = session.read_particles(|world, _system| {
            world
                .world_observation(WorldObservationLimits::reviewed())
                .expect("reviewed observation should include the piston")
                .bodies()
                .iter()
                .filter(|body| body.snapshot().body_type() == BodyType::Dynamic)
                .count()
        });

        // Assert
        assert_eq!(dynamic_count, 1, "the piston is the only dynamic body");
    }

    #[test]
    fn motor_speed_follows_the_half_period() {
        // Arrange
        let mut session = SessionCore::create(SceneId::HydraulicFountain)
            .expect("hydraulic fountain should construct");

        // Act
        session
            .advance(1)
            .expect("the first step should write the advance speed");
        let advance_speed = prismatic_motor_speed(&session);
        for _ in 0..16 {
            session
                .advance(4)
                .expect("the second half of the period should still step");
        }
        let retract_speed = prismatic_motor_speed(&session);

        // Assert
        assert_eq!(advance_speed.to_bits(), ADVANCE_SPEED.to_bits());
        assert_eq!(retract_speed.to_bits(), (-ADVANCE_SPEED).to_bits());
    }

    #[test]
    fn controls_and_pointer_stay_watch_first() {
        // Arrange
        let mut session = SessionCore::create(SceneId::HydraulicFountain)
            .expect("hydraulic fountain should construct");

        // Act
        let period = session.apply_control("period", "2");
        let aim = session.apply_action("aim");
        let pointer = session.apply_pointer("up", 0.0, 0.5);

        // Assert
        assert_eq!(period, Err(SessionError::UnknownControl));
        assert_eq!(aim, Err(SessionError::UnknownControl));
        assert_eq!(pointer, Ok(()));
    }

    #[test]
    fn non_empty_presets_are_rejected_and_gravity_is_stripped() {
        // Arrange
        let period = [("period".to_owned(), "2".to_owned())];
        let gravity = [("gravity".to_owned(), "10".to_owned())];

        // Act
        let rejected = build(&period);
        let built = build_scene(SceneId::HydraulicFountain, &gravity);

        // Assert
        assert!(matches!(rejected, Err(SessionError::UnknownControl)));
        let built = built.expect("gravity is stripped before the scene builder");
        assert_eq!(built.world.gravity().x.to_bits(), 0.0_f32.to_bits());
        assert_eq!(built.world.gravity().y.to_bits(), (-10.0_f32).to_bits());
    }

    fn prismatic_motor_speed(session: &SessionCore) -> f32 {
        session.read_particles(|world, _system| {
            let observation = world
                .world_observation(WorldObservationLimits::reviewed())
                .expect("reviewed observation should include the piston joint");
            let joint = observation
                .joints()
                .iter()
                .find(|joint| joint.snapshot().kind() == JointKind::Prismatic)
                .expect("the piston rides a prismatic joint");
            let JointDef::Prismatic(definition) = joint.snapshot().definition() else {
                panic!("the piston joint should be prismatic");
            };
            definition.motor_speed()
        })
    }
}
