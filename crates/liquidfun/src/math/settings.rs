//! Fixed compatibility settings from the selected upstream `b2Settings.h`.
//!
//! LiquidFun is tuned for meters-kilograms-seconds (MKS): moving shapes are
//! normally about 0.1 to 10 meters, linear velocities are meters per second,
//! and angular quantities use radians. Rendering-scale conversion belongs
//! outside the physics layer. These immutable values are compatibility inputs,
//! not runtime policy knobs.

/// Largest finite `f32`, corresponding to upstream `b2_maxFloat`.
pub const MAX_FLOAT: f32 = f32::MAX;

/// Difference between `1.0_f32` and the next representable value, corresponding
/// to upstream `b2_epsilon`.
pub const EPSILON: f32 = f32::EPSILON;

/// Upstream `b2_pi`, retained with its selected decimal spelling for mapping.
#[allow(
    clippy::approx_constant,
    clippy::excessive_precision,
    clippy::unreadable_literal
)] // Preserve the selected b2_pi token exactly.
pub const PI: f32 = 3.14159265359_f32;

/// One full rotation in radians, derived from [`PI`].
pub const TAU: f32 = 2.0 * PI;

/// Maximum contact points between two convex shapes (`b2_maxManifoldPoints`).
pub const MAX_MANIFOLD_POINTS: usize = 2;

/// Maximum vertices in a convex polygon (`b2_maxPolygonVertices`).
pub const MAX_POLYGON_VERTICES: usize = 8;

/// Dynamic-tree AABB extension in meters (`b2_aabbExtension`).
pub const AABB_EXTENSION: f32 = 0.1;

/// Dimensionless predicted-displacement multiplier (`b2_aabbMultiplier`).
pub const AABB_MULTIPLIER: f32 = 2.0;

/// Collision and constraint length tolerance in meters (`b2_linearSlop`).
pub const LINEAR_SLOP: f32 = 0.005;

/// Collision and constraint angular tolerance in radians (`b2_angularSlop`).
pub const ANGULAR_SLOP: f32 = 2.0 / 180.0 * PI;

/// Polygon and edge collision skin radius in meters (`b2_polygonRadius`).
pub const POLYGON_RADIUS: f32 = 2.0 * LINEAR_SLOP;

/// Maximum continuous-collision substeps per contact (`b2_maxSubSteps`).
pub const MAX_SUB_STEPS: usize = 8;

/// Maximum contacts solved for one time-of-impact event (`b2_maxTOIContacts`).
pub const MAX_TOI_CONTACTS: usize = 32;

/// Inelastic-collision threshold in meters per second (`b2_velocityThreshold`).
pub const VELOCITY_THRESHOLD: f32 = 1.0;

/// Maximum linear constraint correction in meters (`b2_maxLinearCorrection`).
pub const MAX_LINEAR_CORRECTION: f32 = 0.2;

/// Maximum angular constraint correction in radians (`b2_maxAngularCorrection`).
pub const MAX_ANGULAR_CORRECTION: f32 = 8.0 / 180.0 * PI;

/// Maximum translation during one simulation step, in meters (`b2_maxTranslation`).
pub const MAX_TRANSLATION: f32 = 2.0;

/// Squared maximum per-step translation in square meters (`b2_maxTranslationSquared`).
pub const MAX_TRANSLATION_SQUARED: f32 = MAX_TRANSLATION * MAX_TRANSLATION;

/// Maximum rotation during one simulation step, in radians (`b2_maxRotation`).
pub const MAX_ROTATION: f32 = 0.5 * PI;

/// Squared maximum per-step rotation in radians squared (`b2_maxRotationSquared`).
pub const MAX_ROTATION_SQUARED: f32 = MAX_ROTATION * MAX_ROTATION;

/// Dimensionless overlap-resolution factor (`b2_baumgarte`).
pub const BAUMGARTE: f32 = 0.2;

/// Dimensionless time-of-impact overlap-resolution factor.
///
/// The selected upstream source misspells this as `b2_toiBaugarte`.
pub const TOI_BAUMGARTE: f32 = 0.75;

/// Particle spacing as a dimensionless multiple of diameter (`b2_particleStride`).
pub const PARTICLE_STRIDE: f32 = 0.75;

/// Dimensionless minimum particle weight that produces pressure (`b2_minParticleWeight`).
pub const MIN_PARTICLE_WEIGHT: f32 = 1.0;

/// Dimensionless cap relative to critical particle pressure (`b2_maxParticlePressure`).
pub const MAX_PARTICLE_PRESSURE: f32 = 0.25;

/// Dimensionless cap relative to critical particle force (`b2_maxParticleForce`).
pub const MAX_PARTICLE_FORCE: f32 = 0.5;

/// Maximum triad separation as a dimensionless multiple of particle diameter
/// (`b2_maxTriadDistance`).
pub const MAX_TRIAD_DISTANCE: f32 = 2.0;

/// Squared maximum dimensionless triad separation (`b2_maxTriadDistanceSquared`).
pub const MAX_TRIAD_DISTANCE_SQUARED: f32 = MAX_TRIAD_DISTANCE * MAX_TRIAD_DISTANCE;

/// Initial particle-system buffer capacity (`b2_minParticleSystemBufferCapacity`).
pub const MIN_PARTICLE_SYSTEM_BUFFER_CAPACITY: usize = 256;

/// Dimensionless look-ahead multiplier applied to the step duration for barrier
/// collisions (`b2_barrierCollisionTime`).
pub const BARRIER_COLLISION_TIME: f32 = 2.5;

/// Stillness duration in seconds before a body may sleep (`b2_timeToSleep`).
pub const TIME_TO_SLEEP: f32 = 0.5;

/// Maximum sleeping linear velocity in meters per second (`b2_linearSleepTolerance`).
pub const LINEAR_SLEEP_TOLERANCE: f32 = 0.01;

/// Maximum sleeping angular velocity in radians per second (`b2_angularSleepTolerance`).
pub const ANGULAR_SLEEP_TOLERANCE: f32 = 2.0 / 180.0 * PI;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_foundation_has_pinned_bits() {
        // Arrange
        let expected = [0x7f7f_ffff, 0x3400_0000, 0x4049_0fdb, 0x40c9_0fdb];

        // Act
        let actual = [
            MAX_FLOAT.to_bits(),
            EPSILON.to_bits(),
            PI.to_bits(),
            TAU.to_bits(),
        ];

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(TAU.to_bits(), (2.0 * PI).to_bits());
    }

    #[test]
    fn collision_settings_have_pinned_values_and_relationships() {
        // Arrange
        let expected_bits = [
            0x3dcc_cccd,
            0x4000_0000,
            0x3ba3_d70a,
            0x3d0e_fa36,
            0x3c23_d70a,
        ];

        // Act
        let actual_bits = [
            AABB_EXTENSION.to_bits(),
            AABB_MULTIPLIER.to_bits(),
            LINEAR_SLOP.to_bits(),
            ANGULAR_SLOP.to_bits(),
            POLYGON_RADIUS.to_bits(),
        ];

        // Assert
        assert_eq!(MAX_MANIFOLD_POINTS, 2);
        assert_eq!(MAX_POLYGON_VERTICES, 8);
        assert_eq!(MAX_SUB_STEPS, 8);
        assert_eq!(actual_bits, expected_bits);
        assert_eq!(POLYGON_RADIUS.to_bits(), (2.0 * LINEAR_SLOP).to_bits());
    }

    #[test]
    fn dynamics_settings_have_pinned_values_and_relationships() {
        // Arrange
        let expected_bits = [
            0x3f80_0000,
            0x3e4c_cccd,
            0x3e0e_fa36,
            0x4000_0000,
            0x4080_0000,
            0x3fc9_0fdb,
            0x401d_e9e7,
            0x3e4c_cccd,
            0x3f40_0000,
        ];

        // Act
        let actual_bits = [
            VELOCITY_THRESHOLD.to_bits(),
            MAX_LINEAR_CORRECTION.to_bits(),
            MAX_ANGULAR_CORRECTION.to_bits(),
            MAX_TRANSLATION.to_bits(),
            MAX_TRANSLATION_SQUARED.to_bits(),
            MAX_ROTATION.to_bits(),
            MAX_ROTATION_SQUARED.to_bits(),
            BAUMGARTE.to_bits(),
            TOI_BAUMGARTE.to_bits(),
        ];

        // Assert
        assert_eq!(MAX_TOI_CONTACTS, 32);
        assert_eq!(actual_bits, expected_bits);
        assert_eq!(
            MAX_TRANSLATION_SQUARED.to_bits(),
            (MAX_TRANSLATION * MAX_TRANSLATION).to_bits()
        );
        assert_eq!(
            MAX_ROTATION_SQUARED.to_bits(),
            (MAX_ROTATION * MAX_ROTATION).to_bits()
        );
    }

    #[test]
    fn particle_settings_have_pinned_values_and_relationships() {
        // Arrange
        let expected_bits = [
            0x3f40_0000,
            0x3f80_0000,
            0x3e80_0000,
            0x3f00_0000,
            0x4000_0000,
            0x4080_0000,
            0x4020_0000,
        ];

        // Act
        let actual_bits = [
            PARTICLE_STRIDE.to_bits(),
            MIN_PARTICLE_WEIGHT.to_bits(),
            MAX_PARTICLE_PRESSURE.to_bits(),
            MAX_PARTICLE_FORCE.to_bits(),
            MAX_TRIAD_DISTANCE.to_bits(),
            MAX_TRIAD_DISTANCE_SQUARED.to_bits(),
            BARRIER_COLLISION_TIME.to_bits(),
        ];

        // Assert
        assert_eq!(MIN_PARTICLE_SYSTEM_BUFFER_CAPACITY, 256);
        assert_eq!(actual_bits, expected_bits);
        assert_eq!(
            MAX_TRIAD_DISTANCE_SQUARED.to_bits(),
            (MAX_TRIAD_DISTANCE * MAX_TRIAD_DISTANCE).to_bits()
        );
    }

    #[test]
    fn sleep_settings_have_pinned_values_and_angular_mapping() {
        // Arrange
        let expected_bits = [0x3f00_0000, 0x3c23_d70a, 0x3d0e_fa36];

        // Act
        let actual_bits = [
            TIME_TO_SLEEP.to_bits(),
            LINEAR_SLEEP_TOLERANCE.to_bits(),
            ANGULAR_SLEEP_TOLERANCE.to_bits(),
        ];

        // Assert
        assert_eq!(actual_bits, expected_bits);
        assert_eq!(ANGULAR_SLEEP_TOLERANCE.to_bits(), ANGULAR_SLOP.to_bits());
    }
}
