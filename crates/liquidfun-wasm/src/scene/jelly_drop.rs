//! Native Jelly Drop scene: a bounded elastic particle group on two rigid bars.

use liquidfun::collision::{CircleShape, PolygonShape, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{ParticleGroupId, ParticleSystemDef, ParticleSystemId, World};

use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
    attach_basin_fixture,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.16;
const MAXIMUM_PARTICLE_COUNT: usize = 220;
const JELLY_COLOR: ParticleColor = ParticleColor::new(244, 114, 182, 255);
const JELLY_HALF_EXTENT: f32 = 1.2;
const JELLY_CENTER: Vec2 = Vec2::new(0.0, 3.6);
const SOFT_STRENGTH: f32 = 0.4;
const MEDIUM_STRENGTH: f32 = 1.0;
const FIRM_STRENGTH: f32 = 2.0;
/// Downward labeled poke; applied to the contiguous group member range.
const POKE_IMPULSE: Vec2 = Vec2::new(0.0, -8.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JellyShape {
    Circle,
    Square,
}

impl JellyShape {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "circle" => Some(Self::Circle),
            "square" => Some(Self::Square),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Softness {
    Soft,
    Medium,
    Firm,
}

impl Softness {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "soft" => Some(Self::Soft),
            "medium" => Some(Self::Medium),
            "firm" => Some(Self::Firm),
            _ => None,
        }
    }

    fn strength(self) -> f32 {
        match self {
            Self::Soft => SOFT_STRENGTH,
            Self::Medium => MEDIUM_STRENGTH,
            Self::Firm => FIRM_STRENGTH,
        }
    }
}

struct JellyDropHooks {
    bar_segments: [RigidSegment; 2],
    group: ParticleGroupId,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    let shape = preset_value(presets, "shape").and_then(JellyShape::parse);
    let softness = preset_value(presets, "softness").and_then(Softness::parse);
    build_jelly(
        shape.unwrap_or(JellyShape::Circle),
        softness.unwrap_or(Softness::Medium),
    )
    .map_err(|_error| SessionError::SceneConstruction)
}

fn preset_value<'a>(presets: &'a [(String, String)], name: &str) -> Option<&'a str> {
    presets
        .iter()
        .rev()
        .find(|(key, _value)| key == name)
        .map(|(_key, value)| value.as_str())
}

fn build_jelly(shape: JellyShape, softness: Softness) -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(Vec2::new(0.0, -10.0))
        .map_err(|_error| SceneError::Gravity)?;

    let ground = world
        .create_body(&liquidfun::BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    attach_basin_fixture(
        &mut world,
        ground,
        &[
            Vec2::new(-3.2, 0.85),
            Vec2::new(0.6, 0.85),
            Vec2::new(0.6, 1.15),
            Vec2::new(-3.2, 1.15),
        ],
    )?;
    attach_basin_fixture(
        &mut world,
        ground,
        &[
            Vec2::new(-0.6, 0.85),
            Vec2::new(3.2, 0.85),
            Vec2::new(3.2, 1.15),
            Vec2::new(-0.6, 1.15),
        ],
    )?;

    let (particle_system, group) = create_jelly_group(&mut world, shape, softness)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(JellyDropHooks {
            bar_segments: [
                RigidSegment {
                    start: Vec2::new(-3.2, 1.0),
                    end: Vec2::new(0.6, 1.0),
                },
                RigidSegment {
                    start: Vec2::new(-0.6, 1.0),
                    end: Vec2::new(3.2, 1.0),
                },
            ],
            group,
        }),
    })
}

fn create_jelly_group(
    world: &mut World,
    shape: JellyShape,
    softness: Softness,
) -> Result<(ParticleSystemId, ParticleGroupId), SceneError> {
    let system_definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_damping(1.2)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_elastic_strength(0.75)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_spring_strength(0.75)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?;
    let system = world
        .create_particle_system_with_def(&system_definition)
        .map_err(|_error| SceneError::ParticleSystem)?;

    let source = jelly_source(shape)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(ParticleFlags::ELASTIC | ParticleFlags::SPRING)
        .with_strength(softness.strength())
        .map_err(|_error| SceneError::Particle)?
        .with_color(JELLY_COLOR)
        .with_transform(Transform::from_position_angle(JELLY_CENTER, 0.0))
        .map_err(|_error| SceneError::Particle)?;
    let group = world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok((system, group))
}

fn jelly_source(shape: JellyShape) -> Result<ParticleGroupSource, SceneError> {
    let filled = match shape {
        JellyShape::Circle => Shape::from(
            CircleShape::new(Vec2::ZERO, JELLY_HALF_EXTENT)
                .map_err(|_error| SceneError::Geometry)?,
        ),
        JellyShape::Square => Shape::from(
            PolygonShape::new(&[
                Vec2::new(-JELLY_HALF_EXTENT, -JELLY_HALF_EXTENT),
                Vec2::new(JELLY_HALF_EXTENT, -JELLY_HALF_EXTENT),
                Vec2::new(JELLY_HALF_EXTENT, JELLY_HALF_EXTENT),
                Vec2::new(-JELLY_HALF_EXTENT, JELLY_HALF_EXTENT),
            ])
            .map_err(|_error| SceneError::Geometry)?,
        ),
    };
    ParticleGroupSource::filled_shapes(vec![filled]).map_err(|_error| SceneError::Particle)
}

fn poke_jelly(
    world: &mut World,
    system: ParticleSystemId,
    group: ParticleGroupId,
) -> Result<(), SessionError> {
    let members = {
        let view = world
            .particle_group_view(group)
            .map_err(|_error| SessionError::SceneConstruction)?;
        view.member_ids().to_vec()
    };
    let poke_len = members.len().div_ceil(3).max(1).min(members.len());
    let poked = &members[..poke_len];
    world
        .apply_particle_linear_impulse_range(system, poked, POKE_IMPULSE)
        .map_err(|_error| SessionError::SceneConstruction)?;
    Ok(())
}

impl SceneHooks for JellyDropHooks {
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
            "shape" => {
                let Some(_shape) = JellyShape::parse(value) else {
                    return Err(SessionError::UnknownControl);
                };
                Ok(ControlEffect::Recreated)
            }
            "softness" => {
                let Some(_softness) = Softness::parse(value) else {
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
        system: ParticleSystemId,
        name: &str,
    ) -> Result<(), SessionError> {
        if name != "poke-jelly" {
            return Err(SessionError::UnknownControl);
        }
        poke_jelly(world, system, self.group)
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
            PointerKind::Down | PointerKind::Move | PointerKind::Up | PointerKind::Cancel => Ok(()),
        }
    }

    fn collect_segments(&self, _world: &World) -> Result<Vec<RigidSegment>, SessionError> {
        Ok(self.bar_segments.to_vec())
    }

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use liquidfun::particle::ParticleFlags;

    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::SessionCore;

    #[test]
    fn create_jelly_drop_builds_a_bounded_elastic_group_on_two_bars() {
        // Arrange / Act
        let session = SessionCore::create(SceneId::JellyDrop)
            .expect("Jelly Drop should construct a native elastic group");
        let frame = capture(&session);

        // Assert
        assert!((8..=220).contains(&session.particle_count()));
        assert!(session.particle_count() <= 512);
        assert!(
            frame.rigid_segments().len() >= 8,
            "two rigid bars report at least eight segment floats"
        );
    }

    #[test]
    fn constructed_group_flags_include_elastic_and_spring() {
        // Arrange / Act
        let super::BuiltScene {
            world,
            particle_system,
            ..
        } = super::build(&[]).expect("Jelly Drop should construct");
        let view = world
            .particle_system_view(particle_system)
            .expect("constructed system should stay live");
        let expected = ParticleFlags::ELASTIC | ParticleFlags::SPRING;

        // Assert
        assert!(
            view.flags()
                .iter()
                .any(|flags| flags.contains(ParticleFlags::ELASTIC)
                    && flags.contains(ParticleFlags::SPRING)),
            "group particles should carry {expected:?}"
        );
    }

    #[test]
    fn thirty_steps_keep_the_particle_count_constant() {
        // Arrange
        let mut session = SessionCore::create(SceneId::JellyDrop)
            .expect("Jelly Drop should construct a native elastic group");
        let before = capture(&session).particle_count();

        // Act
        advance_steps(&mut session, 30);
        let after = capture(&session).particle_count();

        // Assert
        assert_eq!(after, before, "Jelly Drop must not emit extra particles");
        assert!((8..=220).contains(&after));
    }

    #[test]
    fn shape_and_softness_recreate_the_world_from_presets() {
        // Arrange
        let mut session = SessionCore::create(SceneId::JellyDrop)
            .expect("Jelly Drop should construct a native elastic group");
        let circle_count = session.particle_count();

        // Act
        let shape_recreated = session
            .apply_control("shape", "square")
            .expect("shape=square should recreate");
        let square_count = session.particle_count();
        let softness_recreated = session
            .apply_control("softness", "firm")
            .expect("softness=firm should recreate");

        let square_scene = super::build(&[
            ("shape".to_owned(), "square".to_owned()),
            ("softness".to_owned(), "firm".to_owned()),
        ])
        .expect("square firm presets should construct");
        let square_view = square_scene
            .world
            .particle_system_view(square_scene.particle_system)
            .expect("square system should stay live");

        // Assert
        assert!(shape_recreated);
        assert!(softness_recreated);
        assert!(
            square_count > circle_count,
            "filled square should sample more particles than the default circle"
        );
        assert_eq!(session.particle_count(), square_count);
        assert_eq!(square_view.positions().len(), square_count);
        assert!(square_view.flags().iter().any(|flags| {
            flags.contains(ParticleFlags::ELASTIC) && flags.contains(ParticleFlags::SPRING)
        }));
    }

    #[test]
    fn poke_jelly_deforms_without_resetting_or_changing_count() {
        // Arrange
        let mut session = SessionCore::create(SceneId::JellyDrop)
            .expect("Jelly Drop should construct a native elastic group");
        let before = capture(&session);
        let before_count = before.particle_count();
        let before_positions = before.particle_positions();

        // Act
        session
            .apply_action("poke-jelly")
            .expect("poke-jelly should apply an in-engine impulse");
        advance_steps(&mut session, 4);
        let after = capture(&session);

        // Assert
        assert_eq!(after.particle_count(), before_count);
        assert_ne!(
            after.particle_positions().as_ref(),
            before_positions.as_ref(),
            "poke impulse should move particles without recreating the group"
        );
    }

    #[test]
    fn poke_then_sixty_steps_keep_particles_inside_the_camera_box() {
        // Arrange
        let mut session = SessionCore::create(SceneId::JellyDrop)
            .expect("Jelly Drop should construct a native elastic group");

        // Act
        session
            .apply_action("poke-jelly")
            .expect("poke-jelly should apply an in-engine impulse");
        advance_steps(&mut session, 60);
        let frame = capture(&session);
        let positions = frame.particle_positions();

        // Assert
        assert_eq!(frame.particle_count(), session.particle_count());
        assert!(positions.iter().all(|value| value.is_finite()));
        for pair in positions.chunks_exact(2) {
            let x = pair[0];
            let y = pair[1];
            assert!(
                (-7.0..7.0).contains(&x) && (-2.0..9.0).contains(&y),
                "particle ({x}, {y}) escaped (-7,-2)..(7,9)"
            );
        }
    }

    #[test]
    fn pointer_down_pokes_nearby_jelly_without_changing_count() {
        // Arrange
        let mut control = SessionCore::create(SceneId::JellyDrop)
            .expect("Jelly Drop should construct a native elastic group");
        let mut poked = SessionCore::create(SceneId::JellyDrop)
            .expect("Jelly Drop should construct a native elastic group");
        let before_count = poked.particle_count();

        // Act
        poked
            .apply_pointer("down", 0.0, 2.0)
            .expect("down should poke particles near the click");
        advance_steps(&mut control, 4);
        advance_steps(&mut poked, 4);
        let control_frame = capture(&control);
        let poked_frame = capture(&poked);

        // Assert
        assert_eq!(poked_frame.particle_count(), before_count);
        assert_eq!(control_frame.particle_count(), before_count);
        assert_ne!(
            poked_frame.particle_positions().as_ref(),
            control_frame.particle_positions().as_ref(),
            "pointer poke should move nearby jelly particles versus a no-pointer world"
        );
    }

    #[test]
    fn pointer_cancel_after_down_does_not_apply_a_second_impulse() {
        // Arrange
        let mut down_only = SessionCore::create(SceneId::JellyDrop)
            .expect("Jelly Drop should construct a native elastic group");
        let mut canceled = SessionCore::create(SceneId::JellyDrop)
            .expect("Jelly Drop should construct a native elastic group");

        // Act
        down_only
            .apply_pointer("down", 0.0, 2.0)
            .expect("down should poke once");
        canceled
            .apply_pointer("down", 0.0, 2.0)
            .expect("down should poke once");
        canceled
            .apply_pointer("cancel", 0.0, 0.0)
            .expect("cancel after down must not poke again");
        advance_steps(&mut down_only, 4);
        advance_steps(&mut canceled, 4);

        // Assert
        assert_eq!(
            capture(&canceled).particle_positions().as_ref(),
            capture(&down_only).particle_positions().as_ref(),
            "cancel after down must not apply a second impulse"
        );
    }

    #[test]
    fn labeled_poke_still_uses_contiguous_first_third_range() {
        // Arrange
        let source = include_str!("jelly_drop.rs");
        let impl_source = source
            .split("#[cfg(test)]")
            .next()
            .expect("implementation precedes tests");

        // Assert
        assert!(
            impl_source.contains("apply_particle_linear_impulse("),
            "localized poke must call apply_particle_linear_impulse per nearby id"
        );
        assert!(
            impl_source.contains("apply_particle_linear_impulse_range"),
            "labeled poke-jelly must keep the contiguous range API"
        );
        assert!(
            impl_source.contains("members[..poke_len]"),
            "labeled poke must stay on the first-third contiguous slice"
        );
    }

    #[test]
    fn unknown_shape_softness_and_actions_fail_closed() {
        // Arrange
        let mut session = SessionCore::create(SceneId::JellyDrop)
            .expect("Jelly Drop should construct a native elastic group");

        // Act
        let bad_shape = session.apply_control("shape", "triangle");
        let bad_softness = session.apply_control("softness", "jello");
        let unknown_name = session.apply_control("water-amount", "medium");
        let unknown_action = session.apply_action("drop-body");

        // Assert
        assert_eq!(bad_shape, Err(crate::session::SessionError::UnknownControl));
        assert_eq!(
            bad_softness,
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
                .expect("Jelly Drop should capture a frame"),
        )
    }
}
