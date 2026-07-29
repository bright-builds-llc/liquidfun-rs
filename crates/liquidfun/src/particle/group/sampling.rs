#[cfg(test)]
mod tests;

use std::error::Error;
use std::fmt;

use crate::collision::{Aabb, CollisionError, EdgeShape, Shape};
use crate::math::{Transform, Vec2};

use super::{
    FilledParticleGroupShapes, ParticleGroupRecipe, ParticleGroupSource, ParticleGroupStrokeShape,
};

/// Resource ceilings applied before particle-group samples are materialized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SamplingLimits {
    maximum_work: usize,
    maximum_samples: usize,
}

impl SamplingLimits {
    pub(crate) const fn new(maximum_work: usize, maximum_samples: usize) -> Self {
        Self {
            maximum_work,
            maximum_samples,
        }
    }
}

/// One source-ordered particle position and its exact initial velocity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ParticleSample {
    position: Vec2,
    velocity: Vec2,
}

impl ParticleSample {
    pub(crate) const fn position(self) -> Vec2 {
        self.position
    }

    pub(crate) const fn velocity(self) -> Vec2 {
        self.velocity
    }
}

/// Pure owned output from one completely validated sampling operation.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SamplePlan {
    samples: Box<[ParticleSample]>,
}

impl SamplePlan {
    pub(crate) fn samples(&self) -> &[ParticleSample] {
        &self.samples
    }

    pub(crate) fn into_samples(self) -> Box<[ParticleSample]> {
        self.samples
    }
}

/// A failure while planning bounded particle-group samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParticleGroupSamplingError {
    NonFiniteDefaultStride,
    NonPositiveDefaultStride,
    WorkLimitExceeded { required: usize, limit: usize },
    CapacityExceeded { required: usize, limit: usize },
    ArithmeticOverflow,
    NonFiniteDerivedGeometry,
    NonFiniteDerivedPosition,
    NonFiniteDerivedVelocity,
    AllocationFailed,
    Shape(CollisionError),
}

impl fmt::Display for ParticleGroupSamplingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteDefaultStride => {
                formatter.write_str("default particle-group stride must be finite")
            }
            Self::NonPositiveDefaultStride => {
                formatter.write_str("default particle-group stride must be positive")
            }
            Self::WorkLimitExceeded { required, limit } => write!(
                formatter,
                "particle-group sampling requires {required} work units but the limit is {limit}",
            ),
            Self::CapacityExceeded { required, limit } => write!(
                formatter,
                "particle-group sampling requires {required} particles but capacity is {limit}",
            ),
            Self::ArithmeticOverflow => {
                formatter.write_str("particle-group sampling bounds overflow")
            }
            Self::NonFiniteDerivedGeometry => {
                formatter.write_str("particle-group sampling derived non-finite geometry")
            }
            Self::NonFiniteDerivedPosition => {
                formatter.write_str("particle-group sampling derived a non-finite position")
            }
            Self::NonFiniteDerivedVelocity => {
                formatter.write_str("particle-group sampling derived a non-finite velocity")
            }
            Self::AllocationFailed => {
                formatter.write_str("particle-group sample allocation failed")
            }
            Self::Shape(error) => write!(formatter, "particle-group shape query failed: {error}"),
        }
    }
}

impl Error for ParticleGroupSamplingError {}

impl From<CollisionError> for ParticleGroupSamplingError {
    fn from(error: CollisionError) -> Self {
        Self::Shape(error)
    }
}

#[derive(Debug, Clone, Copy)]
struct SamplingContext {
    transform: Transform,
    linear_velocity: Vec2,
    angular_velocity: f32,
    stride: f32,
    limits: SamplingLimits,
}

pub(crate) fn plan_samples<UserAssociation>(
    recipe: &ParticleGroupRecipe<UserAssociation>,
    default_stride: f32,
    limits: SamplingLimits,
) -> Result<SamplePlan, ParticleGroupSamplingError> {
    let stride = recipe.maybe_stride().unwrap_or(default_stride);
    if !stride.is_finite() {
        return Err(ParticleGroupSamplingError::NonFiniteDefaultStride);
    }
    if stride <= 0.0 {
        return Err(ParticleGroupSamplingError::NonPositiveDefaultStride);
    }
    let context = SamplingContext {
        transform: recipe.transform(),
        linear_velocity: recipe.linear_velocity(),
        angular_velocity: recipe.angular_velocity(),
        stride,
        limits,
    };
    match recipe.source() {
        ParticleGroupSource::FilledShapes(shapes) => sample_filled_union(shapes, context),
        ParticleGroupSource::StrokeShape(shape) => sample_stroke(shape, context),
        ParticleGroupSource::Positions(positions) => {
            sample_explicit_positions(positions.positions(), context)
        }
    }
}

fn sample_filled_union(
    shapes: &FilledParticleGroupShapes,
    context: SamplingContext,
) -> Result<SamplePlan, ParticleGroupSamplingError> {
    let bounds = union_bounds(shapes)?;
    let x_axis = preflight_axis(
        bounds.lower_bound().x,
        bounds.upper_bound().x,
        context.stride,
        context.limits.maximum_work,
    )?;
    let y_axis = preflight_axis(
        bounds.lower_bound().y,
        bounds.upper_bound().y,
        context.stride,
        context.limits.maximum_work,
    )?;
    let grid_points = x_axis
        .count
        .checked_mul(y_axis.count)
        .ok_or(ParticleGroupSamplingError::ArithmeticOverflow)?;
    let point_tests = grid_points
        .checked_mul(shapes.shapes().len())
        .and_then(|work| work.checked_mul(2))
        .ok_or(ParticleGroupSamplingError::ArithmeticOverflow)?;
    check_work(point_tests, context.limits.maximum_work)?;

    let mut sample_count = 0_usize;
    visit_fill_points(shapes, x_axis, y_axis, |local_position| {
        validate_sample(local_position, context)?;
        sample_count = sample_count
            .checked_add(1)
            .ok_or(ParticleGroupSamplingError::ArithmeticOverflow)?;
        Ok(())
    })?;
    check_capacity(sample_count, context.limits.maximum_samples)?;

    materialize(sample_count, |push| {
        visit_fill_points(shapes, x_axis, y_axis, |local_position| {
            push(make_sample(local_position, context)?);
            Ok(())
        })
    })
}

fn sample_stroke(
    source: &ParticleGroupStrokeShape,
    context: SamplingContext,
) -> Result<SamplePlan, ParticleGroupSamplingError> {
    let sample_upper_bound = stroke_sample_upper_bound(source.shape(), context.stride)?;
    let work = sample_upper_bound
        .checked_mul(2)
        .and_then(|samples| samples.checked_add(source.shape().child_count()))
        .ok_or(ParticleGroupSamplingError::ArithmeticOverflow)?;
    check_work(work, context.limits.maximum_work)?;

    let mut sample_count = 0_usize;
    visit_stroke_points(source.shape(), context.stride, |local_position| {
        validate_sample(local_position, context)?;
        sample_count = sample_count
            .checked_add(1)
            .ok_or(ParticleGroupSamplingError::ArithmeticOverflow)?;
        Ok(())
    })?;
    check_capacity(sample_count, context.limits.maximum_samples)?;

    materialize(sample_count, |push| {
        visit_stroke_points(source.shape(), context.stride, |local_position| {
            push(make_sample(local_position, context)?);
            Ok(())
        })
    })
}

fn sample_explicit_positions(
    positions: &[Vec2],
    context: SamplingContext,
) -> Result<SamplePlan, ParticleGroupSamplingError> {
    check_work(positions.len(), context.limits.maximum_work)?;
    check_capacity(positions.len(), context.limits.maximum_samples)?;
    for position in positions {
        validate_sample(*position, context)?;
    }
    materialize(positions.len(), |push| {
        for position in positions {
            push(make_sample(*position, context)?);
        }
        Ok(())
    })
}

fn union_bounds(shapes: &FilledParticleGroupShapes) -> Result<Aabb, ParticleGroupSamplingError> {
    let mut bounds = None;
    for shape in shapes.shapes() {
        let child = shape.child_index(0)?;
        let shape_bounds = shape.compute_aabb(Transform::IDENTITY, child)?;
        bounds = Some(bounds.map_or(shape_bounds, |current: Aabb| current.combined(shape_bounds)));
    }
    bounds.ok_or(ParticleGroupSamplingError::NonFiniteDerivedGeometry)
}

#[derive(Debug, Clone, Copy)]
struct SampleAxis {
    start: f32,
    stride: f32,
    count: usize,
}

fn preflight_axis(
    lower: f32,
    upper: f32,
    stride: f32,
    maximum_work: usize,
) -> Result<SampleAxis, ParticleGroupSamplingError> {
    let start = (lower / stride).floor() * stride;
    if !start.is_finite() {
        return Err(ParticleGroupSamplingError::NonFiniteDerivedGeometry);
    }
    if start >= upper {
        return Ok(SampleAxis {
            start,
            stride,
            count: 0,
        });
    }

    let approximate_count = ((f64::from(upper) - f64::from(start)) / f64::from(stride)).ceil();
    if !approximate_count.is_finite() || approximate_count < 0.0 {
        return Err(ParticleGroupSamplingError::NonFiniteDerivedGeometry);
    }
    if let Ok(maximum_work_u32) = u32::try_from(maximum_work.saturating_add(1))
        && approximate_count > f64::from(maximum_work_u32)
    {
        return Err(ParticleGroupSamplingError::WorkLimitExceeded {
            required: usize::MAX,
            limit: maximum_work,
        });
    }

    let mut count = 0_usize;
    let mut coordinate = start;
    while coordinate < upper {
        count = count
            .checked_add(1)
            .ok_or(ParticleGroupSamplingError::ArithmeticOverflow)?;
        if count > maximum_work {
            return Err(ParticleGroupSamplingError::WorkLimitExceeded {
                required: count,
                limit: maximum_work,
            });
        }
        let next = coordinate + stride;
        if !next.is_finite() || next <= coordinate {
            return Err(ParticleGroupSamplingError::NonFiniteDerivedGeometry);
        }
        coordinate = next;
    }
    Ok(SampleAxis {
        start,
        stride,
        count,
    })
}

fn visit_fill_points(
    shapes: &FilledParticleGroupShapes,
    x_axis: SampleAxis,
    y_axis: SampleAxis,
    mut visit: impl FnMut(Vec2) -> Result<(), ParticleGroupSamplingError>,
) -> Result<(), ParticleGroupSamplingError> {
    let mut y = y_axis.start;
    for _ in 0..y_axis.count {
        let mut x = x_axis.start;
        for _ in 0..x_axis.count {
            let point = Vec2::new(x, y);
            if union_contains(shapes, point)? {
                visit(point)?;
            }
            x += x_axis.stride;
        }
        y += y_axis.stride;
    }
    Ok(())
}

fn union_contains(
    shapes: &FilledParticleGroupShapes,
    point: Vec2,
) -> Result<bool, ParticleGroupSamplingError> {
    for shape in shapes.shapes() {
        if shape.test_point(Transform::IDENTITY, point)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn stroke_sample_upper_bound(
    shape: &Shape,
    stride: f32,
) -> Result<usize, ParticleGroupSamplingError> {
    let mut upper_bound = 0_usize;
    visit_edges(shape, |edge| {
        let edge_length = edge_length(edge)?;
        let edge_bound = (f64::from(edge_length) / f64::from(stride)).ceil();
        let edge_bound = bounded_f64_count(edge_bound)?;
        upper_bound = upper_bound
            .checked_add(edge_bound)
            .and_then(|count| count.checked_add(1))
            .ok_or(ParticleGroupSamplingError::ArithmeticOverflow)?;
        Ok(())
    })?;
    Ok(upper_bound)
}

fn bounded_f64_count(value: f64) -> Result<usize, ParticleGroupSamplingError> {
    if !value.is_finite() || value < 0.0 || value > f64::from(i32::MAX) {
        return Err(ParticleGroupSamplingError::ArithmeticOverflow);
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the finite non-negative value is checked against i32::MAX before conversion"
    )]
    let count = value as u32;
    usize::try_from(count).map_err(|_| ParticleGroupSamplingError::ArithmeticOverflow)
}

fn visit_stroke_points(
    shape: &Shape,
    stride: f32,
    mut visit: impl FnMut(Vec2) -> Result<(), ParticleGroupSamplingError>,
) -> Result<(), ParticleGroupSamplingError> {
    let mut position_on_edge = 0.0_f32;
    visit_edges(shape, |edge| {
        let direction = edge.end() - edge.start();
        let edge_length = edge_length(edge)?;
        while position_on_edge < edge_length {
            let point = edge.start() + (position_on_edge / edge_length) * direction;
            visit(point)?;
            let next = position_on_edge + stride;
            if !next.is_finite() || next <= position_on_edge {
                return Err(ParticleGroupSamplingError::NonFiniteDerivedGeometry);
            }
            position_on_edge = next;
        }
        position_on_edge -= edge_length;
        if !position_on_edge.is_finite() {
            return Err(ParticleGroupSamplingError::NonFiniteDerivedGeometry);
        }
        Ok(())
    })
}

fn visit_edges(
    shape: &Shape,
    mut visit: impl FnMut(&EdgeShape) -> Result<(), ParticleGroupSamplingError>,
) -> Result<(), ParticleGroupSamplingError> {
    match shape {
        Shape::Edge(edge) => visit(edge),
        Shape::Chain(chain) => {
            for index in 0..chain.child_count() {
                let child = chain.child_index(index)?;
                let edge = chain.child_edge(child)?;
                visit(&edge)?;
            }
            Ok(())
        }
        Shape::Circle(_) | Shape::Polygon(_) => {
            Err(ParticleGroupSamplingError::NonFiniteDerivedGeometry)
        }
    }
}

fn edge_length(edge: &EdgeShape) -> Result<f32, ParticleGroupSamplingError> {
    let length = (edge.end() - edge.start()).length();
    if !length.is_finite() || length <= 0.0 {
        return Err(ParticleGroupSamplingError::NonFiniteDerivedGeometry);
    }
    Ok(length)
}

fn validate_sample(
    local_position: Vec2,
    context: SamplingContext,
) -> Result<(), ParticleGroupSamplingError> {
    make_sample(local_position, context).map(|_| ())
}

fn make_sample(
    local_position: Vec2,
    context: SamplingContext,
) -> Result<ParticleSample, ParticleGroupSamplingError> {
    let position = context.transform.apply(local_position);
    if !position.is_valid() {
        return Err(ParticleGroupSamplingError::NonFiniteDerivedPosition);
    }
    let velocity = context.linear_velocity
        + Vec2::scalar_cross(
            context.angular_velocity,
            position - context.transform.position(),
        );
    if !velocity.is_valid() {
        return Err(ParticleGroupSamplingError::NonFiniteDerivedVelocity);
    }
    Ok(ParticleSample { position, velocity })
}

fn materialize(
    sample_count: usize,
    build: impl FnOnce(&mut dyn FnMut(ParticleSample)) -> Result<(), ParticleGroupSamplingError>,
) -> Result<SamplePlan, ParticleGroupSamplingError> {
    let mut samples = Vec::new();
    samples
        .try_reserve_exact(sample_count)
        .map_err(|_| ParticleGroupSamplingError::AllocationFailed)?;
    build(&mut |sample| samples.push(sample))?;
    if samples.len() != sample_count {
        return Err(ParticleGroupSamplingError::ArithmeticOverflow);
    }
    Ok(SamplePlan {
        samples: samples.into_boxed_slice(),
    })
}

fn check_work(required: usize, limit: usize) -> Result<(), ParticleGroupSamplingError> {
    if required > limit {
        return Err(ParticleGroupSamplingError::WorkLimitExceeded { required, limit });
    }
    Ok(())
}

fn check_capacity(required: usize, limit: usize) -> Result<(), ParticleGroupSamplingError> {
    if required > limit {
        return Err(ParticleGroupSamplingError::CapacityExceeded { required, limit });
    }
    Ok(())
}
