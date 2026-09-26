//! Drinking-glass tumbler at real meters, with two walls and a floor.
//!
//! The glass is 74 mm wide and 120 mm tall, with about 55 mm of water.
//! Particles are 0.6 mm across. Earth gravity is 9.8 m/s², matching a resting
//! phone accelerometer. [`PARTICLE_ITERATIONS`] raises the pressure cap enough
//! for that column to hold; the shared 2 substeps cannot.
//!
//! Camera frame, kept in sync with `LIQUID_TUMBLER_VIEW_BOUNDS`:
//! x = -0.055..0.055, y = -0.02..0.15.

use liquidfun::collision::{ChainShape, FilterData, Shape};
use liquidfun::math::{Transform, Vec2};
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupRecipe,
    ParticleGroupSource,
};
use liquidfun::{BodyDef, FixtureDef, ParticleSystemDef, ParticleSystemId, World};

use super::{BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks};
use crate::session::SessionError;

/// Inner width of the glass, in meters.
const INNER_WIDTH: f32 = 0.074;
/// Inner height from the floor to the rim, in meters.
const INNER_HEIGHT: f32 = 0.120;
/// Water depth above the floor, in meters.
const WATER_DEPTH: f32 = 0.055;
const PARTICLE_RADIUS: f32 = 0.0006;
/// Lattice that fills the glass with exactly 9,000 particles.
const PARTICLE_COLUMNS: usize = 100;
const PARTICLE_ROWS: usize = 90;
const PARTICLE_DAMPING: f32 = 0.25;
const MAXIMUM_PARTICLE_COUNT: usize = PARTICLE_COLUMNS * PARTICLE_ROWS;
const WATER_COLOR: ParticleColor = ParticleColor::new(77, 163, 255, 255);
const GRAVITY: Vec2 = Vec2::new(0.0, -9.8);
/// Substeps that keep radius times substep count at the resting 2 mm glass.
/// The pressure cap grows with that product. Fewer substeps let the column
/// sink into the floor and get thrown back up.
pub(crate) const PARTICLE_ITERATIONS: u32 = 54;
const _: () = assert!(MAXIMUM_PARTICLE_COUNT == 9_000);

struct LiquidTumblerHooks {
    basin_segments: [RigidSegment; 3],
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_tumbler().map_err(|_error| SessionError::SceneConstruction)
}

fn build_tumbler() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;

    let basin_body = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    let half_width = INNER_WIDTH * 0.5;
    let rim = INNER_HEIGHT;
    attach_glass_chain(&mut world, basin_body, half_width, rim)?;

    let particle_system = create_water(&mut world, half_width, 0.0)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(LiquidTumblerHooks {
            basin_segments: [
                RigidSegment {
                    start: Vec2::new(-half_width, 0.0),
                    end: Vec2::new(half_width, 0.0),
                },
                RigidSegment {
                    start: Vec2::new(-half_width, 0.0),
                    end: Vec2::new(-half_width, rim),
                },
                RigidSegment {
                    start: Vec2::new(half_width, 0.0),
                    end: Vec2::new(half_width, rim),
                },
            ],
        }),
    })
}

fn attach_glass_chain(
    world: &mut World,
    body: liquidfun::BodyId,
    wall_x: f32,
    rim: f32,
) -> Result<(), SceneError> {
    let chain = ChainShape::open(
        &[
            Vec2::new(-wall_x, rim),
            Vec2::new(-wall_x, 0.0),
            Vec2::new(wall_x, 0.0),
            Vec2::new(wall_x, rim),
        ],
        None,
        None,
    )
    .map_err(|_error| SceneError::Geometry)?;
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

fn create_water(
    world: &mut World,
    half_contact: f32,
    contact_floor: f32,
) -> Result<ParticleSystemId, SceneError> {
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

    let source = ParticleGroupSource::positions(water_lattice(half_contact, contact_floor)?)
        .map_err(|_error| SceneError::Particle)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(ParticleFlags::WATER)
        .with_color(WATER_COLOR)
        .with_default_stride()
        .with_transform(Transform::IDENTITY)
        .map_err(|_error| SceneError::Particle)?;
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SceneError::Particle)?;
    Ok(system)
}

/// Even lattice inside the water, one diameter clear of the glass.
fn water_lattice(half_contact: f32, contact_floor: f32) -> Result<Vec<Vec2>, SceneError> {
    let inset = PARTICLE_RADIUS * 2.0;
    let left = -half_contact + inset;
    let bottom = contact_floor + inset;
    let x_span = (half_contact - inset) - left;
    let y_span = (contact_floor + WATER_DEPTH) - bottom;
    let x_stride = span_stride(x_span, PARTICLE_COLUMNS)?;
    let y_stride = span_stride(y_span, PARTICLE_ROWS)?;
    let mut positions = Vec::new();
    positions
        .try_reserve_exact(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::Particle)?;
    let mut y = bottom;
    for _row in 0..PARTICLE_ROWS {
        let mut x = left;
        for _column in 0..PARTICLE_COLUMNS {
            positions.push(Vec2::new(x, y));
            x += x_stride;
        }
        y += y_stride;
    }
    Ok(positions)
}

fn span_stride(span: f32, samples: usize) -> Result<f32, SceneError> {
    let gaps = samples.checked_sub(1).ok_or(SceneError::Particle)?;
    let gaps = u16::try_from(gaps).map_err(|_error| SceneError::Particle)?;
    let stride = span / f32::from(gaps);
    if stride.is_finite() && stride > 0.0 {
        Ok(stride)
    } else {
        Err(SceneError::Particle)
    }
}

impl SceneHooks for LiquidTumblerHooks {
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

    fn collect_circles(&self, _world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use crate::ProofFrame;
    use crate::scene::SceneId;
    use crate::session::{SessionCore, SessionError};

    use super::{INNER_HEIGHT, INNER_WIDTH, PARTICLE_RADIUS};

    #[test]
    fn create_fills_a_glass_with_millimeter_particles() {
        // Arrange / Act
        let session =
            SessionCore::create(SceneId::LiquidTumbler).expect("Liquid Tumbler should construct");
        let frame = capture(&session);
        let radii = frame.particle_radii();

        // Assert
        assert_eq!(
            session.particle_count(),
            9_000,
            "the 0.6 mm fill should be 9000 particles"
        );
        assert_eq!(session.rigid_shape_count(), 3, "floor and two walls");
        assert!(
            radii
                .iter()
                .all(|radius| radius.to_bits() == PARTICLE_RADIUS.to_bits())
        );
    }

    #[test]
    fn first_steps_keep_the_water_column_up() {
        // Arrange
        let mut session =
            SessionCore::create(SceneId::LiquidTumbler).expect("Liquid Tumbler should construct");
        let before = session.particle_count();

        // Act
        session
            .advance(4)
            .expect("Liquid Tumbler advance must stay within the catch-up cap");
        let frame = capture(&session);
        let positions = frame.particle_positions();

        // Assert
        assert_eq!(session.live_particle_count().expect("live count"), before);
        let speed = frame.max_speed();
        assert!(speed.is_finite());
        assert!(
            speed < 1.0,
            "glass water should not be launched by the contact slop, got {speed}"
        );
        // The 9,000-particle lattice starts denser than the resting spacing,
        // so the surface can lift during these steps. The one-second test
        // checks that the column comes back down.
        let surface = highest_particle(&positions);
        assert!(
            (0.03..INNER_HEIGHT).contains(&surface),
            "the first steps should keep the water inside the glass, got {surface}"
        );
        assert_positions_inside_glass(&positions);
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn one_second_keeps_water_inside_the_glass() {
        // Arrange
        let mut session =
            SessionCore::create(SceneId::LiquidTumbler).expect("Liquid Tumbler should construct");
        let before = session.particle_count();

        // Act
        for _ in 0..15 {
            session
                .advance(4)
                .expect("Liquid Tumbler advance must stay within the catch-up cap");
        }
        let frame = capture(&session);
        let positions = frame.particle_positions();

        // Assert
        assert_eq!(session.live_particle_count().expect("live count"), before);
        let speed = frame.max_speed();
        assert!(speed.is_finite());
        assert!(
            speed < 0.2,
            "settled glass water should come to rest, got {speed}"
        );
        let surface = highest_particle(&positions);
        assert!(
            (0.03..0.08).contains(&surface),
            "the water column should stay a few centimeters deep, got {surface}"
        );
        assert_positions_inside_glass(&positions);
    }

    #[test]
    fn unknown_control_and_action_are_rejected() {
        // Arrange
        let mut session =
            SessionCore::create(SceneId::LiquidTumbler).expect("Liquid Tumbler should construct");

        // Act
        let control = session.apply_control("gravity", "10");
        let action = session.apply_action("drop-obstacle");

        // Assert
        assert_eq!(control, Err(SessionError::UnknownControl));
        assert_eq!(action, Err(SessionError::UnknownControl));
    }

    fn highest_particle(positions: &[f32]) -> f32 {
        let mut highest = f32::MIN;
        for pair in positions.chunks_exact(2) {
            highest = highest.max(pair[1]);
        }
        highest
    }

    fn assert_positions_inside_glass(positions: &[f32]) {
        let half_width = INNER_WIDTH * 0.5 + PARTICLE_RADIUS;
        let floor = -PARTICLE_RADIUS;
        let rim = INNER_HEIGHT + PARTICLE_RADIUS;
        for pair in positions.chunks_exact(2) {
            let x = pair[0];
            let y = pair[1];
            assert!(x.abs() <= half_width, "particle x {x} left the 74 mm glass");
            assert!((floor..=rim).contains(&y), "particle y {y} left the glass");
        }
    }

    fn capture(session: &SessionCore) -> ProofFrame {
        ProofFrame::from(
            session
                .capture_frame()
                .expect("Liquid Tumbler frame capture should succeed"),
        )
    }
}
