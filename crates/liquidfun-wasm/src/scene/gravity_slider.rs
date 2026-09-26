//! Shared playground gravity slider.
//!
//! Every catalog scene accepts this control. The magnitude is whole m/s²
//! straight down, from 2 through five times the former Dam Break High preset.
//! Keep the ends in sync with `GRAVITY_SLIDER_MIN` and `GRAVITY_SLIDER_MAX`
//! in `web/src/catalog/scenes.ts`.

use liquidfun::World;
use liquidfun::math::Vec2;

use crate::session::SessionError;

/// Control name shared by the playground catalog and this crate.
pub(crate) const GRAVITY_CONTROL_NAME: &str = "gravity";
/// Lowest slider magnitude, in m/s². Keep in sync with `GRAVITY_SLIDER_MIN`.
pub(crate) const MIN_GRAVITY_MAGNITUDE: u16 = 2;
/// Former Dam Break High preset. The slider maximum is five times this magnitude.
pub(crate) const FORMER_HIGH_GRAVITY_MAGNITUDE: u16 = 16;
pub(crate) const MAX_GRAVITY_MAGNITUDE: u16 = FORMER_HIGH_GRAVITY_MAGNITUDE * 5;
/// Documented normal gravity. Keep in sync with `GRAVITY_SLIDER_DEFAULT`.
pub(crate) const DEFAULT_GRAVITY_MAGNITUDE: f32 = 10.0;

/// Parses a slider magnitude in whole m/s² inside the shared slider.
pub(crate) fn parse_gravity_magnitude(value: &str) -> Option<f32> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if value.len() > 1 && value.starts_with('0') {
        return None;
    }

    let magnitude = value.parse::<u16>().ok()?;
    if !(MIN_GRAVITY_MAGNITUDE..=MAX_GRAVITY_MAGNITUDE).contains(&magnitude) {
        return None;
    }

    Some(f32::from(magnitude))
}

pub(crate) fn gravity_vector(magnitude: f32) -> Vec2 {
    Vec2::new(0.0, -magnitude)
}

/// Gravity slider value removed from the presets a scene builder may see.
pub(crate) struct SplitGravityPreset {
    pub(crate) maybe_magnitude: Option<f32>,
    pub(crate) scene_presets: Vec<(String, String)>,
}

/// Removes the gravity preset so scene builders keep their own allowlists.
///
/// The last gravity entry wins. A missing entry leaves the scene's authored
/// gravity in place. An out-of-range token fails closed.
pub(crate) fn split_gravity_preset(
    presets: &[(String, String)],
) -> Result<SplitGravityPreset, SessionError> {
    let maybe_magnitude = presets
        .iter()
        .rev()
        .find(|(name, _value)| name == GRAVITY_CONTROL_NAME)
        .map(|(_name, value)| parse_gravity_magnitude(value).ok_or(SessionError::UnknownControl))
        .transpose()?;
    let scene_presets = presets
        .iter()
        .filter(|(name, _value)| name != GRAVITY_CONTROL_NAME)
        .cloned()
        .collect();
    Ok(SplitGravityPreset {
        maybe_magnitude,
        scene_presets,
    })
}

/// Replaces world gravity when the slider preset is present.
pub(crate) fn apply_gravity_preset(
    world: &mut World,
    maybe_magnitude: Option<f32>,
) -> Result<(), SessionError> {
    let Some(magnitude) = maybe_magnitude else {
        return Ok(());
    };
    world
        .set_gravity(gravity_vector(magnitude))
        .map_err(|_error| SessionError::SceneConstruction)
}

#[cfg(test)]
mod tests {
    use super::{
        FORMER_HIGH_GRAVITY_MAGNITUDE, MAX_GRAVITY_MAGNITUDE, MIN_GRAVITY_MAGNITUDE,
        parse_gravity_magnitude,
    };
    use crate::scene::{SceneId, build_scene};

    #[test]
    fn parse_accepts_whole_magnitudes_from_two_through_the_cap() {
        // Arrange
        let accepted = [("2", 2.0_f32), ("5", 5.0), ("10", 10.0), ("80", 80.0)];
        let rejected = ["", "0", "1", "81", "08", "10.0", "high", "low"];

        // Act / Assert
        assert_eq!(MAX_GRAVITY_MAGNITUDE, FORMER_HIGH_GRAVITY_MAGNITUDE * 5);
        assert_eq!(MIN_GRAVITY_MAGNITUDE, 2);
        for (token, magnitude) in accepted {
            assert_eq!(
                parse_gravity_magnitude(token),
                Some(magnitude),
                "gravity={token}"
            );
        }
        for token in rejected {
            assert_eq!(parse_gravity_magnitude(token), None, "gravity={token}");
        }
    }

    #[test]
    fn every_scene_builds_downward_gravity_from_the_slider() {
        // Arrange
        let ids = all_scene_ids();

        // Act / Assert
        let mut seen = [false; 17];
        for id in ids {
            seen[variant_index(id)] = true;
            let low = build_scene(id, &[("gravity".to_owned(), "2".to_owned())])
                .unwrap_or_else(|_| panic!("{id:?} should accept gravity 2"));
            let high = build_scene(id, &[("gravity".to_owned(), "80".to_owned())])
                .unwrap_or_else(|_| panic!("{id:?} should accept gravity 80"));
            let rejected = build_scene(id, &[("gravity".to_owned(), "1".to_owned())]);

            assert_eq!(low.world.gravity().x.to_bits(), 0.0_f32.to_bits(), "{id:?}");
            assert_eq!(
                low.world.gravity().y.to_bits(),
                (-2.0_f32).to_bits(),
                "{id:?}"
            );
            assert_eq!(
                high.world.gravity().y.to_bits(),
                (-80.0_f32).to_bits(),
                "{id:?}"
            );
            assert!(rejected.is_err(), "{id:?} must reject gravity 1");
        }
        assert_eq!(
            seen, [true; 17],
            "every SceneId variant must be built with the gravity slider"
        );
    }

    #[test]
    fn liquid_tumbler_keeps_earth_gravity_until_the_slider_is_set() {
        // Arrange / Act
        let scene = build_scene(SceneId::LiquidTumbler, &[])
            .expect("Liquid Tumbler should construct at its authored gravity");

        // Assert
        assert_eq!(scene.world.gravity().x.to_bits(), 0.0_f32.to_bits());
        assert_eq!(scene.world.gravity().y.to_bits(), (-9.8_f32).to_bits());
    }

    fn variant_index(id: SceneId) -> usize {
        match id {
            SceneId::DamBreak => 0,
            SceneId::Fountain => 1,
            SceneId::FloatOrSink => 2,
            SceneId::ColorMixer => 3,
            SceneId::JellyDrop => 4,
            SceneId::WaterWheel => 5,
            SceneId::Particles => 6,
            SceneId::LiquidTimer => 7,
            SceneId::SurfaceTension => 8,
            SceneId::ElasticParticles => 9,
            SceneId::RigidParticles => 10,
            SceneId::Soup => 11,
            SceneId::SoupStirrer => 12,
            SceneId::Impulse => 13,
            SceneId::WaveMachine => 14,
            SceneId::TheoJansen => 15,
            SceneId::LiquidTumbler => 16,
        }
    }

    fn all_scene_ids() -> [SceneId; 17] {
        [
            SceneId::DamBreak,
            SceneId::Fountain,
            SceneId::FloatOrSink,
            SceneId::ColorMixer,
            SceneId::JellyDrop,
            SceneId::WaterWheel,
            SceneId::Particles,
            SceneId::LiquidTimer,
            SceneId::SurfaceTension,
            SceneId::ElasticParticles,
            SceneId::RigidParticles,
            SceneId::Soup,
            SceneId::SoupStirrer,
            SceneId::Impulse,
            SceneId::WaveMachine,
            SceneId::TheoJansen,
            SceneId::LiquidTumbler,
        ]
    }
}
