use std::collections::BTreeSet;

use super::{fnv1a64, level_hash_hex};
use crate::puzzle::first_pour::FIRST_POUR;
use crate::puzzle::spec::LevelSpec;
use crate::puzzle::{LEVEL_IDS, PuzzleLevelInfo, level_spec};

const PINS: &str = include_str!("../level-hashes.txt");

/// Parses the pin file into `(level id, hex)` pairs, failing on malformed lines.
fn pinned_hashes() -> Vec<(&'static str, &'static str)> {
    PINS.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let fields: Vec<&str> = line.split(' ').collect();
            let [id, hex] = fields.as_slice() else {
                panic!("pin line {line:?} must have exactly two space-separated fields");
            };
            (*id, *hex)
        })
        .collect()
}

#[test]
fn fnv1a64_matches_the_empty_input_vector() {
    // Arrange
    let bytes = b"";

    // Act
    let hash = fnv1a64(bytes);

    // Assert
    assert_eq!(hash, 0xcbf2_9ce4_8422_2325);
}

#[test]
fn fnv1a64_matches_the_single_byte_vector() {
    // Arrange
    let bytes = b"a";

    // Act
    let hash = fnv1a64(bytes);

    // Assert
    assert_eq!(hash, 0xaf63_dc4c_8601_ec8c);
}

#[test]
fn fnv1a64_matches_the_foobar_vector() {
    // Arrange
    let bytes = b"foobar";

    // Act
    let hash = fnv1a64(bytes);

    // Assert
    assert_eq!(hash, 0x8594_4171_f739_67e8);
}

#[test]
fn every_level_hash_is_sixteen_lowercase_hex_digits() {
    // Arrange
    let specs = LEVEL_IDS.map(level_spec);

    // Act
    let hashes = specs.map(level_hash_hex);

    // Assert
    for hash in hashes {
        assert_eq!(hash.len(), 16, "{hash}");
        assert!(
            hash.bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
            "{hash}"
        );
    }
}

#[test]
fn level_hashes_are_pairwise_distinct() {
    // Arrange
    let specs = LEVEL_IDS.map(level_spec);

    // Act
    let distinct: BTreeSet<String> = specs.into_iter().map(level_hash_hex).collect();

    // Assert
    assert_eq!(distinct.len(), LEVEL_IDS.len());
}

#[test]
fn level_hashes_match_the_pin_file_in_level_order() {
    // Arrange
    let expected: Vec<(&str, String)> = LEVEL_IDS
        .into_iter()
        .map(|id| (id.as_str(), level_hash_hex(level_spec(id))))
        .collect();

    // Act
    let pinned: Vec<(&str, String)> = pinned_hashes()
        .into_iter()
        .map(|(id, hex)| (id, hex.to_owned()))
        .collect();

    // Assert
    assert_eq!(
        pinned, expected,
        "a LevelSpec changed: update level-hashes.txt (saved placements for it are discarded)"
    );
}

#[test]
fn changing_one_spec_field_changes_the_hash() {
    // Arrange
    let changed = LevelSpec {
        target_count: FIRST_POUR.target_count + 1,
        ..FIRST_POUR
    };

    // Act
    let hash = level_hash_hex(&changed);

    // Assert
    assert_ne!(hash, level_hash_hex(&FIRST_POUR));
}

#[test]
fn level_hash_is_repeatable() {
    // Arrange
    let spec = &FIRST_POUR;

    // Act
    let (first, second) = (level_hash_hex(spec), level_hash_hex(spec));

    // Assert
    assert_eq!(first, second);
}

#[test]
fn level_info_getter_returns_the_level_hash() {
    for id in LEVEL_IDS {
        // Arrange
        let info = PuzzleLevelInfo::from_spec(level_spec(id));

        // Act
        let hash = info.level_hash();

        // Assert
        assert_eq!(hash, level_hash_hex(level_spec(id)), "{}", id.as_str());
    }
}

#[test]
#[ignore = "prints the pin file body"]
fn print_level_hashes() {
    for id in LEVEL_IDS {
        println!("{} {}", id.as_str(), level_hash_hex(level_spec(id)));
    }
}
