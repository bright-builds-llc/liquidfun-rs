use liquidfun::math::Vec2;
use liquidfun::particle::{ParticleFlags, ParticleGroupFlags};

use super::build_level;
use crate::puzzle::first_pour::{FIRST_POUR, REFERENCE_PLANK_POSE};
use crate::puzzle::goal::{Outcome, PuzzleStatus};
use crate::puzzle::spec::{LevelSpec, LiquidMaterial};
use crate::puzzle::tools::{
    GRID_CELLS_PER_METER, PLANK_HALF_THICKNESS, Placements, ToolAllowance, ToolKind,
};
use crate::puzzle::{LEVEL_IDS, LevelId, level_spec};
use crate::scene::{BuiltScene, PointerKind, SceneId, build_scene};
use crate::session::SessionError;

fn first_pour() -> &'static LevelSpec {
    level_spec(LevelId::FirstPour)
}

fn build_empty_first_pour() -> BuiltScene {
    build_level(first_pour(), &Placements::empty()).expect("empty first-pour builds")
}

fn reference_placements() -> Placements {
    let spec = first_pour();
    Placements::parse(&[ToolKind::Plank.code()], &REFERENCE_PLANK_POSE, spec)
        .expect("reference plank pose parses")
}

/// Axis-aligned rectangles `(min, max)` overlap with positive area.
fn rects_overlap(a: (Vec2, Vec2), b: (Vec2, Vec2)) -> bool {
    a.0.x < b.1.x && b.0.x < a.1.x && a.0.y < b.1.y && b.0.y < a.1.y
}

fn build_area_meters(spec: &LevelSpec) -> (Vec2, Vec2) {
    let [min_x, min_y, max_x, max_y] = spec.build_area.bounds().map(f32::from);
    (
        Vec2::new(min_x / GRID_CELLS_PER_METER, min_y / GRID_CELLS_PER_METER),
        Vec2::new(max_x / GRID_CELLS_PER_METER, max_y / GRID_CELLS_PER_METER),
    )
}

#[test]
fn level_spec_reports_its_own_id_and_title() {
    // Arrange
    let id = LevelId::FirstPour;

    // Act
    let spec = level_spec(id);

    // Assert
    assert_eq!(spec.id, id);
    assert_eq!(spec.title, "First Pour");
}

#[test]
fn liquid_materials_map_to_particle_and_group_flags() {
    // Arrange
    let materials = [
        LiquidMaterial::Water,
        LiquidMaterial::Viscous,
        LiquidMaterial::Powder,
        LiquidMaterial::Elastic,
    ];

    // Act
    let flags = materials.map(LiquidMaterial::flags);

    // Assert
    assert_eq!(
        flags,
        [
            (ParticleFlags::WATER, ParticleGroupFlags::empty()),
            (ParticleFlags::VISCOUS, ParticleGroupFlags::empty()),
            (ParticleFlags::POWDER, ParticleGroupFlags::empty()),
            (ParticleFlags::ELASTIC, ParticleGroupFlags::SOLID),
        ]
    );
}

#[test]
fn level_counts_are_consistent() {
    // Arrange
    let specs = LEVEL_IDS.map(level_spec);

    for spec in specs {
        // Act
        let counts = (spec.target_count, spec.full_count, spec.settle_steps);

        // Assert
        assert!(1 <= counts.0 && counts.0 <= counts.1, "{}", spec.title);
        assert_eq!(counts.2, 30, "{}", spec.title);
        assert!((600..=900).contains(&spec.step_limit), "{}", spec.title);
        assert!(spec.settle_steps <= spec.step_limit, "{}", spec.title);
    }
}

#[test]
fn levels_use_the_shared_particle_iterations() {
    // Arrange
    let specs = LEVEL_IDS.map(level_spec);

    // Act
    let iterations = specs.map(|spec| spec.particle_iterations);

    // Assert
    assert!(iterations.iter().all(|&count| count == 2));
}

#[test]
fn cup_interior_is_a_valid_rectangle() {
    // Arrange
    let specs = LEVEL_IDS.map(level_spec);

    for spec in specs {
        // Act
        let is_valid = spec.cup_interior.is_valid();

        // Assert
        assert!(is_valid, "{}", spec.title);
    }
}

#[test]
fn liquid_starts_outside_the_cup_and_the_build_area() {
    // Arrange
    let specs = LEVEL_IDS.map(level_spec);

    for spec in specs {
        let cup = (spec.cup_interior.min(), spec.cup_interior.max());
        let build_area = build_area_meters(spec);

        // Act
        let overlapping = spec
            .liquid_blocks
            .iter()
            .filter(|block| {
                let liquid = (block.min, block.max);
                rects_overlap(liquid, cup) || rects_overlap(liquid, build_area)
            })
            .count();

        // Assert
        assert_eq!(overlapping, 0, "{}", spec.title);
    }
}

#[test]
fn plank_is_at_least_three_particle_diameters_thick() {
    // Arrange
    let specs = LEVEL_IDS.map(level_spec);

    for spec in specs {
        // Act
        let three_diameters = 3.0 * 2.0 * spec.particle_radius;

        // Assert
        assert!(
            2.0 * PLANK_HALF_THICKNESS >= three_diameters,
            "{}",
            spec.title
        );
    }
}

#[test]
fn first_pour_allows_two_planks_with_par_one() {
    // Arrange
    let spec = first_pour();

    // Act
    let star_rules = (spec.tool_budget, spec.par_tools, spec.brim_count);

    // Assert
    assert_eq!(
        star_rules,
        (
            [ToolAllowance {
                kind: ToolKind::Plank,
                count: 2,
            }]
            .as_slice(),
            1,
            810
        )
    );
    assert!(spec.no_place.is_empty());
}

#[test]
fn specs_with_unordered_star_counts_are_rejected() {
    // Arrange
    const BRIM_AT_TARGET: LevelSpec = LevelSpec {
        brim_count: 540,
        ..FIRST_POUR
    };
    const BRIM_ABOVE_FULL: LevelSpec = LevelSpec {
        brim_count: 901,
        ..FIRST_POUR
    };
    const PAR_ABOVE_BUDGET: LevelSpec = LevelSpec {
        par_tools: 3,
        ..FIRST_POUR
    };
    let specs: [&'static LevelSpec; 3] = [&BRIM_AT_TARGET, &BRIM_ABOVE_FULL, &PAR_ABOVE_BUDGET];

    // Act
    let results = specs.map(|spec| build_level(spec, &Placements::empty()).err());

    // Assert
    assert_eq!(results, [Some(SessionError::SceneConstruction); 3]);
}

#[test]
fn empty_first_pour_builds_within_the_particle_budget() {
    // Arrange
    let spec = first_pour();

    // Act
    let built = build_empty_first_pour();

    // Assert
    let particle_count = built
        .world
        .particle_system_snapshot(built.particle_system)
        .expect("built particle system is live")
        .particle_count();
    assert!(
        (1500..=3000).contains(&particle_count),
        "particle count {particle_count}"
    );
    let full_count = usize::try_from(spec.full_count).expect("full count fits usize");
    assert!(
        particle_count >= full_count,
        "particle count {particle_count}"
    );
}

#[test]
fn empty_first_pour_starts_running_with_an_empty_cup() {
    // Arrange
    let spec = first_pour();

    // Act
    let built = build_empty_first_pour();

    // Assert
    assert_eq!(
        built.hooks.maybe_puzzle_status(),
        Some(PuzzleStatus {
            goal_count: 0,
            target_count: spec.target_count,
            full_count: spec.full_count,
            outcome: Outcome::Running,
            observed_step: 0,
            step_limit: spec.step_limit,
            peak_held_count: 0,
            brim_count: spec.brim_count,
            maybe_brim_step: None,
            maybe_decided_step: None,
        })
    );
}

#[test]
fn a_plank_adds_one_outline() {
    // Arrange
    let empty = build_empty_first_pour();
    let with_plank =
        build_level(first_pour(), &reference_placements()).expect("first-pour with a plank builds");

    // Act
    let empty_segments = empty
        .hooks
        .collect_segments(&empty.world)
        .expect("segments collect");
    let plank_segments = with_plank
        .hooks
        .collect_segments(&with_plank.world)
        .expect("segments collect");
    let empty_circles = empty.hooks.collect_circles(&empty.world);
    let plank_circles = with_plank.hooks.collect_circles(&with_plank.world);

    // Assert
    assert_eq!(plank_segments.len(), empty_segments.len() + 4);
    assert_eq!(empty_circles.map(|circles| circles.len()), Ok(0));
    assert_eq!(plank_circles.map(|circles| circles.len()), Ok(0));
}

#[test]
fn puzzle_hooks_reject_scene_input() {
    // Arrange
    let BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = build_empty_first_pour();

    // Act
    let control = hooks
        .apply_control(&mut world, particle_system, "gravity", "10")
        .map(|_effect| ());
    let action = hooks.apply_action(&mut world, particle_system, "drop");
    let pointer = hooks.apply_pointer(&mut world, particle_system, PointerKind::Down, 0.0, 0.0);

    // Assert
    assert_eq!(control, Err(SessionError::UnknownControl));
    assert_eq!(action, Err(SessionError::UnknownControl));
    assert_eq!(pointer, Err(SessionError::UnknownControl));
}

#[test]
fn after_step_hook_counts_its_own_steps() {
    // Arrange
    let BuiltScene {
        mut world,
        particle_system,
        mut hooks,
        ..
    } = build_empty_first_pour();

    // Act
    for _ in 0..2 {
        hooks
            .on_after_step(&mut world, particle_system, &[])
            .expect("after-step hook succeeds");
    }

    // Assert
    let status = hooks
        .maybe_puzzle_status()
        .expect("puzzle hooks report status");
    assert_eq!(status.observed_step, 2);
    assert_eq!(status.outcome, Outcome::Running);
}

#[test]
fn scene_hooks_report_no_puzzle_status() {
    // Arrange
    let built = build_scene(SceneId::DamBreak, &[]).expect("dam-break builds");

    // Act
    let maybe_status = built.hooks.maybe_puzzle_status();

    // Assert
    assert_eq!(maybe_status, None);
}
