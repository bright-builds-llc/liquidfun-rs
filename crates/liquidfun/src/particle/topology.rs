use std::error::Error;
use std::fmt;

use crate::particle::ParticleFlags;
use crate::particle::storage::ParticleIndex;
use crate::particle::topology::constraints::{
    ConnectionFilter, ConstraintError, GeneratedConstraints, TopologyInput,
    generate_pairs_and_triads,
};

pub(in crate::particle) mod connectivity;
#[allow(
    dead_code,
    reason = "consumed by the Phase 10 group join and reactive topology integration"
)]
pub(in crate::particle) mod constraints;
mod voronoi;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VoronoiLimits {
    generators: usize,
    cells: usize,
    queue_tasks: usize,
    work: usize,
    nodes: usize,
}

impl VoronoiLimits {
    pub(crate) const fn new(
        maximum_generators: usize,
        maximum_cells: usize,
        maximum_queue_tasks: usize,
        maximum_work: usize,
        maximum_nodes: usize,
    ) -> Self {
        Self {
            generators: maximum_generators,
            cells: maximum_cells,
            queue_tasks: maximum_queue_tasks,
            work: maximum_work,
            nodes: maximum_nodes,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::particle) enum VoronoiError {
    NonFiniteRadius,
    NonPositiveRadius,
    NonFiniteMargin,
    NegativeMargin,
    NonFiniteGenerator { ordinal: usize },
    GeneratorLimitExceeded { required: usize, limit: usize },
    AxisCountOutOfRange,
    ArithmeticOverflow,
    GridLimitExceeded { required: usize, limit: usize },
    QueueLimitExceeded { required: usize, limit: usize },
    WorkLimitExceeded { required: usize, limit: usize },
    NodeLimitExceeded { required: usize, limit: usize },
    NonFiniteDerivedGeometry,
    IncompleteDiagram,
    AllocationFailed,
}

impl fmt::Display for VoronoiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "bounded Voronoi generation failed: {self:?}")
    }
}

impl Error for VoronoiError {}

struct ReactiveFilter<'a> {
    flags: &'a [ParticleFlags],
}

impl ConnectionFilter for ReactiveFilter<'_> {
    fn is_necessary(&self, index: ParticleIndex) -> bool {
        self.flags[index.0].contains(ParticleFlags::REACTIVE)
    }

    fn should_create_pair(&self, _indices: [ParticleIndex; 2]) -> bool {
        true
    }

    fn should_create_triad(&self, _indices: [ParticleIndex; 3]) -> bool {
        true
    }
}

pub(in crate::particle) fn generate_reactive_pairs_and_triads(
    input: &TopologyInput<'_>,
) -> Result<GeneratedConstraints, ConstraintError> {
    if !input
        .flags
        .iter()
        .any(|flags| flags.contains(ParticleFlags::REACTIVE))
    {
        return Ok(GeneratedConstraints {
            pairs: Vec::new(),
            triads: Vec::new(),
        });
    }
    generate_pairs_and_triads(input, &ReactiveFilter { flags: input.flags })
}

#[cfg(test)]
mod reactive {
    use crate::identity::{HandleIdentity, Identity, ParticleSystemId, WorldKey};
    use crate::math::Vec2;
    use crate::particle::storage::lanes::ParticleContact;
    use crate::particle::topology::constraints::TopologyGroup;

    use super::*;

    fn owner() -> ParticleSystemId {
        let world = WorldKey::fresh().expect("test world key remains available");
        ParticleSystemId::from_identity(Identity::new(world, 0, 0))
    }

    fn limits() -> VoronoiLimits {
        VoronoiLimits::new(64, 4_096, 16_384, 2_000_000, 8_192)
    }

    fn input<'a>(
        owner: ParticleSystemId,
        positions: &'a [Vec2],
        flags: &'a [ParticleFlags],
        groups: &'a [Option<TopologyGroup>],
        contacts: &'a [ParticleContact],
    ) -> TopologyInput<'a> {
        TopologyInput {
            owner,
            positions,
            flags,
            groups,
            contacts,
            range: 0..positions.len(),
            particle_diameter: 1.0,
            voronoi_limits: limits(),
        }
    }

    #[test]
    fn reactive_control_without_marked_particles_generates_nothing() {
        // Arrange
        let owner = owner();
        let positions = [
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
        ];
        let flags = [ParticleFlags::ELASTIC; 3];
        let groups = [None; 3];

        // Act
        let generated =
            generate_reactive_pairs_and_triads(&input(owner, &positions, &flags, &groups, &[]))
                .expect("an inactive reactive pass is a no-op");

        // Assert
        assert!(generated.pairs.is_empty());
        assert!(generated.triads.is_empty());
    }

    #[test]
    fn reactive_filter_generates_only_records_touching_marked_particles() {
        // Arrange
        let owner = owner();
        let positions = [
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(2.0, 0.0),
        ];
        let flags = [
            ParticleFlags::SPRING | ParticleFlags::REACTIVE,
            ParticleFlags::SPRING,
            ParticleFlags::SPRING,
        ];
        let groups = [None; 3];
        let contacts = [
            ParticleContact {
                indices: [ParticleIndex(1), ParticleIndex(2)],
                flags: ParticleFlags::SPRING,
                weight: 0.5,
                normal: Vec2::new(1.0, 0.0),
            },
            ParticleContact {
                indices: [ParticleIndex(0), ParticleIndex(1)],
                flags: ParticleFlags::SPRING,
                weight: 0.5,
                normal: Vec2::new(1.0, 0.0),
            },
        ];

        // Act
        let generated = generate_reactive_pairs_and_triads(&input(
            owner, &positions, &flags, &groups, &contacts,
        ))
        .expect("finite reactive pair should generate");

        // Assert
        assert_eq!(generated.pairs.len(), 1);
        assert_eq!(
            generated.pairs[0].indices,
            [ParticleIndex(0), ParticleIndex(1)]
        );
    }

    #[test]
    fn reactive_voronoi_emits_only_triads_touching_a_marked_generator() {
        // Arrange
        let owner = owner();
        let positions = [
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
        ];
        let flags = [
            ParticleFlags::ELASTIC | ParticleFlags::REACTIVE,
            ParticleFlags::ELASTIC,
            ParticleFlags::ELASTIC,
            ParticleFlags::ELASTIC,
        ];
        let groups = [None; 4];

        // Act
        let generated =
            generate_reactive_pairs_and_triads(&input(owner, &positions, &flags, &groups, &[]))
                .expect("bounded reactive Voronoi should generate");

        // Assert
        assert_eq!(generated.triads.len(), 1);
        assert!(generated.triads[0].indices.contains(&ParticleIndex(0)));
    }

    #[test]
    fn reactive_generation_reports_geometry_failure_before_any_commit() {
        // Arrange
        let owner = owner();
        let positions = [Vec2::ZERO; 2];
        let flags = [
            ParticleFlags::SPRING | ParticleFlags::REACTIVE,
            ParticleFlags::SPRING,
        ];
        let groups = [None; 2];
        let contacts = [ParticleContact {
            indices: [ParticleIndex(0), ParticleIndex(1)],
            flags: ParticleFlags::SPRING,
            weight: 1.0,
            normal: Vec2::new(1.0, 0.0),
        }];

        // Act
        let result = generate_reactive_pairs_and_triads(&input(
            owner, &positions, &flags, &groups, &contacts,
        ));

        // Assert
        assert_eq!(result, Err(ConstraintError::ZeroLengthPairDistance));
    }
}
