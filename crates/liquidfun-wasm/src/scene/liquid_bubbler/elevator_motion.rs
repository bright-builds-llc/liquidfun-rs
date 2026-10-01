use liquidfun::math::{Rotation, Vec2};

use super::{
    CYCLE, DWELL, PLATE_HALF_HEIGHT, PLATE_HALF_WIDTH, PLATE_SLANT, PLATE_SPEED, RISE_SECONDS,
    STROKE, TOP_DWELL,
};

pub(super) fn plate_local_corners() -> [Vec2; 4] {
    let rotation = Rotation::from_angle(PLATE_SLANT);
    [
        Vec2::new(-PLATE_HALF_WIDTH, -PLATE_HALF_HEIGHT),
        Vec2::new(PLATE_HALF_WIDTH, -PLATE_HALF_HEIGHT),
        Vec2::new(PLATE_HALF_WIDTH, PLATE_HALF_HEIGHT),
        Vec2::new(-PLATE_HALF_WIDTH, PLATE_HALF_HEIGHT),
    ]
    .map(|corner| rotation.apply(corner))
}

pub(super) fn scheduled_plate_speed(elapsed: f32) -> f32 {
    let phase = elapsed.rem_euclid(CYCLE);
    let rise_end = DWELL + RISE_SECONDS;
    let top_end = rise_end + TOP_DWELL;
    if phase < DWELL || (phase >= rise_end && phase < top_end) {
        0.0
    } else if phase < rise_end {
        PLATE_SPEED
    } else {
        -PLATE_SPEED
    }
}

pub(super) fn scheduled_plate_offset(elapsed: f32) -> f32 {
    let phase = elapsed.rem_euclid(CYCLE);
    let rise_end = DWELL + RISE_SECONDS;
    let top_end = rise_end + TOP_DWELL;
    if phase < DWELL {
        0.0
    } else if phase < rise_end {
        (phase - DWELL) * PLATE_SPEED
    } else if phase < top_end {
        STROKE
    } else {
        (CYCLE - phase) * PLATE_SPEED
    }
}
