//! Level materials, system tuning and floats (Phase 40 D-10 to D-12, D-28,
//! D-29): jelly never counts, a tuned viscous level steps cleanly, and a
//! level float is reported as a frame circle and rises in water.

use liquidfun::math::Vec2;
use liquidfun::particle::{ParticleColor, ParticleFlags};

use crate::frame::ProofFrame;
use crate::puzzle::first_pour::FIRST_POUR;
use crate::puzzle::spec::{FloatBody, LevelSpec, LiquidBlock, LiquidMaterial, SystemTuning};
use crate::puzzle::tools::Placements;
use crate::session::{SessionCore, SessionError};

const JELLY_COLOR: ParticleColor = ParticleColor::new(255, 120, 200, 255);

/// First-pour's reservoir water, then an uncounted jelly pad inside the cup.
const JELLY_IN_CUP: LevelSpec = LevelSpec {
    liquid_blocks: &[
        FIRST_POUR.liquid_blocks[0],
        LiquidBlock {
            min: Vec2::new(1.8, 0.2),
            max: Vec2::new(3.2, 0.8),
            spacing: 0.06,
            color: JELLY_COLOR,
            counted: false,
            material: LiquidMaterial::Elastic,
        },
    ],
    ..FIRST_POUR
};

/// First-pour's reservoir as viscous liquid at a level-band strength.
const VISCOUS_RESERVOIR: LevelSpec = LevelSpec {
    system_tuning: SystemTuning {
        maybe_viscous_strength: Some(1.25),
        ..SystemTuning::NONE
    },
    liquid_blocks: &[LiquidBlock {
        material: LiquidMaterial::Viscous,
        ..FIRST_POUR.liquid_blocks[0]
    }],
    ..FIRST_POUR
};

const FLOAT_START: Vec2 = Vec2::new(2.5, 0.3);
const FLOAT_RADIUS: f32 = 0.25;

/// Water filling the cup above a light circle resting near its floor.
const FLOAT_IN_CUP: LevelSpec = LevelSpec {
    liquid_blocks: &[LiquidBlock {
        min: Vec2::new(1.64, 0.6),
        max: Vec2::new(3.36, 1.76),
        ..FIRST_POUR.liquid_blocks[0]
    }],
    floats: &[FloatBody {
        center: FLOAT_START,
        radius: FLOAT_RADIUS,
        density: 0.4,
    }],
    ..FIRST_POUR
};

/// A float with a zero radius, which the builder must reject.
const FLAT_FLOAT: LevelSpec = LevelSpec {
    floats: &[FloatBody {
        radius: 0.0,
        ..FLOAT_IN_CUP.floats[0]
    }],
    ..FLOAT_IN_CUP
};

fn session(spec: &'static LevelSpec) -> SessionCore {
    SessionCore::create_puzzle_from_spec(spec, Placements::empty()).expect("test level builds")
}

fn goal_count(core: &SessionCore) -> u32 {
    core.maybe_puzzle_status()
        .expect("puzzle sessions report status")
        .goal_count
}

/// `(elastic, other)` particles strictly inside `spec.cup_interior`.
fn cup_particles_by_material(core: &SessionCore, spec: &LevelSpec) -> (usize, usize) {
    core.read_particles(|world, system| {
        let view = world
            .particle_system_view(system)
            .expect("live particle system has a view");
        view.positions()
            .iter()
            .zip(view.flags())
            .filter(|(position, _)| spec.cup_interior.contains_strictly(**position))
            .fold((0, 0), |(elastic, other), (_, flags)| {
                if flags.contains(ParticleFlags::ELASTIC) {
                    (elastic + 1, other)
                } else {
                    (elastic, other + 1)
                }
            })
    })
}

/// `(center, radius)` of every frame circle.
fn frame_circles(core: &SessionCore) -> Vec<(Vec2, f32)> {
    let frame = ProofFrame::from(core.capture_frame().expect("puzzle frame captures"));
    frame
        .rigid_circles()
        .chunks_exact(3)
        .map(|circle| (Vec2::new(circle[0], circle[1]), circle[2]))
        .collect()
}

fn advance_steps(core: &mut SessionCore, steps: u32) -> Result<(), SessionError> {
    for _ in 0..steps {
        core.advance(1)?;
    }
    Ok(())
}

#[test]
fn first_pour_keeps_plain_water_without_tuning_or_floats() {
    // Arrange
    let spec = &FIRST_POUR;

    // Act
    let materials: Vec<_> = spec
        .liquid_blocks
        .iter()
        .map(|block| block.material)
        .collect();

    // Assert
    assert_eq!(materials, [LiquidMaterial::Water]);
    assert_eq!(spec.system_tuning, SystemTuning::NONE);
    assert!(spec.floats.is_empty());
}

#[test]
fn jelly_inside_the_cup_never_counts_at_step_zero() {
    // Arrange
    let core = session(&JELLY_IN_CUP);

    // Act
    let (pad_in_cup, water_in_cup) = cup_particles_by_material(&core, &JELLY_IN_CUP);

    // Assert
    assert!(
        pad_in_cup > 100,
        "only {pad_in_cup} pad particles start in the cup"
    );
    assert_eq!(water_in_cup, 0);
    assert_eq!(goal_count(&core), 0);
}

#[test]
fn jelly_inside_the_cup_never_counts_after_stepping() {
    // Arrange
    let mut core = session(&JELLY_IN_CUP);

    // Act
    advance_steps(&mut core, 60).expect("the jelly level steps");

    // Assert
    let (pad_in_cup, water_in_cup) = cup_particles_by_material(&core, &JELLY_IN_CUP);
    assert!(
        pad_in_cup > 100,
        "only {pad_in_cup} pad particles stay in the cup"
    );
    assert_eq!(
        u32::try_from(water_in_cup).ok(),
        Some(goal_count(&core)),
        "only counted water fills the cup"
    );
    assert_eq!(goal_count(&core), 0);
}

#[test]
fn tuned_viscous_reservoir_steps_without_failing() {
    // Arrange
    let mut core = session(&VISCOUS_RESERVOIR);

    // Act
    let result = advance_steps(&mut core, 120);

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(core.step_index(), 120);
}

#[test]
fn level_float_is_a_frame_circle_at_step_zero() {
    // Arrange
    let core = session(&FLOAT_IN_CUP);

    // Act
    let circles = frame_circles(&core);

    // Assert
    assert_eq!(circles, [(FLOAT_START, FLOAT_RADIUS)]);
}

#[test]
fn level_float_rises_in_water() {
    // Arrange
    let mut core = session(&FLOAT_IN_CUP);
    let start_y = frame_circles(&core)[0].0.y;

    // Act
    advance_steps(&mut core, 180).expect("the float level steps");

    // Assert
    let circles = frame_circles(&core);
    assert_eq!(circles.len(), 1);
    let rise = circles[0].0.y - start_y;
    assert!(rise >= 0.3, "float rose only {rise} m");
}

#[test]
fn float_with_a_zero_radius_is_rejected() {
    // Arrange
    let spec = &FLAT_FLOAT;

    // Act
    let result = SessionCore::create_puzzle_from_spec(spec, Placements::empty());

    // Assert
    assert!(matches!(result, Err(SessionError::SceneConstruction)));
}
