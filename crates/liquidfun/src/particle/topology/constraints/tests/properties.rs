use proptest::prelude::*;

use super::*;

proptest! {
    #[test]
    fn generated_pair_endpoints_are_bounded_and_output_is_deterministic(
        count in 2_usize..=16,
        raw_contacts in prop::collection::vec((0_usize..32, 0_usize..32), 0..64),
    ) {
        // Arrange
        let positions = (0..count)
            .map(|index| {
                let coordinate = u8::try_from(index)
                    .expect("generated count fits in u8");
                Vec2::new(f32::from(coordinate), 0.0)
            })
            .collect::<Vec<_>>();
        let flags = vec![ParticleFlags::SPRING; count];
        let mut fixture = Fixture::new(positions, flags);
        fixture.contacts = raw_contacts
            .into_iter()
            .filter_map(|(a, b)| {
                let a = a % count;
                let b = b % count;
                (a != b).then(|| contact(a, b, ParticleFlags::SPRING))
            })
            .collect();
        let filter = TestFilter::all(count);

        // Act
        let first = generate_pairs_and_triads(&fixture.input(), &filter)
            .expect("bounded pair case should generate");
        let second = generate_pairs_and_triads(&fixture.input(), &filter)
            .expect("bounded replay should generate");

        // Assert
        prop_assert_eq!(&first, &second);
        let endpoints_are_bounded = first.pairs.iter().all(|pair| {
            pair.indices.iter().all(|index| index.0 < count)
        });
        prop_assert!(endpoints_are_bounded);
    }

    #[test]
    fn append_pair_policy_emits_exactly_one_record_per_endpoint_tuple(
        raw in prop::collection::vec((0_usize..8, 0_usize..8, any::<u16>()), 0..96),
    ) {
        // Arrange
        let mut pairs = raw
            .iter()
            .map(|&(a, b, marker)| pair([a, b], f32::from(marker), 1.0))
            .collect::<Vec<_>>();

        // Act
        apply_pair_policy(&mut pairs, RecordPolicy::Append);

        // Assert
        let pairs_are_unique_and_sorted = pairs.windows(2).all(|window| {
            window[0].indices != window[1].indices
                && window[0].indices.map(|index| index.0)
                    < window[1].indices.map(|index| index.0)
        });
        prop_assert!(pairs_are_unique_and_sorted);
        for pair in &pairs {
            let first_marker = raw
                .iter()
                .find(|&&(a, b, _)| [a, b] == pair.indices.map(|index| index.0))
                .map(|&(_, _, marker)| marker)
                .expect("retained tuple came from input");
            prop_assert_eq!(pair.strength.to_bits(), f32::from(first_marker).to_bits());
        }
    }

    #[test]
    fn append_triad_policy_emits_exactly_one_record_per_endpoint_tuple(
        raw in prop::collection::vec(
            (0_usize..6, 0_usize..6, 0_usize..6, any::<u16>()),
            0..96,
        ),
    ) {
        // Arrange
        let mut triads = raw
            .iter()
            .map(|&(a, b, c, marker)| triad([a, b, c], f32::from(marker)))
            .collect::<Vec<_>>();

        // Act
        apply_triad_policy(&mut triads, RecordPolicy::Append);

        // Assert
        let triads_are_unique_and_sorted = triads.windows(2).all(|window| {
            window[0].indices != window[1].indices
                && window[0].indices.map(|index| index.0)
                    < window[1].indices.map(|index| index.0)
        });
        prop_assert!(triads_are_unique_and_sorted);
        for triad in &triads {
            let first_marker = raw
                .iter()
                .find(|&&(a, b, c, _)| {
                    [a, b, c] == triad.indices.map(|index| index.0)
                })
                .map(|&(_, _, _, marker)| marker)
                .expect("retained tuple came from input");
            prop_assert_eq!(triad.strength.to_bits(), f32::from(first_marker).to_bits());
        }
    }
}
