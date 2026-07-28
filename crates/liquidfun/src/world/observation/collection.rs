use super::{
    BodyObservation, BroadPhaseObservation, ContactObservation, FixtureObservation,
    JointObservation, ParticleBodyContactObservation, ParticleColor, ParticleContactObservation,
    ParticleObservation, ParticleSystemStatistics, ParticleSystemView, ParticleWorldStatistics,
    REVIEWED_MAX_BODIES, REVIEWED_MAX_FIXTURES, REVIEWED_MAX_JOINTS, World, WorldDiagnostics,
    WorldObservation, WorldObservationError, WorldObservationLimitError, WorldObservationLimits,
    WorldObservationResource,
};

struct ObservationCounts {
    particle_contacts: usize,
    particle_body_contacts: usize,
    broad_phase_observations: usize,
}

struct CollectedParticleObservations {
    particles: Vec<ParticleObservation>,
    contacts: Vec<ParticleContactObservation>,
    body_contacts: Vec<ParticleBodyContactObservation>,
    statistics: Vec<ParticleSystemStatistics>,
    world_statistics: ParticleWorldStatistics,
}

impl World {
    /// Collects one owned, bounded renderer-neutral observation.
    ///
    /// Counts and tree metrics are exact. Current rigid contacts preserve
    /// manager order; particle records preserve newest-first system order and
    /// each system's stored contact order; fixture-child AABBs preserve
    /// newest-first body and fixture order. Every storage coordinate is
    /// translated to a stable public identity before it crosses this boundary.
    ///
    /// # Errors
    ///
    /// Returns a typed capacity error before output allocation when any
    /// reviewed collection limit would be exceeded. An invariant error names
    /// only the bounded semantic category that could not be translated.
    pub fn world_observation(
        &self,
        limits: WorldObservationLimits,
    ) -> Result<WorldObservation, WorldObservationError> {
        let diagnostics = self.world_diagnostics();
        let counts = self.preflight_observation(diagnostics, limits)?;
        let (bodies, fixtures) = self.collect_body_fixture_observations()?;
        let joints = self.collect_joint_observations()?;
        let contacts = self
            .contact_manager
            .contacts()
            .iter()
            .map(|contact| ContactObservation::from_snapshot(&contact.snapshot()))
            .collect();
        let particles = self.collect_particle_observations(&counts)?;
        let broad_phase_observations =
            self.collect_broad_phase_observations(counts.broad_phase_observations)?;

        Ok(WorldObservation {
            diagnostics,
            bodies,
            fixtures,
            joints,
            particles: particles.particles,
            contacts,
            particle_contacts: particles.contacts,
            particle_body_contacts: particles.body_contacts,
            broad_phase_observations,
            particle_statistics: particles.statistics,
            particle_world_statistics: particles.world_statistics,
        })
    }

    fn preflight_observation(
        &self,
        diagnostics: WorldDiagnostics,
        limits: WorldObservationLimits,
    ) -> Result<ObservationCounts, WorldObservationError> {
        check_collection_bound(
            WorldObservationResource::Bodies,
            diagnostics.body_count(),
            REVIEWED_MAX_BODIES,
        )?;
        check_collection_bound(
            WorldObservationResource::Fixtures,
            diagnostics.fixture_count(),
            REVIEWED_MAX_FIXTURES,
        )?;
        check_collection_bound(
            WorldObservationResource::Joints,
            diagnostics.joint_count(),
            REVIEWED_MAX_JOINTS,
        )?;
        check_collection_bound(
            WorldObservationResource::Contacts,
            diagnostics.contact_count(),
            limits.contacts,
        )?;
        check_collection_bound(
            WorldObservationResource::BroadPhaseObservations,
            diagnostics.proxy_count(),
            limits.broad_phase_observations,
        )?;
        check_collection_bound(
            WorldObservationResource::ParticleSystems,
            self.particle_system_order.len(),
            limits.particle_systems,
        )?;

        let mut particle_count = 0_usize;
        let mut particle_contact_count = 0_usize;
        let mut particle_body_contact_count = 0_usize;
        for system in &self.particle_system_order {
            let record = self.particle_systems.get(*system).map_err(|_error| {
                WorldObservationError::InvalidState {
                    resource: WorldObservationResource::ParticleSystems,
                }
            })?;
            particle_count = checked_add_resource(
                WorldObservationResource::Particles,
                particle_count,
                record.storage.len(),
            )?;
            particle_contact_count = checked_add_resource(
                WorldObservationResource::ParticleContacts,
                particle_contact_count,
                record.storage.particle_contacts().len(),
            )?;
            let view = ParticleSystemView::new(&record.storage);
            particle_body_contact_count = checked_add_resource(
                WorldObservationResource::ParticleBodyContacts,
                particle_body_contact_count,
                view.body_contacts().len(),
            )?;
        }
        check_collection_bound(
            WorldObservationResource::Particles,
            particle_count,
            limits.particles,
        )?;
        check_collection_bound(
            WorldObservationResource::ParticleContacts,
            particle_contact_count,
            limits.particle_contacts,
        )?;
        check_collection_bound(
            WorldObservationResource::ParticleBodyContacts,
            particle_body_contact_count,
            limits.particle_body_contacts,
        )?;

        Ok(ObservationCounts {
            particle_contacts: particle_contact_count,
            particle_body_contacts: particle_body_contact_count,
            broad_phase_observations: diagnostics.proxy_count(),
        })
    }

    fn collect_particle_observations(
        &self,
        counts: &ObservationCounts,
    ) -> Result<CollectedParticleObservations, WorldObservationError> {
        let mut contacts = Vec::with_capacity(counts.particle_contacts);
        let mut body_contacts = Vec::with_capacity(counts.particle_body_contacts);
        let mut particles = Vec::new();
        let mut particle_statistics = Vec::with_capacity(self.particle_system_order.len());
        let mut particle_world_statistics = ParticleWorldStatistics::default();
        for system in &self.particle_system_order {
            let record = self.particle_systems.get(*system).map_err(|_error| {
                WorldObservationError::InvalidState {
                    resource: WorldObservationResource::ParticleSystems,
                }
            })?;
            let view = ParticleSystemView::new(&record.storage);
            let maybe_colors = view.maybe_colors();
            particles.extend(
                view.particle_ids()
                    .iter()
                    .enumerate()
                    .map(|(index, particle)| ParticleObservation {
                        system: *system,
                        particle: *particle,
                        position: view.positions()[index],
                        radius: record.definition.radius(),
                        color: maybe_colors
                            .map_or_else(ParticleColor::default, |colors| colors[index]),
                    }),
            );
            contacts.extend(
                view.particle_contacts()
                    .map(|contact| ParticleContactObservation {
                        system: *system,
                        particles: contact.particles(),
                        flags: contact.flags(),
                        weight: contact.weight(),
                        normal: contact.normal(),
                    }),
            );
            body_contacts.extend(view.body_contacts().map(|contact| {
                ParticleBodyContactObservation {
                    system: *system,
                    particle: contact.particle(),
                    body: contact.body(),
                    fixture: contact.fixture(),
                    weight: contact.weight(),
                    normal: contact.normal(),
                    mass: contact.mass(),
                }
            }));
            let statistics = ParticleSystemStatistics::from_storage(
                &record.storage,
                record.definition,
                record.groups.len(),
            );
            particle_world_statistics.include(&statistics);
            particle_statistics.push(statistics);
        }

        Ok(CollectedParticleObservations {
            particles,
            contacts,
            body_contacts,
            statistics: particle_statistics,
            world_statistics: particle_world_statistics,
        })
    }

    fn collect_body_fixture_observations(
        &self,
    ) -> Result<(Vec<BodyObservation>, Vec<FixtureObservation>), WorldObservationError> {
        let mut bodies = Vec::with_capacity(self.body_order.len());
        let mut fixtures = Vec::new();
        for body_id in &self.body_order {
            let body = self.bodies.get(*body_id).map_err(|_error| {
                WorldObservationError::InvalidState {
                    resource: WorldObservationResource::Bodies,
                }
            })?;
            bodies.push(BodyObservation {
                id: *body_id,
                snapshot: body.state.snapshot(),
            });
            for fixture_id in &body.fixtures {
                let fixture = self.fixtures.get(*fixture_id).map_err(|_error| {
                    WorldObservationError::InvalidState {
                        resource: WorldObservationResource::Fixtures,
                    }
                })?;
                fixtures.push(FixtureObservation {
                    id: *fixture_id,
                    body: *body_id,
                    snapshot: fixture.definition.snapshot(),
                });
            }
        }
        Ok((bodies, fixtures))
    }

    fn collect_joint_observations(&self) -> Result<Vec<JointObservation>, WorldObservationError> {
        let mut ordered = self.joints.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|(_id, joint)| std::cmp::Reverse(joint.diagnostic_id));
        ordered
            .into_iter()
            .map(|(id, _record)| {
                self.joint_snapshot(id)
                    .map(|snapshot| JointObservation { id, snapshot })
                    .map_err(|_error| WorldObservationError::InvalidState {
                        resource: WorldObservationResource::Joints,
                    })
            })
            .collect()
    }

    fn collect_broad_phase_observations(
        &self,
        expected_count: usize,
    ) -> Result<Vec<BroadPhaseObservation>, WorldObservationError> {
        let mut broad_phase_observations = Vec::with_capacity(expected_count);
        for body_id in &self.body_order {
            let body = self.bodies.get(*body_id).map_err(|_error| {
                WorldObservationError::InvalidState {
                    resource: WorldObservationResource::BroadPhaseObservations,
                }
            })?;
            if !body.state.snapshot().is_active() {
                continue;
            }
            let transform = body.state.transform();
            for fixture_id in &body.fixtures {
                let fixture = self.fixtures.get(*fixture_id).map_err(|_error| {
                    WorldObservationError::InvalidState {
                        resource: WorldObservationResource::BroadPhaseObservations,
                    }
                })?;
                let shape = fixture.definition.shape();
                for requested_child in 0..shape.child_count() {
                    let child_index = shape.child_index(requested_child).map_err(|_error| {
                        WorldObservationError::InvalidState {
                            resource: WorldObservationResource::BroadPhaseObservations,
                        }
                    })?;
                    let aabb = shape
                        .compute_aabb(transform, child_index)
                        .map_err(|_error| WorldObservationError::InvalidState {
                            resource: WorldObservationResource::BroadPhaseObservations,
                        })?;
                    broad_phase_observations.push(BroadPhaseObservation {
                        body: *body_id,
                        fixture: *fixture_id,
                        child_index,
                        aabb,
                    });
                }
            }
        }
        if broad_phase_observations.len() != expected_count {
            return Err(WorldObservationError::InvalidState {
                resource: WorldObservationResource::BroadPhaseObservations,
            });
        }
        Ok(broad_phase_observations)
    }
}

pub(super) fn check_requested_limit(
    resource: WorldObservationResource,
    requested: usize,
    maximum: usize,
) -> Result<(), WorldObservationLimitError> {
    if requested > maximum {
        return Err(WorldObservationLimitError {
            resource,
            requested,
            maximum,
        });
    }
    Ok(())
}

fn check_collection_bound(
    resource: WorldObservationResource,
    count: usize,
    limit: usize,
) -> Result<(), WorldObservationError> {
    if count > limit {
        return Err(WorldObservationError::CapacityExceeded { resource, limit });
    }
    Ok(())
}

fn checked_add_resource(
    resource: WorldObservationResource,
    current: usize,
    additional: usize,
) -> Result<usize, WorldObservationError> {
    current
        .checked_add(additional)
        .ok_or(WorldObservationError::InvalidState { resource })
}
