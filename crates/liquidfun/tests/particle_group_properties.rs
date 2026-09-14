//! Versioned, bounded public-API state-machine evidence for particle groups.

use proptest::prelude::*;

#[path = "particle_group_properties/model.rs"]
mod model;
#[path = "particle_group_properties/snapshot.rs"]
mod snapshot;

#[path = "particle_group_properties/registered.rs"]
mod registered;

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

const CURRENT_WINDOWS_SEED: u64 = 190_752_942_043_209_832;
const CURRENT_WINDOWS_CONTROLS: [u8; 15] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 41, 225];
const CURRENT_WINDOWS_OPERATIONS: [Operation; 15] = [
    Operation {
        kind: OperationKind::CreateExplicit,
        first: 28_139_653_222_007_705,
        second: 1_677_254_034,
    },
    Operation {
        kind: OperationKind::Append,
        first: 67_953_198_315_843_686,
        second: 4_050_326_246,
    },
    Operation {
        kind: OperationKind::CreateFilled,
        first: 57_624_694_102_115_465,
        second: 3_434_699_422,
    },
    Operation {
        kind: OperationKind::CreateStroke,
        first: 56_058_469_386_270_371,
        second: 3_341_345_154,
    },
    Operation {
        kind: OperationKind::CreateReactive,
        first: 35_720_330_941_525_231,
        second: 2_129_097_637,
    },
    Operation {
        kind: OperationKind::Join,
        first: 48_367_258_220_285_286,
        second: 2_882_913_244,
    },
    Operation {
        kind: OperationKind::Split,
        first: 49_537_496_458_564_977,
        second: 2_952_664_879,
    },
    Operation {
        kind: OperationKind::SetFlags,
        first: 61_828_390_193_329_344,
        second: 3_685_259_234,
    },
    Operation {
        kind: OperationKind::CreateLifetime,
        first: 56_646_850_988_661_860,
        second: 3_376_415_430,
    },
    Operation {
        kind: OperationKind::Step,
        first: 1_651_900_154_810_332,
        second: 98_460_921,
    },
    Operation {
        kind: OperationKind::DestroyMembers,
        first: 13_568_787_666_523_991,
        second: 808_762_768,
    },
    Operation {
        kind: OperationKind::Compact,
        first: 51_631_641_503_332_339,
        second: 3_077_485_650,
    },
    Operation {
        kind: OperationKind::InvalidJoin,
        first: 8_514_166_349_699_835,
        second: 507_483_860,
    },
    Operation {
        kind: OperationKind::CreateStroke,
        first: 31_611_878_473_071_265,
        second: 1_884_214_787,
    },
    Operation {
        kind: OperationKind::Append,
        first: 59_734_381_729_377_835,
        second: 3_560_446_603,
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
enum Rejection {
    CreationTopology,
    MutationTopology,
    WrongParticleSystem,
    PendingDelete,
    StepTopology,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    Applied { created: usize, lifecycle: usize },
    Rejected(Rejection),
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
    let (seed, controls) = registered::load("audited_windows");
    assert_eq!(seed, AUDITED_WINDOWS_SEED);
    assert_eq!(controls, AUDITED_WINDOWS_CONTROLS);
    let parsed_operations = operations(seed, &controls);
    assert_eq!(parsed_operations, AUDITED_WINDOWS_OPERATIONS);
    assert_eq!(
        run_operations(&parsed_operations),
        run_operations(&parsed_operations)
    );
    let mut model = Model::new();
    for (index, operation) in parsed_operations[..13].iter().enumerate() {
        let entry = model.apply(*operation);
        if index == 12 {
            assert_eq!(
                entry.outcome,
                Outcome::Rejected(Rejection::WrongParticleSystem)
            );
        } else {
            assert!(
                matches!(entry.outcome, Outcome::Applied { .. }),
                "operation {index}: {entry:?}"
            );
        }
    }
    let groups = model.live_groups();
    let target = groups[AUDITED_WINDOWS_OPERATIONS[13].first % groups.len()];
    let before_members = model
        .world
        .particle_group_view(target)
        .expect("append target remains live")
        .member_ids()
        .to_vec();
    let before_count = model
        .world
        .particle_system_statistics(model.system)
        .expect("system remains live")
        .particle_count();

    // Act
    let entry = model.apply(parsed_operations[13]);

    // Assert
    assert_eq!(entry.operation, AUDITED_WINDOWS_OPERATIONS[13]);
    assert_eq!(
        entry.outcome,
        Outcome::Applied {
            created: 0,
            lifecycle: 0
        }
    );
    assert_eq!(model.live_groups(), groups);
    let after_members = model
        .world
        .particle_group_view(target)
        .expect("append retains its target")
        .member_ids()
        .to_vec();
    assert_eq!(after_members.len(), before_members.len() + 1);
    assert!(before_members.iter().all(|id| after_members.contains(id)));
    assert_eq!(
        model
            .world
            .particle_system_statistics(model.system)
            .expect("system remains live")
            .particle_count(),
        before_count + 1
    );
}

#[test]
fn persisted_current_windows_seed() {
    // Arrange
    let (seed, controls) = registered::load("current_windows");
    assert_eq!(seed, CURRENT_WINDOWS_SEED);
    assert_eq!(controls, CURRENT_WINDOWS_CONTROLS);
    let parsed_operations = operations(seed, &controls);
    assert_eq!(parsed_operations, CURRENT_WINDOWS_OPERATIONS);
    // Act
    let trace = run_operations(&parsed_operations);
    // Assert
    assert_eq!(trace, run_operations(&parsed_operations));
    assert_eq!(trace.len(), 15);
    for (index, entry) in trace.iter().enumerate() {
        if index == 12 {
            assert_eq!(
                entry.outcome,
                Outcome::Rejected(Rejection::WrongParticleSystem)
            );
        } else {
            assert!(
                matches!(entry.outcome, Outcome::Applied { .. }),
                "operation {index}: {entry:?}"
            );
        }
    }
    assert_eq!(
        trace[14].outcome,
        Outcome::Applied {
            created: 0,
            lifecycle: 0
        }
    );
}

fn reactive_groups(second_selector: usize) -> Model {
    let mut model = Model::new();
    for first in [1, second_selector] {
        let entry = model.apply(Operation {
            kind: OperationKind::CreateReactive,
            first,
            second: 0,
        });
        assert_eq!(
            entry.outcome,
            Outcome::Applied {
                created: 1,
                lifecycle: 0
            }
        );
    }
    model
}

#[test]
fn coincident_reactive_springs_reject_step_without_effects() {
    // Arrange
    let mut model = reactive_groups(1);
    let before = snapshot::rollback_snapshot(&model);
    // Act
    let entry = model.apply(Operation {
        kind: OperationKind::Step,
        first: 0,
        second: 0,
    });
    // Assert
    assert_eq!(entry.outcome, Outcome::Rejected(Rejection::StepTopology));
    assert_eq!(snapshot::rollback_snapshot(&model), before);
    assert!(!model.world.is_locked());
    assert!(!model.world.is_poisoned());
    model
        .world
        .set_gravity(liquidfun::math::Vec2::ZERO)
        .expect("rejected world remains mutable and unpoisoned");
}

#[test]
fn shifted_reactive_springs_complete_the_same_step() {
    // Arrange
    let mut model = reactive_groups(2);
    // Act
    let entry = model.apply(Operation {
        kind: OperationKind::Step,
        first: 0,
        second: 0,
    });
    // Assert
    assert_eq!(
        entry.outcome,
        Outcome::Applied {
            created: 0,
            lifecycle: 0
        }
    );
}
