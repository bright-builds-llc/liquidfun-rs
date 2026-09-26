//! Source-ordered world integration for the closed particle solver graph.

mod body_coupling;
mod executor;

use crate::arena::Arena;
use crate::collision::{Aabb, ChildIndex, RayCastInput, Shape};
use crate::math::{Transform, Vec2, max, min};
use crate::particle::body_contact::{self, FixtureContactSource};
use crate::particle::contact::{
    listener_effects_from_stored, particle_contact_listeners_active, semantic_from_stored,
};
use crate::particle::contact_scan::{self, ContactFillError, ContactProxy};
use crate::particle::solver::boundary::{
    BoundaryCandidate, FilteredCollisionHit, collision_start_from_previous_transform,
};
use crate::particle::solver::preparation;
use crate::particle::{ParticleFlags, ParticleSystemView};
use crate::{
    BodyId, CollisionDecisionHook, FixtureId, ParticleSystemId, StepConfiguration, StepError, World,
};

use self::executor::{SystemPassExecutor, boundary_error};
use super::object::{Body, ParticleSystem};
use super::step::ContactHookRun;

impl World {
    pub(super) fn run_particle_solver<H: CollisionDecisionHook>(
        &mut self,
        configuration: StepConfiguration,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        let backup_bodies = self.bodies.clone();
        let backup_groups = self.particle_groups.clone();
        self.run_particle_lifecycle_step(configuration.time_step(), hook_run)?;
        let mut candidate_bodies = self.bodies.replace_with_empty();
        let mut candidate_systems = self.particle_systems.replace_with_empty();
        let system_order = self.particle_system_order.clone();
        let result = (|| {
            for system in system_order {
                let mut executor = SystemPassExecutor::new(
                    self,
                    system,
                    configuration,
                    &mut candidate_systems,
                    &mut candidate_bodies,
                    hook_run,
                );
                crate::particle::solver::run_particle_solver(configuration, &mut executor)?;
            }
            Ok(())
        })();
        self.bodies = candidate_bodies;
        self.particle_systems = candidate_systems;
        if result.is_err() {
            self.bodies = backup_bodies;
            self.particle_groups = backup_groups;
        }
        result
    }

    fn fixture_contact_sources(&self, bodies: &Arena<Body, BodyId>) -> Vec<FixtureContactSource> {
        let mut sources = Vec::new();
        for body_id in &self.body_order {
            let body = bodies
                .get(*body_id)
                .expect("world body order contains only live bodies");
            if !body.state.snapshot().is_active() {
                continue;
            }
            for fixture_id in &body.fixtures {
                let fixture = self
                    .fixtures
                    .get(*fixture_id)
                    .expect("body fixture adjacency contains only live fixtures");
                if fixture.definition.is_sensor() {
                    continue;
                }
                sources.push(FixtureContactSource {
                    fixture: *fixture_id,
                    body: *body_id,
                    shape: fixture.definition.shape().clone(),
                    transform: body.state.transform(),
                    center: body.state.sweep().center(),
                    inverse_mass: body.state.inverse_mass(),
                    inverse_inertia: body.state.inverse_inertia(),
                });
            }
        }
        sources
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the collision query keeps the candidate, fixture expansion, and sorted proxies explicit"
    )]
    fn filtered_collision_hits<H: CollisionDecisionHook>(
        &self,
        candidate: &BoundaryCandidate,
        bodies: &Arena<Body, BodyId>,
        time_step: f32,
        particle_iteration: u32,
        hook_run: &mut ContactHookRun<'_, H>,
        expansion: Vec2,
        proxies: &[ContactProxy],
        diameter: f32,
    ) -> Result<Vec<FilteredCollisionHit>, StepError> {
        let fixtures = ccd_fixture_records(self, bodies, expansion)?;
        let mut hits = Vec::new();
        let motion = max_particle_motion(&candidate.velocities, time_step);
        let proxies_match = proxies.len() == candidate.positions.len();
        for fixture in &fixtures {
            let static_fixture = fixture.previous_transform == fixture.current_transform;
            for (child, maybe_aabb) in &fixture.children {
                // Later iterations raycast from the current position, so the
                // fixture AABB plus the max travel pad is a superset for
                // moving fixtures too. Iteration 0 remaps the ray start
                // through the previous body transform.
                let spatially_filtered =
                    proxies_match && (static_fixture || particle_iteration != 0);
                if spatially_filtered && let Some(aabb) = *maybe_aabb {
                    let queried =
                        query_particles_for_fixture(proxies, diameter, aabb, motion, |row| {
                            push_fixture_particle_hit(
                                candidate,
                                fixture,
                                *child,
                                maybe_aabb.as_ref(),
                                row,
                                time_step,
                                particle_iteration,
                                hook_run,
                                &mut hits,
                            )
                        })?;
                    if queried {
                        continue;
                    }
                }
                for particle in 0..candidate.positions.len() {
                    push_fixture_particle_hit(
                        candidate,
                        fixture,
                        *child,
                        maybe_aabb.as_ref(),
                        particle,
                        time_step,
                        particle_iteration,
                        hook_run,
                        &mut hits,
                    )?;
                }
            }
        }
        hits.sort_by_key(|hit| hit.particle);
        Ok(hits)
    }

    fn update_particle_contacts<H: CollisionDecisionHook>(
        system: ParticleSystemId,
        systems: &mut Arena<ParticleSystem, ParticleSystemId>,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        let (diameter, listeners, previous) = {
            let record = systems
                .get(system)
                .expect("system remains live during particle contact update");
            let view = ParticleSystemView::new(&record.storage);
            let listeners = particle_contact_listeners_active(&view);
            let previous = if listeners {
                view.stored_particle_contacts().to_vec()
            } else {
                Vec::new()
            };
            (2.0 * record.definition.radius(), listeners, previous)
        };
        let (contacts, proxies, filled) = {
            let record = systems
                .get_mut(system)
                .expect("system remains live during particle contact update");
            let mut contacts = record.storage.take_particle_contacts();
            let mut proxies = record.storage.take_contact_proxies();
            let filled = contact_scan::fill_stored_contacts(
                record.storage.positions(),
                record.storage.flags(),
                record.storage.particle_ids(),
                diameter,
                &mut proxies,
                &mut contacts,
                &mut |contact| hook_run.should_collide_particle_pair(contact),
            );
            (contacts, proxies, filled)
        };
        let record = systems
            .get_mut(system)
            .expect("system remains live during particle contact commit");
        record.storage.install_contact_proxies(proxies);
        record.storage.install_particle_contacts(contacts);
        filled.map_err(|error| match error {
            ContactFillError::Proxy(error) => StepError::ParticleProxy(error),
            ContactFillError::Contact(error) => StepError::ParticleContact(error),
        })?;
        let effects = if listeners {
            let record = systems
                .get(system)
                .expect("system remains live during particle contact listeners");
            let view = ParticleSystemView::new(&record.storage);
            let semantic = semantic_from_stored(&view, view.stored_particle_contacts())
                .map_err(StepError::ParticleContact)?;
            listener_effects_from_stored(&view, &previous, &semantic)
                .map_err(StepError::ParticleContact)?
        } else {
            Vec::new()
        };
        hook_run.ensure_lifecycle_capacity(effects.len())?;
        for effect in effects {
            hook_run.record_particle_contact(effect)?;
        }
        Ok(())
    }

    fn update_body_contacts<H: CollisionDecisionHook>(
        system: ParticleSystemId,
        systems: &mut Arena<ParticleSystem, ParticleSystemId>,
        sources: &[FixtureContactSource],
        timestamp: u32,
        hook_run: &mut ContactHookRun<'_, H>,
    ) -> Result<(), StepError> {
        let record = systems
            .get(system)
            .expect("system remains live during body contact update");
        let view = ParticleSystemView::new(&record.storage);
        let previous = if body_contact::fixture_contact_listeners_active(&view) {
            record.storage.semantic_body_contacts()
        } else {
            Vec::new()
        };
        let diameter = 2.0 * record.definition.radius();
        let proxies = record.storage.contact_proxies();
        let update = body_contact::generate(
            &view,
            proxies,
            sources,
            &previous,
            diameter,
            record.definition.density(),
            record.definition.uses_strict_contact_check(),
            |contact| hook_run.should_collide_fixture_particle(contact),
        );
        hook_run.ensure_lifecycle_capacity(update.effects().len())?;
        for effect in update.effects().iter().copied() {
            hook_run.record_particle_body_contact(effect)?;
        }
        let record = systems
            .get_mut(system)
            .expect("system remains live during body contact commit");
        preparation::body_contacts(
            &mut record.storage,
            update.contacts(),
            timestamp,
            record.definition.stuck_threshold(),
        )
        .map_err(|_error| StepError::ParticleLifecycleInvariant)
    }
}

struct CcdFixtureRecord {
    body: BodyId,
    fixture: FixtureId,
    previous_transform: Transform,
    current_transform: Transform,
    body_local_center: Vec2,
    is_circle: bool,
    shape: Shape,
    children: Vec<(ChildIndex, Option<Aabb>)>,
}

fn max_particle_motion(velocities: &[Vec2], time_step: f32) -> f32 {
    let mut max_squared = 0.0_f32;
    for velocity in velocities {
        let speed_squared = velocity.length_squared();
        if speed_squared > max_squared {
            max_squared = speed_squared;
        }
    }
    // Extra slop covers the static-transform round trip inside collision start.
    time_step * max_squared.sqrt() + 1.0e-3
}

fn query_particles_for_fixture(
    proxies: &[ContactProxy],
    diameter: f32,
    fixture_aabb: Aabb,
    motion: f32,
    mut visit_row: impl FnMut(usize) -> Result<(), StepError>,
) -> Result<bool, StepError> {
    let pad = Vec2::new(motion, motion);
    let Some(query) = Aabb::new(
        fixture_aabb.lower_bound() - pad,
        fixture_aabb.upper_bound() + pad,
    )
    .ok() else {
        return Ok(false);
    };
    let mut failure = None;
    let queried = crate::particle::proxy::visit_sorted_tag_indices_in_aabb(
        proxies.len(),
        |index| proxies[index].tag,
        diameter,
        query,
        |index| {
            if failure.is_some() {
                return;
            }
            if let Err(error) = visit_row(proxies[index].row) {
                failure = Some(error);
            }
        },
    );
    if let Some(error) = failure {
        return Err(error);
    }
    match queried {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "one fixture-particle ray keeps the candidate, fixture, and step clock explicit"
)]
fn push_fixture_particle_hit<H: CollisionDecisionHook>(
    candidate: &BoundaryCandidate,
    fixture: &CcdFixtureRecord,
    child: ChildIndex,
    maybe_aabb: Option<&Aabb>,
    particle: usize,
    time_step: f32,
    particle_iteration: u32,
    hook_run: &mut ContactHookRun<'_, H>,
    hits: &mut Vec<FilteredCollisionHit>,
) -> Result<(), StepError> {
    let position = candidate.positions[particle];
    let velocity = candidate.velocities[particle];
    let end = position + time_step * velocity;
    let start = collision_start_from_previous_transform(
        position,
        fixture.previous_transform,
        fixture.current_transform,
        fixture.body_local_center,
        fixture.is_circle,
        particle_iteration,
    )
    .map_err(boundary_error)?;
    if start == end {
        return Ok(());
    }
    let maybe_travel = particle_travel_aabb(start, end);
    if let (Some(travel), Some(aabb)) = (maybe_travel, maybe_aabb.copied())
        && !travel.overlaps(aabb)
    {
        return Ok(());
    }
    let input = RayCastInput::new(start, end, 1.0)
        .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
    let Some(hit) = fixture
        .shape
        .ray_cast(input, fixture.current_transform, child)
        .map_err(|_error| StepError::ParticleLifecycleInvariant)?
    else {
        return Ok(());
    };
    let filter_contact = crate::ParticleBodyContact::new_internal(
        candidate.particle_ids[particle],
        fixture.body,
        fixture.fixture,
        0.0,
        hit.normal(),
        0.0,
    );
    if candidate.flags[particle].contains(ParticleFlags::FIXTURE_CONTACT_FILTER)
        && !hook_run.should_collide_fixture_particle(&filter_contact)
    {
        return Ok(());
    }
    hits.try_reserve(1)
        .map_err(|_error| StepError::LimitExceeded {
            resource: "filtered collision hits",
            limit: hits.len(),
        })?;
    hits.push(FilteredCollisionHit {
        particle,
        body: fixture.body,
        previous_transform: fixture.previous_transform,
        current_transform: fixture.current_transform,
        body_local_center: fixture.body_local_center,
        is_circle: fixture.is_circle,
        fraction: hit.fraction(),
        normal: hit.normal(),
    });
    Ok(())
}

fn ccd_fixture_records(
    world: &World,
    bodies: &Arena<Body, BodyId>,
    expansion: Vec2,
) -> Result<Vec<CcdFixtureRecord>, StepError> {
    let mut fixtures = Vec::new();
    for body_id in &world.body_order {
        let body = bodies
            .get(*body_id)
            .expect("world body order contains only live bodies");
        if !body.state.snapshot().is_active() {
            continue;
        }
        let previous_transform = body
            .state
            .sweep()
            .transform_at(0.0)
            .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
        let current_transform = body.state.transform();
        let body_local_center = body.state.sweep().local_center();
        for fixture_id in &body.fixtures {
            let fixture = world
                .fixtures
                .get(*fixture_id)
                .expect("body fixture adjacency contains only live fixtures");
            if fixture.definition.is_sensor() {
                continue;
            }
            let shape = fixture.definition.shape().clone();
            let mut children = Vec::with_capacity(shape.child_count());
            for child in 0..shape.child_count() {
                let child = shape
                    .child_index(child)
                    .map_err(|_error| StepError::ParticleLifecycleInvariant)?;
                let maybe_aabb = expanded_shape_aabb(&shape, current_transform, child, expansion);
                children.push((child, maybe_aabb));
            }
            fixtures.push(CcdFixtureRecord {
                body: *body_id,
                fixture: *fixture_id,
                previous_transform,
                current_transform,
                body_local_center,
                is_circle: matches!(shape, Shape::Circle(_)),
                shape,
                children,
            });
        }
    }
    Ok(fixtures)
}

fn expanded_shape_aabb(
    shape: &Shape,
    transform: Transform,
    child: ChildIndex,
    expansion: Vec2,
) -> Option<Aabb> {
    let aabb = shape.compute_aabb(transform, child).ok()?;
    Aabb::new(
        aabb.lower_bound() - expansion,
        aabb.upper_bound() + expansion,
    )
    .ok()
}

fn particle_travel_aabb(start: Vec2, end: Vec2) -> Option<Aabb> {
    Aabb::new(
        Vec2::new(min(start.x, end.x), min(start.y, end.y)),
        Vec2::new(max(start.x, end.x), max(start.y, end.y)),
    )
    .ok()
}
