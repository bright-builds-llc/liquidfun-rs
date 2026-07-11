//! Pure, bounded native execution for the Phase 4 math-probe contract.

use liquidfun::math::{self, Mat22, Mat33, Rotation, Sweep, SweepError, Transform, Vec2, Vec3};
use liquidfun_test_protocol::{
    FloatBits, Mat22Bits, Mat33Bits, MathProbeCase, MathProbeDiscrete, MathProbeDiscreteField,
    MathProbeInput, MathProbeOperation, MathProbeRequestRecord, MathProbeResult, MathProbeValue,
    MathProbeValueField, SweepBits, TransformBits, Vec2Bits, Vec3Bits,
};

/// Failure constructing or advancing a checked sweep from validated probe bits.
#[derive(Debug, thiserror::Error)]
pub enum MathProbeExecutionError {
    /// A structured sweep violates the checked public math invariant.
    #[error("case {case_id} contains an invalid checked sweep: {source}")]
    Sweep {
        /// Stable case identity.
        case_id: Box<str>,
        #[source]
        /// Typed checked-sweep failure.
        source: SweepError,
    },
}

/// Stateless native executor; it creates no world, solver, or persistent adapter state.
pub struct NativeMathProbeExecutor;

impl NativeMathProbeExecutor {
    /// Executes all validated cases in request order.
    pub fn execute(
        request: &MathProbeRequestRecord,
    ) -> Result<Box<[MathProbeResult]>, MathProbeExecutionError> {
        request
            .scenario()
            .cases()
            .iter()
            .map(execute_case)
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }
}

fn execute_case(case: &MathProbeCase) -> Result<MathProbeResult, MathProbeExecutionError> {
    let mut values = Vec::new();
    let mut discrete = Vec::new();
    match (case.operation(), case.input()) {
        (MathProbeOperation::IsValid, MathProbeInput::Scalar { value_bits }) => {
            discrete.push(discrete_value(
                MathProbeDiscreteField::Predicate,
                math::is_valid(value_bits.to_f32()),
            ));
        }
        (MathProbeOperation::Abs, MathProbeInput::Scalar { value_bits }) => {
            push(
                &mut values,
                MathProbeValueField::Value,
                math::abs(value_bits.to_f32()),
            );
        }
        (
            operation @ (MathProbeOperation::Min | MathProbeOperation::Max),
            MathProbeInput::Binary { a_bits, b_bits },
        ) => {
            let value = if operation == MathProbeOperation::Min {
                math::min(a_bits.to_f32(), b_bits.to_f32())
            } else {
                math::max(a_bits.to_f32(), b_bits.to_f32())
            };
            push(&mut values, MathProbeValueField::Value, value);
        }
        (
            MathProbeOperation::Clamp,
            MathProbeInput::Clamp {
                value_bits,
                low_bits,
                high_bits,
            },
        ) => {
            push(
                &mut values,
                MathProbeValueField::Value,
                math::clamp(value_bits.to_f32(), low_bits.to_f32(), high_bits.to_f32()),
            );
        }
        (MathProbeOperation::InvSqrt, MathProbeInput::Scalar { value_bits }) => {
            push(
                &mut values,
                MathProbeValueField::Value,
                math::inverse_sqrt(value_bits.to_f32()),
            );
        }
        (MathProbeOperation::VecLength, MathProbeInput::Vector2 { vector }) => {
            push(
                &mut values,
                MathProbeValueField::Length,
                vec2(*vector).length(),
            );
        }
        (MathProbeOperation::VecNormalize, MathProbeInput::Vector2 { vector }) => {
            let mut vector = vec2(*vector);
            let length = vector.normalize();
            push(&mut values, MathProbeValueField::Length, length);
            push_vec2(&mut values, vector);
            discrete.push(discrete_value(
                MathProbeDiscreteField::Normalized,
                length != 0.0,
            ));
        }
        (
            operation @ (MathProbeOperation::Dot | MathProbeOperation::Cross),
            MathProbeInput::VectorPair { a, b },
        ) => {
            let a = vec2(*a);
            let b = vec2(*b);
            let value = if operation == MathProbeOperation::Dot {
                a.dot(b)
            } else {
                a.cross(b)
            };
            push(&mut values, MathProbeValueField::Value, value);
        }
        (MathProbeOperation::Mat22Solve, MathProbeInput::Mat22Solve { matrix, right }) => {
            let determinant = matrix.first.x_bits.to_f32() * matrix.second.y_bits.to_f32()
                - matrix.second.x_bits.to_f32() * matrix.first.y_bits.to_f32();
            push_vec2(&mut values, mat22(*matrix).solve(vec2(*right)));
            discrete.push(discrete_value(
                MathProbeDiscreteField::NonZeroDeterminant,
                determinant != 0.0,
            ));
        }
        (MathProbeOperation::Mat33Solve, MathProbeInput::Mat33Solve { matrix, right }) => {
            let matrix = mat33(*matrix);
            let determinant = matrix
                .first_column()
                .dot(matrix.second_column().cross(matrix.third_column()));
            push_vec3(&mut values, matrix.solve33(vec3(*right)));
            discrete.push(discrete_value(
                MathProbeDiscreteField::NonZeroDeterminant,
                determinant != 0.0,
            ));
        }
        (MathProbeOperation::Mat22Inverse, MathProbeInput::Mat22 { matrix }) => {
            let determinant = matrix.first.x_bits.to_f32() * matrix.second.y_bits.to_f32()
                - matrix.second.x_bits.to_f32() * matrix.first.y_bits.to_f32();
            push_mat22(&mut values, mat22(*matrix).inverse());
            discrete.push(discrete_value(
                MathProbeDiscreteField::NonZeroDeterminant,
                determinant != 0.0,
            ));
        }
        (MathProbeOperation::Mat33SymInverse, MathProbeInput::Mat33 { matrix }) => {
            let matrix = mat33(*matrix);
            let determinant = matrix
                .first_column()
                .dot(matrix.second_column().cross(matrix.third_column()));
            push_mat33(&mut values, matrix.symmetric_inverse33());
            discrete.push(discrete_value(
                MathProbeDiscreteField::NonZeroDeterminant,
                determinant != 0.0,
            ));
        }
        (MathProbeOperation::Rotation, MathProbeInput::Rotation { angle_bits }) => {
            let rotation = Rotation::from_angle(angle_bits.to_f32());
            push(&mut values, MathProbeValueField::Sine, rotation.sine());
            push(&mut values, MathProbeValueField::Cosine, rotation.cosine());
            push(&mut values, MathProbeValueField::Angle, rotation.angle());
        }
        (MathProbeOperation::Transform, MathProbeInput::Transform { left, right, point }) => {
            let mut composed = transform(*left);
            for _ in 0..case.horizon().steps() {
                composed = composed.compose(transform(*right));
            }
            let result = composed.apply(vec2(*point));
            push_position(&mut values, result);
            push(
                &mut values,
                MathProbeValueField::Angle,
                composed.rotation().angle(),
            );
        }
        (
            MathProbeOperation::SweepTransform,
            MathProbeInput::SweepTransform {
                sweep,
                fraction_bits,
            },
        ) => {
            let transform = sweep_from(case, *sweep)?.transform_at(fraction_bits.to_f32());
            push_position(&mut values, transform.position());
            push(
                &mut values,
                MathProbeValueField::Sine,
                transform.rotation().sine(),
            );
            push(
                &mut values,
                MathProbeValueField::Cosine,
                transform.rotation().cosine(),
            );
        }
        (
            MathProbeOperation::SweepAdvance,
            MathProbeInput::SweepAdvance {
                sweep,
                fractions_bits,
            },
        ) => {
            let mut sweep = sweep_from(case, *sweep)?;
            for fraction in fractions_bits.as_slice() {
                sweep
                    .advance(fraction.to_f32())
                    .map_err(|source| sweep_error(case, source))?;
            }
            push_sweep_state(&mut values, sweep);
            discrete.push(discrete_value(MathProbeDiscreteField::Advanced, true));
        }
        (MathProbeOperation::SweepNormalize, MathProbeInput::Sweep { sweep }) => {
            let mut sweep = sweep_from(case, *sweep)?;
            sweep.normalize();
            push(
                &mut values,
                MathProbeValueField::InitialAngle,
                sweep.initial_angle(),
            );
            push(&mut values, MathProbeValueField::Angle, sweep.angle());
        }
        (
            MathProbeOperation::Cancellation,
            MathProbeInput::Cancellation {
                large_bits,
                opposite_bits,
                tail_bits,
            },
        ) => {
            let left_pair = large_bits.to_f32() + opposite_bits.to_f32();
            let left = left_pair + tail_bits.to_f32();
            let right_pair = opposite_bits.to_f32() + tail_bits.to_f32();
            let right = large_bits.to_f32() + right_pair;
            push(&mut values, MathProbeValueField::LeftAssociated, left);
            push(&mut values, MathProbeValueField::RightAssociated, right);
        }
        (
            MathProbeOperation::HalfwayRounding,
            MathProbeInput::HalfwayRounding {
                even_bits,
                odd_bits,
                half_ulp_bits,
            },
        ) => {
            push(
                &mut values,
                MathProbeValueField::EvenMidpoint,
                even_bits.to_f32() + half_ulp_bits.to_f32(),
            );
            push(
                &mut values,
                MathProbeValueField::OddMidpoint,
                odd_bits.to_f32() + half_ulp_bits.to_f32(),
            );
        }
        (
            MathProbeOperation::Overflow | MathProbeOperation::Underflow,
            MathProbeInput::Scale {
                value_bits,
                factor_bits,
            },
        ) => {
            push(
                &mut values,
                MathProbeValueField::Value,
                value_bits.to_f32() * factor_bits.to_f32(),
            );
        }
        (
            MathProbeOperation::FmaWitness,
            MathProbeInput::FmaWitness {
                a_bits,
                b_bits,
                c_bits,
            },
        ) => {
            let product = a_bits.to_f32() * b_bits.to_f32();
            push(
                &mut values,
                MathProbeValueField::Value,
                product + c_bits.to_f32(),
            );
        }
        _ => unreachable!("validated operation/input pairs are exhaustive"),
    }
    Ok(MathProbeResult::new(
        case.case_id(),
        case.operation(),
        case.policy_path(),
        case.horizon(),
        values,
        discrete,
    ))
}

fn vec2(bits: Vec2Bits) -> Vec2 {
    Vec2::new(bits.x_bits.to_f32(), bits.y_bits.to_f32())
}
fn vec3(bits: Vec3Bits) -> Vec3 {
    Vec3::new(
        bits.x_bits.to_f32(),
        bits.y_bits.to_f32(),
        bits.z_bits.to_f32(),
    )
}
fn mat22(bits: Mat22Bits) -> Mat22 {
    Mat22::from_columns(vec2(bits.first), vec2(bits.second))
}
fn mat33(bits: Mat33Bits) -> Mat33 {
    Mat33::from_columns(vec3(bits.first), vec3(bits.second), vec3(bits.third))
}
fn transform(bits: TransformBits) -> Transform {
    Transform::from_position_angle(vec2(bits.position), bits.angle_bits.to_f32())
}

fn sweep_from(case: &MathProbeCase, bits: SweepBits) -> Result<Sweep, MathProbeExecutionError> {
    Sweep::new(
        vec2(bits.local_center),
        vec2(bits.initial_center),
        vec2(bits.center),
        bits.initial_angle_bits.to_f32(),
        bits.angle_bits.to_f32(),
        bits.initial_fraction_bits.to_f32(),
    )
    .map_err(|source| sweep_error(case, source))
}

fn sweep_error(case: &MathProbeCase, source: SweepError) -> MathProbeExecutionError {
    MathProbeExecutionError::Sweep {
        case_id: case.case_id().into(),
        source,
    }
}

fn push(values: &mut Vec<MathProbeValue>, field: MathProbeValueField, value: f32) {
    values.push(MathProbeValue::new(field, FloatBits::from_f32(value)));
}
fn push_vec2(values: &mut Vec<MathProbeValue>, value: Vec2) {
    push(values, MathProbeValueField::X, value.x);
    push(values, MathProbeValueField::Y, value.y);
}
fn push_position(values: &mut Vec<MathProbeValue>, value: Vec2) {
    push(values, MathProbeValueField::PositionX, value.x);
    push(values, MathProbeValueField::PositionY, value.y);
}
fn push_vec3(values: &mut Vec<MathProbeValue>, value: Vec3) {
    push(values, MathProbeValueField::X, value.x);
    push(values, MathProbeValueField::Y, value.y);
    push(values, MathProbeValueField::Z, value.z);
}
fn push_mat22(values: &mut Vec<MathProbeValue>, value: Mat22) {
    push_vec2(values, value.first_column());
    push_vec2(values, value.second_column());
}
fn push_mat33(values: &mut Vec<MathProbeValue>, value: Mat33) {
    push_vec3(values, value.first_column());
    push_vec3(values, value.second_column());
    push_vec3(values, value.third_column());
}
fn push_sweep_state(values: &mut Vec<MathProbeValue>, value: Sweep) {
    push(
        values,
        MathProbeValueField::InitialCenterX,
        value.initial_center().x,
    );
    push(
        values,
        MathProbeValueField::InitialCenterY,
        value.initial_center().y,
    );
    push(
        values,
        MathProbeValueField::InitialAngle,
        value.initial_angle(),
    );
    push(
        values,
        MathProbeValueField::InitialFraction,
        value.initial_fraction(),
    );
}
fn discrete_value(field: MathProbeDiscreteField, value: bool) -> MathProbeDiscrete {
    MathProbeDiscrete::new(field, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use liquidfun_test_protocol::{HarnessLimits, decode_math_probe_request_jsonl};

    const REQUEST: &[u8] =
        include_bytes!("../../../protocol/fixtures/accepted/math-probe-request.jsonl");

    fn results() -> Box<[MathProbeResult]> {
        let request = decode_math_probe_request_jsonl(REQUEST, &HarnessLimits::phase2_default_v1())
            .expect("checked-in probe request should decode");
        NativeMathProbeExecutor::execute(&request).expect("native probes should execute")
    }

    #[test]
    fn native_math_probe_executes_complete_witness_corpus() {
        // Arrange / Act
        let results = results();

        // Assert
        assert!(results.len() >= 36);
        for operation in [
            MathProbeOperation::IsValid,
            MathProbeOperation::Abs,
            MathProbeOperation::Min,
            MathProbeOperation::Max,
            MathProbeOperation::Clamp,
            MathProbeOperation::InvSqrt,
            MathProbeOperation::VecLength,
            MathProbeOperation::VecNormalize,
            MathProbeOperation::Dot,
            MathProbeOperation::Cross,
            MathProbeOperation::Mat22Solve,
            MathProbeOperation::Mat33Solve,
            MathProbeOperation::Mat22Inverse,
            MathProbeOperation::Mat33SymInverse,
            MathProbeOperation::Rotation,
            MathProbeOperation::Transform,
            MathProbeOperation::SweepTransform,
            MathProbeOperation::SweepAdvance,
            MathProbeOperation::SweepNormalize,
            MathProbeOperation::Cancellation,
            MathProbeOperation::HalfwayRounding,
            MathProbeOperation::Overflow,
            MathProbeOperation::Underflow,
            MathProbeOperation::FmaWitness,
        ] {
            assert!(results.iter().any(|result| result.operation() == operation));
        }
    }

    #[test]
    fn native_math_probe_witness_bits_are_exact() {
        // Arrange
        let results = results();
        let expected = [
            ("cancellation", vec![0x4048_f5c3, 0x0000_0000]),
            ("halfway-rounding", vec![0x3f80_0000, 0x3f80_0002]),
            ("overflow", vec![0x7f80_0000]),
            ("underflow", vec![0x0020_0000]),
            ("fma-witness", vec![0x0000_0000]),
        ];

        // Act and Assert
        for (case_id, expected_bits) in expected {
            let result = results
                .iter()
                .find(|result| result.case_id() == case_id)
                .expect("named witness should exist");
            assert_eq!(
                result
                    .values()
                    .iter()
                    .map(|value| value.bits().bits())
                    .collect::<Vec<_>>(),
                expected_bits
            );
        }
    }
}
