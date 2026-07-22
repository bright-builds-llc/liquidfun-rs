//! Layer-specific conversion from one owned observation into one collection.

use crate::collision::{Shape, world_manifold};
use crate::math::Transform;
use crate::{BodySnapshot, JointSpecificSnapshot, WorldObservation};

use super::support::{
    body_color, body_snapshot, check_limit, checked_u32, fixture_observation, metadata,
    particle_observation, primitive_text_bytes, primitive_vertex_count, segment, semantic_hash,
    shape_kind, validate_primitive,
};
use super::{
    AXIS_LENGTH, CONTACT_POINT_RADIUS, DebugCollectionError, DebugCollectionResource,
    DebugDrawOptions, DebugPrimitiveCollection, NORMAL_LENGTH, PARTICLE_CONTACT_RADIUS,
    REVIEWED_MAX_LABEL_BYTES, REVIEWED_MAX_VERTICES_PER_PRIMITIVE,
};
use crate::debug_draw::primitive::{
    DebugColor, DebugLayer, DebugOwnerKey, DebugPrimitive, DebugPrimitiveKey, DebugPrimitiveKind,
};

pub(super) struct Collector {
    options: DebugDrawOptions,
    primitives: Vec<DebugPrimitive>,
    vertices: usize,
    text_bytes: usize,
}

impl Collector {
    pub(super) fn new(options: DebugDrawOptions) -> Self {
        Self {
            options,
            primitives: Vec::new(),
            vertices: 0,
            text_bytes: 0,
        }
    }

    pub(super) fn collect(
        mut self,
        observation: &WorldObservation,
    ) -> Result<DebugPrimitiveCollection, DebugCollectionError> {
        self.collect_shapes(observation)?;
        self.collect_particles(observation)?;
        self.collect_joints(observation)?;
        self.collect_contacts(observation)?;
        self.collect_particle_contacts(observation)?;
        self.collect_broad_phase(observation)?;
        self.collect_centers_and_labels(observation)?;
        Ok(DebugPrimitiveCollection {
            primitives: self.primitives,
        })
    }

    fn collect_shapes(
        &mut self,
        observation: &WorldObservation,
    ) -> Result<(), DebugCollectionError> {
        if !self.options.includes(DebugLayer::Shapes) {
            return Ok(());
        }
        for fixture in observation.fixtures() {
            let body = body_snapshot(observation, fixture.body())?;
            let transform = body.transform();
            let metadata = metadata(
                DebugPrimitiveKey::new(
                    DebugOwnerKey::Fixture(fixture.id()),
                    DebugLayer::Shapes,
                    shape_kind(fixture.snapshot().shape()),
                    0,
                    0,
                ),
                body_color(body),
                true,
            );
            match fixture.snapshot().shape() {
                Shape::Circle(circle) => self.push(DebugPrimitive::Circle {
                    metadata,
                    center: transform.apply(circle.center()),
                    radius: circle.radius(),
                })?,
                Shape::Edge(edge) => self.push(DebugPrimitive::Segment {
                    metadata,
                    start: transform.apply(edge.start()),
                    end: transform.apply(edge.end()),
                })?,
                Shape::Polygon(polygon) => self.push(DebugPrimitive::Polyline {
                    metadata,
                    vertices: polygon
                        .vertices()
                        .iter()
                        .map(|vertex| transform.apply(*vertex))
                        .collect(),
                    closed: true,
                })?,
                Shape::Chain(chain) => self.push(DebugPrimitive::Polyline {
                    metadata,
                    vertices: chain
                        .vertices()
                        .iter()
                        .map(|vertex| transform.apply(*vertex))
                        .collect(),
                    closed: chain.is_closed(),
                })?,
            }
        }
        Ok(())
    }

    fn collect_particles(
        &mut self,
        observation: &WorldObservation,
    ) -> Result<(), DebugCollectionError> {
        if !self.options.includes(DebugLayer::Particles) {
            return Ok(());
        }
        for particle in observation.particles() {
            let [red, green, blue, alpha] = particle.color().components();
            let color = if alpha == 0 {
                DebugColor::rgba(57, 197, 207, 220)
            } else {
                DebugColor::rgba(red, green, blue, alpha)
            };
            self.push(DebugPrimitive::Circle {
                metadata: metadata(
                    DebugPrimitiveKey::new(
                        DebugOwnerKey::Particle(particle.particle()),
                        DebugLayer::Particles,
                        DebugPrimitiveKind::Circle,
                        0,
                        0,
                    ),
                    color,
                    true,
                ),
                center: particle.position(),
                radius: particle.radius(),
            })?;
        }
        Ok(())
    }

    fn collect_joints(
        &mut self,
        observation: &WorldObservation,
    ) -> Result<(), DebugCollectionError> {
        if !self.options.includes(DebugLayer::Joints) {
            return Ok(());
        }
        for joint in observation.joints() {
            let snapshot = joint.snapshot();
            let body_origins = snapshot
                .bodies()
                .map(|body| body_snapshot(observation, body).map(BodySnapshot::position));
            let [origin_a, origin_b] = [body_origins[0]?, body_origins[1]?];
            let anchor_a = snapshot.anchor_a();
            let anchor_b = snapshot.anchor_b();
            let owner = DebugOwnerKey::Joint(joint.id());
            let color = DebugColor::rgba(57, 197, 207, 255);
            match snapshot.specific() {
                JointSpecificSnapshot::Pulley(pulley) => {
                    for (ordinal, [start, end]) in [
                        [pulley.ground_anchor_a(), anchor_a],
                        [pulley.ground_anchor_b(), anchor_b],
                        [pulley.ground_anchor_a(), pulley.ground_anchor_b()],
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        self.push(segment(
                            owner,
                            DebugLayer::Joints,
                            ordinal,
                            start,
                            end,
                            color,
                        )?)?;
                    }
                }
                JointSpecificSnapshot::Mouse(_) => {}
                JointSpecificSnapshot::Distance(_) => {
                    self.push(segment(
                        owner,
                        DebugLayer::Joints,
                        0,
                        anchor_a,
                        anchor_b,
                        color,
                    )?)?;
                }
                _ => {
                    for (ordinal, [start, end]) in [
                        [origin_a, anchor_a],
                        [anchor_a, anchor_b],
                        [origin_b, anchor_b],
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        self.push(segment(
                            owner,
                            DebugLayer::Joints,
                            ordinal,
                            start,
                            end,
                            color,
                        )?)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn collect_contacts(
        &mut self,
        observation: &WorldObservation,
    ) -> Result<(), DebugCollectionError> {
        for (occurrence, contact) in observation.contacts().iter().enumerate() {
            let Some(manifold) = contact.maybe_manifold() else {
                continue;
            };
            let fixtures = contact.fixtures();
            let first = fixture_observation(observation, fixtures[0])?;
            let second = fixture_observation(observation, fixtures[1])?;
            let first_body = body_snapshot(observation, first.body())?;
            let second_body = body_snapshot(observation, second.body())?;
            let maybe_world = world_manifold(
                manifold,
                first_body.transform(),
                first.snapshot().shape().radius(),
                second_body.transform(),
                second.snapshot().shape().radius(),
            )
            .map_err(|_error| DebugCollectionError::InvalidGeometry {
                layer: DebugLayer::Contacts,
            })?;
            let Some(world) = maybe_world else {
                continue;
            };
            let occurrence = checked_u32(occurrence)?;
            let owner = DebugOwnerKey::Contact {
                fixtures,
                occurrence,
            };
            for (point_ordinal, point) in world.points().iter().enumerate() {
                if self.options.includes(DebugLayer::Contacts) {
                    self.push(DebugPrimitive::Point {
                        metadata: metadata(
                            DebugPrimitiveKey::new(
                                owner,
                                DebugLayer::Contacts,
                                DebugPrimitiveKind::Point,
                                checked_u32(point_ordinal)?,
                                occurrence,
                            ),
                            DebugColor::rgba(210, 153, 34, 255),
                            false,
                        ),
                        position: point.point(),
                        radius: CONTACT_POINT_RADIUS,
                    })?;
                }
                if self.options.includes(DebugLayer::ContactNormals) {
                    self.push(DebugPrimitive::Arrow {
                        metadata: metadata(
                            DebugPrimitiveKey::new(
                                owner,
                                DebugLayer::ContactNormals,
                                DebugPrimitiveKind::Arrow,
                                checked_u32(point_ordinal)?,
                                occurrence,
                            ),
                            DebugColor::rgba(210, 153, 34, 255),
                            false,
                        ),
                        start: point.point(),
                        end: point.point() + NORMAL_LENGTH * world.normal(),
                    })?;
                }
            }
        }
        Ok(())
    }

    fn collect_particle_contacts(
        &mut self,
        observation: &WorldObservation,
    ) -> Result<(), DebugCollectionError> {
        if !self.options.includes(DebugLayer::ParticleContacts) {
            return Ok(());
        }
        for (occurrence, contact) in observation.particle_contacts().iter().enumerate() {
            let particles = contact.particles();
            let first = particle_observation(observation, particles[0])?;
            let second = particle_observation(observation, particles[1])?;
            let midpoint = 0.5 * (first.position() + second.position());
            let occurrence = checked_u32(occurrence)?;
            let owner = DebugOwnerKey::ParticleContact {
                system: contact.system(),
                particles,
                occurrence,
            };
            self.push(DebugPrimitive::Point {
                metadata: metadata(
                    DebugPrimitiveKey::new(
                        owner,
                        DebugLayer::ParticleContacts,
                        DebugPrimitiveKind::Point,
                        0,
                        occurrence,
                    ),
                    DebugColor::rgba(57, 197, 207, 255),
                    false,
                ),
                position: midpoint,
                radius: PARTICLE_CONTACT_RADIUS,
            })?;
            self.push(DebugPrimitive::Arrow {
                metadata: metadata(
                    DebugPrimitiveKey::new(
                        owner,
                        DebugLayer::ParticleContacts,
                        DebugPrimitiveKind::Arrow,
                        1,
                        occurrence,
                    ),
                    DebugColor::rgba(57, 197, 207, 255),
                    false,
                ),
                start: midpoint,
                end: midpoint + NORMAL_LENGTH * contact.normal(),
            })?;
        }
        Ok(())
    }

    fn collect_broad_phase(
        &mut self,
        observation: &WorldObservation,
    ) -> Result<(), DebugCollectionError> {
        if !self.options.includes(DebugLayer::BroadPhase) {
            return Ok(());
        }
        let mut entries = observation.broad_phase_observations().to_vec();
        entries.sort_by_key(|entry| {
            let bounds = entry.aabb();
            (
                semantic_hash(&(entry.body(), entry.fixture(), entry.child_index())),
                bounds.lower_bound().x.to_bits(),
                bounds.lower_bound().y.to_bits(),
                bounds.upper_bound().x.to_bits(),
                bounds.upper_bound().y.to_bits(),
            )
        });
        for (ordinal, entry) in entries.into_iter().enumerate() {
            self.push(DebugPrimitive::Aabb {
                metadata: metadata(
                    DebugPrimitiveKey::new(
                        DebugOwnerKey::Fixture(entry.fixture()),
                        DebugLayer::BroadPhase,
                        DebugPrimitiveKind::Aabb,
                        u32::try_from(entry.child_index().get()).map_err(|_error| {
                            DebugCollectionError::InvalidGeometry {
                                layer: DebugLayer::BroadPhase,
                            }
                        })?,
                        checked_u32(ordinal)?,
                    ),
                    DebugColor::rgba(57, 197, 207, 179),
                    false,
                ),
                bounds: entry.aabb(),
            })?;
        }
        Ok(())
    }

    fn collect_centers_and_labels(
        &mut self,
        observation: &WorldObservation,
    ) -> Result<(), DebugCollectionError> {
        for body in observation.bodies() {
            let snapshot = body.snapshot();
            let center = snapshot.transform().apply(snapshot.local_center());
            let transform = Transform::from_position_angle(center, snapshot.angle());
            if self.options.includes(DebugLayer::CentersOfMass) {
                self.push(DebugPrimitive::TransformAxes {
                    metadata: metadata(
                        DebugPrimitiveKey::new(
                            DebugOwnerKey::Body(body.id()),
                            DebugLayer::CentersOfMass,
                            DebugPrimitiveKind::TransformAxes,
                            0,
                            0,
                        ),
                        DebugColor::rgba(88, 166, 255, 255),
                        false,
                    ),
                    transform,
                    scale: AXIS_LENGTH,
                })?;
            }
            if self.options.includes(DebugLayer::Labels) {
                self.push(DebugPrimitive::Label {
                    metadata: metadata(
                        DebugPrimitiveKey::new(
                            DebugOwnerKey::Body(body.id()),
                            DebugLayer::Labels,
                            DebugPrimitiveKind::Label,
                            0,
                            0,
                        ),
                        DebugColor::rgba(240, 246, 252, 255),
                        false,
                    ),
                    position: center,
                    text: "body".to_owned(),
                })?;
            }
        }
        Ok(())
    }

    fn push(&mut self, primitive: DebugPrimitive) -> Result<(), DebugCollectionError> {
        validate_primitive(&primitive)?;
        let vertices = primitive_vertex_count(&primitive);
        let text_bytes = primitive_text_bytes(&primitive);
        if vertices > REVIEWED_MAX_VERTICES_PER_PRIMITIVE {
            return Err(DebugCollectionError::CapacityExceeded {
                resource: DebugCollectionResource::PrimitiveVertices,
                limit: REVIEWED_MAX_VERTICES_PER_PRIMITIVE,
            });
        }
        if text_bytes > REVIEWED_MAX_LABEL_BYTES {
            return Err(DebugCollectionError::CapacityExceeded {
                resource: DebugCollectionResource::LabelBytes,
                limit: REVIEWED_MAX_LABEL_BYTES,
            });
        }
        let next_primitives =
            self.primitives
                .len()
                .checked_add(1)
                .ok_or(DebugCollectionError::CapacityExceeded {
                    resource: DebugCollectionResource::Primitives,
                    limit: self.options.limits.primitives,
                })?;
        let next_vertices =
            self.vertices
                .checked_add(vertices)
                .ok_or(DebugCollectionError::CapacityExceeded {
                    resource: DebugCollectionResource::Vertices,
                    limit: self.options.limits.vertices,
                })?;
        let next_text = self.text_bytes.checked_add(text_bytes).ok_or(
            DebugCollectionError::CapacityExceeded {
                resource: DebugCollectionResource::TextBytes,
                limit: self.options.limits.text_bytes,
            },
        )?;
        check_limit(
            DebugCollectionResource::Primitives,
            next_primitives,
            self.options.limits.primitives,
        )?;
        check_limit(
            DebugCollectionResource::Vertices,
            next_vertices,
            self.options.limits.vertices,
        )?;
        check_limit(
            DebugCollectionResource::TextBytes,
            next_text,
            self.options.limits.text_bytes,
        )?;
        self.primitives.push(primitive);
        self.vertices = next_vertices;
        self.text_bytes = next_text;
        Ok(())
    }
}
