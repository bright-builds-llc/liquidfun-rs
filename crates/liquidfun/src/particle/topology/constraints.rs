use std::ops::Range;

use crate::identity::ParticleSystemId;
use crate::math::Vec2;
use crate::particle::storage::ParticleIndex;
use crate::particle::storage::group::GroupRecord;
use crate::particle::storage::lanes::{ParticleContact, ParticlePair, ParticleTriad};
use crate::particle::{ParticleFlags, ParticleGroupFlags};

use super::voronoi::{VoronoiDiagram, VoronoiGenerator};
use super::{VoronoiError, VoronoiLimits};

const PARTICLE_STRIDE: f32 = 0.75;
const MAX_TRIAD_DISTANCE_SQUARED: f32 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::particle) struct TopologyGroup {
    owner: ParticleSystemId,
    flags: ParticleGroupFlags,
    strength: f32,
}

impl TopologyGroup {
    pub(in crate::particle) const fn from_record(record: GroupRecord) -> Self {
        Self {
            owner: record.system,
            flags: record.flags,
            strength: record.strength,
        }
    }

    #[cfg(test)]
    const fn new(owner: ParticleSystemId, flags: ParticleGroupFlags, strength: f32) -> Self {
        Self {
            owner,
            flags,
            strength,
        }
    }
}

pub(in crate::particle) struct TopologyInput<'a> {
    pub(in crate::particle) owner: ParticleSystemId,
    pub(in crate::particle) positions: &'a [Vec2],
    pub(in crate::particle) flags: &'a [ParticleFlags],
    pub(in crate::particle) groups: &'a [Option<TopologyGroup>],
    pub(in crate::particle) contacts: &'a [ParticleContact],
    pub(in crate::particle) range: Range<usize>,
    pub(in crate::particle) particle_diameter: f32,
    pub(in crate::particle) voronoi_limits: VoronoiLimits,
}

pub(in crate::particle) trait ConnectionFilter {
    fn is_necessary(&self, index: ParticleIndex) -> bool;

    fn should_create_pair(&self, indices: [ParticleIndex; 2]) -> bool;

    fn should_create_triad(&self, indices: [ParticleIndex; 3]) -> bool;
}

#[derive(Debug, Clone, PartialEq)]
pub(in crate::particle) struct GeneratedConstraints {
    pub(in crate::particle) pairs: Vec<ParticlePair>,
    pub(in crate::particle) triads: Vec<ParticleTriad>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::particle) enum ConstraintError {
    MismatchedParticleLanes,
    InvalidRange,
    InvalidContactEndpoint,
    ForeignGroupOwner,
    NonFiniteGroupStrength,
    NonFinitePosition,
    InvalidParticleDiameter,
    NonFiniteDerivedGeometry,
    ZeroLengthPairDistance,
    VoronoiRequiresNecessaryGenerator,
    Voronoi(VoronoiError),
    AllocationFailed,
}

impl From<VoronoiError> for ConstraintError {
    fn from(error: VoronoiError) -> Self {
        Self::Voronoi(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::particle) enum RecordPolicy {
    Append,
    Preserve,
}

pub(in crate::particle) fn generate_pairs_and_triads(
    input: &TopologyInput<'_>,
    filter: &impl ConnectionFilter,
) -> Result<GeneratedConstraints, ConstraintError> {
    validate_input(input)?;
    let mut pairs = generate_pairs(input, filter)?;
    let mut triads = generate_triads(input, filter)?;
    apply_pair_policy(&mut pairs, RecordPolicy::Append);
    apply_triad_policy(&mut triads, RecordPolicy::Append);
    Ok(GeneratedConstraints { pairs, triads })
}

pub(in crate::particle) fn apply_pair_policy(pairs: &mut Vec<ParticlePair>, policy: RecordPolicy) {
    if policy == RecordPolicy::Preserve {
        return;
    }
    pairs.sort_by_key(|pair| pair.indices.map(|index| index.0));
    pairs.dedup_by(|left, right| left.indices == right.indices);
}

pub(in crate::particle) fn apply_triad_policy(
    triads: &mut Vec<ParticleTriad>,
    policy: RecordPolicy,
) {
    if policy == RecordPolicy::Preserve {
        return;
    }
    triads.sort_by_key(|triad| triad.indices.map(|index| index.0));
    triads.dedup_by(|left, right| left.indices == right.indices);
}

fn validate_input(input: &TopologyInput<'_>) -> Result<(), ConstraintError> {
    let count = input.positions.len();
    if input.flags.len() != count || input.groups.len() != count {
        return Err(ConstraintError::MismatchedParticleLanes);
    }
    if input.range.start > input.range.end || input.range.end > count {
        return Err(ConstraintError::InvalidRange);
    }
    if !input.particle_diameter.is_finite() || input.particle_diameter <= 0.0 {
        return Err(ConstraintError::InvalidParticleDiameter);
    }
    if input.positions.iter().any(|position| !position.is_valid()) {
        return Err(ConstraintError::NonFinitePosition);
    }
    for maybe_group in input.groups {
        let Some(group) = maybe_group else {
            continue;
        };
        if group.owner != input.owner {
            return Err(ConstraintError::ForeignGroupOwner);
        }
        if !group.strength.is_finite() {
            return Err(ConstraintError::NonFiniteGroupStrength);
        }
    }
    if input
        .contacts
        .iter()
        .any(|contact| contact.indices.iter().any(|index| index.0 >= count))
    {
        return Err(ConstraintError::InvalidContactEndpoint);
    }
    Ok(())
}

fn generate_pairs(
    input: &TopologyInput<'_>,
    filter: &impl ConnectionFilter,
) -> Result<Vec<ParticlePair>, ConstraintError> {
    let mut pairs = Vec::new();
    pairs
        .try_reserve(input.contacts.len())
        .map_err(|_| ConstraintError::AllocationFailed)?;
    for contact in input.contacts {
        let [a, b] = contact.indices;
        let combined_flags = input.flags[a.0] | input.flags[b.0];
        if !pair_is_eligible(input, filter, [a, b], combined_flags) {
            continue;
        }
        let distance = (input.positions[a.0] - input.positions[b.0]).length();
        if distance == 0.0 {
            return Err(ConstraintError::ZeroLengthPairDistance);
        }
        if !distance.is_finite() {
            return Err(ConstraintError::NonFiniteDerivedGeometry);
        }
        pairs.push(ParticlePair {
            indices: [a, b],
            flags: contact.flags,
            strength: minimum_strength(input.groups[a.0], input.groups[b.0]),
            distance,
        });
    }
    Ok(pairs)
}

fn pair_is_eligible(
    input: &TopologyInput<'_>,
    filter: &impl ConnectionFilter,
    indices: [ParticleIndex; 2],
    combined_flags: ParticleFlags,
) -> bool {
    let [a, b] = indices;
    index_in_range(a, &input.range)
        && index_in_range(b, &input.range)
        && !combined_flags.intersects(ParticleFlags::ZOMBIE)
        && combined_flags.intersects(ParticleFlags::SPRING | ParticleFlags::BARRIER)
        && (filter.is_necessary(a) || filter.is_necessary(b))
        && particle_can_be_connected(input.flags[a.0], input.groups[a.0])
        && particle_can_be_connected(input.flags[b.0], input.groups[b.0])
        && filter.should_create_pair(indices)
}

fn generate_triads(
    input: &TopologyInput<'_>,
    filter: &impl ConnectionFilter,
) -> Result<Vec<ParticleTriad>, ConstraintError> {
    if !input.flags[input.range.clone()]
        .iter()
        .any(|flags| flags.intersects(ParticleFlags::ELASTIC))
    {
        return Ok(Vec::new());
    }
    let generators = collect_generators(input, filter)?;
    if !generators
        .iter()
        .any(|generator| generator.generator.necessary())
    {
        return Err(ConstraintError::VoronoiRequiresNecessaryGenerator);
    }
    let stride = PARTICLE_STRIDE * input.particle_diameter;
    let mut voronoi_generators = Vec::new();
    voronoi_generators
        .try_reserve_exact(generators.len())
        .map_err(|_| ConstraintError::AllocationFailed)?;
    voronoi_generators.extend(generators.iter().map(|generator| generator.generator));
    let diagram = VoronoiDiagram::generate(
        &voronoi_generators,
        stride / 2.0,
        stride * 2.0,
        input.voronoi_limits,
    )?;
    build_triads(input, filter, &generators, &diagram)
}

#[derive(Debug, Clone, Copy)]
struct DenseGenerator {
    index: ParticleIndex,
    generator: VoronoiGenerator,
}

fn collect_generators(
    input: &TopologyInput<'_>,
    filter: &impl ConnectionFilter,
) -> Result<Vec<DenseGenerator>, ConstraintError> {
    let mut generators = Vec::new();
    generators
        .try_reserve(input.range.len())
        .map_err(|_| ConstraintError::AllocationFailed)?;
    for index in input.range.clone().map(ParticleIndex) {
        let flags = input.flags[index.0];
        if flags.intersects(ParticleFlags::ZOMBIE)
            || !particle_can_be_connected(flags, input.groups[index.0])
        {
            continue;
        }
        generators.push(DenseGenerator {
            index,
            generator: VoronoiGenerator::new(input.positions[index.0], filter.is_necessary(index)),
        });
    }
    Ok(generators)
}

fn build_triads(
    input: &TopologyInput<'_>,
    filter: &impl ConnectionFilter,
    generators: &[DenseGenerator],
    diagram: &VoronoiDiagram,
) -> Result<Vec<ParticleTriad>, ConstraintError> {
    let mut triads = Vec::new();
    triads
        .try_reserve(diagram.nodes().len())
        .map_err(|_| ConstraintError::AllocationFailed)?;
    let maximum_distance_squared =
        MAX_TRIAD_DISTANCE_SQUARED * input.particle_diameter * input.particle_diameter;
    if !maximum_distance_squared.is_finite() {
        return Err(ConstraintError::NonFiniteDerivedGeometry);
    }
    for node in diagram.nodes() {
        let indices = node
            .generator_ordinals()
            .map(|ordinal| generators[ordinal].index);
        let flags = indices.iter().fold(ParticleFlags::empty(), |all, index| {
            all | input.flags[index.0]
        });
        if !flags.intersects(ParticleFlags::ELASTIC) || !filter.should_create_triad(indices) {
            continue;
        }
        let positions = indices.map(|index| input.positions[index.0]);
        if triad_is_too_large(positions, maximum_distance_squared) {
            continue;
        }
        triads.push(build_triad(input, indices, positions, flags)?);
    }
    Ok(triads)
}

fn triad_is_too_large(positions: [Vec2; 3], maximum_distance_squared: f32) -> bool {
    let [a, b, c] = positions;
    let ab = a - b;
    let bc = b - c;
    let ca = c - a;
    ab.dot(ab) > maximum_distance_squared
        || bc.dot(bc) > maximum_distance_squared
        || ca.dot(ca) > maximum_distance_squared
}

fn build_triad(
    input: &TopologyInput<'_>,
    indices: [ParticleIndex; 3],
    positions: [Vec2; 3],
    flags: ParticleFlags,
) -> Result<ParticleTriad, ConstraintError> {
    let [a, b, c] = positions;
    let ab = a - b;
    let bc = b - c;
    let ca = c - a;
    let midpoint = (1.0 / 3.0) * (a + b + c);
    let triad = ParticleTriad {
        indices,
        flags,
        strength: indices.iter().fold(1.0_f32, |strength, index| {
            strength.min(group_strength(input.groups[index.0]))
        }),
        pa: a - midpoint,
        pb: b - midpoint,
        pc: c - midpoint,
        ka: -ca.dot(ab),
        kb: -ab.dot(bc),
        kc: -bc.dot(ca),
        s: a.cross(b) + b.cross(c) + c.cross(a),
    };
    if !triad.pa.is_valid()
        || !triad.pb.is_valid()
        || !triad.pc.is_valid()
        || !triad.ka.is_finite()
        || !triad.kb.is_finite()
        || !triad.kc.is_finite()
        || !triad.s.is_finite()
    {
        return Err(ConstraintError::NonFiniteDerivedGeometry);
    }
    Ok(triad)
}

fn particle_can_be_connected(flags: ParticleFlags, maybe_group: Option<TopologyGroup>) -> bool {
    flags.intersects(ParticleFlags::WALL | ParticleFlags::SPRING | ParticleFlags::ELASTIC)
        || maybe_group.is_some_and(|group| group.flags.contains(ParticleGroupFlags::RIGID))
}

fn minimum_strength(first: Option<TopologyGroup>, second: Option<TopologyGroup>) -> f32 {
    group_strength(first).min(group_strength(second))
}

fn group_strength(maybe_group: Option<TopologyGroup>) -> f32 {
    maybe_group.map_or(1.0, |group| group.strength)
}

fn index_in_range(index: ParticleIndex, range: &Range<usize>) -> bool {
    range.contains(&index.0)
}

#[cfg(test)]
mod tests;
