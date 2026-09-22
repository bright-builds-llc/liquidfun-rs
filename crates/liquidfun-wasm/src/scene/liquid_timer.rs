//! Pinned `LiquidFun` Liquid Timer test: tensile/viscous drain through shelves.

use liquidfun::collision::{ChainShape, EdgeShape, FilterData, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{BodyDef, BodyId, FixtureDef, ParticleSystemDef, ParticleSystemId, World};

use super::{BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.025;
const MAXIMUM_PARTICLE_COUNT: usize = 10240;
const SLAB_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const SLAB_HALF_WIDTH: f32 = 2.0;
const SLAB_HALF_HEIGHT: f32 = 0.4;
const SLAB_CENTER: Vec2 = Vec2::new(0.0, 3.6);
const EDGE_DENSITY: f32 = 0.1;
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);

const BOWL_VERTICES: [Vec2; 4] = [
    Vec2::new(-2.0, 0.0),
    Vec2::new(2.0, 0.0),
    Vec2::new(2.0, 4.0),
    Vec2::new(-2.0, 4.0),
];

const SHELF_ENDPOINTS: [(Vec2, Vec2); 10] = [
    (Vec2::new(-2.0, 3.2), Vec2::new(-1.2, 3.2)),
    (Vec2::new(-1.1, 3.2), Vec2::new(2.0, 3.2)),
    (Vec2::new(-1.2, 3.2), Vec2::new(-1.2, 2.8)),
    (Vec2::new(-1.1, 3.2), Vec2::new(-1.1, 2.8)),
    (Vec2::new(-1.6, 2.4), Vec2::new(0.8, 2.0)),
    (Vec2::new(1.6, 1.6), Vec2::new(-0.8, 1.2)),
    (Vec2::new(-1.2, 0.8), Vec2::new(-1.2, 0.0)),
    (Vec2::new(-0.4, 0.8), Vec2::new(-0.4, 0.0)),
    (Vec2::new(0.4, 0.8), Vec2::new(0.4, 0.0)),
    (Vec2::new(1.2, 0.8), Vec2::new(1.2, 0.0)),
];

struct LiquidTimerHooks {
    shelf_segments: [RigidSegment; 10],
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_liquid_timer().map_err(|_error| SessionError::SceneConstruction)
}

fn build_liquid_timer() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_closed_bowl(&mut world, ground)?;
    attach_shelf_edges(&mut world)?;

    let particle_system = create_tensile_viscous_slab(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(LiquidTimerHooks {
            shelf_segments: shelf_segments(),
        }),
    })
}

fn shelf_segments() -> [RigidSegment; 10] {
    SHELF_ENDPOINTS.map(|(start, end)| RigidSegment { start, end })
}

fn attach_closed_bowl(world: &mut World, body: BodyId) -> Result<(), SceneError> {
    let chain = ChainShape::closed(&BOWL_VERTICES).map_err(|_error| SceneError::Geometry)?;
    let definition = FixtureDef::new(
        Shape::from(chain),
        0.0,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(body, &definition)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(())
}

fn attach_shelf_edges(world: &mut World) -> Result<(), SceneError> {
    for (start, end) in SHELF_ENDPOINTS {
        let body = world
            .create_body(&BodyDef::default())
            .map_err(|_error| SceneError::Body)?;
        let edge = EdgeShape::new(start, end).map_err(|_error| SceneError::Geometry)?;
        let definition = FixtureDef::new(
            Shape::from(edge),
            EDGE_DENSITY,
            0.2,
            0.0,
            false,
            FilterData::default(),
        )
        .map_err(|_error| SceneError::Fixture)?;
        world
            .create_fixture(body, &definition)
            .map_err(|_error| SceneError::Fixture)?;
    }
    Ok(())
}

fn create_tensile_viscous_slab(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    let filled = Shape::from(
        PolygonShape::oriented_box(SLAB_HALF_WIDTH, SLAB_HALF_HEIGHT, SLAB_CENTER, 0.0)
            .map_err(|_error| SceneError::Geometry)?,
    );
    let source =
        ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(ParticleFlags::TENSILE | ParticleFlags::VISCOUS)
        .with_color(SLAB_COLOR)
        .with_transform(Transform::IDENTITY)
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(system)
}

impl SceneHooks for LiquidTimerHooks {
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
        Ok(self.shelf_segments.to_vec())
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use liquidfun::ParticleFlags;

    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::{SessionCore, SessionError};

    #[test]
    fn create_builds_bowl_slab_and_drain_geometry() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::LiquidTimer)
            .expect("Liquid Timer should construct the pinned drain bowl");
        let frame = capture(&session);

        // Assert
        assert!(
            session.particle_count() > 0,
            "tensile/viscous slab must create at least one particle"
        );
        assert!(
            session.rigid_shape_count() >= 10,
            "collect_segments must export every shelf and column edge"
        );
        assert!(
            frame.rigid_segments().len() >= 40,
            "ten edges × four floats each must be drawable"
        );
    }

    #[test]
    fn constructed_group_flags_include_tensile_and_viscous() {
        // Arrange / Act
        let super::BuiltScene {
            world,
            particle_system,
            ..
        } = super::build(&[]).expect("Liquid Timer should construct");
        let view = world
            .particle_system_view(particle_system)
            .expect("constructed system should stay live");
        let expected = ParticleFlags::TENSILE | ParticleFlags::VISCOUS;

        // Assert
        assert!(
            view.flags().iter().any(|flags| {
                flags.contains(ParticleFlags::TENSILE) && flags.contains(ParticleFlags::VISCOUS)
            }),
            "group particles should carry {expected:?}"
        );
    }

    #[test]
    fn several_advances_keep_particles_alive() {
        // Arrange
        let mut session = SessionCore::create(SceneId::LiquidTimer)
            .expect("Liquid Timer should construct the pinned drain bowl");
        let before = session.particle_count();

        // Act
        for _ in 0..8 {
            session
                .advance(4)
                .expect("Liquid Timer advance must stay within the catch-up cap");
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
        let mut session = SessionCore::create(SceneId::LiquidTimer)
            .expect("Liquid Timer should construct the pinned drain bowl");
        let before_count = session.particle_count();

        // Act
        let control = session.apply_control("viscosity", "high");
        let action = session.apply_action("refill");

        // Assert
        assert_eq!(control, Err(SessionError::UnknownControl));
        assert_eq!(action, Err(SessionError::UnknownControl));
        assert_eq!(session.particle_count(), before_count);
    }

    #[test]
    fn pointer_is_a_noop_without_mutating_particle_count() {
        // Arrange
        let mut session = SessionCore::create(SceneId::LiquidTimer)
            .expect("Liquid Timer should construct the pinned drain bowl");
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
                .expect("Liquid Timer frame capture should succeed"),
        )
    }
}
