//! Test-only helpers shared by the screw's hooks, spike and flow tests.

use liquidfun::StepConfiguration;

/// The session's fixed step: 1/60 s, 8 velocity and 3 position iterations,
/// and the default two particle iterations (no `particle_iterations` arm).
pub(super) fn step_configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3)
        .expect("valid step")
        .with_particle_iterations(2)
        .expect("two particle iterations")
}
