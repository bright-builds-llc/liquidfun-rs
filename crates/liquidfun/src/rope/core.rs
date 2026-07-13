use crate::math::Vec2;
use crate::math::settings::{PI, TAU};

use super::{RopeDef, RopeError, RopeIterations};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RopeCore {
    positions: Vec<Vec2>,
    previous_positions: Vec<Vec2>,
    velocities: Vec<Vec2>,
    inverse_masses: Vec<f32>,
    rest_lengths: Vec<f32>,
    rest_angles: Vec<f32>,
    gravity: Vec2,
    damping: f32,
    stretching_stiffness: f32,
    bending_stiffness: f32,
}

impl RopeCore {
    pub(super) fn new(definition: RopeDef) -> Result<Self, RopeError> {
        let inverse_masses = definition
            .masses
            .iter()
            .copied()
            .enumerate()
            .map(|(index, mass)| inverse_mass(index, mass))
            .collect::<Result<Vec<_>, _>>()?;
        let rest_lengths = rest_lengths(&definition.vertices)?;
        let rest_angles = rest_angles(&definition.vertices)?;
        let count = definition.vertices.len();

        Ok(Self {
            previous_positions: definition.vertices.clone(),
            positions: definition.vertices,
            velocities: vec![Vec2::ZERO; count],
            inverse_masses,
            rest_lengths,
            rest_angles,
            gravity: definition.gravity,
            damping: definition.damping,
            stretching_stiffness: definition.stretching_stiffness,
            bending_stiffness: definition.bending_stiffness,
        })
    }

    pub(super) fn vertices(&self) -> &[Vec2] {
        &self.positions
    }

    pub(super) fn set_angle(&mut self, angle: f32) {
        self.rest_angles.fill(angle);
    }

    pub(super) fn step(
        &mut self,
        time_step: f32,
        iterations: RopeIterations,
    ) -> Result<(), RopeError> {
        let damping_exponent = -time_step * self.damping;
        if !damping_exponent.is_finite() {
            return Err(RopeError::NonFiniteDerivedState { index: 0 });
        }
        let damping = damping_exponent.exp();
        if !damping.is_finite() {
            return Err(RopeError::NonFiniteDerivedState { index: 0 });
        }

        self.integrate(time_step, damping)?;
        for _iteration in 0..iterations.get() {
            self.solve_stretch()?;
            self.solve_bend()?;
            self.solve_stretch()?;
        }
        self.reconstruct_velocities(time_step)
    }

    fn integrate(&mut self, time_step: f32, damping: f32) -> Result<(), RopeError> {
        for index in 0..self.positions.len() {
            self.previous_positions[index] = self.positions[index];
            if self.inverse_masses[index] > 0.0 {
                let gravity_delta = time_step * self.gravity;
                if !gravity_delta.is_valid() {
                    return Err(RopeError::NonFiniteDerivedState { index });
                }
                self.velocities[index] += gravity_delta;
            }

            self.velocities[index] *= damping;
            let position_delta = time_step * self.velocities[index];
            if !self.velocities[index].is_valid() || !position_delta.is_valid() {
                return Err(RopeError::NonFiniteDerivedState { index });
            }
            self.positions[index] += position_delta;
            if !self.positions[index].is_valid() {
                return Err(RopeError::NonFiniteDerivedState { index });
            }
        }

        Ok(())
    }

    fn solve_stretch(&mut self) -> Result<(), RopeError> {
        for index in 0..self.rest_lengths.len() {
            let mut first = self.positions[index];
            let mut second = self.positions[index + 1];
            let mut direction = second - first;
            if !direction.is_valid() {
                return Err(RopeError::NonFiniteDerivedState { index });
            }
            let length = direction.normalize();
            if !length.is_finite() {
                return Err(RopeError::NonFiniteDerivedState { index });
            }

            let first_inverse_mass = self.inverse_masses[index];
            let second_inverse_mass = self.inverse_masses[index + 1];
            let inverse_mass_sum = first_inverse_mass + second_inverse_mass;
            if inverse_mass_sum == 0.0 {
                continue;
            }
            if !inverse_mass_sum.is_finite() {
                return Err(RopeError::NonFiniteDerivedState { index });
            }

            let first_share = first_inverse_mass / inverse_mass_sum;
            let second_share = second_inverse_mass / inverse_mass_sum;
            let length_error = self.rest_lengths[index] - length;
            let first_correction =
                self.stretching_stiffness * first_share * length_error * direction;
            let second_correction =
                self.stretching_stiffness * second_share * length_error * direction;
            if !first_correction.is_valid() || !second_correction.is_valid() {
                return Err(RopeError::NonFiniteDerivedState { index });
            }

            first -= first_correction;
            second += second_correction;
            if !first.is_valid() || !second.is_valid() {
                return Err(RopeError::NonFiniteDerivedState { index });
            }
            self.positions[index] = first;
            self.positions[index + 1] = second;
        }

        Ok(())
    }

    fn solve_bend(&mut self) -> Result<(), RopeError> {
        for index in 0..self.rest_angles.len() {
            let mut first = self.positions[index];
            let mut second = self.positions[index + 1];
            let mut third = self.positions[index + 2];

            let first_inverse_mass = self.inverse_masses[index];
            let second_inverse_mass = self.inverse_masses[index + 1];
            let third_inverse_mass = self.inverse_masses[index + 2];
            let first_delta = second - first;
            let second_delta = third - second;
            let first_length_squared = first_delta.length_squared();
            let second_length_squared = second_delta.length_squared();
            let length_product = first_length_squared * second_length_squared;
            if !first_length_squared.is_finite()
                || !second_length_squared.is_finite()
                || !length_product.is_finite()
            {
                return Err(RopeError::NonFiniteDerivedState { index });
            }
            if length_product == 0.0 {
                continue;
            }

            let cross = first_delta.cross(second_delta);
            let dot = first_delta.dot(second_delta);
            if !cross.is_finite() || !dot.is_finite() {
                return Err(RopeError::NonFiniteDerivedState { index });
            }
            let mut angle = cross.atan2(dot);
            let first_jacobian_delta = (-1.0 / first_length_squared) * first_delta.skew();
            let second_jacobian_delta = (1.0 / second_length_squared) * second_delta.skew();
            let first_jacobian = -first_jacobian_delta;
            let second_jacobian = first_jacobian_delta - second_jacobian_delta;
            let third_jacobian = second_jacobian_delta;
            if !angle.is_finite()
                || !first_jacobian.is_valid()
                || !second_jacobian.is_valid()
                || !third_jacobian.is_valid()
            {
                return Err(RopeError::NonFiniteDerivedState { index });
            }

            let mut mass = first_inverse_mass * first_jacobian.dot(first_jacobian)
                + second_inverse_mass * second_jacobian.dot(second_jacobian)
                + third_inverse_mass * third_jacobian.dot(third_jacobian);
            if mass == 0.0 {
                continue;
            }
            if !mass.is_finite() {
                return Err(RopeError::NonFiniteDerivedState { index });
            }
            mass = 1.0 / mass;

            let mut angular_error = angle - self.rest_angles[index];
            let mut wrap_count = 0;
            while angular_error > PI {
                if wrap_count == RopeIterations::MAX {
                    return Err(RopeError::AngleWrapLimitExceeded { constraint: index });
                }
                angle -= TAU;
                angular_error = angle - self.rest_angles[index];
                wrap_count += 1;
            }
            while angular_error < -PI {
                if wrap_count == RopeIterations::MAX {
                    return Err(RopeError::AngleWrapLimitExceeded { constraint: index });
                }
                angle += TAU;
                angular_error = angle - self.rest_angles[index];
                wrap_count += 1;
            }

            let impulse = -self.bending_stiffness * mass * angular_error;
            let first_correction = (first_inverse_mass * impulse) * first_jacobian;
            let second_correction = (second_inverse_mass * impulse) * second_jacobian;
            let third_correction = (third_inverse_mass * impulse) * third_jacobian;
            if !impulse.is_finite()
                || !first_correction.is_valid()
                || !second_correction.is_valid()
                || !third_correction.is_valid()
            {
                return Err(RopeError::NonFiniteDerivedState { index });
            }

            first += first_correction;
            second += second_correction;
            third += third_correction;
            if !first.is_valid() || !second.is_valid() || !third.is_valid() {
                return Err(RopeError::NonFiniteDerivedState { index });
            }
            self.positions[index] = first;
            self.positions[index + 1] = second;
            self.positions[index + 2] = third;
        }

        Ok(())
    }

    fn reconstruct_velocities(&mut self, time_step: f32) -> Result<(), RopeError> {
        let inverse_time_step = 1.0 / time_step;
        if !inverse_time_step.is_finite() {
            return Err(RopeError::NonFiniteDerivedState { index: 0 });
        }

        for index in 0..self.positions.len() {
            self.velocities[index] =
                inverse_time_step * (self.positions[index] - self.previous_positions[index]);
            if !self.velocities[index].is_valid() {
                return Err(RopeError::NonFiniteDerivedState { index });
            }
        }

        Ok(())
    }
}

fn inverse_mass(index: usize, mass: f32) -> Result<f32, RopeError> {
    if mass == 0.0 {
        return Ok(0.0);
    }

    let inverse = 1.0 / mass;
    if !inverse.is_finite() {
        return Err(RopeError::NonFiniteDerivedState { index });
    }
    Ok(inverse)
}

fn rest_lengths(vertices: &[Vec2]) -> Result<Vec<f32>, RopeError> {
    let mut lengths = Vec::with_capacity(vertices.len() - 1);
    for index in 0..vertices.len() - 1 {
        let delta = vertices[index + 1] - vertices[index];
        let length = delta.length();
        if !delta.is_valid() || !length.is_finite() {
            return Err(RopeError::NonFiniteDerivedState { index });
        }
        lengths.push(length);
    }
    Ok(lengths)
}

fn rest_angles(vertices: &[Vec2]) -> Result<Vec<f32>, RopeError> {
    let mut angles = Vec::with_capacity(vertices.len() - 2);
    for index in 0..vertices.len() - 2 {
        let first_delta = vertices[index + 1] - vertices[index];
        let second_delta = vertices[index + 2] - vertices[index + 1];
        let cross = first_delta.cross(second_delta);
        let dot = first_delta.dot(second_delta);
        let angle = cross.atan2(dot);
        if !first_delta.is_valid()
            || !second_delta.is_valid()
            || !cross.is_finite()
            || !dot.is_finite()
            || !angle.is_finite()
        {
            return Err(RopeError::NonFiniteDerivedState { index });
        }
        angles.push(angle);
    }
    Ok(angles)
}
