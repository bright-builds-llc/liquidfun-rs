//! Versioned, bounded public-API state-machine evidence for particle groups.

use proptest::prelude::*;

#[path = "particle_group_properties/model.rs"]
mod model;
#[path = "particle_group_properties/snapshot.rs"]
mod snapshot;

use model::Model;
use snapshot::SemanticSnapshot;

const GENERATOR_VERSION: u32 = 1;
const MAX_OPERATIONS: usize = 24;
const MAX_GROUPS: usize = 8;
const MAX_PARTICLES: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OperationKind {
    CreateExplicit,
    CreateFilled,
    CreateStroke,
    Append,
    Join,
    Split,
    SetFlags,
    CreateReactive,
    CreateLifetime,
    DestroyMembers,
    Compact,
    Step,
    InvalidJoin,
}

const REQUIRED_OPERATION_KINDS: [OperationKind; 13] = [
    OperationKind::CreateExplicit,
    OperationKind::Append,
    OperationKind::CreateFilled,
    OperationKind::CreateStroke,
    OperationKind::CreateReactive,
    OperationKind::Join,
    OperationKind::Split,
    OperationKind::SetFlags,
    OperationKind::CreateLifetime,
    OperationKind::Step,
    OperationKind::DestroyMembers,
    OperationKind::Compact,
    OperationKind::InvalidJoin,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Operation {
    kind: OperationKind,
    first: usize,
    second: usize,
}

#[derive(Debug, Clone, Copy)]
struct VersionedGenerator {
    state: u64,
}

impl VersionedGenerator {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ u64::from(GENERATOR_VERSION),
        }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }
}

fn operations(seed: u64, controls: &[u8]) -> Vec<Operation> {
    let mut generator = VersionedGenerator::new(seed);
    controls
        .iter()
        .copied()
        .take(MAX_OPERATIONS)
        .enumerate()
        .map(|(index, control)| {
            let random = generator.next() ^ u64::from(control);
            let kind = REQUIRED_OPERATION_KINDS
                .get(index)
                .copied()
                .unwrap_or_else(|| {
                    REQUIRED_OPERATION_KINDS
                        [usize::try_from(random).unwrap_or(0) % REQUIRED_OPERATION_KINDS.len()]
                });
            Operation {
                kind,
                first: usize::from(control) ^ usize::try_from(random >> 8).unwrap_or(0),
                second: usize::try_from(random >> 32).unwrap_or(0),
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Applied { created: usize, lifecycle: usize },
    Rejected,
    SkippedAtBound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TraceEntry {
    operation: Operation,
    outcome: Outcome,
    snapshot: SemanticSnapshot,
}

fn run(seed: u64, controls: &[u8]) -> Vec<TraceEntry> {
    let mut model = Model::new();
    operations(seed, controls)
        .into_iter()
        .map(|operation| model.apply(operation))
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn versioned_public_sequences_replay_exactly(
        seed in any::<u64>(),
        controls in prop::collection::vec(any::<u8>(), REQUIRED_OPERATION_KINDS.len()..=MAX_OPERATIONS),
    ) {
        // Arrange / Act
        let first = run(seed, &controls);
        let second = run(seed, &controls);

        // Assert
        prop_assert_eq!(first, second);
    }
}

#[test]
fn persisted_minimized_regression_covers_the_complete_operation_vocabulary() {
    // Arrange
    const SEED: u64 = 0x7d7b_4a19_10c2_3023;
    const CONTROLS: [u8; 13] = [0, 0, 1, 2, 3, 0, 0, 1, 4, 0, 0, 0, 0];

    // Act
    let trace = run(SEED, &CONTROLS);

    // Assert
    assert_eq!(trace.len(), REQUIRED_OPERATION_KINDS.len());
    assert_eq!(
        trace
            .iter()
            .map(|entry| entry.operation.kind)
            .collect::<Vec<_>>(),
        REQUIRED_OPERATION_KINDS
    );
}
