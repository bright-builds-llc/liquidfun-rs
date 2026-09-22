//! Pinned `LiquidFun` Impulse scene: chain-loop box with whole-group shove.

use liquidfun::collision::{ChainShape, FilterData, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleGroupDestination, ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{
    BodyDef, BodyId, FixtureDef, ParticleGroupId, ParticleSystemDef, ParticleSystemId, World,
};

use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.025;
const PARTICLE_DAMPING: f32 = 0.2;
const MAXIMUM_PARTICLE_COUNT: usize = 10240;
const GROUP_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);

const BOX_LEFT: f32 = -2.0;
const BOX_RIGHT: f32 = 2.0;
const BOX_BOTTOM: f32 = 0.0;
const BOX_TOP: f32 = 4.0;
const BOX_CENTER: Vec2 = Vec2::new(0.0, 2.0);
const BOX_VERTICES: [Vec2; 4] = [
    Vec2::new(BOX_LEFT, BOX_BOTTOM),
    Vec2::new(BOX_RIGHT, BOX_BOTTOM),
    Vec2::new(BOX_RIGHT, BOX_TOP),
    Vec2::new(BOX_LEFT, BOX_TOP),
];

const GROUP_HALF_WIDTH: f32 = 0.8;
const GROUP_HALF_HEIGHT: f32 = 1.0;
const GROUP_CENTER: Vec2 = Vec2::new(0.0, 1.01);

const FORCE_MAGNITUDE: f32 = 1.0;
const IMPULSE_MAGNITUDE: f32 = 0.005;
const PUSH_MODE_CONTROL: &str = "push-mode";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PushMode {
    Force,
    Impulse,
}

impl PushMode {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "force" => Some(Self::Force),
            "impulse" => Some(Self::Impulse),
            _ => None,
        }
    }
}

struct ImpulseHooks {
    box_segments: [RigidSegment; 4],
    group: ParticleGroupId,
    push_mode: PushMode,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_impulse().map_err(|_error| SessionError::SceneConstruction)
}

fn build_impulse() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_chain_loop_box(&mut world, ground)?;

    let (particle_system, group) = create_particle_group(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(ImpulseHooks {
            box_segments: box_segments(),
            group,
            push_mode: PushMode::Force,
        }),
    })
}

fn box_segments() -> [RigidSegment; 4] {
    [
        RigidSegment {
            start: Vec2::new(BOX_LEFT, BOX_BOTTOM),
            end: Vec2::new(BOX_RIGHT, BOX_BOTTOM),
        },
        RigidSegment {
            start: Vec2::new(BOX_RIGHT, BOX_BOTTOM),
            end: Vec2::new(BOX_RIGHT, BOX_TOP),
        },
        RigidSegment {
            start: Vec2::new(BOX_RIGHT, BOX_TOP),
            end: Vec2::new(BOX_LEFT, BOX_TOP),
        },
        RigidSegment {
            start: Vec2::new(BOX_LEFT, BOX_TOP),
            end: Vec2::new(BOX_LEFT, BOX_BOTTOM),
        },
    ]
}

fn attach_chain_loop_box(world: &mut World, body: BodyId) -> Result<(), SceneError> {
    let chain = ChainShape::closed(&BOX_VERTICES).map_err(|_error| SceneError::Geometry)?;
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

fn create_particle_group(
    world: &mut World,
) -> Result<(ParticleSystemId, ParticleGroupId), SceneError> {
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

    let filled = Shape::from(
        PolygonShape::oriented_box(GROUP_HALF_WIDTH, GROUP_HALF_HEIGHT, GROUP_CENTER, 0.0)
            .map_err(|_error| SceneError::Geometry)?,
    );
    let source =
        ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_color(GROUP_COLOR)
        .with_transform(Transform::IDENTITY)
        .map_err(|_error| SceneError::Particle)?;
    let group = world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok((system, group))
}

fn pointer_inside_box(world_x: f32, world_y: f32) -> bool {
    BOX_LEFT <= world_x
        && world_x <= BOX_RIGHT
        && BOX_BOTTOM <= world_y
        && world_y <= BOX_TOP
}

fn shove_group(
    world: &mut World,
    system: ParticleSystemId,
    group: ParticleGroupId,
    push_mode: PushMode,
    pointer: Vec2,
) -> Result<(), SessionError> {
    let mut direction = pointer - BOX_CENTER;
    let length = direction.normalize();
    if length == 0.0 {
        return Ok(());
    }

    let members = {
        let view = world
            .particle_group_view(group)
            .map_err(|_error| SessionError::SceneConstruction)?;
        view.member_ids().to_vec()
    };
    let member_count = members.len() as f32;
    match push_mode {
        PushMode::Force => {
            let force = direction * (FORCE_MAGNITUDE * member_count);
            world
                .apply_particle_force_range(system, &members, force)
                .map_err(|_error| SessionError::SceneConstruction)?;
        }
        PushMode::Impulse => {
            let impulse = direction * (IMPULSE_MAGNITUDE * member_count);
            world
                .apply_particle_linear_impulse_range(system, &members, impulse)
                .map_err(|_error| SessionError::SceneConstruction)?;
        }
    }
    Ok(())
}

impl SceneHooks for ImpulseHooks {
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
        if name != PUSH_MODE_CONTROL {
            return Err(SessionError::UnknownControl);
        }
        let Some(mode) = PushMode::parse(value) else {
            return Err(SessionError::UnknownControl);
        };
        self.push_mode = mode;
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
        system: ParticleSystemId,
        kind: PointerKind,
        world_x: f32,
        world_y: f32,
    ) -> Result<(), SessionError> {
        if kind != PointerKind::Up {
            return Ok(());
        }
        if !pointer_inside_box(world_x, world_y) {
            return Ok(());
        }
        shove_group(
            world,
            system,
            self.group,
            self.push_mode,
            Vec2::new(world_x, world_y),
        )
    }

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(self.box_segments.to_vec())
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use liquidfun::NoDecisionHook;
    use liquidfun::{ParticleGroupId, ParticleSystemId, StepConfiguration, StepLimits, World};

    use super::super::{ControlEffect, PointerKind, SceneId};
    use super::build;
    use crate::session::{SessionCore, SessionError};

    #[test]
    fn create_builds_impulse_with_particles() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::Impulse)
            .expect("Impulse should construct the pinned chain-loop box");

        // Assert
        assert!(
            session.particle_count() > 0,
            "particle group must create at least one particle"
        );
    }

    #[test]
    fn inside_box_shove_changes_group_momentum_outside_is_noop() {
        // Arrange — twin worlds so gravity/settle match; only shove differs
        let mut baseline = build_impulse();
        let mut shoved = build_impulse();
        settle(&mut baseline.world);
        settle(&mut shoved.world);
        let baseline_group = first_group(&baseline.world, baseline.particle_system);
        let shoved_group = first_group(&shoved.world, shoved.particle_system);

        // Act — outside box is a success no-op
        baseline
            .hooks
            .apply_pointer(
                &mut baseline.world,
                baseline.particle_system,
                PointerKind::Up,
                10.0,
                10.0,
            )
            .expect("outside-box pointer up must succeed as a no-op");
        settle(&mut baseline.world);
        let after_outside = group_speed(&baseline.world, baseline_group);

        // Act — inside box shoves the whole blob (default force mode needs a step)
        shoved
            .hooks
            .apply_pointer(
                &mut shoved.world,
                shoved.particle_system,
                PointerKind::Up,
                1.0,
                2.0,
            )
            .expect("inside-box pointer up must shove the group");
        settle(&mut shoved.world);
        let after_inside = group_speed(&shoved.world, shoved_group);

        // Assert — same settle after outside vs after inside shove; inside must diverge
        assert!(
            (after_inside - after_outside).abs() > 0.05,
            "inside-box shove must change group momentum versus the outside twin \
             (after_inside={after_inside}, after_outside={after_outside})"
        );
    }

    #[test]
    fn push_mode_force_and_impulse_return_live() {
        // Arrange
        let super::super::BuiltScene {
            mut world,
            particle_system,
            mut hooks,
            ..
        } = build(&[]).expect("Impulse should construct the pinned chain-loop box");

        // Act
        let impulse = hooks
            .apply_control(&mut world, particle_system, "push-mode", "impulse")
            .expect("push-mode impulse must be allowlisted");
        let force = hooks
            .apply_control(&mut world, particle_system, "push-mode", "force")
            .expect("push-mode force must be allowlisted");

        // Assert
        assert!(matches!(impulse, ControlEffect::Live));
        assert!(matches!(force, ControlEffect::Live));
    }

    #[test]
    fn unknown_control_is_rejected() {
        // Arrange
        let mut session = SessionCore::create(SceneId::Impulse)
            .expect("Impulse should construct the pinned chain-loop box");

        // Act
        let control = session.apply_control("stiffness", "high");
        let action = session.apply_action("refill");
        let bad_mode = session.apply_control("push-mode", "explode");

        // Assert
        assert_eq!(control, Err(SessionError::UnknownControl));
        assert_eq!(action, Err(SessionError::UnknownControl));
        assert_eq!(bad_mode, Err(SessionError::UnknownControl));
    }

    #[test]
    fn shove_uses_full_member_ids_range_apis() {
        // Arrange
        let source = include_str!("impulse.rs");
        let impl_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("implementation precedes tests");

        // Assert — production shove (not Jelly Drop singular poke)
        assert!(
            impl_source.contains("apply_particle_force_range"),
            "force shove must use apply_particle_force_range"
        );
        assert!(
            impl_source.contains("apply_particle_linear_impulse_range"),
            "impulse shove must use apply_particle_linear_impulse_range"
        );
        assert!(
            impl_source.contains("member_ids"),
            "shove must copy full member_ids"
        );
        assert!(
            impl_source.contains("push-mode"),
            "live push-mode preset must be present"
        );
        assert!(
            impl_source.contains("0.025"),
            "particle radius must stay 0.025"
        );
        assert!(
            impl_source.contains("0.005"),
            "impulse magnitude constant must stay 0.005"
        );
        assert!(
            !impl_source.contains("apply_particle_linear_impulse("),
            "Impulse must not shove via singular apply_particle_linear_impulse"
        );
    }

    struct ImpulseBuilt {
        world: World,
        particle_system: ParticleSystemId,
        hooks: Box<dyn super::super::SceneHooks>,
    }

    fn build_impulse() -> ImpulseBuilt {
        let built = build(&[]).expect("Impulse should construct the pinned chain-loop box");
        ImpulseBuilt {
            world: built.world,
            particle_system: built.particle_system,
            hooks: built.hooks,
        }
    }

    fn first_group(world: &World, system: ParticleSystemId) -> ParticleGroupId {
        let view = world
            .particle_system_view(system)
            .expect("Impulse particle system must exist");
        view.group_ids()
            .iter()
            .find_map(|maybe_group| *maybe_group)
            .expect("impulse scene must own one particle group")
    }

    fn group_speed(world: &World, group: ParticleGroupId) -> f32 {
        world
            .particle_group_view(group)
            .expect("group must remain valid")
            .linear_velocity()
            .length()
    }

    fn settle(world: &mut World) {
        let configuration = StepConfiguration::new(1.0 / 60.0, 8, 3)
            .expect("default Impulse settle step configuration");
        let limits = StepLimits::default();
        world
            .step(configuration, &mut NoDecisionHook, limits)
            .expect("impulse settle step must succeed");
    }
}
