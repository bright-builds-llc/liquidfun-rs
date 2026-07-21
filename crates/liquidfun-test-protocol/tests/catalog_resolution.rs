//! Integration coverage for bounded deterministic scenario catalog resolution.

use liquidfun_test_protocol::{
    CATALOG_MAXIMUM_ACTIONS, CatalogDefinition, CatalogErrorKind, CatalogProgram,
    CatalogSchemaVersion, CatalogSlug, FloatBits, GeneratorId, GeneratorVersion, ResolveRequest,
    RunSettings, ScenarioEligibility, ScenarioVersion, SemanticEntityKind, Sha256Hex, Vec2Bits,
    decode_resolved_scenario, resolve_catalog,
};

const TIMESTEP_BITS: u32 = 0x3c88_8889;

fn settings() -> RunSettings {
    RunSettings::new(FloatBits::new(TIMESTEP_BITS), 8, 3, 1)
        .expect("fixture settings should validate")
}

fn named_definition(display_title: &str, step_count: u32) -> CatalogDefinition {
    CatalogDefinition::new(
        CatalogSlug::new("empty-world").expect("fixture slug should validate"),
        display_title,
        ScenarioVersion::CURRENT,
        GeneratorId::new("static-gravity").expect("fixture generator should validate"),
        GeneratorVersion::CURRENT,
        ScenarioEligibility::NamedOnly,
        Vec::new(),
        CatalogProgram::exact_gravity(
            Vec2Bits {
                x_bits: FloatBits::new(0.0_f32.to_bits()),
                y_bits: FloatBits::new((-10.0_f32).to_bits()),
            },
            step_count,
        )
        .expect("fixture program should validate"),
    )
    .expect("fixture definition should validate")
}

fn seeded_definition() -> CatalogDefinition {
    CatalogDefinition::new(
        CatalogSlug::new("seeded-gravity").expect("fixture slug should validate"),
        "Seeded Gravity",
        ScenarioVersion::CURRENT,
        GeneratorId::new("chacha8-gravity-choice").expect("fixture generator should validate"),
        GeneratorVersion::CURRENT,
        ScenarioEligibility::SeedRequired,
        vec![SemanticEntityKind::Body, SemanticEntityKind::ParticleSystem],
        CatalogProgram::seeded_gravity_choices(
            vec![
                Vec2Bits {
                    x_bits: FloatBits::new(0.0_f32.to_bits()),
                    y_bits: FloatBits::new((-10.0_f32).to_bits()),
                },
                Vec2Bits {
                    x_bits: FloatBits::new(1.0_f32.to_bits()),
                    y_bits: FloatBits::new((-5.0_f32).to_bits()),
                },
                Vec2Bits {
                    x_bits: FloatBits::new((-1.0_f32).to_bits()),
                    y_bits: FloatBits::new((-20.0_f32).to_bits()),
                },
            ],
            2,
        )
        .expect("fixture program should validate"),
    )
    .expect("fixture definition should validate")
}

#[test]
fn named_resolution_has_golden_bytes_and_title_independent_identity() {
    // Arrange
    let first = named_definition("Empty World", 1);
    let renamed = named_definition("A Mutable Presentation Title", 1);
    let request = ResolveRequest::new(
        CatalogSlug::new("empty-world").expect("fixture slug should validate"),
        None,
        settings(),
    );

    // Act
    let first_resolved = resolve_catalog(&[first], &request).expect("named run should resolve");
    let renamed_resolved =
        resolve_catalog(&[renamed], &request).expect("renamed run should resolve");

    // Assert
    assert_eq!(first_resolved, renamed_resolved);
    assert_eq!(
        first_resolved.identity().content_sha256().as_str(),
        "0363ca1c2114390f97017fcbf4d093311c9d29c4e5eac040c25b4352bf0b8e2b"
    );
    let canonical_text = std::str::from_utf8(first_resolved.canonical_bytes())
        .expect("canonical JSON should be UTF-8");
    assert!(!canonical_text.contains("Empty World"));
    assert!(!canonical_text.contains("Mutable Presentation"));
}

#[test]
fn seeded_resolution_is_repeatable_and_has_a_golden_identity() {
    // Arrange
    let definition = seeded_definition();
    let request = ResolveRequest::new(
        CatalogSlug::new("seeded-gravity").expect("fixture slug should validate"),
        Some(0x0123_4567_89ab_cdef),
        settings(),
    );

    // Act
    let first = resolve_catalog(std::slice::from_ref(&definition), &request)
        .expect("seeded run should resolve");
    let second = resolve_catalog(&[definition], &request).expect("seeded run should repeat");

    // Assert
    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(first.identity(), second.identity());
    assert_eq!(
        first.identity().content_sha256().as_str(),
        "2fbb3e2a9e990a38559f2cdb0139974fa40c7276415115d580cb0b8c7af5c23b"
    );
    assert_eq!(first.identity().maybe_seed(), Some(0x0123_4567_89ab_cdef));
    assert_eq!(first.entities()[0].semantic_id().ordinal(), 0);
    assert_eq!(first.entities()[1].semantic_id().ordinal(), 1);
    assert_eq!(first.actions()[0].action_id().as_str(), "action-0000");
    assert_eq!(first.actions()[2].action_id().as_str(), "action-0002");
}

#[test]
fn resolved_identity_records_versions_generator_seed_and_exact_settings() {
    // Arrange
    let definition = seeded_definition();
    let run_settings = settings();
    let request = ResolveRequest::new(
        CatalogSlug::new("seeded-gravity").expect("fixture slug should validate"),
        Some(7),
        run_settings,
    );

    // Act
    let resolved = resolve_catalog(&[definition], &request).expect("seeded run should resolve");

    // Assert
    assert_eq!(
        resolved.identity().catalog_schema_version(),
        CatalogSchemaVersion::CURRENT
    );
    assert_eq!(
        resolved.identity().scenario_version(),
        ScenarioVersion::CURRENT
    );
    assert_eq!(
        resolved.identity().generator_version(),
        GeneratorVersion::CURRENT
    );
    assert_eq!(
        resolved.identity().generator_id().as_str(),
        "chacha8-gravity-choice"
    );
    assert_eq!(resolved.identity().maybe_seed(), Some(7));
    assert_eq!(resolved.identity().settings(), run_settings);
    assert_eq!(
        resolved.identity().settings().timestep_bits().bits(),
        TIMESTEP_BITS
    );
}

#[test]
fn canonical_bytes_decode_to_the_identical_domain_value_and_hash() {
    // Arrange
    let definition = seeded_definition();
    let request = ResolveRequest::new(
        CatalogSlug::new("seeded-gravity").expect("fixture slug should validate"),
        Some(42),
        settings(),
    );
    let resolved = resolve_catalog(&[definition], &request).expect("seeded run should resolve");

    // Act
    let replayed = decode_resolved_scenario(
        resolved.canonical_bytes(),
        resolved.identity().content_sha256(),
    )
    .expect("canonical replay should decode");

    // Assert
    assert_eq!(replayed, resolved);
}

#[test]
fn action_and_iteration_bounds_accept_n_and_reject_n_plus_one() {
    // Arrange
    let maximum_steps =
        u32::try_from(CATALOG_MAXIMUM_ACTIONS - 1).expect("catalog action limit should fit u32");
    let at_limit_definition = named_definition("At Limit", maximum_steps);
    let request = ResolveRequest::new(
        CatalogSlug::new("empty-world").expect("fixture slug should validate"),
        None,
        RunSettings::new(FloatBits::new(TIMESTEP_BITS), 1_024, 1_024, 1_024)
            .expect("at-limit settings should validate"),
    );

    // Act
    let at_limit = resolve_catalog(&[at_limit_definition], &request)
        .expect("at-limit action schedule should resolve");
    let one_over_program = CatalogProgram::exact_gravity(
        Vec2Bits {
            x_bits: FloatBits::new(0),
            y_bits: FloatBits::new(0),
        },
        maximum_steps + 1,
    );
    let one_over_settings = RunSettings::new(FloatBits::new(TIMESTEP_BITS), 1_025, 1, 1);

    // Assert
    assert_eq!(at_limit.actions().len(), CATALOG_MAXIMUM_ACTIONS);
    assert_eq!(at_limit.checkpoints().len(), CATALOG_MAXIMUM_ACTIONS - 1);
    assert_eq!(
        one_over_program
            .expect_err("N + 1 actions should fail")
            .kind(),
        CatalogErrorKind::ResolvedLimitExceeded
    );
    assert_eq!(
        one_over_settings
            .expect_err("N + 1 iterations should fail")
            .kind(),
        CatalogErrorKind::InvalidRunSettings
    );
}

#[test]
fn resolver_rejects_invalid_seed_eligibility_and_settings() {
    // Arrange
    let named = named_definition("Empty World", 1);
    let seeded = seeded_definition();
    let named_with_seed = ResolveRequest::new(
        CatalogSlug::new("empty-world").expect("fixture slug should validate"),
        Some(1),
        settings(),
    );
    let seeded_without_seed = ResolveRequest::new(
        CatalogSlug::new("seeded-gravity").expect("fixture slug should validate"),
        None,
        settings(),
    );

    // Act
    let forbidden_seed = resolve_catalog(&[named], &named_with_seed)
        .expect_err("named definition should reject a seed");
    let missing_seed = resolve_catalog(&[seeded], &seeded_without_seed)
        .expect_err("seeded definition should require a seed");
    let zero_timestep = RunSettings::new(FloatBits::new(0.0_f32.to_bits()), 8, 3, 1);
    let nonfinite_timestep = RunSettings::new(FloatBits::new(f32::INFINITY.to_bits()), 8, 3, 1);
    let zero_iterations = RunSettings::new(FloatBits::new(TIMESTEP_BITS), 0, 3, 1);

    // Assert
    assert_eq!(forbidden_seed.kind(), CatalogErrorKind::SeedNotAllowed);
    assert_eq!(missing_seed.kind(), CatalogErrorKind::SeedRequired);
    assert_eq!(
        zero_timestep.expect_err("zero timestep should fail").kind(),
        CatalogErrorKind::InvalidRunSettings
    );
    assert_eq!(
        nonfinite_timestep
            .expect_err("nonfinite timestep should fail")
            .kind(),
        CatalogErrorKind::InvalidRunSettings
    );
    assert_eq!(
        zero_iterations
            .expect_err("zero iterations should fail")
            .kind(),
        CatalogErrorKind::InvalidRunSettings
    );
}

#[test]
fn replay_rejects_hash_tampering_and_noncanonical_bytes() {
    // Arrange
    let definition = named_definition("Empty World", 1);
    let request = ResolveRequest::new(
        CatalogSlug::new("empty-world").expect("fixture slug should validate"),
        None,
        settings(),
    );
    let resolved = resolve_catalog(&[definition], &request).expect("named run should resolve");
    let wrong_hash = Sha256Hex::new("f".repeat(64)).expect("fixture hash should validate");
    let mut noncanonical = resolved.canonical_bytes().to_vec();
    noncanonical.push(b' ');
    let noncanonical_hash = {
        use sha2::{Digest, Sha256};

        Sha256Hex::from_digest(Sha256::digest(&noncanonical).into())
    };

    // Act
    let tampered = decode_resolved_scenario(resolved.canonical_bytes(), &wrong_hash)
        .expect_err("wrong hash should fail");
    let padded = decode_resolved_scenario(&noncanonical, &noncanonical_hash)
        .expect_err("noncanonical whitespace should fail");

    // Assert
    assert_eq!(tampered.kind(), CatalogErrorKind::HashMismatch);
    assert_eq!(padded.kind(), CatalogErrorKind::NonCanonicalBytes);
}

#[test]
fn replay_rejects_actions_that_disagree_with_run_identity() {
    // Arrange
    let definition = named_definition("Empty World", 1);
    let request = ResolveRequest::new(
        CatalogSlug::new("empty-world").expect("fixture slug should validate"),
        None,
        settings(),
    );
    let resolved = resolve_catalog(&[definition], &request).expect("named run should resolve");
    let canonical_text =
        std::str::from_utf8(resolved.canonical_bytes()).expect("canonical JSON should be UTF-8");
    let altered_text =
        canonical_text.replacen("\"velocity_iterations\":8", "\"velocity_iterations\":9", 1);
    assert_ne!(
        altered_text, canonical_text,
        "fixture must alter one action"
    );
    let altered_bytes = altered_text.into_bytes();
    let altered_hash = {
        use sha2::{Digest, Sha256};

        Sha256Hex::from_digest(Sha256::digest(&altered_bytes).into())
    };

    // Act
    let error = decode_resolved_scenario(&altered_bytes, &altered_hash)
        .expect_err("action settings must agree with the hashed run identity");

    // Assert
    assert_eq!(error.kind(), CatalogErrorKind::InvalidRunSettings);
}
