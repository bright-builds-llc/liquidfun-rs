use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::super::{
    PHASE10_MAXIMUM_CONTACTS, PHASE10_MAXIMUM_EVENTS, PHASE10_MAXIMUM_GROUPS,
    PHASE10_MAXIMUM_PAIRS, PHASE10_MAXIMUM_PARTICLES, PHASE10_MAXIMUM_TRIADS,
    PHASE10_MAXIMUM_WITNESSES, PHASE10_PUBLIC_GROUP_FLAG_MASK, PHASE10_PUBLIC_PARTICLE_FLAG_MASK,
    PHASE10_RIGID_WORLD_EXTENSION_VERSION, Phase10Provenance, Phase10ValidationError,
    Phase10ValidationKind, unique_ids, validate_finite, validate_transform, validate_vec2,
};
use crate::{FloatBits, ScenarioId, TransformBits, Vec2Bits};

/// Public semantic role of one Phase 10 behavior witness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WitnessRole {
    Control,
    Activation,
    Interaction,
}

/// Closed semantic behavior-leaf identities; private solver-pass identity is absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase10BehaviorLeaf {
    GroupCreate,
    GroupAppend,
    GroupJoin,
    GroupSplit,
    GroupFlags,
    GroupDestroy,
    Water,
    Zombie,
    Wall,
    Spring,
    Elastic,
    Viscous,
    Powder,
    Tensile,
    ColorMixing,
    Barrier,
    StaticPressure,
    Reactive,
    Repulsive,
    SolidGroup,
    RigidGroup,
    BodyInteraction,
}

/// Typed public observation carried by one behavior-leaf witness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase10WitnessObservation {
    ControlUnchanged,
    FlagActivated {
        flags_bits: u32,
    },
    ParticleVelocity {
        particle_id: ScenarioId,
        before: Vec2Bits,
        after: Vec2Bits,
    },
    Scalar {
        value_bits: FloatBits,
    },
    Count {
        value: u32,
    },
    Occurrence {
        event_ordinal: u32,
    },
    Topology {
        pair_count: u32,
        triad_count: u32,
    },
}

/// One ordered semantic witness, with no pass ID, admission trace, or pass inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10Witness {
    pub ordinal: u32,
    pub behavior_leaf: Phase10BehaviorLeaf,
    pub role: WitnessRole,
    pub observation: Phase10WitnessObservation,
}

/// Stable semantic status of a Phase 10 operation or step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase10SemanticOutcome {
    Completed,
    Rejected { reason: Phase10RejectionReason },
}

/// Closed public rejection surface shared by adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase10RejectionReason {
    CapacityExceeded,
    InvalidHandle,
    InvalidRecipe,
    Locked,
    Poisoned,
    NumericFailure,
}

/// One group snapshot in stable group order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10GroupSnapshot {
    pub ordinal: u32,
    pub group_id: ScenarioId,
    pub system_id: ScenarioId,
    pub member_ids: Box<[ScenarioId]>,
    pub group_flags_bits: u32,
    pub transform: TransformBits,
    pub center: Vec2Bits,
    pub linear_velocity: Vec2Bits,
    pub angular_velocity_bits: FloatBits,
    pub mass_bits: FloatBits,
    pub inertia_bits: FloatBits,
    pub maybe_depths_bits: Option<Box<[FloatBits]>>,
}

/// One particle snapshot in group/member order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10ParticleSnapshot {
    pub particle_id: ScenarioId,
    pub system_id: ScenarioId,
    pub group_id: ScenarioId,
    pub position: Vec2Bits,
    pub velocity: Vec2Bits,
    pub flags_bits: u32,
    pub color: [u8; 4],
    pub weight_bits: FloatBits,
}

/// One exact pair topology record in stable topology order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10PairSnapshot {
    pub ordinal: u32,
    pub particle_a_id: ScenarioId,
    pub particle_b_id: ScenarioId,
    pub flags_bits: u32,
    pub strength_bits: FloatBits,
    pub distance_bits: FloatBits,
}

/// One full exact triad topology record in stable topology order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10TriadSnapshot {
    pub ordinal: u32,
    pub particle_a_id: ScenarioId,
    pub particle_b_id: ScenarioId,
    pub particle_c_id: ScenarioId,
    pub flags_bits: u32,
    pub strength_bits: FloatBits,
    pub pa: Vec2Bits,
    pub pb: Vec2Bits,
    pub pc: Vec2Bits,
    pub ka_bits: FloatBits,
    pub kb_bits: FloatBits,
    pub kc_bits: FloatBits,
    pub s_bits: FloatBits,
}

/// One semantic particle-particle contact in stable contact order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10ParticleContact {
    pub ordinal: u32,
    pub system_id: ScenarioId,
    pub particle_a_id: ScenarioId,
    pub particle_b_id: ScenarioId,
    pub flags_bits: u32,
    pub weight_bits: FloatBits,
    pub normal: Vec2Bits,
}

/// One semantic particle-body contact in stable contact order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10BodyContact {
    pub ordinal: u32,
    pub system_id: ScenarioId,
    pub particle_id: ScenarioId,
    pub body_id: ScenarioId,
    pub fixture_id: ScenarioId,
    pub weight_bits: FloatBits,
    pub normal: Vec2Bits,
    pub mass_bits: FloatBits,
}

/// Closed Phase 10 lifecycle/contact event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase10EventKind {
    GroupCreated,
    GroupsJoined,
    GroupSplit,
    GroupDestroyed,
    ParticleDestroyed,
    ParticleContactBegin,
    ParticleContactEnd,
    BodyContactBegin,
    BodyContactEnd,
}

/// One stable ordered semantic event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10Event {
    pub ordinal: u32,
    pub kind: Phase10EventKind,
    pub system_id: ScenarioId,
    pub maybe_group_id: Option<ScenarioId>,
    pub maybe_particle_id: Option<ScenarioId>,
    pub maybe_other_particle_id: Option<ScenarioId>,
    pub maybe_body_id: Option<ScenarioId>,
}

/// Complete public semantic state emitted for one Phase 10 inspection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Phase10StateObservation {
    pub provenance: Phase10Provenance,
    pub outcome: Phase10SemanticOutcome,
    pub groups: Box<[Phase10GroupSnapshot]>,
    pub particles: Box<[Phase10ParticleSnapshot]>,
    pub pairs: Box<[Phase10PairSnapshot]>,
    pub triads: Box<[Phase10TriadSnapshot]>,
    pub particle_contacts: Box<[Phase10ParticleContact]>,
    pub body_contacts: Box<[Phase10BodyContact]>,
    pub events: Box<[Phase10Event]>,
    pub witnesses: Box<[Phase10Witness]>,
}

/// Phase 10 result extension nested in the existing rigid-world observation enum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Phase10Observation {
    State { state: Phase10StateObservation },
}

impl Phase10Observation {
    /// Validates bounds, identity, ownership, order, topology, flags, witness bindings, and floats.
    ///
    /// # Errors
    ///
    /// Returns [`Phase10ValidationError`] when this observation cannot safely
    /// cross the adapter/comparator boundary.
    pub fn validate_semantics(&self) -> Result<(), Phase10ValidationError> {
        self.validate().map_err(Phase10ValidationError::from_kind)
    }

    pub(crate) fn validate(&self) -> Result<(), Phase10ValidationKind> {
        match self {
            Self::State { state } => state.validate(),
        }
    }
}

impl Phase10StateObservation {
    fn validate(&self) -> Result<(), Phase10ValidationKind> {
        if self.provenance.extension_version != PHASE10_RIGID_WORLD_EXTENSION_VERSION {
            return Err(Phase10ValidationKind::InvalidProvenance);
        }
        self.validate_bounds()?;
        validate_ordinals(self.groups.iter().map(|record| record.ordinal))?;
        validate_ordinals(self.pairs.iter().map(|record| record.ordinal))?;
        validate_ordinals(self.triads.iter().map(|record| record.ordinal))?;
        validate_ordinals(self.particle_contacts.iter().map(|record| record.ordinal))?;
        validate_ordinals(self.body_contacts.iter().map(|record| record.ordinal))?;
        validate_ordinals(self.events.iter().map(|record| record.ordinal))?;
        validate_ordinals(self.witnesses.iter().map(|record| record.ordinal))?;
        self.validate_identity_and_topology()?;
        self.validate_numeric_and_flags()
    }

    fn validate_bounds(&self) -> Result<(), Phase10ValidationKind> {
        if self.groups.len() > PHASE10_MAXIMUM_GROUPS
            || self.particles.len() > PHASE10_MAXIMUM_PARTICLES
            || self.pairs.len() > PHASE10_MAXIMUM_PAIRS
            || self.triads.len() > PHASE10_MAXIMUM_TRIADS
            || self.particle_contacts.len() > PHASE10_MAXIMUM_CONTACTS
            || self.body_contacts.len() > PHASE10_MAXIMUM_CONTACTS
            || self.events.len() > PHASE10_MAXIMUM_EVENTS
            || self.witnesses.len() > PHASE10_MAXIMUM_WITNESSES
        {
            return Err(Phase10ValidationKind::BoundaryLimitExceeded);
        }
        Ok(())
    }

    fn validate_identity_and_topology(&self) -> Result<(), Phase10ValidationKind> {
        let group_ids = self
            .groups
            .iter()
            .map(|group| group.group_id.clone())
            .collect::<Vec<_>>();
        if unique_ids(&group_ids).is_none() {
            return Err(Phase10ValidationKind::DuplicateSemanticId);
        }
        let particle_ids = self
            .particles
            .iter()
            .map(|particle| particle.particle_id.clone())
            .collect::<Vec<_>>();
        if unique_ids(&particle_ids).is_none() {
            return Err(Phase10ValidationKind::DuplicateSemanticId);
        }
        let particles = self
            .particles
            .iter()
            .map(|particle| (particle.particle_id.clone(), particle))
            .collect::<HashMap<_, _>>();
        let groups = self
            .groups
            .iter()
            .map(|group| (group.group_id.clone(), group))
            .collect::<HashMap<_, _>>();
        let expected_particle_order = self
            .groups
            .iter()
            .flat_map(|group| group.member_ids.iter())
            .collect::<Vec<_>>();
        if expected_particle_order.len() != self.particles.len()
            || !expected_particle_order
                .iter()
                .zip(&self.particles)
                .all(|(expected, actual)| *expected == &actual.particle_id)
        {
            return Err(Phase10ValidationKind::InvalidOrdering);
        }
        for particle in &self.particles {
            let Some(group) = groups.get(&particle.group_id) else {
                return Err(Phase10ValidationKind::InvalidOwnership);
            };
            if particle.system_id != group.system_id {
                return Err(Phase10ValidationKind::InvalidOwnership);
            }
        }
        for group in &self.groups {
            if unique_ids(&group.member_ids).is_none()
                || group
                    .member_ids
                    .iter()
                    .any(|id| !particles.contains_key(id))
            {
                return Err(Phase10ValidationKind::InvalidTopology);
            }
            if let Some(depths) = &group.maybe_depths_bits
                && depths.len() != group.member_ids.len()
            {
                return Err(Phase10ValidationKind::InvalidTopology);
            }
        }
        for pair in &self.pairs {
            validate_distinct_known(&particles, [&pair.particle_a_id, &pair.particle_b_id])?;
        }
        for triad in &self.triads {
            validate_distinct_known(
                &particles,
                [
                    &triad.particle_a_id,
                    &triad.particle_b_id,
                    &triad.particle_c_id,
                ],
            )?;
        }
        for contact in &self.particle_contacts {
            validate_distinct_known(&particles, [&contact.particle_a_id, &contact.particle_b_id])?;
            if particles[&contact.particle_a_id].system_id != contact.system_id
                || particles[&contact.particle_b_id].system_id != contact.system_id
            {
                return Err(Phase10ValidationKind::InvalidOwnership);
            }
        }
        Ok(())
    }

    fn validate_numeric_and_flags(&self) -> Result<(), Phase10ValidationKind> {
        for group in &self.groups {
            if group.group_flags_bits & !PHASE10_PUBLIC_GROUP_FLAG_MASK != 0 {
                return Err(Phase10ValidationKind::InvalidFlags);
            }
            validate_transform(group.transform)?;
            validate_vec2(group.center)?;
            validate_vec2(group.linear_velocity)?;
            for value in [
                group.angular_velocity_bits,
                group.mass_bits,
                group.inertia_bits,
            ] {
                validate_finite(value)?;
            }
            if let Some(depths) = &group.maybe_depths_bits {
                for depth in depths {
                    validate_finite(*depth)?;
                }
            }
        }
        for particle in &self.particles {
            if particle.flags_bits & !PHASE10_PUBLIC_PARTICLE_FLAG_MASK != 0 {
                return Err(Phase10ValidationKind::InvalidFlags);
            }
            validate_vec2(particle.position)?;
            validate_vec2(particle.velocity)?;
            validate_finite(particle.weight_bits)?;
        }
        for pair in &self.pairs {
            validate_public_particle_flags(pair.flags_bits)?;
            for value in [pair.strength_bits, pair.distance_bits] {
                validate_finite(value)?;
            }
        }
        for triad in &self.triads {
            validate_public_particle_flags(triad.flags_bits)?;
            for vector in [triad.pa, triad.pb, triad.pc] {
                validate_vec2(vector)?;
            }
            for value in [
                triad.strength_bits,
                triad.ka_bits,
                triad.kb_bits,
                triad.kc_bits,
                triad.s_bits,
            ] {
                validate_finite(value)?;
            }
        }
        for contact in &self.particle_contacts {
            validate_public_particle_flags(contact.flags_bits)?;
            validate_finite(contact.weight_bits)?;
            validate_vec2(contact.normal)?;
        }
        for contact in &self.body_contacts {
            if !self.particles.iter().any(|particle| {
                particle.particle_id == contact.particle_id
                    && particle.system_id == contact.system_id
            }) {
                return Err(Phase10ValidationKind::InvalidOwnership);
            }
            validate_finite(contact.weight_bits)?;
            validate_finite(contact.mass_bits)?;
            validate_vec2(contact.normal)?;
        }
        let mut witness_bindings = HashSet::with_capacity(self.witnesses.len());
        for witness in &self.witnesses {
            if !witness_bindings.insert((witness.behavior_leaf, witness.role)) {
                return Err(Phase10ValidationKind::InvalidWitness);
            }
            validate_witness_observation(&witness.observation)?;
            if !witness_role_matches(witness.role, &witness.observation) {
                return Err(Phase10ValidationKind::InvalidWitness);
            }
            match &witness.observation {
                Phase10WitnessObservation::ParticleVelocity { particle_id, .. }
                    if !self
                        .particles
                        .iter()
                        .any(|particle| &particle.particle_id == particle_id) =>
                {
                    return Err(Phase10ValidationKind::InvalidWitness);
                }
                Phase10WitnessObservation::Occurrence { event_ordinal }
                    if usize::try_from(*event_ordinal)
                        .ok()
                        .is_none_or(|ordinal| ordinal >= self.events.len()) =>
                {
                    return Err(Phase10ValidationKind::InvalidWitness);
                }
                Phase10WitnessObservation::Topology {
                    pair_count,
                    triad_count,
                } if usize::try_from(*pair_count) != Ok(self.pairs.len())
                    || usize::try_from(*triad_count) != Ok(self.triads.len()) =>
                {
                    return Err(Phase10ValidationKind::InvalidWitness);
                }
                _ => {}
            }
        }
        Ok(())
    }
}

fn validate_public_particle_flags(flags: u32) -> Result<(), Phase10ValidationKind> {
    if flags & !PHASE10_PUBLIC_PARTICLE_FLAG_MASK != 0 {
        return Err(Phase10ValidationKind::InvalidFlags);
    }
    Ok(())
}

fn validate_distinct_known<'a, const N: usize>(
    particles: &HashMap<ScenarioId, &'a Phase10ParticleSnapshot>,
    ids: [&ScenarioId; N],
) -> Result<(), Phase10ValidationKind> {
    if ids.iter().any(|id| !particles.contains_key(*id)) {
        return Err(Phase10ValidationKind::InvalidTopology);
    }
    let unique = ids.into_iter().collect::<HashSet<_>>();
    if unique.len() != N {
        return Err(Phase10ValidationKind::InvalidTopology);
    }
    Ok(())
}

fn validate_ordinals(values: impl Iterator<Item = u32>) -> Result<(), Phase10ValidationKind> {
    for (expected, actual) in values.enumerate() {
        if usize::try_from(actual) != Ok(expected) {
            return Err(Phase10ValidationKind::InvalidOrdering);
        }
    }
    Ok(())
}

fn validate_witness_observation(
    observation: &Phase10WitnessObservation,
) -> Result<(), Phase10ValidationKind> {
    match observation {
        Phase10WitnessObservation::FlagActivated { flags_bits } => {
            validate_public_particle_flags(*flags_bits)
        }
        Phase10WitnessObservation::ParticleVelocity { before, after, .. } => {
            validate_vec2(*before)?;
            validate_vec2(*after)
        }
        Phase10WitnessObservation::Scalar { value_bits } => validate_finite(*value_bits),
        Phase10WitnessObservation::ControlUnchanged
        | Phase10WitnessObservation::Count { .. }
        | Phase10WitnessObservation::Occurrence { .. }
        | Phase10WitnessObservation::Topology { .. } => Ok(()),
    }
}

fn witness_role_matches(role: WitnessRole, observation: &Phase10WitnessObservation) -> bool {
    match role {
        WitnessRole::Control => {
            matches!(observation, Phase10WitnessObservation::ControlUnchanged)
        }
        WitnessRole::Activation => matches!(
            observation,
            Phase10WitnessObservation::FlagActivated { .. }
                | Phase10WitnessObservation::Count { .. }
                | Phase10WitnessObservation::Occurrence { .. }
        ),
        WitnessRole::Interaction => matches!(
            observation,
            Phase10WitnessObservation::ParticleVelocity { .. }
                | Phase10WitnessObservation::Scalar { .. }
                | Phase10WitnessObservation::Occurrence { .. }
                | Phase10WitnessObservation::Topology { .. }
        ),
    }
}
