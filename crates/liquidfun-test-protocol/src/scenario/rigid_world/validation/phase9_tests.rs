use super::*;
use crate::Phase9ParticleBufferMode;

fn id(value: &str) -> ScenarioId {
    ScenarioId::new(value).expect("test scenario identity should be valid")
}

fn system_declaration() -> Phase9ParticleSystemDeclaration {
    Phase9ParticleSystemDeclaration {
        system_id: id("system"),
        buffer_mode: Phase9ParticleBufferMode::Growable {
            initial_capacity: 1,
        },
        paused: false,
        strict_contact_check: false,
        stuck_threshold: 0,
        density_bits: FloatBits::from_f32(1.0),
        gravity_scale_bits: FloatBits::from_f32(1.0),
        radius_bits: FloatBits::from_f32(0.1),
        damping_bits: FloatBits::from_f32(0.0),
        destruction_by_age: false,
        lifetime_granularity_bits: FloatBits::from_f32(1.0 / 60.0),
        maximum_count: None,
    }
}

#[test]
fn phase9_declaration_accepts_negative_finite_lifetime_bits() {
    // Arrange
    let systems = [system_declaration()];
    let particles = [Phase9ParticleDeclaration {
        particle_id: id("particle"),
        system_id: id("system"),
        position: crate::Vec2Bits {
            x_bits: FloatBits::from_f32(0.0),
            y_bits: FloatBits::from_f32(0.0),
        },
        velocity: crate::Vec2Bits {
            x_bits: FloatBits::from_f32(0.0),
            y_bits: FloatBits::from_f32(0.0),
        },
        flags_bits: 0,
        color: [0; 4],
        lifetime_bits: FloatBits::from_f32(-1.0),
    }];

    // Act
    let result = validate_phase9_declarations(&systems, &particles);

    // Assert
    assert!(result.is_ok());
}
