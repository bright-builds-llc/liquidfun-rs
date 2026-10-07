use super::super::{ParticleStorageError, combination_limit};

#[test]
fn triad_bound_is_exact_across_the_wasm_numerator_overflow_threshold() {
    // Arrange
    let cases = [(1_626, 715_169_000), (1_627, 716_490_125)];

    // Act / Assert
    for (count, expected) in cases {
        assert_eq!(combination_limit(count, 3), Ok(expected));
    }
}

#[test]
fn triad_bound_clips_the_divided_result_at_the_target_word_limit() {
    // Arrange
    let cases = [
        (2_954, 4_291_795_704_u64),
        (2_955, 4_296_157_285),
        (4_789, 18_294_104_514),
        (3_000_000, 4_499_995_500_001_000_000),
    ];

    // Act / Assert
    for (count, mathematical_bound) in cases {
        let expected = match usize::try_from(mathematical_bound) {
            Ok(bound) => bound,
            Err(_error) => usize::MAX,
        };
        assert_eq!(combination_limit(count, 3), Ok(expected));
    }
}

#[test]
fn pair_bound_is_exact_across_the_wasm_numerator_overflow_threshold() {
    // Arrange
    let cases = [(65_536, 2_147_450_880), (65_537, 2_147_516_416)];

    // Act / Assert
    for (count, expected) in cases {
        assert_eq!(combination_limit(count, 2), Ok(expected));
    }
}

#[test]
fn unrepresentable_pair_and_triad_bounds_saturate_without_an_allocation() {
    // Arrange
    let count = usize::MAX;

    // Act
    let pair_bound = combination_limit(count, 2);
    let triad_bound = combination_limit(count, 3);

    // Assert
    assert_eq!(pair_bound, Ok(usize::MAX));
    assert_eq!(triad_bound, Ok(usize::MAX));
}

#[test]
fn pair_and_triad_bounds_follow_pascal_recurrence() {
    // Arrange
    let mut expected_pairs = 0;
    let mut expected_triads = 0;

    // Act / Assert
    for count in 0..4_790 {
        assert_eq!(combination_limit(count, 2), Ok(expected_pairs));
        assert_eq!(combination_limit(count, 3), Ok(expected_triads));
        expected_triads = expected_triads.saturating_add(expected_pairs);
        expected_pairs += count;
    }
}

#[test]
fn unsupported_connection_widths_remain_invalid() {
    // Arrange
    let count = 10;

    // Act / Assert
    for width in [0, 1, 4] {
        assert_eq!(
            combination_limit(count, width),
            Err(ParticleStorageError::InvalidLaneBundle)
        );
    }
}
