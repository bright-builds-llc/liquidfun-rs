//! Compile-time copies of the exact files consumed by the codec fixture tests.

macro_rules! fixture_table {
    ($($relative:literal),+ $(,)?) => {
        const FIXTURES: &[(&str, &[u8])] = &[
            $(($relative, include_bytes!(concat!("../../../../", $relative)))),+
        ];
    };
}

fixture_table!(
    "protocol/fixtures/accepted/empty-world-request.jsonl",
    "protocol/fixtures/accepted/empty-world-trace.jsonl",
    "protocol/fixtures/rejected/duplicate-member.jsonl",
    "protocol/fixtures/rejected/empty-checkpoint-phase.jsonl",
    "protocol/fixtures/rejected/oversized-id.jsonl",
    "protocol/fixtures/rejected/partial-record.jsonl",
    "protocol/fixtures/rejected/rigid-world-negative-centered-inertia.jsonl",
    "protocol/fixtures/rejected/rigid-world-zero-centered-inertia.jsonl",
    "protocol/fixtures/rejected/unknown-record-kind.jsonl",
    "protocol/fixtures/rejected/unsupported-version.jsonl",
    "protocol/schemas/scenario-v1.schema.json",
    "scenarios/phase-02/empty-world.json",
);

pub(super) fn read(relative: &str) -> &'static [u8] {
    FIXTURES
        .iter()
        .find_map(|(name, bytes)| (*name == relative).then_some(*bytes))
        .expect("codec fixture path must be registered in the compile-time table")
}
