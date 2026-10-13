//! Per-particle speeds for the optional render lane (Phase 43 D-10).
//! Read-only: computed from velocities the capture already reads.

use liquidfun::math::Vec2;

use super::SessionError;
use crate::frame::FrameData;

/// Returns the largest particle speed, rejecting any invalid velocity or a
/// non-finite result.
pub(super) fn max_particle_speed(velocities: &[Vec2]) -> Result<f32, SessionError> {
    let mut max_squared = 0.0_f32;
    for velocity in velocities {
        if !velocity.is_valid() {
            return Err(SessionError::FrameCaptureFailed);
        }
        let squared = velocity.length_squared();
        if squared > max_squared {
            max_squared = squared;
        }
    }

    let speed = max_squared.sqrt();
    if !speed.is_finite() {
        return Err(SessionError::FrameCaptureFailed);
    }

    Ok(speed)
}

/// Returns one speed per velocity in m/s, index-aligned with the input.
///
/// Uses the same `length_squared().sqrt()` math as [`max_particle_speed`];
/// `sqrt` is correctly rounded and monotonic, so the lane maximum equals the
/// frame's max speed bit for bit.
pub(super) fn particle_speeds(velocities: &[Vec2]) -> Result<Vec<f32>, SessionError> {
    let mut speeds = Vec::with_capacity(velocities.len());
    for velocity in velocities {
        if !velocity.is_valid() {
            return Err(SessionError::FrameCaptureFailed);
        }
        let speed = velocity.length_squared().sqrt();
        if !speed.is_finite() {
            return Err(SessionError::FrameCaptureFailed);
        }
        speeds.push(speed);
    }

    Ok(speeds)
}

/// Attaches a requested speed lane to `frame`; `None` leaves the frame
/// unchanged. A lane the frame rejects fails the capture.
pub(super) fn attach_particle_speeds(
    frame: &mut FrameData,
    maybe_speeds: Option<Vec<f32>>,
) -> Result<(), SessionError> {
    let Some(speeds) = maybe_speeds else {
        return Ok(());
    };
    frame
        .set_particle_speeds(speeds)
        .map_err(|_error| SessionError::FrameCaptureFailed)
}

#[cfg(test)]
mod tests;
