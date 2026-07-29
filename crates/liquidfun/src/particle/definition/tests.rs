use super::{ParticleSystemDef, ParticleSystemDefError};

type Builder = fn(ParticleSystemDef, f32) -> Result<ParticleSystemDef, ParticleSystemDefError>;
type Accessor = fn(ParticleSystemDef) -> f32;

struct CoefficientCase {
    name: &'static str,
    default: f32,
    builder: Builder,
    accessor: Accessor,
    non_finite_error: ParticleSystemDefError,
    negative_error: ParticleSystemDefError,
}

fn coefficient_cases() -> [CoefficientCase; 12] {
    [
        CoefficientCase {
            name: "pressure strength",
            default: 0.05,
            builder: ParticleSystemDef::with_pressure_strength,
            accessor: ParticleSystemDef::pressure_strength,
            non_finite_error: ParticleSystemDefError::NonFinitePressureStrength,
            negative_error: ParticleSystemDefError::NegativePressureStrength,
        },
        CoefficientCase {
            name: "elastic strength",
            default: 0.25,
            builder: ParticleSystemDef::with_elastic_strength,
            accessor: ParticleSystemDef::elastic_strength,
            non_finite_error: ParticleSystemDefError::NonFiniteElasticStrength,
            negative_error: ParticleSystemDefError::NegativeElasticStrength,
        },
        CoefficientCase {
            name: "spring strength",
            default: 0.25,
            builder: ParticleSystemDef::with_spring_strength,
            accessor: ParticleSystemDef::spring_strength,
            non_finite_error: ParticleSystemDefError::NonFiniteSpringStrength,
            negative_error: ParticleSystemDefError::NegativeSpringStrength,
        },
        CoefficientCase {
            name: "viscous strength",
            default: 0.25,
            builder: ParticleSystemDef::with_viscous_strength,
            accessor: ParticleSystemDef::viscous_strength,
            non_finite_error: ParticleSystemDefError::NonFiniteViscousStrength,
            negative_error: ParticleSystemDefError::NegativeViscousStrength,
        },
        CoefficientCase {
            name: "surface-tension pressure strength",
            default: 0.2,
            builder: ParticleSystemDef::with_surface_tension_pressure_strength,
            accessor: ParticleSystemDef::surface_tension_pressure_strength,
            non_finite_error: ParticleSystemDefError::NonFiniteSurfaceTensionPressureStrength,
            negative_error: ParticleSystemDefError::NegativeSurfaceTensionPressureStrength,
        },
        CoefficientCase {
            name: "surface-tension normal strength",
            default: 0.2,
            builder: ParticleSystemDef::with_surface_tension_normal_strength,
            accessor: ParticleSystemDef::surface_tension_normal_strength,
            non_finite_error: ParticleSystemDefError::NonFiniteSurfaceTensionNormalStrength,
            negative_error: ParticleSystemDefError::NegativeSurfaceTensionNormalStrength,
        },
        CoefficientCase {
            name: "repulsive strength",
            default: 1.0,
            builder: ParticleSystemDef::with_repulsive_strength,
            accessor: ParticleSystemDef::repulsive_strength,
            non_finite_error: ParticleSystemDefError::NonFiniteRepulsiveStrength,
            negative_error: ParticleSystemDefError::NegativeRepulsiveStrength,
        },
        CoefficientCase {
            name: "powder strength",
            default: 0.5,
            builder: ParticleSystemDef::with_powder_strength,
            accessor: ParticleSystemDef::powder_strength,
            non_finite_error: ParticleSystemDefError::NonFinitePowderStrength,
            negative_error: ParticleSystemDefError::NegativePowderStrength,
        },
        CoefficientCase {
            name: "ejection strength",
            default: 0.5,
            builder: ParticleSystemDef::with_ejection_strength,
            accessor: ParticleSystemDef::ejection_strength,
            non_finite_error: ParticleSystemDefError::NonFiniteEjectionStrength,
            negative_error: ParticleSystemDefError::NegativeEjectionStrength,
        },
        CoefficientCase {
            name: "static-pressure strength",
            default: 0.2,
            builder: ParticleSystemDef::with_static_pressure_strength,
            accessor: ParticleSystemDef::static_pressure_strength,
            non_finite_error: ParticleSystemDefError::NonFiniteStaticPressureStrength,
            negative_error: ParticleSystemDefError::NegativeStaticPressureStrength,
        },
        CoefficientCase {
            name: "static-pressure relaxation",
            default: 0.2,
            builder: ParticleSystemDef::with_static_pressure_relaxation,
            accessor: ParticleSystemDef::static_pressure_relaxation,
            non_finite_error: ParticleSystemDefError::NonFiniteStaticPressureRelaxation,
            negative_error: ParticleSystemDefError::NegativeStaticPressureRelaxation,
        },
        CoefficientCase {
            name: "color-mixing strength",
            default: 0.5,
            builder: ParticleSystemDef::with_color_mixing_strength,
            accessor: ParticleSystemDef::color_mixing_strength,
            non_finite_error: ParticleSystemDefError::NonFiniteColorMixingStrength,
            negative_error: ParticleSystemDefError::NegativeColorMixingStrength,
        },
    ]
}

#[test]
fn solver_coefficient_defaults_match_pinned_bits() {
    // Arrange / Act
    let definition = ParticleSystemDef::default();

    // Assert
    for case in coefficient_cases() {
        assert_eq!(
            (case.accessor)(definition).to_bits(),
            case.default.to_bits(),
            "{}",
            case.name
        );
    }
}

#[test]
fn solver_coefficient_builders_preserve_configured_bits() {
    // Arrange
    let definition = ParticleSystemDef::default();
    let configured = 0.375_f32;

    for case in coefficient_cases() {
        // Act
        let result = (case.builder)(definition, configured)
            .expect("finite non-negative solver coefficient should be valid");

        // Assert
        assert_eq!(
            (case.accessor)(result).to_bits(),
            configured.to_bits(),
            "{}",
            case.name
        );
    }
}

#[test]
fn solver_coefficient_builders_reject_non_finite_values_without_mutation() {
    // Arrange
    let definition = ParticleSystemDef::default();
    let non_finite_values = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY];

    for case in coefficient_cases() {
        for value in non_finite_values {
            // Act
            let result = (case.builder)(definition, value);

            // Assert
            assert_eq!(result, Err(case.non_finite_error), "{}", case.name);
            assert_eq!(
                (case.accessor)(definition).to_bits(),
                case.default.to_bits(),
                "{}",
                case.name
            );
        }
    }
}

#[test]
fn solver_coefficient_builders_reject_negative_values_without_mutation() {
    // Arrange
    let definition = ParticleSystemDef::default();

    for case in coefficient_cases() {
        // Act
        let result = (case.builder)(definition, -0.25);

        // Assert
        assert_eq!(result, Err(case.negative_error), "{}", case.name);
        assert_eq!(
            (case.accessor)(definition).to_bits(),
            case.default.to_bits(),
            "{}",
            case.name
        );
    }
}

#[test]
fn solver_coefficient_builders_accept_exact_positive_zero() {
    // Arrange
    let definition = ParticleSystemDef::default();

    for case in coefficient_cases() {
        // Act
        let result = (case.builder)(definition, 0.0)
            .expect("zero disables or suppresses the corresponding solver effect");

        // Assert
        assert_eq!((case.accessor)(result).to_bits(), 0.0_f32.to_bits());
    }
}
