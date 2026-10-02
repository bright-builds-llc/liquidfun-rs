const WIDTH_MIN_HUNDREDTHS: u16 = 16;
const WIDTH_MAX_HUNDREDTHS: u16 = 96;
const WIDTH_STEP_HUNDREDTHS: u16 = 4;
const SPEED_MAX_TENTHS: u16 = 40;
const AMPLITUDE_MAX_THOUSANDTHS: u16 = 320;
const AMPLITUDE_STEP_THOUSANDTHS: u16 = 8;

pub(super) fn parse_platform_width(value: &str) -> Option<f32> {
    parse_fixed_point(
        value,
        2,
        WIDTH_MIN_HUNDREDTHS,
        WIDTH_MAX_HUNDREDTHS,
        WIDTH_STEP_HUNDREDTHS,
    )
}

pub(super) fn parse_platform_speed(value: &str) -> Option<f32> {
    parse_fixed_point(value, 1, 0, SPEED_MAX_TENTHS, 1)
}

pub(super) fn parse_platform_amplitude(value: &str) -> Option<f32> {
    parse_fixed_point(
        value,
        3,
        0,
        AMPLITUDE_MAX_THOUSANDTHS,
        AMPLITUDE_STEP_THOUSANDTHS,
    )
}

pub(super) fn parse_degrees(value: &str, max_degrees: u16) -> Option<f32> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if value.len() > 1 && value.starts_with('0') {
        return None;
    }
    let degrees = value.parse::<u16>().ok()?;
    if degrees > max_degrees {
        return None;
    }
    Some(f32::from(degrees))
}

fn parse_fixed_point(
    value: &str,
    places: u8,
    min_units: u16,
    max_units: u16,
    step_units: u16,
) -> Option<f32> {
    let (whole, fraction) = value.split_once('.')?;
    if fraction.len() != usize::from(places) || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    if whole.is_empty() || !whole.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if whole.len() > 1 && whole.starts_with('0') {
        return None;
    }
    if step_units == 0 {
        return None;
    }

    let whole_value = whole.parse::<u16>().ok()?;
    let fraction_value = fraction.parse::<u16>().ok()?;
    let scale = 10_u16.checked_pow(u32::from(places))?;
    let units = whole_value
        .checked_mul(scale)?
        .checked_add(fraction_value)?;
    if units < min_units || units > max_units {
        return None;
    }
    if !(units - min_units).is_multiple_of(step_units) {
        return None;
    }

    Some(f32::from(units) / f32::from(scale))
}
