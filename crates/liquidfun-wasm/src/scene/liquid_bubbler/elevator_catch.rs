//! The deck fills the shaft, and the lowered divider spills liquid onto it.

use liquidfun::JointKind;
use liquidfun::WorldObservationLimits;

use crate::scene::SceneId;
use crate::session::SessionCore;

#[test]
fn plate_spans_the_shaft_under_a_lower_divider() {
    // Arrange
    let corners = super::super::plate_local_corners();
    let left = corners
        .iter()
        .fold(f32::MAX, |bound, corner| bound.min(corner.x));
    let right = corners
        .iter()
        .fold(f32::MIN, |bound, corner| bound.max(corner.x));
    let deck_top = corners
        .iter()
        .fold(f32::MIN, |bound, corner| bound.max(corner.y));

    // Act
    let left_gap = left + super::super::PLATE_CENTER.x - (0.52 + super::super::WALL_HALF);
    let right_gap = (1.00 - super::super::WALL_HALF) - (right + super::super::PLATE_CENTER.x);

    // Assert
    assert!(
        left_gap > 0.0 && left_gap < super::super::PARTICLE_RADIUS * 2.0,
        "{left_gap}"
    );
    assert!(
        right_gap > 0.0 && right_gap < super::super::PARTICLE_RADIUS * 2.0,
        "{right_gap}"
    );
    assert!(super::super::DIVIDER_BOTTOM_Y > deck_top + super::super::PLATE_CENTER.y);
    assert!(super::super::DIVIDER_BOTTOM_Y < 0.22);
}

#[test]
fn second_dwell_keeps_a_load_on_the_deck() {
    // Arrange
    let mut session =
        SessionCore::create(SceneId::LiquidBubbler).expect("liquid bubbler should construct");
    let steps = (15.0 / super::super::SIM_DT).ceil() as usize;

    // Act
    for _ in 0..steps.div_ceil(4) {
        session.advance(4).expect("the second dwell should step");
    }
    let (translation, on_deck, under) = session.read_particles(|world, system| {
        let observation = world
            .world_observation(WorldObservationLimits::reviewed())
            .expect("reviewed observation should include the plate joint");
        let joint = observation
            .joints()
            .iter()
            .find(|joint| joint.snapshot().kind() == JointKind::Prismatic)
            .expect("the plate rides a prismatic joint");
        let translation = world
            .prismatic_joint_translation(joint.id())
            .expect("the plate translation should be readable");
        let view = world
            .particle_system_view(system)
            .expect("the water system should be live");
        let deck = super::super::PLATE_CENTER.y + translation;
        let mut on_deck = 0;
        let mut under = 0;
        for position in view.positions() {
            if position.x <= 0.56 || position.x >= 0.96 {
                continue;
            }
            if (deck..deck + 0.15).contains(&position.y) {
                on_deck += 1;
            } else if position.y < 0.03 {
                under += 1;
            }
        }
        (translation, on_deck, under)
    });

    // Assert
    assert!(
        translation.abs() < 0.001,
        "the deck is seated for the second dwell, translation {translation}"
    );
    assert!(
        on_deck > under,
        "the deck should hold more liquid than has slipped underneath, on {on_deck} under {under}"
    );
    assert!(
        on_deck > 300,
        "the deck should be carrying a load, on {on_deck}"
    );
}
