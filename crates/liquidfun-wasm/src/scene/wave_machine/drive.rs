//! Absolute wave-machine motor command.
//!
//! Phase accumulates with `phase += speed * dt`. The feedforward speed is the
//! derivative of `peak(tilt) * sin(phase)`. At 1× and 9° that derivative is the
//! pinned `0.05 * cos(t) * π` motor. A tilt edit rebases phase through the
//! current angle when the new peak still contains it. Otherwise a bounded pull
//! eases the tank onto `peak * sin(phase)`.

use std::f32::consts::TAU;

/// Pull time constant. A third of a second eases a tilt cut back without a snap.
const CORRECTION_HORIZON_SECONDS: f32 = 0.35;
/// Error below this, in radians, is already on the waveform.
const CORRECTION_RELEASE: f32 = 1.0e-3;
/// Peaks below this are treated as level.
const LEVEL_PEAK: f32 = 1.0e-3;

/// Rocking command whose speed and tilt are absolute about level.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct WaveDrive {
    phase: f32,
    speed_multiplier: f32,
    tilt_degrees: f32,
    correct_toward_target: bool,
}

impl WaveDrive {
    /// Pinned Wave Machine: 1×, 9°, phase zero, no pull.
    #[must_use]
    pub(super) fn pinned() -> Self {
        Self {
            phase: 0.0,
            speed_multiplier: super::DEFAULT_WAVE_SPEED,
            tilt_degrees: super::DEFAULT_TILT_DEGREES,
            correct_toward_target: false,
        }
    }

    /// Advances phase by `speed * dt`. A stopped tank keeps its phase.
    pub(super) fn advance(&mut self, dt: f32) {
        self.phase = wrap_phase(self.phase + self.speed_multiplier * dt);
    }

    /// Sets the rocking frequency and keeps the current phase.
    pub(super) fn set_speed(&mut self, speed_multiplier: f32) {
        self.speed_multiplier = speed_multiplier;
    }

    /// Sets the absolute peak tilt in degrees.
    ///
    /// When `angle` still fits the new peak, phase is rebased so the waveform
    /// passes through that angle and keeps its current direction. When it does
    /// not, the next [`Self::motor_speed`] pulls toward the absolute waveform.
    pub(super) fn set_tilt(&mut self, tilt_degrees: f32, angle: f32) {
        self.tilt_degrees = tilt_degrees;
        let peak = peak_angle(tilt_degrees);
        if peak > LEVEL_PEAK && angle.abs() <= peak + LEVEL_PEAK {
            self.phase = phase_on_waveform(self.phase, angle, peak);
            self.correct_toward_target = false;
            return;
        }

        let target = target_angle(self.phase, tilt_degrees);
        self.correct_toward_target = (angle - target).abs() > CORRECTION_RELEASE;
    }

    /// Feedforward derivative, plus a bounded pull while a tilt edit is settling.
    #[must_use]
    pub(super) fn motor_speed(&mut self, angle: f32) -> f32 {
        let feedforward = feedforward_speed(self.phase, self.speed_multiplier, self.tilt_degrees);
        let target = target_angle(self.phase, self.tilt_degrees);
        if self.correct_toward_target && (angle - target).abs() <= CORRECTION_RELEASE {
            self.correct_toward_target = false;
        }
        if !self.correct_toward_target {
            return feedforward;
        }

        feedforward + correction_speed(target - angle)
    }
}

/// Open-loop speed for a constant speed and tilt, matching `speed * time` phase.
#[cfg(test)]
#[must_use]
pub(super) fn open_loop_speed(time: f32, speed_multiplier: f32, tilt_degrees: f32) -> f32 {
    let phase = speed_multiplier * time;
    feedforward_speed(phase, speed_multiplier, tilt_degrees)
}

fn peak_angle(tilt_degrees: f32) -> f32 {
    super::MOTOR_SPEED_SCALE * TAU * 0.5 * (tilt_degrees / super::DEFAULT_TILT_DEGREES)
}

fn target_angle(phase: f32, tilt_degrees: f32) -> f32 {
    peak_angle(tilt_degrees) * phase.sin()
}

fn feedforward_speed(phase: f32, speed_multiplier: f32, tilt_degrees: f32) -> f32 {
    peak_angle(tilt_degrees) * speed_multiplier * phase.cos()
}

fn phase_on_waveform(phase: f32, angle: f32, peak: f32) -> f32 {
    let ratio = (angle / peak).clamp(-1.0, 1.0);
    let principal = ratio.asin();
    let rebased = if phase.cos() >= 0.0 {
        principal
    } else {
        TAU / 2.0 - principal
    };
    wrap_phase(rebased)
}

fn wrap_phase(phase: f32) -> f32 {
    phase.rem_euclid(TAU)
}

fn correction_speed(angle_error: f32) -> f32 {
    let limit = peak_angle(f32::from(super::MAX_TILT_DEGREES))
        * (f32::from(super::MAX_WAVE_SPEED_TENTHS) / 10.0);
    (angle_error / CORRECTION_HORIZON_SECONDS).clamp(-limit, limit)
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{PI, TAU};

    use super::{CORRECTION_HORIZON_SECONDS, WaveDrive, open_loop_speed, peak_angle};

    const DT: f32 = 1.0 / 60.0;

    fn assert_near(actual: f32, expected: f32, tolerance: f32) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "expected {expected} ± {tolerance}, got {actual}"
        );
    }

    #[test]
    fn pinned_feedforward_matches_the_original_motor() {
        // Arrange
        let mut drive = WaveDrive::pinned();

        // Act
        let speed = drive.motor_speed(0.0);

        // Assert
        assert_near(speed, 0.05 * PI, 1.0e-5);
        assert!(!drive.correct_toward_target);
    }

    #[test]
    fn advancing_at_constant_speed_matches_sim_time() {
        // Arrange
        let mut drive = WaveDrive::pinned();
        let advances = 7_u16;

        // Act
        for _ in 0..advances {
            drive.advance(DT);
        }
        let speed = drive.motor_speed(0.0);
        let time = f32::from(advances) * DT;

        // Assert
        assert_near(speed, open_loop_speed(time, 1.0, 9.0), 1.0e-5);
    }

    #[test]
    fn speed_edit_keeps_the_current_phase() {
        // Arrange
        let mut drive = WaveDrive::pinned();
        let advances = 40_u16;
        for _ in 0..advances {
            drive.advance(DT);
        }
        let phase = f32::from(advances) * DT;

        // Act
        drive.set_speed(4.0);
        let speed = drive.motor_speed(0.0);

        // Assert
        let kept = peak_angle(9.0) * 4.0 * phase.cos();
        let jumped = peak_angle(9.0) * 4.0 * (4.0 * phase).cos();
        assert_near(speed, kept, 1.0e-5);
        assert!(
            (speed - jumped).abs() > 0.05,
            "a speed edit must not retarget phase to speed * time (got {speed}, jumped {jumped})"
        );
    }

    #[test]
    fn zero_speed_freezes_phase_and_stops_the_motor() {
        // Arrange
        let mut drive = WaveDrive::pinned();
        for _ in 0..12 {
            drive.advance(DT);
        }
        let phase = drive.phase;

        // Act
        drive.set_speed(0.0);
        let stopped = drive.motor_speed(0.0);
        drive.advance(DT);
        let after = drive.motor_speed(0.0);

        // Assert
        assert_near(stopped, 0.0, 1.0e-6);
        assert_near(after, 0.0, 1.0e-6);
        assert_near(drive.phase, phase, 1.0e-6);
    }

    #[test]
    fn tilt_increase_rebases_through_the_current_angle() {
        // Arrange — falling side of the pinned wave.
        let mut drive = WaveDrive::pinned();
        let mut angle = 0.0_f32;
        for _ in 0..150 {
            drive.advance(DT);
            angle += drive.motor_speed(angle) * DT;
        }
        let direction = drive.phase.cos();

        // Act
        drive.set_tilt(18.0, angle);
        let target = peak_angle(18.0) * drive.phase.sin();

        // Assert
        assert!(direction < 0.0, "setup should be on the falling side");
        assert!(
            drive.phase.cos() < 0.0,
            "rebase must keep the falling direction"
        );
        assert_near(target, angle, 1.0e-4);
        assert!(!drive.correct_toward_target);
    }

    #[test]
    fn tilt_cut_pulls_toward_the_absolute_waveform() {
        // Arrange — sit past the crest, then ask for a smaller peak.
        let mut drive = WaveDrive::pinned();
        for _ in 0..94 {
            drive.advance(DT);
        }
        let angle = 0.30_f32;

        // Act
        drive.set_tilt(9.0, angle);
        let speed = drive.motor_speed(angle);
        let target = peak_angle(9.0) * drive.phase.sin();
        let feedforward = peak_angle(9.0) * drive.speed_multiplier * drive.phase.cos();

        // Assert
        assert!(drive.correct_toward_target);
        assert!(
            speed < 0.0,
            "a tank leaned past the new peak must be pulled back (got {speed})"
        );
        assert_near(
            speed,
            feedforward + (target - angle) / CORRECTION_HORIZON_SECONDS,
            1.0e-4,
        );
    }

    #[test]
    fn tilt_zero_pulls_a_leaned_tank_back_to_level() {
        // Arrange
        let mut drive = WaveDrive::pinned();
        let angle = 0.20_f32;

        // Act
        drive.set_tilt(0.0, angle);
        let speed = drive.motor_speed(angle);

        // Assert
        assert!(drive.correct_toward_target);
        assert_near(speed, -angle / CORRECTION_HORIZON_SECONDS, 1.0e-4);
    }

    #[test]
    fn integrated_tilt_increase_stays_centered_on_level() {
        // Arrange
        let mut drive = WaveDrive::pinned();
        let mut angle = 0.0_f32;
        for _ in 0..40 {
            drive.advance(DT);
            angle += drive.motor_speed(angle) * DT;
        }

        // Act
        drive.set_tilt(18.0, angle);
        let (min_angle, max_angle) = integrate_turn(&mut drive, &mut angle);

        // Assert
        let center = 0.5 * (min_angle + max_angle);
        let peak = peak_angle(18.0);
        assert_near(center, 0.0, 0.01);
        assert_near(max_angle, peak, 0.02);
        assert_near(min_angle, -peak, 0.02);
    }

    #[test]
    fn integrated_tilt_cut_returns_to_the_new_peaks() {
        // Arrange
        let mut drive = WaveDrive::pinned();
        let mut angle = 0.0_f32;
        for _ in 0..94 {
            drive.advance(DT);
            angle += drive.motor_speed(angle) * DT;
        }
        angle = 0.30;

        // Act
        drive.set_tilt(9.0, angle);
        for _ in 0..200 {
            drive.advance(DT);
            angle += drive.motor_speed(angle) * DT;
        }
        let (min_angle, max_angle) = integrate_turn(&mut drive, &mut angle);

        // Assert
        let center = 0.5 * (min_angle + max_angle);
        let peak = peak_angle(9.0);
        assert_near(center, 0.0, 0.01);
        assert_near(max_angle, peak, 0.02);
        assert_near(min_angle, -peak, 0.02);
        assert!(!drive.correct_toward_target);
    }

    #[test]
    fn long_run_keeps_phase_inside_one_turn() {
        // Arrange
        let mut drive = WaveDrive::pinned();
        drive.set_speed(10.0);

        // Act
        for _ in 0..1_000 {
            drive.advance(DT);
        }

        // Assert
        assert!(
            (0.0..TAU).contains(&drive.phase),
            "phase should stay wrapped, got {}",
            drive.phase
        );
    }

    fn integrate_turn(drive: &mut WaveDrive, angle: &mut f32) -> (f32, f32) {
        let mut min_angle = *angle;
        let mut max_angle = *angle;
        for _ in 0..377 {
            drive.advance(DT);
            let speed = drive.motor_speed(*angle);
            *angle += speed * DT;
            min_angle = min_angle.min(*angle);
            max_angle = max_angle.max(*angle);
        }
        (min_angle, max_angle)
    }
}
