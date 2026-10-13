//! The checked-in fixture table, by file name (Phase 40 D-15).

/// Every checked-in fixture by file name.
///
/// Level plans add their pair here; extras such as timed-gate early/late
/// losses go in too.
pub(super) const FIXTURES: &[(&str, &str)] = &[
    (
        "first-pour-solution.txt",
        include_str!("../fixtures/first-pour-solution.txt"),
    ),
    (
        "first-pour-empty.txt",
        include_str!("../fixtures/first-pour-empty.txt"),
    ),
    (
        "thick-syrup-solution.txt",
        include_str!("../fixtures/thick-syrup-solution.txt"),
    ),
    (
        "thick-syrup-empty.txt",
        include_str!("../fixtures/thick-syrup-empty.txt"),
    ),
    (
        "jelly-pad-solution.txt",
        include_str!("../fixtures/jelly-pad-solution.txt"),
    ),
    (
        "jelly-pad-empty.txt",
        include_str!("../fixtures/jelly-pad-empty.txt"),
    ),
    (
        "float-up-solution.txt",
        include_str!("../fixtures/float-up-solution.txt"),
    ),
    (
        "float-up-empty.txt",
        include_str!("../fixtures/float-up-empty.txt"),
    ),
    (
        "timed-gate-solution.txt",
        include_str!("../fixtures/timed-gate-solution.txt"),
    ),
    (
        "timed-gate-empty.txt",
        include_str!("../fixtures/timed-gate-empty.txt"),
    ),
    (
        "timed-gate-early.txt",
        include_str!("../fixtures/timed-gate-early.txt"),
    ),
    (
        "timed-gate-late.txt",
        include_str!("../fixtures/timed-gate-late.txt"),
    ),
];
