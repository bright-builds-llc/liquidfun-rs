#include "math_probe.hpp"

#include "protocol.hpp"

#include <Box2D/Common/b2Math.h>

#include <stdexcept>

namespace liquidfun::reference {
namespace {

b2Vec2 vec2(const Vec2Bits& bits) {
  return {float_from_bits(bits.x), float_from_bits(bits.y)};
}

b2Vec3 vec3(const Vec3Bits& bits) {
  return {float_from_bits(bits.x), float_from_bits(bits.y),
          float_from_bits(bits.z)};
}

b2Mat22 mat22(const Mat22Bits& bits) {
  return {vec2(bits.first), vec2(bits.second)};
}

b2Mat33 mat33(const Mat33Bits& bits) {
  return {vec3(bits.first), vec3(bits.second), vec3(bits.third)};
}

b2Transform transform(const TransformBits& bits) {
  return {vec2(bits.position), b2Rot(float_from_bits(bits.angle))};
}

b2Sweep sweep(const SweepBits& bits) {
  b2Sweep value;
  value.localCenter = vec2(bits.local_center);
  value.c0 = vec2(bits.initial_center);
  value.c = vec2(bits.center);
  value.a0 = float_from_bits(bits.initial_angle);
  value.a = float_from_bits(bits.angle);
  value.alpha0 = float_from_bits(bits.initial_fraction);
  return value;
}

void push(
    std::vector<MathProbeValue>& values,
    MathProbeValueField field,
    float value) {
  values.push_back({field, bits_from_float(value)});
}

void push_vec2(std::vector<MathProbeValue>& values, const b2Vec2& value) {
  push(values, MathProbeValueField::x, value.x);
  push(values, MathProbeValueField::y, value.y);
}

void push_position(
    std::vector<MathProbeValue>& values,
    const b2Vec2& value) {
  push(values, MathProbeValueField::position_x, value.x);
  push(values, MathProbeValueField::position_y, value.y);
}

void push_vec3(std::vector<MathProbeValue>& values, const b2Vec3& value) {
  push(values, MathProbeValueField::x, value.x);
  push(values, MathProbeValueField::y, value.y);
  push(values, MathProbeValueField::z, value.z);
}

void push_mat22(std::vector<MathProbeValue>& values, const b2Mat22& value) {
  push_vec2(values, value.ex);
  push_vec2(values, value.ey);
}

void push_mat33(std::vector<MathProbeValue>& values, const b2Mat33& value) {
  push_vec3(values, value.ex);
  push_vec3(values, value.ey);
  push_vec3(values, value.ez);
}

MathProbeResult execute_case(const MathProbeCase& probe) {
  MathProbeResult result{
      probe.case_id,
      probe.operation,
      probe.policy_path,
      probe.horizon,
      {},
      {}};
  switch (probe.operation) {
    case MathProbeOperation::is_valid: {
      const auto& input = std::get<ScalarInput>(probe.input);
      result.discrete.push_back({
          MathProbeDiscreteField::predicate,
          b2IsValid(float_from_bits(input.value))});
      break;
    }
    case MathProbeOperation::abs: {
      const auto& input = std::get<ScalarInput>(probe.input);
      push(result.values, MathProbeValueField::value,
           b2Abs(float_from_bits(input.value)));
      break;
    }
    case MathProbeOperation::min:
    case MathProbeOperation::max: {
      const auto& input = std::get<BinaryInput>(probe.input);
      const auto a = float_from_bits(input.a);
      const auto b = float_from_bits(input.b);
      push(result.values, MathProbeValueField::value,
           probe.operation == MathProbeOperation::min ? b2Min(a, b)
                                                      : b2Max(a, b));
      break;
    }
    case MathProbeOperation::clamp: {
      const auto& input = std::get<ClampInput>(probe.input);
      push(result.values, MathProbeValueField::value,
           b2Clamp(float_from_bits(input.value), float_from_bits(input.low),
                   float_from_bits(input.high)));
      break;
    }
    case MathProbeOperation::inv_sqrt: {
      const auto& input = std::get<ScalarInput>(probe.input);
      push(result.values, MathProbeValueField::value,
           b2InvSqrt(float_from_bits(input.value)));
      break;
    }
    case MathProbeOperation::vec_length: {
      const auto& input = std::get<Vector2Input>(probe.input);
      push(result.values, MathProbeValueField::length,
           vec2(input.vector).Length());
      break;
    }
    case MathProbeOperation::vec_normalize: {
      const auto& input = std::get<Vector2Input>(probe.input);
      auto value = vec2(input.vector);
      const auto length = value.Normalize();
      push(result.values, MathProbeValueField::length, length);
      push_vec2(result.values, value);
      result.discrete.push_back(
          {MathProbeDiscreteField::normalized, length != 0.0F});
      break;
    }
    case MathProbeOperation::dot:
    case MathProbeOperation::cross: {
      const auto& input = std::get<VectorPairInput>(probe.input);
      const auto a = vec2(input.a);
      const auto b = vec2(input.b);
      push(result.values, MathProbeValueField::value,
           probe.operation == MathProbeOperation::dot ? b2Dot(a, b)
                                                      : b2Cross(a, b));
      break;
    }
    case MathProbeOperation::mat22_solve: {
      const auto& input = std::get<Mat22SolveInput>(probe.input);
      const auto matrix = mat22(input.matrix);
      const auto determinant =
          matrix.ex.x * matrix.ey.y - matrix.ey.x * matrix.ex.y;
      push_vec2(result.values, matrix.Solve(vec2(input.right)));
      result.discrete.push_back(
          {MathProbeDiscreteField::non_zero_determinant,
           determinant != 0.0F});
      break;
    }
    case MathProbeOperation::mat33_solve: {
      const auto& input = std::get<Mat33SolveInput>(probe.input);
      const auto matrix = mat33(input.matrix);
      const auto determinant = b2Dot(matrix.ex, b2Cross(matrix.ey, matrix.ez));
      push_vec3(result.values, matrix.Solve33(vec3(input.right)));
      result.discrete.push_back(
          {MathProbeDiscreteField::non_zero_determinant,
           determinant != 0.0F});
      break;
    }
    case MathProbeOperation::mat22_inverse: {
      const auto& input = std::get<Mat22Input>(probe.input);
      const auto matrix = mat22(input.matrix);
      const auto determinant =
          matrix.ex.x * matrix.ey.y - matrix.ey.x * matrix.ex.y;
      push_mat22(result.values, matrix.GetInverse());
      result.discrete.push_back(
          {MathProbeDiscreteField::non_zero_determinant,
           determinant != 0.0F});
      break;
    }
    case MathProbeOperation::mat33_sym_inverse: {
      const auto& input = std::get<Mat33Input>(probe.input);
      const auto matrix = mat33(input.matrix);
      const auto determinant = b2Dot(matrix.ex, b2Cross(matrix.ey, matrix.ez));
      b2Mat33 inverse;
      matrix.GetSymInverse33(&inverse);
      push_mat33(result.values, inverse);
      result.discrete.push_back(
          {MathProbeDiscreteField::non_zero_determinant,
           determinant != 0.0F});
      break;
    }
    case MathProbeOperation::rotation: {
      const auto& input = std::get<RotationInput>(probe.input);
      const b2Rot value(float_from_bits(input.angle));
      push(result.values, MathProbeValueField::sine, value.s);
      push(result.values, MathProbeValueField::cosine, value.c);
      push(result.values, MathProbeValueField::angle, value.GetAngle());
      break;
    }
    case MathProbeOperation::transform: {
      const auto& input = std::get<TransformInput>(probe.input);
      auto composed = transform(input.left);
      for (std::uint32_t index = 0; index < probe.horizon.steps; ++index) {
        composed = b2Mul(composed, transform(input.right));
      }
      push_position(result.values, b2Mul(composed, vec2(input.point)));
      push(result.values, MathProbeValueField::angle, composed.q.GetAngle());
      break;
    }
    case MathProbeOperation::sweep_transform: {
      const auto& input = std::get<SweepTransformInput>(probe.input);
      b2Transform value;
      sweep(input.sweep).GetTransform(&value, float_from_bits(input.fraction));
      push_position(result.values, value.p);
      push(result.values, MathProbeValueField::sine, value.q.s);
      push(result.values, MathProbeValueField::cosine, value.q.c);
      break;
    }
    case MathProbeOperation::sweep_advance: {
      const auto& input = std::get<SweepAdvanceInput>(probe.input);
      auto value = sweep(input.sweep);
      for (const auto fraction : input.fractions) {
        value.Advance(float_from_bits(fraction));
      }
      push(result.values, MathProbeValueField::initial_center_x, value.c0.x);
      push(result.values, MathProbeValueField::initial_center_y, value.c0.y);
      push(result.values, MathProbeValueField::initial_angle, value.a0);
      push(result.values, MathProbeValueField::initial_fraction, value.alpha0);
      result.discrete.push_back({MathProbeDiscreteField::advanced, true});
      break;
    }
    case MathProbeOperation::sweep_normalize: {
      const auto& input = std::get<SweepInput>(probe.input);
      auto value = sweep(input.sweep);
      value.Normalize();
      push(result.values, MathProbeValueField::initial_angle, value.a0);
      push(result.values, MathProbeValueField::angle, value.a);
      break;
    }
    case MathProbeOperation::cancellation: {
      const auto& input = std::get<CancellationInput>(probe.input);
      const auto large = float_from_bits(input.large);
      const auto opposite = float_from_bits(input.opposite);
      const auto tail = float_from_bits(input.tail);
      const auto left_pair = large + opposite;
      const auto right_pair = opposite + tail;
      push(result.values, MathProbeValueField::left_associated,
           left_pair + tail);
      push(result.values, MathProbeValueField::right_associated,
           large + right_pair);
      break;
    }
    case MathProbeOperation::halfway_rounding: {
      const auto& input = std::get<HalfwayRoundingInput>(probe.input);
      push(result.values, MathProbeValueField::even_midpoint,
           float_from_bits(input.even) + float_from_bits(input.half_ulp));
      push(result.values, MathProbeValueField::odd_midpoint,
           float_from_bits(input.odd) + float_from_bits(input.half_ulp));
      break;
    }
    case MathProbeOperation::overflow:
    case MathProbeOperation::underflow: {
      const auto& input = std::get<ScaleInput>(probe.input);
      push(result.values, MathProbeValueField::value,
           float_from_bits(input.value) * float_from_bits(input.factor));
      break;
    }
    case MathProbeOperation::fma_witness: {
      const auto& input = std::get<FmaWitnessInput>(probe.input);
      const auto product = float_from_bits(input.a) * float_from_bits(input.b);
      push(result.values, MathProbeValueField::value,
           product + float_from_bits(input.c));
      break;
    }
  }
  return result;
}

}  // namespace

std::vector<MathProbeResult> execute_math_probe(
    const MathProbeRequest& request) {
  std::vector<MathProbeResult> results;
  results.reserve(request.cases.size());
  for (const auto& probe : request.cases) {
    results.push_back(execute_case(probe));
  }
  return results;
}

}  // namespace liquidfun::reference
