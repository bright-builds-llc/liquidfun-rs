//! Versioned, bounded public-API state-machine evidence for particle groups.

use std::panic::{AssertUnwindSafe, catch_unwind};

#[cfg(target_os = "windows")]
use std::any::Any;

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

const AUDITED_WINDOWS_SEED: u64 = 0x3995_60c9_ead9_4a3f;
const AUDITED_WINDOWS_CONTROLS: [u8; 14] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 59];
const AUDITED_WINDOWS_OPERATIONS: [Operation; 14] = [
    Operation {
        kind: OperationKind::CreateExplicit,
        first: 71_787_583_439_247_902,
        second: 4_278_873_410,
    },
    Operation {
        kind: OperationKind::Append,
        first: 47_981_589_295_040_740,
        second: 2_859_925_585,
    },
    Operation {
        kind: OperationKind::CreateFilled,
        first: 56_436_091_559_000_168,
        second: 3_363_853_189,
    },
    Operation {
        kind: OperationKind::CreateStroke,
        first: 26_405_087_307_191_014,
        second: 1_573_865_849,
    },
    Operation {
        kind: OperationKind::CreateReactive,
        first: 33_181_658_083_951_976,
        second: 1_977_780_943,
    },
    Operation {
        kind: OperationKind::Join,
        first: 6_396_812_461_881_086,
        second: 381_279_734,
    },
    Operation {
        kind: OperationKind::Split,
        first: 10_357_184_829_122_123,
        second: 617_336_322,
    },
    Operation {
        kind: OperationKind::SetFlags,
        first: 5_749_946_986_639_520,
        second: 342_723_547,
    },
    Operation {
        kind: OperationKind::CreateLifetime,
        first: 25_775_792_647_897_003,
        second: 1_536_356_964,
    },
    Operation {
        kind: OperationKind::Step,
        first: 53_393_286_103_193_178,
        second: 3_182_487_851,
    },
    Operation {
        kind: OperationKind::DestroyMembers,
        first: 5_176_720_587_242_457,
        second: 308_556_591,
    },
    Operation {
        kind: OperationKind::Compact,
        first: 1_779_414_347_435_098,
        second: 106_061_360,
    },
    Operation {
        kind: OperationKind::InvalidJoin,
        first: 13_697_499_052_459_992,
        second: 816_434_565,
    },
    Operation {
        kind: OperationKind::Append,
        first: 31_141_724_268_561_272,
        second: 1_856_191_412,
    },
];

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

fn run_operations(operation_sequence: &[Operation]) -> Vec<TraceEntry> {
    let mut model = Model::new();
    operation_sequence
        .iter()
        .copied()
        .map(|operation| model.apply(operation))
        .collect()
}

fn run(seed: u64, controls: &[u8]) -> Vec<TraceEntry> {
    run_operations(&operations(seed, controls))
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

#[test]
fn persisted_audited_windows_seed() {
    // Arrange
    assert_eq!(
        operations(AUDITED_WINDOWS_SEED, &AUDITED_WINDOWS_CONTROLS),
        AUDITED_WINDOWS_OPERATIONS
    );
    let mut model = Model::new();
    for operation in &AUDITED_WINDOWS_OPERATIONS[..13] {
        model.apply(*operation);
    }
    #[cfg(target_os = "windows")]
    let before = snapshot::rollback_snapshot(&model);

    // Act
    let result = catch_unwind(AssertUnwindSafe(|| {
        model.apply(AUDITED_WINDOWS_OPERATIONS[13])
    }));

    // Assert
    #[cfg(target_os = "windows")]
    {
        let panic = result.expect_err(
            "Plan 14-02 must replace this temporary audited panic expectation with no-panic behavior",
        );
        assert_eq!(
            panic_message(panic.as_ref()),
            "internal error: entered unreachable code: checked creation cannot invalidate authoritative storage"
        );
        assert_eq!(
            snapshot::rollback_snapshot(&model),
            before,
            "the audited candidate panic must not mutate public semantic state"
        );
    }
    #[cfg(not(target_os = "windows"))]
    {
        let entry =
            result.expect("the audited failure is specific to the supported Windows runner");
        assert_eq!(entry.operation, AUDITED_WINDOWS_OPERATIONS[13]);
        assert!(matches!(entry.outcome, Outcome::Applied { .. }));
    }
}

#[cfg(target_os = "windows")]
fn panic_message(payload: &(dyn Any + Send)) -> &str {
    payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("<non-string panic payload>")
}
