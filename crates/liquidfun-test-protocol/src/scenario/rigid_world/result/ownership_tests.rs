use super::fixture_belongs_to_live_body;
use crate::ScenarioId;

fn id(value: &str) -> ScenarioId {
    ScenarioId::new(value).expect("test semantic ID should be valid")
}

#[test]
fn live_fixture_identity_must_match_its_claimed_body_owner() {
    // Arrange
    let body_a = id("body-a");
    let body_b = id("body-b");
    let fixture_a = id("fixture-a");
    let fixture_b = id("fixture-b");
    let live_body_ids = [&body_a, &body_b];
    let live_fixture_owners = [(&fixture_a, &body_a), (&fixture_b, &body_b)];

    // Act
    let a_owns_a = fixture_belongs_to_live_body(
        &live_body_ids,
        live_fixture_owners.iter().copied(),
        &fixture_a,
        &body_a,
    );
    let b_owns_b = fixture_belongs_to_live_body(
        &live_body_ids,
        live_fixture_owners.iter().copied(),
        &fixture_b,
        &body_b,
    );
    let a_owns_b = fixture_belongs_to_live_body(
        &live_body_ids,
        live_fixture_owners.iter().copied(),
        &fixture_b,
        &body_a,
    );
    let b_owns_a = fixture_belongs_to_live_body(
        &live_body_ids,
        live_fixture_owners.iter().copied(),
        &fixture_a,
        &body_b,
    );

    // Assert
    assert!(a_owns_a);
    assert!(b_owns_b);
    assert!(!a_owns_b);
    assert!(!b_owns_a);
}
