//! D-26 flip measurement: one batched rebuild of all sand (including frozen
//! grains) at the point reflection, a 10 s rerun, then the per-particle
//! `set_particle_position` comparison. Never flips by inverting gravity.

use std::collections::HashMap;
use std::time::Instant;

use liquidfun::math::Vec2;
use liquidfun::{NoDecisionHook, ParticleFlags, ParticleGroupId, ParticleId, StepLimits, World};

use super::geometry::reflect;
use super::{
    Config, ORDER_TOLERANCE, POST_FLIP_STEPS, POST_SAMPLE_STEPS, RERUN_FRACTION, Rule, Run,
    STEPS_PER_SECOND, SpikeResult, counts, frozen, sand_recipe,
};

/// Particle state copied before the rebuild.
struct Copied {
    particles: Vec<ParticleId>,
    positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    flags: Vec<ParticleFlags>,
    groups: Vec<Option<ParticleGroupId>>,
}

fn copy_state(world: &World, run: &Run) -> SpikeResult<Copied> {
    let view = world.particle_system_view(run.system)?;
    Ok(Copied {
        particles: view.particle_ids().to_vec(),
        positions: view.positions().to_vec(),
        velocities: view.velocities().to_vec(),
        flags: view.flags().to_vec(),
        groups: view.group_ids().to_vec(),
    })
}

/// Index batches to rebuild: one per old group for rigid clumps (first
/// appearance order), otherwise one batch with every particle.
fn batches(config: &Config, copied: &Copied) -> Vec<Vec<usize>> {
    if !matches!(config.rule, Rule::Clumps) {
        return vec![(0..copied.particles.len()).collect()];
    }
    let mut slots: HashMap<Option<ParticleGroupId>, usize> = HashMap::new();
    let mut batches: Vec<Vec<usize>> = Vec::new();
    for (index, group) in copied.groups.iter().enumerate() {
        let slot = *slots.entry(*group).or_insert_with(|| {
            batches.push(Vec::new());
            batches.len() - 1
        });
        batches[slot].push(index);
    }
    batches
}

/// Rebuilt members per batch, paired with the source indices.
type Rebuilt = Vec<(Vec<ParticleId>, Vec<usize>)>;

/// The timed batched flip: mark all, compact once, one group per batch at
/// `−p`, then `−v` per member in member order.
fn batched_flip(config: &Config, run: &mut Run, copied: &Copied) -> SpikeResult<Rebuilt> {
    for particle in &copied.particles {
        run.world.mark_particle_for_destruction(*particle)?;
    }
    run.world.compact_pending_particles(run.system)?;
    let mut rebuilt = Vec::new();
    for batch in batches(config, copied) {
        let reflected: Vec<Vec2> = batch
            .iter()
            .map(|&i| reflect(copied.positions[i]))
            .collect();
        let group = run
            .world
            .create_particle_group(run.system, &sand_recipe(config, reflected)?)?;
        let members = run.world.particle_group_view(group)?.member_ids().to_vec();
        for (member, &index) in members.iter().zip(&batch) {
            run.world
                .set_particle_velocity(*member, copied.velocities[index] * -1.0)?;
        }
        rebuilt.push((members, batch));
    }
    Ok(rebuilt)
}

/// Assumption A1: members come back in source order. On a mismatch the
/// velocities are re-assigned by nearest position (untimed) and `false`
/// is returned.
fn check_order(run: &mut Run, copied: &Copied, rebuilt: &Rebuilt) -> SpikeResult<bool> {
    let mut in_order = true;
    for (members, batch) in rebuilt {
        for (member, &index) in members.iter().zip(batch) {
            let position = run.world.particle_snapshot(*member)?.position();
            if (position - reflect(copied.positions[index])).length() > ORDER_TOLERANCE {
                in_order = false;
            }
        }
    }
    if in_order {
        return Ok(true);
    }
    for (members, batch) in rebuilt {
        for member in members {
            let position = run.world.particle_snapshot(*member)?.position();
            let nearest = batch
                .iter()
                .copied()
                .min_by(|&a, &b| {
                    let da = (position - reflect(copied.positions[a])).length();
                    let db = (position - reflect(copied.positions[b])).length();
                    da.total_cmp(&db)
                })
                .ok_or("empty rebuild batch")?;
            run.world
                .set_particle_velocity(*member, copied.velocities[nearest] * -1.0)?;
        }
    }
    Ok(false)
}

/// Runs the hook and world for the post-flip seconds; returns `lower` at the end.
fn rerun(config: &Config, run: &mut Run) -> SpikeResult<usize> {
    let mut lower_end = 0;
    for step in 1..=POST_FLIP_STEPS {
        run.hook.on_advance(&mut run.world, run.system)?;
        run.world
            .step(run.step_config, &mut NoDecisionHook, StepLimits::default())?;
        if step % POST_SAMPLE_STEPS == 0 {
            let (upper, lower) = counts(run.world.particle_system_view(run.system)?.positions());
            let (frozen_count, _) = frozen(&run.world, run.system)?;
            println!(
                "POST hourglass {} t {:.2} upper {upper} lower {lower} frozen {frozen_count}",
                config.label,
                step as f32 / STEPS_PER_SECOND as f32
            );
            lower_end = lower;
        }
    }
    Ok(lower_end)
}

/// Timed per-particle comparison over non-`WALL` particles; a `WALL`
/// particle, if any, is tried once untimed and reported.
fn per_particle_flip(run: &mut Run) -> SpikeResult<(f64, usize, String)> {
    let copied = copy_state(&run.world, run)?;
    let movable: Vec<usize> = (0..copied.particles.len())
        .filter(|&index| !copied.flags[index].contains(ParticleFlags::WALL))
        .collect();
    let started = Instant::now();
    for &index in &movable {
        run.world
            .set_particle_position(copied.particles[index], reflect(copied.positions[index]))?;
    }
    for &index in &movable {
        run.world
            .set_particle_velocity(copied.particles[index], copied.velocities[index] * -1.0)?;
    }
    let elapsed_ms = started.elapsed().as_secs_f64() * 1e3;
    let maybe_wall = (0..copied.particles.len())
        .find(|&index| copied.flags[index].contains(ParticleFlags::WALL));
    let wall_edit = match maybe_wall {
        None => "none".to_owned(),
        Some(index) => match run
            .world
            .set_particle_position(copied.particles[index], reflect(copied.positions[index]))
        {
            Ok(()) => "ok".to_owned(),
            Err(error) => {
                println!("NOTE hourglass wall position edit error {error:?}");
                "error".to_owned()
            }
        },
    };
    Ok((elapsed_ms, movable.len(), wall_edit))
}

/// D-26 flip cost, post-flip rerun and per-particle comparison.
pub(super) fn measure_flip(config: &Config, run: &mut Run) -> SpikeResult<()> {
    let started = Instant::now();
    let copied = copy_state(&run.world, run)?;
    let rebuilt = batched_flip(config, run, &copied)?;
    let batched_ms = started.elapsed().as_secs_f64() * 1e3;
    let total = copied.particles.len();
    let after = run
        .world
        .particle_system_view(run.system)?
        .particle_ids()
        .len();
    let count_ok = after == total
        && rebuilt
            .iter()
            .all(|(members, batch)| members.len() == batch.len());
    let order_ok = check_order(run, &copied, &rebuilt)?;
    let frozen_in = copied
        .flags
        .iter()
        .filter(|flags| flags.contains(ParticleFlags::WALL))
        .count();
    let lower = rerun(config, run)?;
    let (per_particle_ms, moved, wall_edit) = per_particle_flip(run)?;
    println!(
        "FLIP hourglass {} batched_ms {batched_ms:.3} per_particle_ms {per_particle_ms:.3} \
         count_ok {count_ok} order_ok {order_ok} rerun_lower_10s {lower}/{total} \
         rerun_90pct {} groups {} frozen_rebuilt {frozen_in} moved {moved} wall_edit {wall_edit}",
        config.label,
        lower as f32 >= RERUN_FRACTION * total as f32,
        rebuilt.len()
    );
    Ok(())
}
