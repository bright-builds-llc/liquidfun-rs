#pragma once

#include <cstdint>
#include <string>
#include <variant>
#include <vector>

namespace liquidfun::reference {

enum class MathProbeOperation {
  is_valid,
  abs,
  min,
  max,
  clamp,
  inv_sqrt,
  vec_length,
  vec_normalize,
  dot,
  cross,
  mat22_solve,
  mat33_solve,
  mat22_inverse,
  mat33_sym_inverse,
  rotation,
  transform,
  sweep_transform,
  sweep_advance,
  sweep_normalize,
  cancellation,
  halfway_rounding,
  overflow,
  underflow,
  fma_witness,
};

enum class MathProbePolicyPath {
  branch_is_valid,
  operation_abs,
  operation_min,
  pass_through_max,
  operation_clamp,
  operation_inv_sqrt,
  vector_length,
  vector_normalize,
  vector_dot,
  vector_cross,
  matrix22_solve,
  matrix33_solve,
  matrix22_inverse,
  matrix33_symmetric_inverse,
  rotation,
  transform_operation,
  transform_steps_32,
  sweep_transform,
  sweep_advance_steps_4,
  sweep_normalize,
  arithmetic_cancellation,
  arithmetic_halfway_rounding,
  arithmetic_overflow,
  arithmetic_underflow,
  arithmetic_fma_witness,
};

struct MathProbeHorizon {
  bool is_operation = true;
  std::uint32_t steps = 1;
};

struct Vec2Bits {
  std::uint32_t x = 0;
  std::uint32_t y = 0;
};

struct Vec3Bits {
  std::uint32_t x = 0;
  std::uint32_t y = 0;
  std::uint32_t z = 0;
};

struct Mat22Bits {
  Vec2Bits first;
  Vec2Bits second;
};

struct Mat33Bits {
  Vec3Bits first;
  Vec3Bits second;
  Vec3Bits third;
};

struct TransformBits {
  Vec2Bits position;
  std::uint32_t angle = 0;
};

struct SweepBits {
  Vec2Bits local_center;
  Vec2Bits initial_center;
  Vec2Bits center;
  std::uint32_t initial_angle = 0;
  std::uint32_t angle = 0;
  std::uint32_t initial_fraction = 0;
};

struct ScalarInput { std::uint32_t value = 0; };
struct BinaryInput { std::uint32_t a = 0; std::uint32_t b = 0; };
struct ClampInput { std::uint32_t value = 0; std::uint32_t low = 0; std::uint32_t high = 0; };
struct Vector2Input { Vec2Bits vector; };
struct VectorPairInput { Vec2Bits a; Vec2Bits b; };
struct Mat22SolveInput { Mat22Bits matrix; Vec2Bits right; };
struct Mat33SolveInput { Mat33Bits matrix; Vec3Bits right; };
struct Mat22Input { Mat22Bits matrix; };
struct Mat33Input { Mat33Bits matrix; };
struct RotationInput { std::uint32_t angle = 0; };
struct TransformInput { TransformBits left; TransformBits right; Vec2Bits point; };
struct SweepTransformInput { SweepBits sweep; std::uint32_t fraction = 0; };
struct SweepAdvanceInput { SweepBits sweep; std::vector<std::uint32_t> fractions; };
struct SweepInput { SweepBits sweep; };
struct CancellationInput { std::uint32_t large = 0; std::uint32_t opposite = 0; std::uint32_t tail = 0; };
struct HalfwayRoundingInput { std::uint32_t even = 0; std::uint32_t odd = 0; std::uint32_t half_ulp = 0; };
struct ScaleInput { std::uint32_t value = 0; std::uint32_t factor = 0; };
struct FmaWitnessInput { std::uint32_t a = 0; std::uint32_t b = 0; std::uint32_t c = 0; };

using MathProbeInput = std::variant<
    ScalarInput,
    BinaryInput,
    ClampInput,
    Vector2Input,
    VectorPairInput,
    Mat22SolveInput,
    Mat33SolveInput,
    Mat22Input,
    Mat33Input,
    RotationInput,
    TransformInput,
    SweepTransformInput,
    SweepAdvanceInput,
    SweepInput,
    CancellationInput,
    HalfwayRoundingInput,
    ScaleInput,
    FmaWitnessInput>;

struct MathProbeCase {
  std::string case_id;
  MathProbeOperation operation;
  MathProbePolicyPath policy_path;
  MathProbeHorizon horizon;
  MathProbeInput input;
};

struct MathProbeRequest {
  std::string request_id;
  std::vector<MathProbeCase> cases;
};

enum class MathProbeValueField {
  value, x, y, z, length, sine, cosine, position_x, position_y, angle,
  initial_center_x, initial_center_y, initial_angle, initial_fraction,
  left_associated, right_associated, even_midpoint, odd_midpoint,
};

enum class MathProbeDiscreteField {
  predicate,
  non_zero_determinant,
  normalized,
  advanced,
};

struct MathProbeValue {
  MathProbeValueField field;
  std::uint32_t bits = 0;
};

struct MathProbeDiscrete {
  MathProbeDiscreteField field;
  bool value = false;
};

struct MathProbeResult {
  std::string case_id;
  MathProbeOperation operation;
  MathProbePolicyPath policy_path;
  MathProbeHorizon horizon;
  std::vector<MathProbeValue> values;
  std::vector<MathProbeDiscrete> discrete;
};

std::vector<MathProbeResult> execute_math_probe(const MathProbeRequest& request);

}  // namespace liquidfun::reference
