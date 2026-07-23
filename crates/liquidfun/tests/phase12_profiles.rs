//! Public Phase 12 profile-structure and non-authority evidence.

use std::collections::BTreeSet;
use std::time::Duration;

use liquidfun::{
    DiagnosticProfileChild, DiagnosticProfileParent, DiagnosticProfileSchema, DiagnosticStepPhase,
    DiagnosticStepProfile, NoDecisionHook, ParticleSystemDef, StepConfiguration, StepLimits, World,
    WorldObservationLimits,
};

const PROFILE_SOURCE: &str = include_str!("../src/world/observation/profile.rs");
const STEP_SOURCE: &str = include_str!("../src/world/step.rs");

fn configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3).expect("fixture step configuration should validate")
}

#[test]
fn phase12_parent_vocabulary_is_exact_and_storage_neutral() {
    // Arrange
    let expected = [
        "contact_update",
        "rigid_solve",
        "continuous_solve",
        "particle_prepare",
        "particle_solve",
        "finalize",
    ];

    // Act
    let actual = DiagnosticProfileParent::ALL.map(DiagnosticProfileParent::as_str);

    // Assert
    assert_eq!(actual, expected);
    assert_eq!(
        DiagnosticProfileSchema::Phase12V1.as_str(),
        "phase12-profile-v1"
    );
}

#[test]
fn rust_only_children_have_one_explicit_common_parent() {
    // Arrange
    let expected = BTreeSet::from(DiagnosticProfileParent::ALL);

    // Act
    let mapped = DiagnosticProfileChild::ALL
        .into_iter()
        .map(DiagnosticProfileChild::parent)
        .collect::<BTreeSet<_>>();

    // Assert
    assert_eq!(mapped, expected);
    assert!(DiagnosticProfileChild::ALL.iter().all(|child| {
        child.as_str().starts_with("rust_")
            && !child.as_str().contains("index")
            && !child.as_str().contains("cache")
    }));
}

#[test]
fn profiled_and_unprofiled_steps_have_identical_semantic_evidence() {
    // Arrange
    let mut profiled_world = World::new().expect("world key should remain available");
    let mut ordinary_world = World::new().expect("world key should remain available");

    // Act
    let (profiled_report, profile) = profiled_world
        .step_profiled(configuration(), &mut NoDecisionHook, StepLimits::default())
        .expect("profiled step should succeed");
    let ordinary_report = ordinary_world
        .step(configuration(), &mut NoDecisionHook, StepLimits::default())
        .expect("ordinary step should succeed");
    let profiled_observation = profiled_world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("profiled observation should fit");
    let ordinary_observation = ordinary_world
        .world_observation(WorldObservationLimits::reviewed())
        .expect("ordinary observation should fit");

    // Assert
    assert_eq!(profiled_report, ordinary_report);
    assert_eq!(profiled_observation, ordinary_observation);
    assert_eq!(profile.schema(), DiagnosticProfileSchema::Phase12V1);
    assert_eq!(DiagnosticStepProfile::MAXIMUM_PHASES, 32);
    assert!(profile.phases().len() <= DiagnosticStepProfile::MAXIMUM_PHASES);
    assert!(profile.is_complete());
    assert!(profile.phases().iter().all(|timing| {
        timing.duration() <= Duration::from_mins(1)
            && matches!(
                timing.phase(),
                DiagnosticStepPhase::Common(_) | DiagnosticStepPhase::RustChild(_)
            )
    }));
    assert_eq!(
        profile
            .phases()
            .iter()
            .filter_map(|timing| timing.phase().maybe_common_parent())
            .map(DiagnosticProfileParent::as_str)
            .collect::<Vec<_>>(),
        vec![
            "contact_update",
            "particle_solve",
            "rigid_solve",
            "continuous_solve",
            "finalize",
        ]
    );
}

#[test]
fn particle_world_emits_the_particle_prepare_parent() {
    // Arrange
    let mut world = World::new().expect("world key should remain available");
    world
        .create_particle_system_with_def(&ParticleSystemDef::default())
        .expect("fixture particle system should fit");

    // Act
    let (_report, profile) = world
        .step_profiled(configuration(), &mut NoDecisionHook, StepLimits::default())
        .expect("profiled particle step should succeed");

    // Assert
    assert!(profile.phases().iter().any(|timing| {
        timing.phase().maybe_common_parent() == Some(DiagnosticProfileParent::ParticlePrepare)
    }));
}

#[test]
fn duration_types_remain_outside_semantic_and_serializable_contracts() {
    // Arrange
    let timing_start = PROFILE_SOURCE
        .find("pub struct DiagnosticStepPhaseTiming")
        .expect("timing type should remain present");
    let timing_derive_start = PROFILE_SOURCE[..timing_start]
        .rfind("#[derive")
        .expect("timing derives should remain explicit");
    let timing_end = PROFILE_SOURCE[timing_start..]
        .find("impl DiagnosticStepPhaseTiming")
        .map(|offset| timing_start + offset)
        .expect("timing implementation should remain present");
    let timing_declaration = &PROFILE_SOURCE[timing_derive_start..timing_end];
    let report_start = STEP_SOURCE
        .find("pub struct StepReport")
        .expect("step report should remain present");
    let report_end = STEP_SOURCE[report_start..]
        .find("impl StepReport")
        .map(|offset| report_start + offset)
        .expect("step report implementation should remain present");
    let report_declaration = &STEP_SOURCE[report_start..report_end];

    // Act
    let timing_has_authority_traits = ["PartialEq", "Eq", "Hash", "Serialize", "Deserialize"]
        .iter()
        .any(|trait_name| timing_declaration.contains(trait_name));
    let report_contains_timing =
        report_declaration.contains("Duration") || report_declaration.contains("Diagnostic");

    // Assert
    assert!(!timing_has_authority_traits);
    assert!(!report_contains_timing);
    assert!(!PROFILE_SOURCE.contains("serde"));
    assert!(!PROFILE_SOURCE.contains("impl From<DiagnosticStepPhaseTiming"));
    assert!(!PROFILE_SOURCE.contains("impl From<DiagnosticStepProfile"));
}
