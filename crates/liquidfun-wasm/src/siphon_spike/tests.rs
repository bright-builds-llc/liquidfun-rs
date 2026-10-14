//! Pure evaluator and geometry checks for the siphon spike.

use liquidfun::math::Vec2;

use super::geometry::{self, CREST, Rect, SOURCE};
use super::{MAXIMUM_COUNT, RuleA, Sample, evaluate_rule_a, evaluate_rule_b, surface};

/// Builds a 0.25 s sample series from per-sample source surfaces, dest
/// surfaces and `count_d` values.
fn series(sources: &[f32], dests: &[f32], counts: &[usize]) -> Vec<Sample> {
    sources
        .iter()
        .zip(dests)
        .zip(counts)
        .enumerate()
        .map(
            |(index, ((&source_surface, &dest_surface), &count_d))| Sample {
                seconds: index as f32 * 0.25,
                source_surface,
                dest_surface,
                count_d,
                count_s: 0,
                leaks: 0,
                lost: 0,
            },
        )
        .collect()
}

#[test]
fn rule_a_passes_a_steadily_rising_series() {
    // Arrange
    let sources: Vec<f32> = (0..20).map(|index| 2.0 - 0.05 * index as f32).collect();
    let dests = vec![0.0; 20];
    let counts: Vec<usize> = (0..20).map(|index| 200 + 3 * index).collect();
    let samples = series(&sources, &dests, &counts);

    // Act
    let rule_a = evaluate_rule_a(&samples, CREST);

    // Assert
    assert_eq!(
        rule_a,
        RuleA {
            maybe_t_star: Some(8),
            pass: true
        }
    );
}

#[test]
fn rule_a_fails_when_transfer_stops_after_t_star() {
    // Arrange
    let sources: Vec<f32> = (0..20).map(|index| 2.0 - 0.05 * index as f32).collect();
    let dests = vec![0.0; 20];
    let counts: Vec<usize> = (0..20).map(|index| 200 + 3 * index.min(10)).collect();
    let samples = series(&sources, &dests, &counts);

    // Act
    let rule_a = evaluate_rule_a(&samples, CREST);

    // Assert
    assert!(!rule_a.pass);
}

#[test]
fn rule_a_fails_when_the_source_never_reaches_crest_minus_0_3() {
    // Arrange
    let sources = vec![2.0; 20];
    let dests = vec![0.0; 20];
    let counts: Vec<usize> = (0..20).map(|index| 200 + 3 * index).collect();
    let samples = series(&sources, &dests, &counts);

    // Act
    let rule_a = evaluate_rule_a(&samples, CREST);

    // Assert
    assert_eq!(
        rule_a,
        RuleA {
            maybe_t_star: None,
            pass: false
        }
    );
}

#[test]
fn rule_a_ignores_a_priming_drain_in_the_first_two_seconds() {
    // Arrange
    let sources = vec![1.5; 20];
    let dests = vec![0.0; 20];
    let counts: Vec<usize> = (0..20).map(|index| 200 + 30 * index.min(8)).collect();
    let samples = series(&sources, &dests, &counts);

    // Act
    let rule_a = evaluate_rule_a(&samples, CREST);

    // Assert
    assert_eq!(
        rule_a,
        RuleA {
            maybe_t_star: Some(8),
            pass: false
        }
    );
}

#[test]
fn rule_b_passes_a_final_level_difference_of_0_1() {
    // Arrange
    let samples = series(&[1.8, 1.0], &[0.0, 0.9], &[200, 300]);

    // Act
    let equal = evaluate_rule_b(&samples, CREST);

    // Assert
    assert!(equal);
}

#[test]
fn rule_b_fails_a_final_level_difference_of_0_2() {
    // Arrange
    let samples = series(&[1.8, 1.0], &[0.0, 0.8], &[200, 300]);

    // Act
    let equal = evaluate_rule_b(&samples, CREST);

    // Assert
    assert!(!equal);
}

#[test]
fn surface_of_a_flat_column_is_its_top_row() {
    // Arrange
    let region = Rect {
        min: Vec2::new(0.0, 0.0),
        max: Vec2::new(1.0, 3.0),
    };
    let column: Vec<Vec2> = (0..27)
        .flat_map(|row| {
            (0..13).map(move |column| {
                Vec2::new(0.05 + 0.075 * column as f32, 0.05 + 0.075 * row as f32)
            })
        })
        .collect();

    // Act
    let top = surface(&column, region);

    // Assert
    assert!((top - 2.0).abs() < 1e-4, "surface {top}");
}

#[test]
fn primed_fill_fits_the_budget_and_starts_inside() {
    // Arrange
    let points = geometry::primed_fill();

    // Act
    let leaks = points.iter().filter(|p| geometry::is_leak(**p)).count();
    let in_source = points.iter().filter(|p| SOURCE.contains(**p)).count();

    // Assert
    println!(
        "GEOMETRY siphon particles {} in_source {in_source}",
        points.len()
    );
    assert!(points.len() <= 3_000 && points.len() < MAXIMUM_COUNT);
    assert_eq!(leaks, 0);
}
