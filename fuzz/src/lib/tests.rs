    use super::{
        FailureClassification, FuzzDisposition, MAX_GROUPS, MAX_MUTATION_OPERATIONS, MAX_PARTICLES,
        MAX_PROTOCOL_BYTES, MAX_WORLD_ENTITIES, RAW_OPERATION_BYTES, fuzz_groups_ownership,
        fuzz_particles, fuzz_protocol, fuzz_shapes_collision, fuzz_world_mutation,
    };

    #[test]
    fn protocol_accepts_exact_one_mebibyte_and_rejects_n_plus_one() {
        // Arrange
        let at_limit = vec![0_u8; MAX_PROTOCOL_BYTES];
        let over_limit = vec![0_u8; MAX_PROTOCOL_BYTES + 1];

        // Act
        let accepted = fuzz_protocol(&at_limit);
        let rejected = fuzz_protocol(&over_limit);

        // Assert
        assert_eq!(accepted, FuzzDisposition::Executed);
        assert_eq!(rejected, FuzzDisposition::Rejected);
    }

    #[test]
    fn shapes_accept_256_operations_and_reject_n_plus_one() {
        // Arrange
        let at_limit = vec![0_u8; MAX_MUTATION_OPERATIONS * RAW_OPERATION_BYTES];
        let over_limit = vec![0_u8; (MAX_MUTATION_OPERATIONS + 1) * RAW_OPERATION_BYTES];

        // Act
        let accepted = fuzz_shapes_collision(&at_limit);
        let rejected = fuzz_shapes_collision(&over_limit);

        // Assert
        assert_eq!(accepted, FuzzDisposition::Executed);
        assert_eq!(rejected, FuzzDisposition::Rejected);
    }

    #[test]
    fn world_accepts_128_creations_and_rejects_n_plus_one() {
        // Arrange
        let at_limit = vec![0_u8; MAX_WORLD_ENTITIES * RAW_OPERATION_BYTES];
        let over_limit = vec![0_u8; (MAX_WORLD_ENTITIES + 1) * RAW_OPERATION_BYTES];

        // Act
        let accepted = fuzz_world_mutation(&at_limit);
        let rejected = fuzz_world_mutation(&over_limit);

        // Assert
        assert_eq!(accepted, FuzzDisposition::Executed);
        assert_eq!(rejected, FuzzDisposition::Rejected);
    }

    #[test]
    fn particles_accept_4096_creation_attempts_and_reject_n_plus_one() {
        // Arrange
        let at_limit = particle_program(MAX_PARTICLES);
        let over_limit = particle_program(MAX_PARTICLES + 1);

        // Act
        let accepted = fuzz_particles(&at_limit);
        let rejected = fuzz_particles(&over_limit);

        // Assert
        assert_eq!(accepted, FuzzDisposition::Executed);
        assert_eq!(rejected, FuzzDisposition::Rejected);
    }

    #[test]
    fn groups_accept_64_creations_and_reject_n_plus_one() {
        // Arrange
        let at_limit = vec![0_u8; MAX_GROUPS * RAW_OPERATION_BYTES];
        let over_limit = vec![0_u8; (MAX_GROUPS + 1) * RAW_OPERATION_BYTES];

        // Act
        let accepted = fuzz_groups_ownership(&at_limit);
        let rejected = fuzz_groups_ownership(&over_limit);

        // Assert
        assert_eq!(accepted, FuzzDisposition::Executed);
        assert_eq!(rejected, FuzzDisposition::Rejected);
    }

    #[test]
    fn regression_failure_classification_is_closed_and_exact() {
        // Arrange / Act
        let spellings = [
            FailureClassification::Harness,
            FailureClassification::PhysicsMismatch,
            FailureClassification::Sanitizer,
            FailureClassification::Timeout,
            FailureClassification::Schema,
        ]
        .map(FailureClassification::as_str);

        // Assert
        assert_eq!(
            spellings,
            [
                "Harness",
                "PhysicsMismatch",
                "Sanitizer",
                "Timeout",
                "Schema",
            ]
        );
    }

    fn particle_program(total: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        let full_operations = total / 32;
        let remainder = total % 32;
        for _ in 0..full_operations {
            bytes.extend(operation_with_first(31));
        }
        if remainder != 0 {
            bytes.extend(operation_with_first(
                u32::try_from(remainder - 1).unwrap_or_default(),
            ));
        }
        bytes
    }

    fn operation_with_first(first: u32) -> [u8; RAW_OPERATION_BYTES] {
        let mut operation = [0_u8; RAW_OPERATION_BYTES];
        operation[1..5].copy_from_slice(&first.to_le_bytes());
        operation
    }
