//! Timed valve scheduling (Phase 40 D-01 to D-03).

/// Fixed steps per half-second open tick (D-02).
pub(crate) const STEPS_PER_OPEN_TICK: u32 = 30;

/// First-pour with one valve allowed and a build area reaching under the
/// reservoir outlet, for valve tests only.
#[cfg(test)]
pub(crate) const VALVE_TEST_SPEC: super::spec::LevelSpec = super::spec::LevelSpec {
    tool_budget: &[
        super::tools::ToolAllowance {
            kind: super::tools::ToolKind::Plank,
            count: 2,
        },
        super::tools::ToolAllowance {
            kind: super::tools::ToolKind::Valve,
            count: 1,
        },
    ],
    build_area: super::tools::GridRect::new(-40, 40, 40, 90),
    ..super::first_pour::FIRST_POUR
};

/// A valve centred 0.125 m under the first-pour outlet (x −0.8 m, y 4.35 m),
/// opening at tick 4 (step 120).
#[cfg(test)]
pub(crate) const REFERENCE_VALVE_POSE: [i32; 4] = [-16, 87, 0, 4];
