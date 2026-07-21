use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{FloatBits, ScenarioId, TransformBits, Vec2Bits};

/// Named Phase 10 extension version carried by the version-1 rigid-world schema.
pub const PHASE10_RIGID_WORLD_EXTENSION_VERSION: u32 = 1;
/// Maximum Phase 10 group operations in one rigid-world timeline.
pub const PHASE10_MAXIMUM_OPERATIONS: usize = 128;
/// Maximum live or declared particle groups in one Phase 10 timeline.
pub const PHASE10_MAXIMUM_GROUPS: usize = 64;
/// Maximum group-created particles in one Phase 10 timeline or observation.
pub const PHASE10_MAXIMUM_PARTICLES: usize = 512;
/// Maximum shapes in one filled particle-group source.
pub const PHASE10_MAXIMUM_SHAPES: usize = 32;
/// Maximum vertices in one particle-group polygon or chain.
pub const PHASE10_MAXIMUM_SHAPE_VERTICES: usize = 64;
/// Maximum pair topology records in one Phase 10 observation.
pub const PHASE10_MAXIMUM_PAIRS: usize = 1_024;
/// Maximum triad topology records in one Phase 10 observation.
pub const PHASE10_MAXIMUM_TRIADS: usize = 1_024;
/// Maximum particle/body contact records in one Phase 10 observation.
pub const PHASE10_MAXIMUM_CONTACTS: usize = 1_024;
/// Maximum semantic lifecycle events in one Phase 10 observation.
pub const PHASE10_MAXIMUM_EVENTS: usize = 1_024;
/// Maximum semantic witness records in one Phase 10 observation.
pub const PHASE10_MAXIMUM_WITNESSES: usize = 256;
/// Maximum solver steps expressed by one Phase 10 timeline.
pub const PHASE10_MAXIMUM_STEPS: u32 = 1_024;
/// Maximum bytes in one Phase 10 provenance text field.
pub const PHASE10_MAXIMUM_TEXT_BYTES: usize = 4_096;

/// Closed mask for every public particle flag supported by the pinned engine.
pub const PHASE10_PUBLIC_PARTICLE_FLAG_MASK: u32 = 0x0003_fffe;
/// Closed mask for public particle-group flags; private `0x0018` bits are excluded.
pub const PHASE10_PUBLIC_GROUP_FLAG_MASK: u32 = 0x0000_0007;

/// Exact source provenance required for a deterministic Phase 10 scenario.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10Provenance {
    pub extension_version: u32,
    pub generator_id: ScenarioId,
    pub generator_version: ScenarioId,
    pub upstream_revision: ScenarioId,
    pub toolchain_id: ScenarioId,
    pub seed: u64,
}

/// Closed exact-bit geometry accepted by particle-group sampling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase10Shape {
    Circle {
        center: Vec2Bits,
        radius_bits: FloatBits,
    },
    Polygon {
        vertices: Box<[Vec2Bits]>,
    },
    Edge {
        vertex_a: Vec2Bits,
        vertex_b: Vec2Bits,
    },
    Chain {
        vertices: Box<[Vec2Bits]>,
        looped: bool,
    },
}

/// Exactly one source for deterministic group member creation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase10GroupSource {
    Filled { shapes: Box<[Phase10Shape]> },
    Stroke { shape: Phase10Shape },
    Explicit { positions: Box<[Vec2Bits]> },
}

/// Whether a definition creates a new group or appends directly to a live one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase10GroupDestination {
    New,
    AppendTo { target_group_id: ScenarioId },
}

/// Complete exact-bit group definition shared by both adapters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10GroupDefinition {
    pub provenance: Phase10Provenance,
    pub system_id: ScenarioId,
    /// The new identity, or the append target identity for `append_to`.
    pub group_id: ScenarioId,
    /// Stable identities assigned in exact source member order.
    pub member_ids: Box<[ScenarioId]>,
    pub source: Phase10GroupSource,
    pub destination: Phase10GroupDestination,
    pub particle_flags_bits: u32,
    pub group_flags_bits: u32,
    pub transform: TransformBits,
    pub linear_velocity: Vec2Bits,
    pub angular_velocity_bits: FloatBits,
    pub color: [u8; 4],
    pub strength_bits: FloatBits,
    pub maybe_stride_bits: Option<FloatBits>,
    pub lifetime_bits: FloatBits,
}

/// Complete tagged Phase 10 action surface nested in the rigid-world timeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase10Operation {
    CreateGroup {
        definition: Phase10GroupDefinition,
    },
    JoinGroups {
        target_group_id: ScenarioId,
        source_group_id: ScenarioId,
    },
    SplitGroup {
        group_id: ScenarioId,
        created_group_ids: Box<[ScenarioId]>,
    },
    SetGroupFlags {
        group_id: ScenarioId,
        group_flags_bits: u32,
    },
    DestroyGroup {
        group_id: ScenarioId,
    },
    Step {
        timestep_bits: FloatBits,
        velocity_iterations: u32,
        position_iterations: u32,
        particle_iterations: u32,
    },
    InspectState,
}

impl Phase10GroupDefinition {
    /// Validates finite values, closed flags, source shape, and per-definition bounds.
    pub(crate) fn validate(&self) -> Result<(), Phase10ValidationKind> {
        if self.provenance.extension_version != PHASE10_RIGID_WORLD_EXTENSION_VERSION {
            return Err(Phase10ValidationKind::InvalidProvenance);
        }
        if self.member_ids.is_empty() || self.member_ids.len() > PHASE10_MAXIMUM_PARTICLES {
            return Err(Phase10ValidationKind::BoundaryLimitExceeded);
        }
        if unique_ids(&self.member_ids).is_none() {
            return Err(Phase10ValidationKind::DuplicateSemanticId);
        }
        if self.particle_flags_bits & !PHASE10_PUBLIC_PARTICLE_FLAG_MASK != 0
            || self.group_flags_bits & !PHASE10_PUBLIC_GROUP_FLAG_MASK != 0
        {
            return Err(Phase10ValidationKind::InvalidFlags);
        }
        if let Phase10GroupDestination::AppendTo { target_group_id } = &self.destination
            && target_group_id != &self.group_id
        {
            return Err(Phase10ValidationKind::InvalidOwnership);
        }
        validate_source(&self.source)?;
        if let Phase10GroupSource::Explicit { positions } = &self.source
            && positions.len() != self.member_ids.len()
        {
            return Err(Phase10ValidationKind::InvalidOrdering);
        }
        validate_transform(self.transform)?;
        validate_vec2(self.linear_velocity)?;
        validate_finite(self.angular_velocity_bits)?;
        validate_nonnegative(self.strength_bits)?;
        validate_finite(self.lifetime_bits)?;
        if let Some(stride) = self.maybe_stride_bits {
            validate_positive(stride)?;
        }
        Ok(())
    }
}

/// Stable validation categories used by strict Phase 10 request and result decoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase10ValidationKind {
    BoundaryLimitExceeded,
    DuplicateSemanticId,
    UnknownSemanticId,
    InvalidOwnership,
    InvalidOrdering,
    InvalidTopology,
    InvalidFlags,
    InvalidFloat,
    InvalidWitness,
    InvalidProvenance,
}

/// Context-free stable failure returned by Phase 10 semantic constructors and validators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("invalid Phase 10 semantic contract: {kind:?}")]
pub struct Phase10ValidationError {
    kind: Phase10ValidationKind,
}

impl Phase10ValidationError {
    pub(crate) const fn from_kind(kind: Phase10ValidationKind) -> Self {
        Self { kind }
    }

    /// Returns the stable validation category without exposing untrusted record bytes.
    #[must_use]
    pub const fn kind(self) -> Phase10ValidationKind {
        self.kind
    }
}

/// Validates one standalone Phase 10 operation before adapter dispatch.
///
/// Cross-operation identity and lifecycle checks are additionally enforced by
/// [`decode_rigid_world_request_jsonl`](super::decode_rigid_world_request_jsonl).
///
/// # Errors
///
/// Returns [`Phase10ValidationError`] for non-finite values, closed-flag
/// violations, invalid source topology, or per-operation resource excess.
pub fn validate_phase10_operation(
    operation: &Phase10Operation,
) -> Result<(), Phase10ValidationError> {
    validate_operation_shape(operation).map_err(Phase10ValidationError::from_kind)
}

pub(crate) fn validate_operation_shape(
    operation: &Phase10Operation,
) -> Result<(), Phase10ValidationKind> {
    match operation {
        Phase10Operation::CreateGroup { definition } => definition.validate(),
        Phase10Operation::SplitGroup {
            created_group_ids, ..
        } => {
            if created_group_ids.is_empty() || created_group_ids.len() > PHASE10_MAXIMUM_GROUPS {
                return Err(Phase10ValidationKind::BoundaryLimitExceeded);
            }
            if unique_ids(created_group_ids).is_none() {
                return Err(Phase10ValidationKind::DuplicateSemanticId);
            }
            Ok(())
        }
        Phase10Operation::SetGroupFlags {
            group_flags_bits, ..
        } => {
            if group_flags_bits & !PHASE10_PUBLIC_GROUP_FLAG_MASK != 0 {
                return Err(Phase10ValidationKind::InvalidFlags);
            }
            Ok(())
        }
        Phase10Operation::Step {
            timestep_bits,
            velocity_iterations,
            position_iterations,
            particle_iterations,
        } => {
            validate_positive(*timestep_bits)?;
            if *velocity_iterations == 0
                || *position_iterations == 0
                || *particle_iterations == 0
                || *velocity_iterations > PHASE10_MAXIMUM_STEPS
                || *position_iterations > PHASE10_MAXIMUM_STEPS
                || *particle_iterations > PHASE10_MAXIMUM_STEPS
            {
                return Err(Phase10ValidationKind::BoundaryLimitExceeded);
            }
            Ok(())
        }
        Phase10Operation::JoinGroups { .. }
        | Phase10Operation::DestroyGroup { .. }
        | Phase10Operation::InspectState => Ok(()),
    }
}

#[derive(Default)]
pub(crate) struct Phase10ActionState {
    operation_count: usize,
    step_count: u32,
    created_groups: HashSet<ScenarioId>,
    live_group_owners: HashMap<ScenarioId, ScenarioId>,
    created_particles: HashSet<ScenarioId>,
    maybe_provenance: Option<Phase10Provenance>,
}

impl Phase10ActionState {
    pub(crate) fn apply(
        &mut self,
        operation: &Phase10Operation,
        declared_systems: &HashSet<ScenarioId>,
        live_systems: &HashSet<ScenarioId>,
        reserved_ids: &HashSet<ScenarioId>,
    ) -> Result<(), Phase10ValidationKind> {
        self.operation_count = self
            .operation_count
            .checked_add(1)
            .ok_or(Phase10ValidationKind::BoundaryLimitExceeded)?;
        if self.operation_count > PHASE10_MAXIMUM_OPERATIONS {
            return Err(Phase10ValidationKind::BoundaryLimitExceeded);
        }
        validate_operation_shape(operation)?;
        match operation {
            Phase10Operation::CreateGroup { definition } => {
                self.create_group(definition, declared_systems, live_systems, reserved_ids)
            }
            Phase10Operation::JoinGroups {
                target_group_id,
                source_group_id,
            } => {
                if target_group_id == source_group_id {
                    return Err(Phase10ValidationKind::InvalidTopology);
                }
                let Some(target_owner) = self.live_group_owners.get(target_group_id) else {
                    return Err(Phase10ValidationKind::UnknownSemanticId);
                };
                let Some(source_owner) = self.live_group_owners.get(source_group_id) else {
                    return Err(Phase10ValidationKind::UnknownSemanticId);
                };
                if target_owner != source_owner {
                    return Err(Phase10ValidationKind::InvalidOwnership);
                }
                self.live_group_owners.remove(source_group_id);
                Ok(())
            }
            Phase10Operation::SplitGroup {
                group_id,
                created_group_ids,
            } => {
                let Some(owner) = self.live_group_owners.get(group_id).cloned() else {
                    return Err(Phase10ValidationKind::UnknownSemanticId);
                };
                self.ensure_group_identity_capacity(created_group_ids.len())?;
                for created_id in created_group_ids {
                    if reserved_ids.contains(created_id)
                        || self.created_particles.contains(created_id)
                        || !self.created_groups.insert(created_id.clone())
                    {
                        return Err(Phase10ValidationKind::DuplicateSemanticId);
                    }
                    self.live_group_owners
                        .insert(created_id.clone(), owner.clone());
                }
                Ok(())
            }
            Phase10Operation::SetGroupFlags { group_id, .. } => self.require_live_group(group_id),
            Phase10Operation::DestroyGroup { group_id } => {
                if self.live_group_owners.remove(group_id).is_none() {
                    return Err(Phase10ValidationKind::UnknownSemanticId);
                }
                Ok(())
            }
            Phase10Operation::Step { .. } => {
                self.step_count = self
                    .step_count
                    .checked_add(1)
                    .ok_or(Phase10ValidationKind::BoundaryLimitExceeded)?;
                if self.step_count > PHASE10_MAXIMUM_STEPS {
                    return Err(Phase10ValidationKind::BoundaryLimitExceeded);
                }
                Ok(())
            }
            Phase10Operation::InspectState => Ok(()),
        }
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.live_group_owners.is_empty()
    }

    fn create_group(
        &mut self,
        definition: &Phase10GroupDefinition,
        declared_systems: &HashSet<ScenarioId>,
        live_systems: &HashSet<ScenarioId>,
        reserved_ids: &HashSet<ScenarioId>,
    ) -> Result<(), Phase10ValidationKind> {
        if !declared_systems.contains(&definition.system_id)
            || !live_systems.contains(&definition.system_id)
        {
            return Err(Phase10ValidationKind::InvalidOwnership);
        }
        if let Some(provenance) = &self.maybe_provenance {
            if provenance != &definition.provenance {
                return Err(Phase10ValidationKind::InvalidProvenance);
            }
        } else {
            self.maybe_provenance = Some(definition.provenance.clone());
        }
        if self
            .created_particles
            .len()
            .checked_add(definition.member_ids.len())
            .is_none_or(|count| count > PHASE10_MAXIMUM_PARTICLES)
        {
            return Err(Phase10ValidationKind::BoundaryLimitExceeded);
        }
        for particle_id in &definition.member_ids {
            if reserved_ids.contains(particle_id)
                || self.created_groups.contains(particle_id)
                || !self.created_particles.insert(particle_id.clone())
            {
                return Err(Phase10ValidationKind::DuplicateSemanticId);
            }
        }
        match &definition.destination {
            Phase10GroupDestination::New => {
                self.ensure_group_identity_capacity(1)?;
                if reserved_ids.contains(&definition.group_id)
                    || self.created_particles.contains(&definition.group_id)
                    || !self.created_groups.insert(definition.group_id.clone())
                {
                    return Err(Phase10ValidationKind::DuplicateSemanticId);
                }
                self.live_group_owners
                    .insert(definition.group_id.clone(), definition.system_id.clone());
            }
            Phase10GroupDestination::AppendTo { target_group_id } => {
                let Some(owner) = self.live_group_owners.get(target_group_id) else {
                    return Err(Phase10ValidationKind::UnknownSemanticId);
                };
                if owner != &definition.system_id {
                    return Err(Phase10ValidationKind::InvalidOwnership);
                }
            }
        }
        Ok(())
    }

    fn require_live_group(&self, group_id: &ScenarioId) -> Result<(), Phase10ValidationKind> {
        if !self.live_group_owners.contains_key(group_id) {
            return Err(Phase10ValidationKind::UnknownSemanticId);
        }
        Ok(())
    }

    fn ensure_group_identity_capacity(
        &self,
        additional: usize,
    ) -> Result<(), Phase10ValidationKind> {
        if self
            .created_groups
            .len()
            .checked_add(additional)
            .is_none_or(|count| count > PHASE10_MAXIMUM_GROUPS)
        {
            return Err(Phase10ValidationKind::BoundaryLimitExceeded);
        }
        Ok(())
    }
}

fn validate_source(source: &Phase10GroupSource) -> Result<(), Phase10ValidationKind> {
    match source {
        Phase10GroupSource::Filled { shapes } => {
            if shapes.is_empty() || shapes.len() > PHASE10_MAXIMUM_SHAPES {
                return Err(Phase10ValidationKind::BoundaryLimitExceeded);
            }
            for shape in shapes {
                if !matches!(
                    shape,
                    Phase10Shape::Circle { .. } | Phase10Shape::Polygon { .. }
                ) {
                    return Err(Phase10ValidationKind::InvalidTopology);
                }
                validate_shape(shape)?;
            }
        }
        Phase10GroupSource::Stroke { shape } => {
            if !matches!(
                shape,
                Phase10Shape::Edge { .. } | Phase10Shape::Chain { .. }
            ) {
                return Err(Phase10ValidationKind::InvalidTopology);
            }
            validate_shape(shape)?;
        }
        Phase10GroupSource::Explicit { positions } => {
            if positions.is_empty() || positions.len() > PHASE10_MAXIMUM_PARTICLES {
                return Err(Phase10ValidationKind::BoundaryLimitExceeded);
            }
            for position in positions {
                validate_vec2(*position)?;
            }
        }
    }
    Ok(())
}

fn validate_shape(shape: &Phase10Shape) -> Result<(), Phase10ValidationKind> {
    match shape {
        Phase10Shape::Circle {
            center,
            radius_bits,
        } => {
            validate_vec2(*center)?;
            validate_positive(*radius_bits)
        }
        Phase10Shape::Polygon { vertices } => validate_vertices(vertices, 3, 8),
        Phase10Shape::Edge { vertex_a, vertex_b } => {
            validate_vec2(*vertex_a)?;
            validate_vec2(*vertex_b)?;
            if vertex_a == vertex_b {
                return Err(Phase10ValidationKind::InvalidTopology);
            }
            Ok(())
        }
        Phase10Shape::Chain { vertices, looped } => validate_vertices(
            vertices,
            if *looped { 3 } else { 2 },
            PHASE10_MAXIMUM_SHAPE_VERTICES,
        ),
    }
}

fn validate_vertices(
    vertices: &[Vec2Bits],
    minimum: usize,
    maximum: usize,
) -> Result<(), Phase10ValidationKind> {
    if vertices.len() < minimum || vertices.len() > maximum {
        return Err(Phase10ValidationKind::BoundaryLimitExceeded);
    }
    for vertex in vertices {
        validate_vec2(*vertex)?;
    }
    Ok(())
}

pub(crate) fn validate_vec2(value: Vec2Bits) -> Result<(), Phase10ValidationKind> {
    validate_finite(value.x_bits)?;
    validate_finite(value.y_bits)
}

pub(crate) fn validate_transform(value: TransformBits) -> Result<(), Phase10ValidationKind> {
    validate_vec2(value.position)?;
    validate_finite(value.angle_bits)
}

pub(crate) fn validate_finite(value: FloatBits) -> Result<(), Phase10ValidationKind> {
    if !value.to_f32().is_finite() {
        return Err(Phase10ValidationKind::InvalidFloat);
    }
    Ok(())
}

fn validate_positive(value: FloatBits) -> Result<(), Phase10ValidationKind> {
    validate_finite(value)?;
    if value.to_f32() <= 0.0 {
        return Err(Phase10ValidationKind::InvalidFloat);
    }
    Ok(())
}

fn validate_nonnegative(value: FloatBits) -> Result<(), Phase10ValidationKind> {
    validate_finite(value)?;
    if value.to_f32() < 0.0 {
        return Err(Phase10ValidationKind::InvalidFloat);
    }
    Ok(())
}

pub(crate) fn unique_ids(ids: &[ScenarioId]) -> Option<HashSet<&ScenarioId>> {
    let mut unique = HashSet::with_capacity(ids.len());
    for id in ids {
        if !unique.insert(id) {
            return None;
        }
    }
    Some(unique)
}
