//! Per-step sand-rule hooks (Phase 45 D-24), SceneHooks-shaped so a
//! passing rule can be lifted into Phase 51.

use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use liquidfun::math::Vec2;
use liquidfun::particle::ParticleColor;
use liquidfun::{ParticleDef, ParticleFlags, ParticleId, ParticleSystemId, World};

use super::metrics::nearest_rank;
use super::{Config, Rule, SAND_COLOR, SpikeResult};

/// Supported grains weigh at least this fraction of the lower-bulb median.
const SUPPORT_WEIGHT_FRACTION: f32 = 0.5;
const FREEZE_MAX_Y: f32 = -0.2;
const FREEZE_MAX_SPEED: f32 = 0.05;
const FREEZE_STEPS: u32 = 30;

/// Freeze bookkeeping (candidate 2).
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct FreezeStats {
    pub(super) total: usize,
    pub(super) max_per_step: usize,
    pub(super) steps: u32,
}

/// SceneHooks-shaped per-step hook so a passing rule can be lifted into
/// Phase 51.
pub(super) trait SpikeHook {
    fn on_advance(&mut self, world: &mut World, system: ParticleSystemId) -> SpikeResult<()>;

    /// Releases every frozen grain (flip or gravity change); returns how many.
    fn release_all(&mut self, _world: &mut World, _system: ParticleSystemId) -> SpikeResult<usize> {
        Ok(0)
    }

    fn freeze_stats(&self) -> FreezeStats {
        FreezeStats::default()
    }
}

struct NoHook;

impl SpikeHook for NoHook {
    fn on_advance(&mut self, _world: &mut World, _system: ParticleSystemId) -> SpikeResult<()> {
        Ok(())
    }
}

/// Weight threshold for "supported": half the median weight of the
/// non-wall lower-bulb particles this step.
fn support_threshold(positions: &[Vec2], weights: &[f32], flags: &[ParticleFlags]) -> f32 {
    let lower: Vec<f32> = (0..positions.len())
        .filter(|&index| positions[index].y < 0.0 && !flags[index].contains(ParticleFlags::WALL))
        .map(|index| weights[index])
        .collect();
    if lower.is_empty() {
        return f32::INFINITY;
    }
    SUPPORT_WEIGHT_FRACTION * nearest_rank(&lower, 0.5)
}

/// Candidate 1: slow, supported lower-bulb grains get `v · (1 − k)`.
struct StickSlip {
    v_slip: f32,
    k: f32,
}

impl SpikeHook for StickSlip {
    fn on_advance(&mut self, world: &mut World, system: ParticleSystemId) -> SpikeResult<()> {
        let edits = {
            let view = world.particle_system_view(system)?;
            let (positions, velocities) = (view.positions(), view.velocities());
            let threshold = support_threshold(positions, view.weights(), view.flags());
            (0..positions.len())
                .filter(|&index| positions[index].y < 0.0)
                .filter(|&index| velocities[index].length() < self.v_slip)
                .filter(|&index| view.weights()[index] >= threshold)
                .map(|index| {
                    (
                        view.particle_ids()[index],
                        velocities[index] * (1.0 - self.k),
                    )
                })
                .collect::<Vec<(ParticleId, Vec2)>>()
        };
        for (particle, velocity) in edits {
            world.set_particle_velocity(particle, velocity)?;
        }
        Ok(())
    }
}

/// Deterministic ordering key for a particle (`ParticleId` has no `Ord`).
fn grain_key(particle: ParticleId) -> u64 {
    let mut hasher = DefaultHasher::new();
    particle.hash(&mut hasher);
    hasher.finish()
}

/// Marks every listed particle, then compacts once.
fn remove_particles(
    world: &mut World,
    system: ParticleSystemId,
    particles: &[ParticleId],
) -> SpikeResult<()> {
    for particle in particles {
        world.mark_particle_for_destruction(*particle)?;
    }
    world.compact_pending_particles(system)?;
    Ok(())
}

/// Re-creates grains at rest with `flags`, keeping position and color.
fn create_grains(
    world: &mut World,
    system: ParticleSystemId,
    grains: &[(Vec2, ParticleColor)],
    flags: ParticleFlags,
) -> SpikeResult<()> {
    for (position, color) in grains {
        let definition = ParticleDef::default()
            .with_flags(flags)
            .with_color(*color)
            .with_position(*position)?;
        let receipt = world.create_particle_with_def(system, None, &definition)?;
        if !receipt.destruction_occurrences().is_empty() {
            return Err("re-creating a grain evicted particles".into());
        }
    }
    Ok(())
}

/// Candidate 2: grains slow and supported for 30 steps become `WALL`, at
/// most `per_step` per step, lowest first.
struct FreezeToWall {
    per_step: usize,
    base_flags: ParticleFlags,
    slow_steps: BTreeMap<u64, u32>,
    stats: FreezeStats,
}

/// A grain ready to freeze: `(y, key, id, position, color)`.
type Ready = (f32, u64, ParticleId, Vec2, ParticleColor);

impl FreezeToWall {
    fn ready_grains(&mut self, world: &World, system: ParticleSystemId) -> SpikeResult<Vec<Ready>> {
        let view = world.particle_system_view(system)?;
        let (positions, velocities, flags) = (view.positions(), view.velocities(), view.flags());
        let threshold = support_threshold(positions, view.weights(), flags);
        let maybe_colors = view.maybe_colors();
        let mut next = BTreeMap::new();
        let mut ready = Vec::new();
        for index in 0..positions.len() {
            let position = positions[index];
            let qualifies = !flags[index].contains(ParticleFlags::WALL)
                && position.y < FREEZE_MAX_Y
                && velocities[index].length() < FREEZE_MAX_SPEED
                && view.weights()[index] >= threshold;
            if !qualifies {
                continue;
            }
            let particle = view.particle_ids()[index];
            let key = grain_key(particle);
            let steps = self.slow_steps.get(&key).copied().unwrap_or(0) + 1;
            if next.insert(key, steps).is_some() {
                return Err(format!("grain key collision {key:016x}").into());
            }
            if steps >= FREEZE_STEPS {
                let color = maybe_colors.map_or(SAND_COLOR, |colors| colors[index]);
                ready.push((position.y, key, particle, position, color));
            }
        }
        self.slow_steps = next;
        ready.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        ready.truncate(self.per_step);
        Ok(ready)
    }
}

impl SpikeHook for FreezeToWall {
    fn on_advance(&mut self, world: &mut World, system: ParticleSystemId) -> SpikeResult<()> {
        let ready = self.ready_grains(world, system)?;
        self.stats.steps += 1;
        if ready.is_empty() {
            return Ok(());
        }
        let particles: Vec<ParticleId> = ready.iter().map(|grain| grain.2).collect();
        let grains: Vec<(Vec2, ParticleColor)> =
            ready.iter().map(|grain| (grain.3, grain.4)).collect();
        for grain in &ready {
            self.slow_steps.remove(&grain.1);
        }
        remove_particles(world, system, &particles)?;
        create_grains(world, system, &grains, ParticleFlags::WALL)?;
        self.stats.total += ready.len();
        self.stats.max_per_step = self.stats.max_per_step.max(ready.len());
        Ok(())
    }

    fn release_all(&mut self, world: &mut World, system: ParticleSystemId) -> SpikeResult<usize> {
        let (particles, grains) = {
            let view = world.particle_system_view(system)?;
            let maybe_colors = view.maybe_colors();
            let frozen: Vec<usize> = (0..view.flags().len())
                .filter(|&index| view.flags()[index].contains(ParticleFlags::WALL))
                .collect();
            let particles: Vec<ParticleId> = frozen
                .iter()
                .map(|&index| view.particle_ids()[index])
                .collect();
            let grains: Vec<(Vec2, ParticleColor)> = frozen
                .iter()
                .map(|&index| {
                    let color = maybe_colors.map_or(SAND_COLOR, |colors| colors[index]);
                    (view.positions()[index], color)
                })
                .collect();
            (particles, grains)
        };
        if particles.is_empty() {
            return Ok(0);
        }
        remove_particles(world, system, &particles)?;
        create_grains(world, system, &grains, self.base_flags)?;
        Ok(particles.len())
    }

    fn freeze_stats(&self) -> FreezeStats {
        self.stats
    }
}

pub(super) fn hook_for(config: &Config) -> Box<dyn SpikeHook> {
    match config.rule {
        Rule::Plain | Rule::Clumps => Box::new(NoHook),
        Rule::StickSlip { v_slip, k } => Box::new(StickSlip { v_slip, k }),
        Rule::Freeze { per_step } => Box::new(FreezeToWall {
            per_step,
            base_flags: config.flags,
            slow_steps: BTreeMap::new(),
            stats: FreezeStats::default(),
        }),
    }
}
