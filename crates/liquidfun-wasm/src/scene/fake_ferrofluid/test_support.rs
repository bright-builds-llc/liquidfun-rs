//! Test-only helpers shared by the ferrofluid unit and spike tests: the fixed
//! step and the binned surface-peak metric (Research "Surface-Peak Metric").

use liquidfun::StepConfiguration;
use liquidfun::math::Vec2;

use super::geometry::{DISH_INNER_HALF_WIDTH, PARTICLE_DIAMETER};
use crate::scene::{SceneId, particle_iterations};

/// Least topographic prominence of a counted surface peak: about two diameters.
pub(super) const PEAK_PROMINENCE: f32 = 0.15;
/// Largest smoothed surface range that still reads as flat: 1.5 diameters.
pub(super) const FLAT_RANGE: f32 = 0.12;
/// Profile bins stay two diameters clear of each inner wall face.
const BIN_START: f32 = -DISH_INNER_HALF_WIDTH + 2.0 * PARTICLE_DIAMETER;
/// One bin per particle diameter across the binned span.
const BIN_COUNT: u16 = 71;

/// The session's fixed step: 1/60 s, 8 velocity and 3 position iterations,
/// and the scene's particle iterations.
pub(super) fn step_configuration() -> StepConfiguration {
    StepConfiguration::new(1.0 / 60.0, 8, 3)
        .expect("valid step")
        .with_particle_iterations(particle_iterations(SceneId::FakeFerrofluid))
        .expect("particle iterations")
}

/// A 3-bin-median smoothed max-height profile across the dish (gravity straight down).
#[derive(Debug, Clone, PartialEq)]
pub(super) struct SurfaceProfile {
    tops: Vec<f32>,
}

/// Bins particle centers one diameter wide, keeps each bin's highest center
/// (an empty bin reads as the floor, 0), then smooths with a 3-bin median.
pub(super) fn surface_profile(positions: &[Vec2]) -> SurfaceProfile {
    let bins = usize::from(BIN_COUNT);
    let mut raw = vec![0.0_f32; bins];
    for position in positions {
        let Some(index) = bin_index(position.x) else {
            continue;
        };
        raw[index] = raw[index].max(position.y);
    }
    let tops = (0..bins)
        .map(|index| {
            let left = raw[index.saturating_sub(1)];
            let right = raw[(index + 1).min(bins - 1)];
            median3(left, raw[index], right)
        })
        .collect();
    SurfaceProfile { tops }
}

/// The bin holding `x`, or `None` outside the binned span.
fn bin_index(x: f32) -> Option<usize> {
    let offset = ((x - BIN_START) / PARTICLE_DIAMETER).floor();
    if !(0.0..f32::from(BIN_COUNT)).contains(&offset) {
        return None;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "offset is a whole number inside 0..BIN_COUNT"
    )]
    let index = offset as usize;
    Some(index)
}

fn median3(a: f32, b: f32, c: f32) -> f32 {
    a.max(b).min(a.min(b).max(c))
}

impl SurfaceProfile {
    /// Bins that are local maxima (`top[i-1] < top[i] >= top[i+1]`, edges
    /// allowed) with a prominence of at least `PEAK_PROMINENCE`.
    fn peak_bins(&self) -> Vec<usize> {
        let tops = &self.tops;
        (0..tops.len())
            .filter(|&index| {
                let rises = index == 0 || tops[index - 1] < tops[index];
                let holds = index + 1 == tops.len() || tops[index] >= tops[index + 1];
                rises && holds && self.prominence(index) >= PEAK_PROMINENCE
            })
            .collect()
    }

    /// Topographic prominence: the peak height above the higher of its two
    /// bases, each the lowest bin before a higher bin or the profile edge.
    fn prominence(&self, index: usize) -> f32 {
        let tops = &self.tops;
        let top = tops[index];
        let left_min = tops[..=index]
            .iter()
            .rev()
            .take_while(|height| **height <= top)
            .fold(top, |low, height| low.min(*height));
        let right_min = tops[index..]
            .iter()
            .take_while(|height| **height <= top)
            .fold(top, |low, height| low.min(*height));
        top - left_min.max(right_min)
    }

    /// Number of separate surface peaks.
    pub(super) fn peak_count(&self) -> usize {
        self.peak_bins().len()
    }

    /// Mean x of the peak bin centers, or `None` without peaks.
    pub(super) fn peak_centroid_x(&self) -> Option<f32> {
        let peaks = self.peak_bins();
        if peaks.is_empty() {
            return None;
        }
        let sum: f32 = peaks.iter().map(|&index| bin_center_x(index)).sum();
        Some(sum / count_as_f32(peaks.len()))
    }

    /// No peaks and a smoothed range of at most `FLAT_RANGE`.
    pub(super) fn is_flat(&self) -> bool {
        let highest = self.tops.iter().copied().fold(f32::MIN, f32::max);
        let lowest = self.tops.iter().copied().fold(f32::MAX, f32::min);
        self.peak_count() == 0 && highest - lowest <= FLAT_RANGE
    }
}

fn bin_center_x(index: usize) -> f32 {
    BIN_START + (count_as_f32(index) + 0.5) * PARTICLE_DIAMETER
}

/// A small count as `f32`; test counts stay far below `u16::MAX`.
pub(super) fn count_as_f32(count: usize) -> f32 {
    f32::from(u16::try_from(count).expect("test counts fit in u16"))
}
