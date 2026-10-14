//! Optional per-particle speed lane on `ProofFrame` (Phase 43 D-10); undefined
//! in JS unless the capture asked for it. The lane is copied output only and
//! never feeds the simulation (D-11).

use wasm_bindgen::prelude::*;

use super::{FrameData, FrameError, PARTICLE_POSITION_STRIDE, ProofFrame};

impl FrameData {
    /// Attaches one finite, non-negative speed per particle. Rejects a lane
    /// whose length differs from the particle count, or any NaN, infinite or
    /// negative value.
    pub(crate) fn set_particle_speeds(&mut self, speeds: Vec<f32>) -> Result<(), FrameError> {
        let particle_count = self.particle_positions.len() / PARTICLE_POSITION_STRIDE;
        if speeds.len() != particle_count {
            return Err(FrameError::LaneLengthMismatch);
        }
        if speeds.iter().any(|speed| !speed.is_finite()) {
            return Err(FrameError::NonFiniteValue);
        }
        if speeds.iter().any(|speed| *speed < 0.0) {
            return Err(FrameError::NegativeSpeed);
        }
        self.maybe_particle_speeds = Some(speeds);
        Ok(())
    }
}

#[wasm_bindgen]
impl ProofFrame {
    /// Returns per-particle speeds in m/s, or undefined unless the capture
    /// asked for them.
    #[must_use]
    #[wasm_bindgen(js_name = particleSpeeds)]
    pub fn particle_speeds(&self) -> Option<Box<[f32]>> {
        self.data
            .maybe_particle_speeds
            .clone()
            .map(Vec::into_boxed_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::super::FrameDiagnostics;
    use super::*;

    const PARTICLE_COUNT: usize = 3;

    fn frame_data() -> FrameData {
        FrameData::new(
            0,
            vec![0.0; PARTICLE_COUNT * PARTICLE_POSITION_STRIDE],
            vec![255; PARTICLE_COUNT * 4],
            vec![0.05; PARTICLE_COUNT],
            Vec::new(),
            Vec::new(),
            FrameDiagnostics::at_rest(),
        )
        .expect("valid test frame")
    }

    #[test]
    fn set_particle_speeds_rejects_a_lane_of_the_wrong_length() {
        for length in [0, PARTICLE_COUNT - 1, PARTICLE_COUNT + 1] {
            // Arrange
            let mut frame = frame_data();

            // Act
            let result = frame.set_particle_speeds(vec![1.0; length]);

            // Assert
            assert_eq!(
                result,
                Err(FrameError::LaneLengthMismatch),
                "length {length}"
            );
        }
    }

    #[test]
    fn set_particle_speeds_rejects_non_finite_values() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            // Arrange
            let mut frame = frame_data();

            // Act
            let result = frame.set_particle_speeds(vec![0.0, bad, 1.0]);

            // Assert
            assert_eq!(result, Err(FrameError::NonFiniteValue), "value {bad}");
        }
    }

    #[test]
    fn set_particle_speeds_rejects_negative_values() {
        // Arrange
        let mut frame = frame_data();

        // Act
        let result = frame.set_particle_speeds(vec![0.0, -0.5, 1.0]);

        // Assert
        assert_eq!(result, Err(FrameError::NegativeSpeed));
    }

    #[test]
    fn set_particle_speeds_accepts_a_matching_finite_lane() {
        // Arrange
        let mut frame = frame_data();

        // Act
        let result = frame.set_particle_speeds(vec![0.0, 1.5, 3.0]);

        // Assert
        assert_eq!(result, Ok(()));
        assert_eq!(
            ProofFrame::from(frame).particle_speeds().as_deref(),
            Some([0.0, 1.5, 3.0].as_slice())
        );
    }
}
