//! Validated owned frame data exported as copied JavaScript typed arrays.

use wasm_bindgen::prelude::*;

const MAX_PARTICLE_COUNT: usize = 10240;
const MAX_RIGID_SEGMENTS: usize = 16;
const MAX_RIGID_CIRCLES: usize = 8;
const PARTICLE_POSITION_STRIDE: usize = 2;
const PARTICLE_COLOR_STRIDE: usize = 4;
const PARTICLE_RADIUS_STRIDE: usize = 1;
const RIGID_SEGMENT_STRIDE: usize = 4;
const RIGID_CIRCLE_STRIDE: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FrameError {
    ParticleCountExceeded,
    RigidSegmentCountExceeded,
    RigidCircleCountExceeded,
    LaneLengthMismatch,
    NonFiniteValue,
    NonPositiveRadius,
    InvalidDiagnostic,
    ArithmeticOverflow,
}

/// Scalar simulation readouts copied with one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct FrameDiagnostics {
    pub(crate) max_speed: f32,
    pub(crate) stuck_candidate_count: usize,
    pub(crate) body_contact_count: usize,
}

impl FrameDiagnostics {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) const fn at_rest() -> Self {
        Self {
            max_speed: 0.0,
            stuck_candidate_count: 0,
            body_contact_count: 0,
        }
    }
}

pub(crate) struct FrameData {
    step_index: u32,
    particle_positions: Vec<f32>,
    particle_colors: Vec<u8>,
    particle_radii: Vec<f32>,
    rigid_segments: Vec<f32>,
    rigid_circles: Vec<f32>,
    circle_labels: Vec<String>,
    diagnostics: FrameDiagnostics,
}

impl FrameData {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        step_index: u32,
        particle_positions: Vec<f32>,
        particle_colors: Vec<u8>,
        particle_radii: Vec<f32>,
        rigid_segments: Vec<f32>,
        rigid_circles: Vec<f32>,
        diagnostics: FrameDiagnostics,
    ) -> Result<Self, FrameError> {
        let particle_count =
            checked_lane_count(particle_positions.len(), PARTICLE_POSITION_STRIDE)?;
        if particle_count > MAX_PARTICLE_COUNT {
            return Err(FrameError::ParticleCountExceeded);
        }
        require_lane_length(particle_colors.len(), particle_count, PARTICLE_COLOR_STRIDE)?;
        require_lane_length(particle_radii.len(), particle_count, PARTICLE_RADIUS_STRIDE)?;

        let rigid_segment_count = checked_lane_count(rigid_segments.len(), RIGID_SEGMENT_STRIDE)?;
        if rigid_segment_count > MAX_RIGID_SEGMENTS {
            return Err(FrameError::RigidSegmentCountExceeded);
        }

        let rigid_circle_count = checked_lane_count(rigid_circles.len(), RIGID_CIRCLE_STRIDE)?;
        if rigid_circle_count > MAX_RIGID_CIRCLES {
            return Err(FrameError::RigidCircleCountExceeded);
        }

        require_finite(&particle_positions)?;
        require_positive_finite(&particle_radii)?;
        require_finite(&rigid_segments)?;
        require_finite(&rigid_circles)?;
        require_positive_circle_radii(&rigid_circles)?;
        if !diagnostics.max_speed.is_finite() || diagnostics.max_speed < 0.0 {
            return Err(FrameError::InvalidDiagnostic);
        }

        Ok(Self {
            step_index,
            particle_positions,
            particle_colors,
            particle_radii,
            rigid_segments,
            rigid_circles,
            circle_labels: vec![String::new(); rigid_circle_count],
            diagnostics,
        })
    }

    pub(crate) fn set_circle_labels(&mut self, labels: Vec<String>) -> Result<(), FrameError> {
        let count = self.rigid_circles.len() / RIGID_CIRCLE_STRIDE;
        if labels.len() != count {
            return Err(FrameError::LaneLengthMismatch);
        }
        self.circle_labels = labels;
        Ok(())
    }

    pub(crate) const fn max_speed(&self) -> f32 {
        self.diagnostics.max_speed
    }

    pub(crate) const fn stuck_candidate_count(&self) -> usize {
        self.diagnostics.stuck_candidate_count
    }

    pub(crate) const fn body_contact_count(&self) -> usize {
        self.diagnostics.body_contact_count
    }
}

fn checked_lane_count(lane_length: usize, stride: usize) -> Result<usize, FrameError> {
    let count = lane_length / stride;
    require_lane_length(lane_length, count, stride)?;
    Ok(count)
}

fn require_lane_length(
    actual_length: usize,
    item_count: usize,
    stride: usize,
) -> Result<(), FrameError> {
    let expected_length = item_count
        .checked_mul(stride)
        .ok_or(FrameError::ArithmeticOverflow)?;
    if actual_length != expected_length {
        return Err(FrameError::LaneLengthMismatch);
    }

    Ok(())
}

fn require_finite(values: &[f32]) -> Result<(), FrameError> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(FrameError::NonFiniteValue);
    }

    Ok(())
}

fn require_positive_finite(radii: &[f32]) -> Result<(), FrameError> {
    require_finite(radii)?;
    if radii.iter().any(|radius| *radius <= 0.0) {
        return Err(FrameError::NonPositiveRadius);
    }

    Ok(())
}

fn require_positive_circle_radii(circles: &[f32]) -> Result<(), FrameError> {
    if circles
        .chunks_exact(RIGID_CIRCLE_STRIDE)
        .any(|circle| circle[2] <= 0.0)
    {
        return Err(FrameError::NonPositiveRadius);
    }

    Ok(())
}

/// One coherent validated frame copied out of the Rust simulation.
#[wasm_bindgen]
pub struct ProofFrame {
    data: FrameData,
}

impl From<FrameData> for ProofFrame {
    fn from(data: FrameData) -> Self {
        Self { data }
    }
}

#[wasm_bindgen]
impl ProofFrame {
    /// Returns the fixed-step index represented by this frame.
    #[must_use]
    #[wasm_bindgen(js_name = stepIndex)]
    pub fn step_index(&self) -> u32 {
        self.data.step_index
    }

    /// Returns the number of particles represented by this frame.
    #[must_use]
    #[wasm_bindgen(js_name = particleCount)]
    pub fn particle_count(&self) -> usize {
        self.data.particle_positions.len() / PARTICLE_POSITION_STRIDE
    }

    /// Returns the total number of bounded rigid shapes.
    #[must_use]
    #[wasm_bindgen(js_name = rigidShapeCount)]
    pub fn rigid_shape_count(&self) -> usize {
        self.data.rigid_segments.len() / RIGID_SEGMENT_STRIDE
            + self.data.rigid_circles.len() / RIGID_CIRCLE_STRIDE
    }

    /// Returns a fresh copied `x, y` lane for every particle.
    #[must_use]
    #[wasm_bindgen(js_name = particlePositions)]
    pub fn particle_positions(&self) -> Box<[f32]> {
        self.data.particle_positions.clone().into_boxed_slice()
    }

    /// Returns a fresh copied `r, g, b, a` lane for every particle.
    #[must_use]
    #[wasm_bindgen(js_name = particleColors)]
    pub fn particle_colors(&self) -> Box<[u8]> {
        self.data.particle_colors.clone().into_boxed_slice()
    }

    /// Returns a fresh copied radius lane for every particle.
    #[must_use]
    #[wasm_bindgen(js_name = particleRadii)]
    pub fn particle_radii(&self) -> Box<[f32]> {
        self.data.particle_radii.clone().into_boxed_slice()
    }

    /// Returns fresh copied `x1, y1, x2, y2` rigid segment data.
    #[must_use]
    #[wasm_bindgen(js_name = rigidSegments)]
    pub fn rigid_segments(&self) -> Box<[f32]> {
        self.data.rigid_segments.clone().into_boxed_slice()
    }

    /// Returns fresh copied `center_x, center_y, radius` rigid circle data.
    #[must_use]
    #[wasm_bindgen(js_name = rigidCircles)]
    pub fn rigid_circles(&self) -> Box<[f32]> {
        self.data.rigid_circles.clone().into_boxed_slice()
    }

    /// Returns the fastest particle speed in meters per second.
    #[must_use]
    #[wasm_bindgen(js_name = maxSpeed)]
    pub fn max_speed(&self) -> f32 {
        self.data.max_speed()
    }

    /// Returns the number of particles currently marked as stuck candidates.
    #[must_use]
    #[wasm_bindgen(js_name = stuckCandidateCount)]
    pub fn stuck_candidate_count(&self) -> usize {
        self.data.stuck_candidate_count()
    }

    /// Returns the number of particle–body contacts.
    #[must_use]
    #[wasm_bindgen(js_name = bodyContactCount)]
    pub fn body_contact_count(&self) -> usize {
        self.data.body_contact_count()
    }

    /// Returns one label per rigid circle, empty when the circle is unlabeled.
    #[must_use]
    #[wasm_bindgen(js_name = circleLabels)]
    pub fn circle_labels(&self) -> Vec<String> {
        self.data.circle_labels.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PARTICLE_COUNT: usize = 192;

    fn valid_frame_data() -> Result<FrameData, FrameError> {
        FrameData::new(
            7,
            vec![1.0; PARTICLE_COUNT * PARTICLE_POSITION_STRIDE],
            vec![255; PARTICLE_COUNT * PARTICLE_COLOR_STRIDE],
            vec![0.2; PARTICLE_COUNT * PARTICLE_RADIUS_STRIDE],
            vec![0.0; 3 * RIGID_SEGMENT_STRIDE],
            vec![0.0, 1.0, 0.75],
            FrameDiagnostics::at_rest(),
        )
    }

    #[test]
    fn valid_frame_reports_counts_and_copied_lanes() {
        // Arrange
        let data = valid_frame_data().expect("valid frame should pass validation");
        let frame = ProofFrame::from(data);

        // Act
        let mut first_positions = frame.particle_positions();
        first_positions[0] = 99.0;
        let second_positions = frame.particle_positions();

        // Assert
        assert_eq!(frame.step_index(), 7);
        assert_eq!(frame.particle_count(), PARTICLE_COUNT);
        assert_eq!(frame.rigid_shape_count(), 4);
        assert_eq!(second_positions[0].to_bits(), 1.0_f32.to_bits());
        assert_eq!(
            frame.particle_colors().len(),
            PARTICLE_COUNT * PARTICLE_COLOR_STRIDE
        );
        assert_eq!(frame.particle_radii().len(), PARTICLE_COUNT);
        assert_eq!(frame.rigid_segments().len(), 3 * RIGID_SEGMENT_STRIDE);
        assert_eq!(frame.rigid_circles().len(), RIGID_CIRCLE_STRIDE);
        assert_eq!(frame.max_speed().to_bits(), 0.0_f32.to_bits());
        assert_eq!(frame.stuck_candidate_count(), 0);
        assert_eq!(frame.body_contact_count(), 0);
    }

    #[test]
    fn rejects_particle_count_above_limit() {
        // Arrange
        let positions = vec![0.0; (MAX_PARTICLE_COUNT + 1) * PARTICLE_POSITION_STRIDE];

        // Act
        let result = FrameData::new(
            0,
            positions,
            vec![0; (MAX_PARTICLE_COUNT + 1) * PARTICLE_COLOR_STRIDE],
            vec![0.2; MAX_PARTICLE_COUNT + 1],
            Vec::new(),
            Vec::new(),
            FrameDiagnostics::at_rest(),
        );

        // Assert
        assert!(matches!(result, Err(FrameError::ParticleCountExceeded)));
    }

    #[test]
    fn rejects_rigid_segment_count_above_limit() {
        // Arrange
        let segments = vec![0.0; (MAX_RIGID_SEGMENTS + 1) * RIGID_SEGMENT_STRIDE];

        // Act
        let result = FrameData::new(
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            segments,
            Vec::new(),
            FrameDiagnostics::at_rest(),
        );

        // Assert
        assert!(matches!(result, Err(FrameError::RigidSegmentCountExceeded)));
    }

    #[test]
    fn rejects_rigid_circle_count_above_limit() {
        // Arrange
        let circles = vec![1.0; (MAX_RIGID_CIRCLES + 1) * RIGID_CIRCLE_STRIDE];

        // Act
        let result = FrameData::new(
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            circles,
            FrameDiagnostics::at_rest(),
        );

        // Assert
        assert!(matches!(result, Err(FrameError::RigidCircleCountExceeded)));
    }

    #[test]
    fn rejects_mismatched_particle_lane_lengths() {
        // Arrange
        let positions = vec![0.0; PARTICLE_POSITION_STRIDE];

        // Act
        let result = FrameData::new(
            0,
            positions,
            vec![0; PARTICLE_COLOR_STRIDE - 1],
            vec![0.2],
            Vec::new(),
            Vec::new(),
            FrameDiagnostics::at_rest(),
        );

        // Assert
        assert!(matches!(result, Err(FrameError::LaneLengthMismatch)));
    }

    #[test]
    fn rejects_non_finite_float_lane_values() {
        // Arrange
        let positions = vec![f32::NAN, 0.0];

        // Act
        let result = FrameData::new(
            0,
            positions,
            vec![0; PARTICLE_COLOR_STRIDE],
            vec![0.2],
            Vec::new(),
            Vec::new(),
            FrameDiagnostics::at_rest(),
        );

        // Assert
        assert!(matches!(result, Err(FrameError::NonFiniteValue)));
    }

    #[test]
    fn rejects_non_positive_particle_radius() {
        // Arrange
        let radius = 0.0;

        // Act
        let result = FrameData::new(
            0,
            vec![0.0; PARTICLE_POSITION_STRIDE],
            vec![0; PARTICLE_COLOR_STRIDE],
            vec![radius],
            Vec::new(),
            Vec::new(),
            FrameDiagnostics::at_rest(),
        );

        // Assert
        assert!(matches!(result, Err(FrameError::NonPositiveRadius)));
    }

    #[test]
    fn rejects_non_positive_rigid_circle_radius() {
        // Arrange
        let circles = vec![0.0, 0.0, -0.5];

        // Act
        let result = FrameData::new(
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            circles,
            FrameDiagnostics::at_rest(),
        );

        // Assert
        assert!(matches!(result, Err(FrameError::NonPositiveRadius)));
    }

    #[test]
    fn rejects_a_negative_max_speed() {
        // Arrange
        let diagnostics = FrameDiagnostics {
            max_speed: -1.0,
            stuck_candidate_count: 0,
            body_contact_count: 0,
        };

        // Act
        let result = FrameData::new(
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            diagnostics,
        );

        // Assert
        assert!(matches!(result, Err(FrameError::InvalidDiagnostic)));
    }
}
