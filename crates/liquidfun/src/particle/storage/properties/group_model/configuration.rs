//! Preserve the same property sampling while adapting persistence to Miri isolation.

use proptest::test_runner::{Config, RngSeed};

pub(super) fn property_config(seed: u64) -> Config {
    Config {
        cases: 128,
        rng_seed: RngSeed::Fixed(seed),
        // Keep Miri isolated without changing seeded cases or shrinking behavior.
        #[cfg(miri)]
        failure_persistence: Some(Box::new(
            proptest::test_runner::MapFailurePersistence::default(),
        )),
        ..Config::default()
    }
}

#[test]
fn property_configuration_preserves_sampling_with_boundary_specific_persistence() {
    // Arrange / Act
    let config = property_config(super::MODEL_SEED);

    // Assert
    assert_eq!(config.cases, 128);
    assert_eq!(config.rng_seed, RngSeed::Fixed(super::MODEL_SEED));
    let persistence = config
        .failure_persistence
        .expect("both boundaries retain failure persistence");
    #[cfg(miri)]
    assert!(
        persistence
            .as_any()
            .is::<proptest::test_runner::MapFailurePersistence>()
    );
    #[cfg(not(miri))]
    assert!(
        persistence
            .as_any()
            .is::<proptest::test_runner::FileFailurePersistence>()
    );
}
