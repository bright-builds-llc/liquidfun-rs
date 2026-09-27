//! Sparky is an experimental native Rust port of the pinned `LiquidFun` tests.
//! The playground shows recognizable behavior. Catalog previews are static illustrations.

#[cfg(test)]
mod tests;

use liquidfun::collision::{CircleShape, FilterData, Shape};
use liquidfun::math::Vec2;
use liquidfun::particle::{
    ParticleColor, ParticleFlags, ParticleGroupDestination, ParticleGroupFlags,
    ParticleGroupRecipe, ParticleGroupSource,
};
use liquidfun::{
    BodyDef, BodyId, BodyType, ContactTransition, ContactTransitionKind, FixtureDef,
    ManagedContactSnapshot, ParticleGroupId, ParticleSystemDef, ParticleSystemId, World,
};

use super::{
    BuiltScene, ControlEffect, PointerKind, RigidSegment, SceneError, SceneHooks,
    attach_basin_fixture,
};
use crate::session::SessionError;

const PARTICLE_RADIUS: f32 = 0.25;
const MAXIMUM_PARTICLE_COUNT: usize = 10240;
const GRAVITY: Vec2 = Vec2::new(0.0, -10.0);
const CIRCLE_RADIUS: f32 = 2.0;
const CIRCLE_DENSITY: f32 = 0.5;
const CIRCLE_COUNT: usize = 6;
const STEP_SECONDS: f32 = 1.0 / 60.0;
// Ring length 16 (upstream `c_maxVFX` is 50) and splash radius 1.5 m.
// Speed 15 and lifetime 0.75 are the deterministic playground choices inside the upstream bands.
const SPARK_RING_LEN: usize = 16;
const SPARK_RADIUS: f32 = 1.5;
const SPARK_SPEED: f32 = 15.0;
const SPARK_LIFETIME_SECONDS: f32 = 0.75;
const SPARK_PALETTE: [ParticleColor; 4] = [
    ParticleColor::new(248, 113, 113, 255),
    ParticleColor::new(61, 220, 151, 255),
    ParticleColor::new(57, 211, 199, 255),
    ParticleColor::new(244, 247, 250, 255),
];

struct BurstSlot {
    maybe_group: Option<ParticleGroupId>,
    remaining_seconds: f32,
    original: ParticleColor,
}

impl BurstSlot {
    const fn empty() -> Self {
        Self {
            maybe_group: None,
            remaining_seconds: 0.0,
            original: ParticleColor::ZERO,
        }
    }
}

const FLOOR: [Vec2; 4] = [
    Vec2::new(-40.0, -10.0),
    Vec2::new(40.0, -10.0),
    Vec2::new(40.0, 0.0),
    Vec2::new(-40.0, 0.0),
];
const CEILING: [Vec2; 4] = [
    Vec2::new(-40.0, 40.0),
    Vec2::new(40.0, 40.0),
    Vec2::new(40.0, 50.0),
    Vec2::new(-40.0, 50.0),
];
const LEFT_WALL: [Vec2; 4] = [
    Vec2::new(-40.0, -1.0),
    Vec2::new(-20.0, -1.0),
    Vec2::new(-20.0, 40.0),
    Vec2::new(-40.0, 40.0),
];
const RIGHT_WALL: [Vec2; 4] = [
    Vec2::new(20.0, -1.0),
    Vec2::new(40.0, -1.0),
    Vec2::new(40.0, 40.0),
    Vec2::new(20.0, 40.0),
];
const WALLS: [[Vec2; 4]; 4] = [FLOOR, CEILING, LEFT_WALL, RIGHT_WALL];

struct SparkyHooks {
    wall_segments: [RigidSegment; 16],
    sparkable_bodies: [BodyId; CIRCLE_COUNT],
    bursts: [BurstSlot; SPARK_RING_LEN],
    next_slot: usize,
}

pub(crate) fn build(presets: &[(String, String)]) -> Result<BuiltScene, SessionError> {
    if !presets.is_empty() {
        return Err(SessionError::UnknownControl);
    }
    build_sparky().map_err(|_error| SessionError::SceneConstruction)
}

fn build_sparky() -> Result<BuiltScene, SceneError> {
    let mut world = World::new().map_err(|_error| SceneError::World)?;
    world
        .set_gravity(GRAVITY)
        .map_err(|_error| SceneError::Gravity)?;
    let ground = world
        .create_body(&BodyDef::default())
        .map_err(|_error| SceneError::Body)?;
    for vertices in WALLS {
        attach_basin_fixture(&mut world, ground, &vertices)?;
    }
    let mut sparkable_bodies = [ground; CIRCLE_COUNT];
    for (index, slot) in sparkable_bodies.iter_mut().enumerate() {
        *slot = create_sparkable_circle(&mut world, index)?;
    }
    let particle_system = create_particle_system(&mut world)?;

    Ok(BuiltScene {
        world,
        particle_system,
        particle_radius: PARTICLE_RADIUS,
        hooks: Box::new(SparkyHooks {
            wall_segments: wall_segments(),
            sparkable_bodies,
            bursts: [const { BurstSlot::empty() }; SPARK_RING_LEN],
            next_slot: 0,
        }),
    })
}

fn create_particle_system(world: &mut World) -> Result<ParticleSystemId, SceneError> {
    let definition = ParticleSystemDef::default()
        .with_radius(PARTICLE_RADIUS)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_maximum_count(MAXIMUM_PARTICLE_COUNT)
        .map_err(|_error| SceneError::ParticleSystem)?
        .with_destruction_by_age(false);
    world
        .create_particle_system_with_def(&definition)
        .map_err(|_error| SceneError::ParticleSystem)
}

fn create_sparkable_circle(world: &mut World, index: usize) -> Result<BodyId, SceneError> {
    let center = Vec2::new(circle_x(index), circle_y(index));
    let definition =
        BodyDef::new(BodyType::Dynamic, center, 0.0, true).map_err(|_error| SceneError::Body)?;
    let body = world
        .create_body(&definition)
        .map_err(|_error| SceneError::Body)?;
    let circle =
        CircleShape::new(Vec2::ZERO, CIRCLE_RADIUS).map_err(|_error| SceneError::Geometry)?;
    let fixture = FixtureDef::new(
        Shape::from(circle),
        CIRCLE_DENSITY,
        0.2,
        0.0,
        false,
        FilterData::default(),
    )
    .map_err(|_error| SceneError::Fixture)?;
    world
        .create_fixture(body, &fixture)
        .map_err(|_error| SceneError::Fixture)?;
    Ok(body)
}

fn circle_x(index: usize) -> f32 {
    if index.is_multiple_of(2) { -1.5 } else { 1.5 }
}

fn circle_y(index: usize) -> f32 {
    let step = u8::try_from(index).unwrap_or(0);
    7.0 + 4.5 * f32::from(step)
}

fn wall_segments() -> [RigidSegment; 16] {
    let mut segments = [RigidSegment {
        start: Vec2::ZERO,
        end: Vec2::ZERO,
    }; 16];
    let mut index = 0usize;
    for wall in WALLS {
        for edge in 0..4 {
            segments[index] = RigidSegment {
                start: wall[edge],
                end: wall[(edge + 1) % 4],
            };
            index += 1;
        }
    }
    segments
}

fn burst_origin(
    world: &World,
    contact: &ManagedContactSnapshot,
    sparkable_bodies: &[BodyId; CIRCLE_COUNT],
) -> Result<Vec2, SessionError> {
    if let Some(point) = manifold_point(world, contact) {
        return Ok(point);
    }
    let Some(body) = contact
        .bodies()
        .into_iter()
        .find(|body| sparkable_bodies.contains(body))
    else {
        return Err(SessionError::StepFailed);
    };
    world
        .body_snapshot(body)
        .map(|snapshot| snapshot.transform().position())
        .map_err(|_error| SessionError::StepFailed)
}

fn manifold_point(world: &World, contact: &ManagedContactSnapshot) -> Option<Vec2> {
    let manifold = contact.maybe_manifold()?;
    let bodies = contact.bodies();
    let fixtures = contact.fixtures();
    let transform_a = world.body_snapshot(bodies[0]).ok()?.transform();
    let transform_b = world.body_snapshot(bodies[1]).ok()?.transform();
    let radius_a = world.fixture_snapshot(fixtures[0]).ok()?.shape().radius();
    let radius_b = world.fixture_snapshot(fixtures[1]).ok()?.shape().radius();
    liquidfun::collision::world_manifold(manifold, transform_a, radius_a, transform_b, radius_b)
        .ok()
        .flatten()
        .and_then(|world_manifold| world_manifold.points().first().copied())
        .map(liquidfun::collision::WorldManifoldPoint::point)
}

fn create_powder_group(
    world: &mut World,
    system: ParticleSystemId,
    origin: Vec2,
    color: ParticleColor,
) -> Result<ParticleGroupId, SessionError> {
    let circle =
        CircleShape::new(origin, SPARK_RADIUS).map_err(|_error| SessionError::StepFailed)?;
    let source = ParticleGroupSource::filled_shapes(vec![Shape::from(circle)])
        .map_err(|_error| SessionError::StepFailed)?;
    let recipe = ParticleGroupRecipe::new(source, ParticleGroupDestination::New)
        .with_particle_flags(ParticleFlags::POWDER)
        .with_group_flags(ParticleGroupFlags::empty())
        .with_color(color);
    world
        .create_particle_group(system, &recipe)
        .map_err(|_error| SessionError::StepFailed)
}

fn launch_members(
    world: &mut World,
    system: ParticleSystemId,
    group: ParticleGroupId,
    origin: Vec2,
) -> Result<(), SessionError> {
    let member_ids = world
        .particle_group_view(group)
        .map_err(|_error| SessionError::StepFailed)?
        .member_ids()
        .to_vec();
    let motions = member_motions(world, system, &member_ids, origin)?;
    for (member, velocity) in motions {
        world
            .set_particle_velocity(member, velocity)
            .map_err(|_error| SessionError::StepFailed)?;
    }
    Ok(())
}

fn member_motions(
    world: &World,
    system: ParticleSystemId,
    member_ids: &[liquidfun::ParticleId],
    origin: Vec2,
) -> Result<Vec<(liquidfun::ParticleId, Vec2)>, SessionError> {
    let view = world
        .particle_system_view(system)
        .map_err(|_error| SessionError::StepFailed)?;
    let mut motions = Vec::with_capacity(member_ids.len());
    for &member in member_ids {
        let Some(index) = view.particle_ids().iter().position(|id| *id == member) else {
            return Err(SessionError::StepFailed);
        };
        let position = view.positions()[index];
        motions.push((member, (position - origin) * SPARK_SPEED));
    }
    Ok(motions)
}

fn faded_color(original: ParticleColor, remaining_seconds: f32) -> ParticleColor {
    let coefficient = color_coefficient(remaining_seconds);
    let [red, green, blue, _alpha] = original.components();
    ParticleColor::new(
        scale_channel(red, coefficient),
        scale_channel(green, coefficient),
        scale_channel(blue, coefficient),
        255,
    )
}

fn color_coefficient(remaining_seconds: f32) -> f32 {
    let half_life = SPARK_LIFETIME_SECONDS * 0.5;
    if remaining_seconds >= half_life {
        return 1.0;
    }
    1.0 - ((half_life - remaining_seconds) / half_life)
}

fn scale_channel(channel: u8, coefficient: f32) -> u8 {
    let scaled = f32::from(channel) * coefficient;
    if !scaled.is_finite() || scaled <= 0.0 {
        return 0;
    }
    if scaled >= f32::from(u8::MAX) {
        return u8::MAX;
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "scaled is already inside 0.0..255.0"
    )]
    let byte = scaled.trunc() as u8;
    byte
}

impl SparkyHooks {
    fn is_sparkable(&self, body: BodyId) -> bool {
        self.sparkable_bodies.contains(&body)
    }

    fn spawn_from_begins(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
        transitions: &[ContactTransition],
    ) -> Result<(), SessionError> {
        for transition in transitions {
            if transition.kind() != ContactTransitionKind::Begin {
                continue;
            }
            let contact = transition.contact();
            if !contact
                .bodies()
                .into_iter()
                .any(|body| self.is_sparkable(body))
            {
                continue;
            }
            self.spawn_burst(world, system, contact)?;
        }
        Ok(())
    }

    fn spawn_burst(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
        contact: &ManagedContactSnapshot,
    ) -> Result<(), SessionError> {
        let origin = burst_origin(world, contact, &self.sparkable_bodies)?;
        let slot_index = self.next_slot;
        self.release_slot(world, system, slot_index)?;
        let original = SPARK_PALETTE[slot_index % SPARK_PALETTE.len()];
        let group = create_powder_group(world, system, origin, original)?;
        launch_members(world, system, group, origin)?;
        self.bursts[slot_index] = BurstSlot {
            maybe_group: Some(group),
            remaining_seconds: SPARK_LIFETIME_SECONDS,
            original,
        };
        self.next_slot = slot_index.saturating_add(1) % SPARK_RING_LEN;
        Ok(())
    }

    fn age_bursts(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
    ) -> Result<(), SessionError> {
        for index in 0..SPARK_RING_LEN {
            if self.bursts[index].maybe_group.is_none() {
                continue;
            }
            let remaining = (self.bursts[index].remaining_seconds - STEP_SECONDS).max(0.0);
            self.bursts[index].remaining_seconds = remaining;
            if remaining <= 0.0 {
                self.release_slot(world, system, index)?;
                continue;
            }
            self.write_fade(world, index)?;
        }
        Ok(())
    }

    fn write_fade(&self, world: &mut World, index: usize) -> Result<(), SessionError> {
        let Some(group) = self.bursts[index].maybe_group else {
            return Ok(());
        };
        let members = world
            .particle_group_view(group)
            .map_err(|_error| SessionError::StepFailed)?
            .member_ids()
            .to_vec();
        let color = faded_color(
            self.bursts[index].original,
            self.bursts[index].remaining_seconds,
        );
        world
            .set_particle_colors(&members, color)
            .map_err(|_error| SessionError::StepFailed)
    }

    fn release_slot(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
        index: usize,
    ) -> Result<(), SessionError> {
        let Some(group) = self.bursts[index].maybe_group else {
            return Ok(());
        };
        if world.particle_group_view(group).is_ok() {
            world
                .destroy_particle_group_particles(group, false)
                .map_err(|_error| SessionError::StepFailed)?;
            world
                .compact_pending_particles(system)
                .map_err(|_error| SessionError::StepFailed)?;
        }
        self.bursts[index].maybe_group = None;
        self.bursts[index].remaining_seconds = 0.0;
        Ok(())
    }
}

impl SceneHooks for SparkyHooks {
    fn on_after_step(
        &mut self,
        world: &mut World,
        system: ParticleSystemId,
        transitions: &[ContactTransition],
    ) -> Result<(), SessionError> {
        self.spawn_from_begins(world, system, transitions)?;
        self.age_bursts(world, system)
    }

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
        Ok(self.wall_segments.to_vec())
    }

    fn collect_circles(&self, world: &World) -> Result<Vec<(Vec2, f32)>, SessionError> {
        let mut circles = Vec::with_capacity(self.sparkable_bodies.len());
        for &body in &self.sparkable_bodies {
            let position = world
                .body_snapshot(body)
                .map_err(|_error| SessionError::FrameCaptureFailed)?
                .transform()
                .position();
            circles.push((position, CIRCLE_RADIUS));
        }
        Ok(circles)
    }
}
