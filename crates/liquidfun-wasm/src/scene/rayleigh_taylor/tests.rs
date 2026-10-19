//! Debug-safe Rayleigh–Taylor tests: tank geometry, the polynomial seed, the
//! two-layer lattice and the `weight-ratio` parser.

use liquidfun::collision::PolygonShape;
use liquidfun::math::Vec2;

use super::geometry::{
    FILL_TOP_Y, FLOOR_TOP_Y, FRAME_MAX_X, FRAME_MAX_Y, FRAME_MIN_X, FRAME_MIN_Y, INTERFACE_MEAN_Y,
    LID_BOTTOM_Y, TANK_HALF_WIDTH, TANK_INNER_HEIGHT, WALL_THICKNESS, interface_y, lattice,
    lattice_stride, split_layers, tank_quads, tank_segments, wave,
};
use super::test_support::{
    DEPTH_LINE, FINGER_BIN_WIDTH, Layers, down_frame, finger_count, is_overturned,
};
use super::{DEFAULT_WEIGHT_RATIO_TOKEN, TUNING, WEIGHT_RATIO_TOKENS, WeightRatio};

const WAVE_TOLERANCE: f32 = 1e-6;
const FRAME_MARGIN: f32 = 0.1;
const MIN_TROUGH_DEPTH: f32 = 0.06;
const MIN_LATTICE_POINTS: usize = 1_500;
const MAX_LATTICE_POINTS: usize = 3_000;

/// `u` from -1 to 1 in steps of 0.01.
fn wave_grid() -> impl Iterator<Item = f32> {
    (-100_i16..=100).map(|step| f32::from(step) * 0.01)
}

/// `per_bin` heavy particles below the depth line in each listed finger bin.
fn clusters(bins: &[u16], per_bin: u16) -> Layers {
    let mut heavy = Vec::new();
    for bin in bins {
        let x = -TANK_HALF_WIDTH + FINGER_BIN_WIDTH * (f32::from(*bin) + 0.5);
        for index in 0..per_bin {
            heavy.push(Vec2::new(x, DEPTH_LINE - 0.1 - 0.05 * f32::from(index)));
        }
    }
    Layers {
        heavy,
        ..Layers::default()
    }
}

/// Two flat rows of particles at heights `heavy_y` and `light_y`.
fn two_rows(heavy_y: f32, light_y: f32) -> Layers {
    let row = |y: f32| -> Vec<Vec2> {
        (-10_i16..=10)
            .map(|step| Vec2::new(f32::from(step) * 0.1, y))
            .collect()
    };
    Layers {
        heavy: row(heavy_y),
        light: row(light_y),
        velocities: Vec::new(),
    }
}

fn point_bits(points: &[Vec2]) -> Vec<(u32, u32)> {
    points
        .iter()
        .map(|point| (point.x.to_bits(), point.y.to_bits()))
        .collect()
}

#[test]
fn wave_peaks_at_plus_and_minus_a_quarter_period() {
    // Act
    let values = [wave(0.25), wave(-0.25), wave(0.0)];

    // Assert
    assert_eq!(values[0].to_bits(), 1.0_f32.to_bits());
    assert_eq!(values[1].to_bits(), (-1.0_f32).to_bits());
    assert!(values[2].abs() <= 0.0, "{}", values[2]);
}

#[test]
fn wave_has_period_one() {
    for u in wave_grid() {
        // Act
        let shifted = wave(u + 1.0);

        // Assert
        assert!((shifted - wave(u)).abs() <= WAVE_TOLERANCE, "u = {u}");
    }
}

#[test]
fn wave_is_odd() {
    for u in wave_grid() {
        // Act
        let mirrored = wave(-u);

        // Assert
        assert!((mirrored + wave(u)).abs() <= WAVE_TOLERANCE, "u = {u}");
    }
}

#[test]
fn wave_stays_within_one() {
    for u in wave_grid() {
        // Act
        let value = wave(u);

        // Assert
        assert!(value.abs() <= 1.0, "u = {u}: {value}");
    }
}

#[test]
fn seeded_interface_dips_at_each_interior_trough() {
    // Arrange
    let troughs = f32::from(TUNING.modes[0].troughs);

    for k in 0..TUNING.modes[0].troughs {
        let x = -TANK_HALF_WIDTH + 2.0 * TANK_HALF_WIDTH * (f32::from(k) + 0.5) / troughs;

        // Act
        let height = interface_y(x, &TUNING.modes);

        // Assert
        assert!(
            INTERFACE_MEAN_Y - height >= MIN_TROUGH_DEPTH,
            "trough {k} at x = {x}: {height}"
        );
    }
}

#[test]
fn seeded_interface_crests_at_both_walls() {
    for x in [-TANK_HALF_WIDTH, TANK_HALF_WIDTH] {
        // Act
        let height = interface_y(x, &TUNING.modes);

        // Assert
        assert!(height > INTERFACE_MEAN_Y, "x = {x}: {height}");
    }
}

#[test]
fn tank_quads_are_convex_and_counter_clockwise() {
    for quad in tank_quads() {
        // Act
        let turns = (0..4).map(|index| {
            let a = quad[index];
            let b = quad[(index + 1) % 4];
            let c = quad[(index + 2) % 4];
            (b - a).cross(c - b)
        });

        // Assert
        assert!(turns.into_iter().all(|turn| turn > 0.0), "{quad:?}");
        assert!(PolygonShape::new(&quad).is_ok(), "{quad:?}");
    }
}

#[test]
fn walls_and_lid_are_at_least_three_diameters_thick() {
    // Arrange
    let quads = tank_quads();

    for quad in &quads[1..] {
        // Act
        let thickness = (quad[2].x - quad[0].x).min(quad[2].y - quad[0].y);

        // Assert
        assert!(thickness >= WALL_THICKNESS - 1e-6, "{quad:?}");
    }
}

#[test]
fn tank_is_as_tall_as_wide_or_a_little_taller() {
    // Act
    let aspect = TANK_INNER_HEIGHT / (2.0 * TANK_HALF_WIDTH);

    // Assert
    assert!((1.0..=1.3).contains(&aspect), "{aspect}");
}

#[test]
fn walls_overlap_the_floor_and_the_lid() {
    // Arrange
    let [floor, left, right, lid] = tank_quads();

    for wall in [left, right] {
        // Act
        let reaches_floor = wall[0].y <= floor[0].y && wall[2].y >= lid[2].y;
        let spans_floor = wall[0].x >= floor[0].x && wall[2].x <= floor[2].x;

        // Assert
        assert!(reaches_floor && spans_floor, "{wall:?}");
    }
}

#[test]
fn every_tank_vertex_lies_inside_the_frame_with_a_margin() {
    for vertex in tank_quads().iter().flatten() {
        // Act
        let inside = vertex.x >= FRAME_MIN_X + FRAME_MARGIN
            && vertex.x <= FRAME_MAX_X - FRAME_MARGIN
            && vertex.y >= FRAME_MIN_Y + FRAME_MARGIN
            && vertex.y <= FRAME_MAX_Y - FRAME_MARGIN;

        // Assert
        assert!(inside, "{vertex:?}");
    }
}

#[test]
fn tank_segments_form_four_closed_loops() {
    // Act
    let segments = tank_segments();

    // Assert
    assert_eq!(segments.len(), 16);
    for loop_segments in segments.chunks(4) {
        for index in 0..4 {
            let end = loop_segments[index].end;
            let next_start = loop_segments[(index + 1) % 4].start;
            assert_eq!(
                (end.x.to_bits(), end.y.to_bits()),
                (next_start.x.to_bits(), next_start.y.to_bits())
            );
        }
    }
}

#[test]
fn lattice_size_fits_the_budget() {
    // Act
    let points = lattice(TUNING.radius);

    // Assert
    assert!(
        (MIN_LATTICE_POINTS..=MAX_LATTICE_POINTS).contains(&points.len()),
        "{}",
        points.len()
    );
}

#[test]
fn lattice_stays_inside_the_tank_below_the_fill_line() {
    // Arrange
    let radius = TUNING.radius;

    for point in lattice(radius) {
        // Act
        let inside = point.x.abs() <= TANK_HALF_WIDTH - radius + 1e-6
            && point.y >= FLOOR_TOP_Y + radius - 1e-6
            && point.y <= FILL_TOP_Y
            && point.y <= LID_BOTTOM_Y - radius;

        // Assert
        assert!(inside, "{point:?}");
    }
}

#[test]
fn lattice_points_keep_one_stride_apart() {
    // Arrange
    let points = lattice(TUNING.radius);
    let min_distance = 0.99 * lattice_stride(TUNING.radius);

    // Act
    let maybe_close = points.iter().enumerate().find_map(|(index, a)| {
        points[index + 1..]
            .iter()
            .find(|b| (**b - *a).length() < min_distance)
            .map(|b| (*a, *b))
    });

    // Assert
    assert!(maybe_close.is_none(), "{maybe_close:?}");
}

#[test]
fn layer_split_puts_heavy_points_at_or_above_the_interface() {
    // Act
    let (light, heavy) = split_layers(lattice(TUNING.radius), &TUNING.modes);

    // Assert
    assert!(
        heavy
            .iter()
            .all(|point| point.y >= interface_y(point.x, &TUNING.modes))
    );
    assert!(
        light
            .iter()
            .all(|point| point.y < interface_y(point.x, &TUNING.modes))
    );
}

#[test]
fn layer_split_keeps_every_point_in_about_equal_layers() {
    // Arrange
    let total = lattice(TUNING.radius).len();

    // Act
    let (light, heavy) = split_layers(lattice(TUNING.radius), &TUNING.modes);

    // Assert
    assert_eq!(light.len() + heavy.len(), total);
    for layer in [light.len(), heavy.len()] {
        assert!(
            layer * 10 >= total * 4 && layer * 10 <= total * 6,
            "{layer} of {total}"
        );
    }
}

#[test]
fn lattice_is_bit_identical_across_builds() {
    // Act
    let first = split_layers(lattice(TUNING.radius), &TUNING.modes);
    let second = split_layers(lattice(TUNING.radius), &TUNING.modes);

    // Assert
    assert_eq!(point_bits(&first.0), point_bits(&second.0));
    assert_eq!(point_bits(&first.1), point_bits(&second.1));
}

#[test]
fn weight_ratio_accepts_each_token_with_its_exact_extra_weight() {
    // Arrange
    let expected = [0.2_f32, 0.5, 1.0, 2.0];

    for (token, extra) in WEIGHT_RATIO_TOKENS.into_iter().zip(expected) {
        // Act
        let maybe_ratio = WeightRatio::parse(token);

        // Assert
        let ratio = maybe_ratio.expect("listed token parses");
        assert_eq!(ratio.extra_weight().to_bits(), extra.to_bits(), "{token}");
    }
}

#[test]
fn weight_ratio_rejects_malformed_tokens() {
    for token in ["", "2.0", "4", "1", "1,2", " 2", "2 ", "-2", "+2", "abc"] {
        // Act
        let maybe_ratio = WeightRatio::parse(token);

        // Assert
        assert_eq!(maybe_ratio, None, "{token:?}");
    }
}

#[test]
fn weight_ratio_default_is_two_and_matches_the_default_token() {
    // Act
    let maybe_default = WeightRatio::parse(DEFAULT_WEIGHT_RATIO_TOKEN);

    // Assert
    assert_eq!(WeightRatio::DEFAULT, WeightRatio::Two);
    assert_eq!(maybe_default, Some(WeightRatio::DEFAULT));
}

#[test]
fn three_separate_four_particle_clusters_count_three_fingers() {
    // Arrange
    let layers = clusters(&[3, 7, 11], 4);

    // Act
    let fingers = finger_count(&layers, down_frame());

    // Assert
    assert_eq!(fingers, 3);
}

#[test]
fn three_particle_clusters_count_no_fingers() {
    // Arrange
    let layers = clusters(&[3, 7, 11], 3);

    // Act
    let fingers = finger_count(&layers, down_frame());

    // Assert
    assert_eq!(fingers, 0);
}

#[test]
fn a_cluster_in_the_wall_bin_counts_no_finger() {
    // Arrange
    let layers = clusters(&[0], 4);

    // Act
    let fingers = finger_count(&layers, down_frame());

    // Assert
    assert_eq!(fingers, 0);
}

#[test]
fn clusters_in_adjacent_bins_count_one_finger() {
    // Arrange
    let layers = clusters(&[5, 6], 4);

    // Act
    let fingers = finger_count(&layers, down_frame());

    // Assert
    assert_eq!(fingers, 1);
}

#[test]
fn heavy_above_light_is_not_overturned() {
    // Arrange
    let layers = two_rows(2.0, 1.0);

    // Act
    let overturned = is_overturned(&layers, down_frame());

    // Assert
    assert!(!overturned);
}

#[test]
fn heavy_below_light_is_overturned() {
    // Arrange
    let layers = two_rows(1.0, 2.0);

    // Act
    let overturned = is_overturned(&layers, down_frame());

    // Assert
    assert!(overturned);
}
