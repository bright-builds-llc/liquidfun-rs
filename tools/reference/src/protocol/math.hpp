MathProbeOperation decode_math_operation(std::string_view value) {
  if (value == "is_valid") return MathProbeOperation::is_valid;
  if (value == "abs") return MathProbeOperation::abs;
  if (value == "min") return MathProbeOperation::min;
  if (value == "max") return MathProbeOperation::max;
  if (value == "clamp") return MathProbeOperation::clamp;
  if (value == "inv_sqrt") return MathProbeOperation::inv_sqrt;
  if (value == "vec_length") return MathProbeOperation::vec_length;
  if (value == "vec_normalize") return MathProbeOperation::vec_normalize;
  if (value == "dot") return MathProbeOperation::dot;
  if (value == "cross") return MathProbeOperation::cross;
  if (value == "mat22_solve") return MathProbeOperation::mat22_solve;
  if (value == "mat33_solve") return MathProbeOperation::mat33_solve;
  if (value == "mat22_inverse") return MathProbeOperation::mat22_inverse;
  if (value == "mat33_sym_inverse") return MathProbeOperation::mat33_sym_inverse;
  if (value == "rotation") return MathProbeOperation::rotation;
  if (value == "transform") return MathProbeOperation::transform;
  if (value == "sweep_transform") return MathProbeOperation::sweep_transform;
  if (value == "sweep_advance") return MathProbeOperation::sweep_advance;
  if (value == "sweep_normalize") return MathProbeOperation::sweep_normalize;
  if (value == "cancellation") return MathProbeOperation::cancellation;
  if (value == "halfway_rounding") return MathProbeOperation::halfway_rounding;
  if (value == "overflow") return MathProbeOperation::overflow;
  if (value == "underflow") return MathProbeOperation::underflow;
  if (value == "fma_witness") return MathProbeOperation::fma_witness;
  throw std::runtime_error("unsupported math probe operation");
}

MathProbePolicyPath decode_math_policy_path(std::string_view value) {
  if (value == "math.branch.is_valid") {
    return MathProbePolicyPath::branch_is_valid;
  }
  if (value == "math.operation.abs") return MathProbePolicyPath::operation_abs;
  if (value == "math.operation.min") return MathProbePolicyPath::operation_min;
  if (value == "math.pass_through.max") return MathProbePolicyPath::pass_through_max;
  if (value == "math.operation.clamp") return MathProbePolicyPath::operation_clamp;
  if (value == "math.operation.inv_sqrt") return MathProbePolicyPath::operation_inv_sqrt;
  if (value == "math.vector.length") return MathProbePolicyPath::vector_length;
  if (value == "math.vector.normalize") return MathProbePolicyPath::vector_normalize;
  if (value == "math.vector.dot") return MathProbePolicyPath::vector_dot;
  if (value == "math.vector.cross") return MathProbePolicyPath::vector_cross;
  if (value == "math.matrix22.solve") return MathProbePolicyPath::matrix22_solve;
  if (value == "math.matrix33.solve") return MathProbePolicyPath::matrix33_solve;
  if (value == "math.matrix22.inverse") return MathProbePolicyPath::matrix22_inverse;
  if (value == "math.matrix33.symmetric_inverse") return MathProbePolicyPath::matrix33_symmetric_inverse;
  if (value == "math.rotation") return MathProbePolicyPath::rotation;
  if (value == "math.transform.operation") return MathProbePolicyPath::transform_operation;
  if (value == "math.transform.steps_32") return MathProbePolicyPath::transform_steps_32;
  if (value == "math.sweep.transform") return MathProbePolicyPath::sweep_transform;
  if (value == "math.sweep.advance_steps_4") return MathProbePolicyPath::sweep_advance_steps_4;
  if (value == "math.sweep.normalize") return MathProbePolicyPath::sweep_normalize;
  if (value == "math.arithmetic.cancellation") return MathProbePolicyPath::arithmetic_cancellation;
  if (value == "math.arithmetic.halfway_rounding") return MathProbePolicyPath::arithmetic_halfway_rounding;
  if (value == "math.arithmetic.overflow") return MathProbePolicyPath::arithmetic_overflow;
  if (value == "math.arithmetic.underflow") return MathProbePolicyPath::arithmetic_underflow;
  if (value == "math.arithmetic.fma_witness") return MathProbePolicyPath::arithmetic_fma_witness;
  throw std::runtime_error("unsupported math probe policy path");
}

Vec2Bits decode_vec2_bits(const Node& node, std::string_view context) {
  const auto& object = as_object(node, context);
  require_members(object, {"x_bits", "y_bits"}, context);
  return {
      as_u32(member(object, "x_bits", context), "x bits"),
      as_u32(member(object, "y_bits", context), "y bits")};
}

Vec3Bits decode_vec3_bits(const Node& node, std::string_view context) {
  const auto& object = as_object(node, context);
  require_members(object, {"x_bits", "y_bits", "z_bits"}, context);
  return {
      as_u32(member(object, "x_bits", context), "x bits"),
      as_u32(member(object, "y_bits", context), "y bits"),
      as_u32(member(object, "z_bits", context), "z bits")};
}

Mat22Bits decode_mat22_bits(const Node& node) {
  const auto& object = as_object(node, "mat22 bits");
  require_members(object, {"first", "second"}, "mat22 bits");
  return {
      decode_vec2_bits(member(object, "first", "mat22 bits"), "first column"),
      decode_vec2_bits(member(object, "second", "mat22 bits"), "second column")};
}

Mat33Bits decode_mat33_bits(const Node& node) {
  const auto& object = as_object(node, "mat33 bits");
  require_members(object, {"first", "second", "third"}, "mat33 bits");
  return {
      decode_vec3_bits(member(object, "first", "mat33 bits"), "first column"),
      decode_vec3_bits(member(object, "second", "mat33 bits"), "second column"),
      decode_vec3_bits(member(object, "third", "mat33 bits"), "third column")};
}

TransformBits decode_transform_bits(const Node& node) {
  const auto& object = as_object(node, "transform bits");
  require_members(object, {"position", "angle_bits"}, "transform bits");
  return {
      decode_vec2_bits(member(object, "position", "transform bits"), "position"),
      as_u32(member(object, "angle_bits", "transform bits"), "angle bits")};
}

SweepBits decode_sweep_bits(const Node& node) {
  const auto& object = as_object(node, "sweep bits");
  require_members(
      object,
      {"local_center", "initial_center", "center", "initial_angle_bits",
       "angle_bits", "initial_fraction_bits"},
      "sweep bits");
  return {
      decode_vec2_bits(member(object, "local_center", "sweep bits"), "local center"),
      decode_vec2_bits(member(object, "initial_center", "sweep bits"), "initial center"),
      decode_vec2_bits(member(object, "center", "sweep bits"), "center"),
      as_u32(member(object, "initial_angle_bits", "sweep bits"), "initial angle bits"),
      as_u32(member(object, "angle_bits", "sweep bits"), "angle bits"),
      as_u32(member(object, "initial_fraction_bits", "sweep bits"), "initial fraction bits")};
}

MathProbeHorizon decode_math_horizon(const Node& node) {
  const auto& object = as_object(node, "math probe horizon");
  const auto& kind = as_string(
      member(object, "kind", "math probe horizon"), "horizon kind");
  if (kind == "operation") {
    require_members(object, {"kind"}, "operation horizon");
    return {};
  }
  if (kind == "scenario_steps") {
    require_members(object, {"kind", "steps"}, "scenario-steps horizon");
    const auto steps = as_u32(
        member(object, "steps", "scenario-steps horizon"), "horizon steps");
    if (steps == 0 || steps > 32) {
      throw std::runtime_error("math probe horizon is outside reviewed bounds");
    }
    return {false, steps};
  }
  throw std::runtime_error("unsupported math probe horizon");
}

MathProbeInput decode_math_input(const Node& node) {
  const auto& object = as_object(node, "math probe input");
  const auto& kind = as_string(
      member(object, "kind", "math probe input"), "math input kind");
  if (kind == "scalar") {
    require_members(object, {"kind", "value_bits"}, "scalar input");
    return ScalarInput{as_u32(member(object, "value_bits", "scalar input"), "value bits")};
  }
  if (kind == "binary") {
    require_members(object, {"kind", "a_bits", "b_bits"}, "binary input");
    return BinaryInput{
        as_u32(member(object, "a_bits", "binary input"), "a bits"),
        as_u32(member(object, "b_bits", "binary input"), "b bits")};
  }
  if (kind == "clamp") {
    require_members(object, {"kind", "value_bits", "low_bits", "high_bits"}, "clamp input");
    return ClampInput{
        as_u32(member(object, "value_bits", "clamp input"), "value bits"),
        as_u32(member(object, "low_bits", "clamp input"), "low bits"),
        as_u32(member(object, "high_bits", "clamp input"), "high bits")};
  }
  if (kind == "vector2") {
    require_members(object, {"kind", "vector"}, "vector2 input");
    return Vector2Input{decode_vec2_bits(member(object, "vector", "vector2 input"), "vector")};
  }
  if (kind == "vector_pair") {
    require_members(object, {"kind", "a", "b"}, "vector-pair input");
    return VectorPairInput{
        decode_vec2_bits(member(object, "a", "vector-pair input"), "a vector"),
        decode_vec2_bits(member(object, "b", "vector-pair input"), "b vector")};
  }
  if (kind == "mat22_solve") {
    require_members(object, {"kind", "matrix", "right"}, "mat22-solve input");
    return Mat22SolveInput{
        decode_mat22_bits(member(object, "matrix", "mat22-solve input")),
        decode_vec2_bits(member(object, "right", "mat22-solve input"), "right vector")};
  }
  if (kind == "mat33_solve") {
    require_members(object, {"kind", "matrix", "right"}, "mat33-solve input");
    return Mat33SolveInput{
        decode_mat33_bits(member(object, "matrix", "mat33-solve input")),
        decode_vec3_bits(member(object, "right", "mat33-solve input"), "right vector")};
  }
  if (kind == "mat22") {
    require_members(object, {"kind", "matrix"}, "mat22 input");
    return Mat22Input{decode_mat22_bits(member(object, "matrix", "mat22 input"))};
  }
  if (kind == "mat33") {
    require_members(object, {"kind", "matrix"}, "mat33 input");
    return Mat33Input{decode_mat33_bits(member(object, "matrix", "mat33 input"))};
  }
  if (kind == "rotation") {
    require_members(object, {"kind", "angle_bits"}, "rotation input");
    return RotationInput{as_u32(member(object, "angle_bits", "rotation input"), "angle bits")};
  }
  if (kind == "transform") {
    require_members(object, {"kind", "left", "right", "point"}, "transform input");
    return TransformInput{
        decode_transform_bits(member(object, "left", "transform input")),
        decode_transform_bits(member(object, "right", "transform input")),
        decode_vec2_bits(member(object, "point", "transform input"), "point")};
  }
  if (kind == "sweep_transform") {
    require_members(object, {"kind", "sweep", "fraction_bits"}, "sweep-transform input");
    return SweepTransformInput{
        decode_sweep_bits(member(object, "sweep", "sweep-transform input")),
        as_u32(member(object, "fraction_bits", "sweep-transform input"), "fraction bits")};
  }
  if (kind == "sweep_advance") {
    require_members(object, {"kind", "sweep", "fractions_bits"}, "sweep-advance input");
    const auto& raw_fractions = as_array(
        member(object, "fractions_bits", "sweep-advance input"), "fractions bits");
    if (raw_fractions.size() > 32) {
      throw std::runtime_error("math probe step collection exceeds reviewed limit");
    }
    std::vector<std::uint32_t> fractions;
    fractions.reserve(raw_fractions.size());
    for (const auto& raw_fraction : raw_fractions) {
      fractions.push_back(as_u32(raw_fraction, "fraction bits"));
    }
    return SweepAdvanceInput{
        decode_sweep_bits(member(object, "sweep", "sweep-advance input")),
        std::move(fractions)};
  }
  if (kind == "sweep") {
    require_members(object, {"kind", "sweep"}, "sweep input");
    return SweepInput{decode_sweep_bits(member(object, "sweep", "sweep input"))};
  }
  if (kind == "cancellation") {
    require_members(object, {"kind", "large_bits", "opposite_bits", "tail_bits"}, "cancellation input");
    return CancellationInput{
        as_u32(member(object, "large_bits", "cancellation input"), "large bits"),
        as_u32(member(object, "opposite_bits", "cancellation input"), "opposite bits"),
        as_u32(member(object, "tail_bits", "cancellation input"), "tail bits")};
  }
  if (kind == "halfway_rounding") {
    require_members(object, {"kind", "even_bits", "odd_bits", "half_ulp_bits"}, "halfway-rounding input");
    return HalfwayRoundingInput{
        as_u32(member(object, "even_bits", "halfway-rounding input"), "even bits"),
        as_u32(member(object, "odd_bits", "halfway-rounding input"), "odd bits"),
        as_u32(member(object, "half_ulp_bits", "halfway-rounding input"), "half ULP bits")};
  }
  if (kind == "scale") {
    require_members(object, {"kind", "value_bits", "factor_bits"}, "scale input");
    return ScaleInput{
        as_u32(member(object, "value_bits", "scale input"), "value bits"),
        as_u32(member(object, "factor_bits", "scale input"), "factor bits")};
  }
  if (kind == "fma_witness") {
    require_members(object, {"kind", "a_bits", "b_bits", "c_bits"}, "FMA-witness input");
    return FmaWitnessInput{
        as_u32(member(object, "a_bits", "FMA-witness input"), "a bits"),
        as_u32(member(object, "b_bits", "FMA-witness input"), "b bits"),
        as_u32(member(object, "c_bits", "FMA-witness input"), "c bits")};
  }
  throw std::runtime_error("unsupported math probe input kind");
}

bool input_matches_operation(
    MathProbeOperation operation,
    const MathProbeInput& input) {
  switch (operation) {
    case MathProbeOperation::is_valid:
    case MathProbeOperation::abs:
    case MathProbeOperation::inv_sqrt:
      return std::holds_alternative<ScalarInput>(input);
    case MathProbeOperation::min:
    case MathProbeOperation::max:
      return std::holds_alternative<BinaryInput>(input);
    case MathProbeOperation::clamp:
      return std::holds_alternative<ClampInput>(input);
    case MathProbeOperation::vec_length:
    case MathProbeOperation::vec_normalize:
      return std::holds_alternative<Vector2Input>(input);
    case MathProbeOperation::dot:
    case MathProbeOperation::cross:
      return std::holds_alternative<VectorPairInput>(input);
    case MathProbeOperation::mat22_solve:
      return std::holds_alternative<Mat22SolveInput>(input);
    case MathProbeOperation::mat33_solve:
      return std::holds_alternative<Mat33SolveInput>(input);
    case MathProbeOperation::mat22_inverse:
      return std::holds_alternative<Mat22Input>(input);
    case MathProbeOperation::mat33_sym_inverse:
      return std::holds_alternative<Mat33Input>(input);
    case MathProbeOperation::rotation:
      return std::holds_alternative<RotationInput>(input);
    case MathProbeOperation::transform:
      return std::holds_alternative<TransformInput>(input);
    case MathProbeOperation::sweep_transform:
      return std::holds_alternative<SweepTransformInput>(input);
    case MathProbeOperation::sweep_advance:
      return std::holds_alternative<SweepAdvanceInput>(input);
    case MathProbeOperation::sweep_normalize:
      return std::holds_alternative<SweepInput>(input);
    case MathProbeOperation::cancellation:
      return std::holds_alternative<CancellationInput>(input);
    case MathProbeOperation::halfway_rounding:
      return std::holds_alternative<HalfwayRoundingInput>(input);
    case MathProbeOperation::overflow:
    case MathProbeOperation::underflow:
      return std::holds_alternative<ScaleInput>(input);
    case MathProbeOperation::fma_witness:
      return std::holds_alternative<FmaWitnessInput>(input);
  }
  return false;
}

MathProbePolicyPath expected_policy_path(
    MathProbeOperation operation,
    const MathProbeHorizon& horizon) {
  switch (operation) {
    case MathProbeOperation::is_valid:
      return MathProbePolicyPath::branch_is_valid;
    case MathProbeOperation::abs: return MathProbePolicyPath::operation_abs;
    case MathProbeOperation::min: return MathProbePolicyPath::operation_min;
    case MathProbeOperation::max: return MathProbePolicyPath::pass_through_max;
    case MathProbeOperation::clamp: return MathProbePolicyPath::operation_clamp;
    case MathProbeOperation::inv_sqrt: return MathProbePolicyPath::operation_inv_sqrt;
    case MathProbeOperation::vec_length: return MathProbePolicyPath::vector_length;
    case MathProbeOperation::vec_normalize: return MathProbePolicyPath::vector_normalize;
    case MathProbeOperation::dot: return MathProbePolicyPath::vector_dot;
    case MathProbeOperation::cross: return MathProbePolicyPath::vector_cross;
    case MathProbeOperation::mat22_solve: return MathProbePolicyPath::matrix22_solve;
    case MathProbeOperation::mat33_solve: return MathProbePolicyPath::matrix33_solve;
    case MathProbeOperation::mat22_inverse: return MathProbePolicyPath::matrix22_inverse;
    case MathProbeOperation::mat33_sym_inverse:
      return MathProbePolicyPath::matrix33_symmetric_inverse;
    case MathProbeOperation::rotation: return MathProbePolicyPath::rotation;
    case MathProbeOperation::transform:
      return horizon.is_operation ? MathProbePolicyPath::transform_operation
                                  : MathProbePolicyPath::transform_steps_32;
    case MathProbeOperation::sweep_transform:
      return MathProbePolicyPath::sweep_transform;
    case MathProbeOperation::sweep_advance:
      return MathProbePolicyPath::sweep_advance_steps_4;
    case MathProbeOperation::sweep_normalize:
      return MathProbePolicyPath::sweep_normalize;
    case MathProbeOperation::cancellation:
      return MathProbePolicyPath::arithmetic_cancellation;
    case MathProbeOperation::halfway_rounding:
      return MathProbePolicyPath::arithmetic_halfway_rounding;
    case MathProbeOperation::overflow:
      return MathProbePolicyPath::arithmetic_overflow;
    case MathProbeOperation::underflow:
      return MathProbePolicyPath::arithmetic_underflow;
    case MathProbeOperation::fma_witness:
      return MathProbePolicyPath::arithmetic_fma_witness;
  }
  throw std::runtime_error("unreachable math probe operation");
}

MathProbeCase decode_math_case(const Node& node) {
  const auto& object = as_object(node, "math probe case");
  require_members(
      object, {"case_id", "operation", "policy_path", "horizon", "input"},
      "math probe case");
  MathProbeCase probe{
      as_string(member(object, "case_id", "math probe case"), "case ID"),
      decode_math_operation(as_string(
          member(object, "operation", "math probe case"), "math operation")),
      decode_math_policy_path(as_string(
          member(object, "policy_path", "math probe case"), "policy path")),
      decode_math_horizon(member(object, "horizon", "math probe case")),
      decode_math_input(member(object, "input", "math probe case"))};
  require_id(probe.case_id, "math probe case ID");
  if (!input_matches_operation(probe.operation, probe.input)) {
    throw std::runtime_error("math probe operation/input mismatch");
  }
  if (probe.policy_path != expected_policy_path(probe.operation, probe.horizon)) {
    throw std::runtime_error("math probe policy path mismatch");
  }
  if (const auto* advance = std::get_if<SweepAdvanceInput>(&probe.input)) {
    if (advance->fractions.empty() ||
        advance->fractions.size() != probe.horizon.steps ||
        probe.horizon.steps != 4) {
      throw std::runtime_error("math probe advance horizon mismatch");
    }
  } else if (!probe.horizon.is_operation) {
    if (!std::holds_alternative<TransformInput>(probe.input) ||
        probe.horizon.steps != 32) {
      throw std::runtime_error("math probe scenario horizon is invalid");
    }
  }
  return probe;
}

std::string_view math_operation_name(MathProbeOperation operation) {
  switch (operation) {
    case MathProbeOperation::is_valid: return "is_valid";
    case MathProbeOperation::abs: return "abs";
    case MathProbeOperation::min: return "min";
    case MathProbeOperation::max: return "max";
    case MathProbeOperation::clamp: return "clamp";
    case MathProbeOperation::inv_sqrt: return "inv_sqrt";
    case MathProbeOperation::vec_length: return "vec_length";
    case MathProbeOperation::vec_normalize: return "vec_normalize";
    case MathProbeOperation::dot: return "dot";
    case MathProbeOperation::cross: return "cross";
    case MathProbeOperation::mat22_solve: return "mat22_solve";
    case MathProbeOperation::mat33_solve: return "mat33_solve";
    case MathProbeOperation::mat22_inverse: return "mat22_inverse";
    case MathProbeOperation::mat33_sym_inverse: return "mat33_sym_inverse";
    case MathProbeOperation::rotation: return "rotation";
    case MathProbeOperation::transform: return "transform";
    case MathProbeOperation::sweep_transform: return "sweep_transform";
    case MathProbeOperation::sweep_advance: return "sweep_advance";
    case MathProbeOperation::sweep_normalize: return "sweep_normalize";
    case MathProbeOperation::cancellation: return "cancellation";
    case MathProbeOperation::halfway_rounding: return "halfway_rounding";
    case MathProbeOperation::overflow: return "overflow";
    case MathProbeOperation::underflow: return "underflow";
    case MathProbeOperation::fma_witness: return "fma_witness";
  }
  throw std::runtime_error("unreachable math probe operation");
}

std::string_view math_policy_path_name(MathProbePolicyPath path) {
  switch (path) {
    case MathProbePolicyPath::branch_is_valid: return "math.branch.is_valid";
    case MathProbePolicyPath::operation_abs: return "math.operation.abs";
    case MathProbePolicyPath::operation_min: return "math.operation.min";
    case MathProbePolicyPath::pass_through_max: return "math.pass_through.max";
    case MathProbePolicyPath::operation_clamp: return "math.operation.clamp";
    case MathProbePolicyPath::operation_inv_sqrt: return "math.operation.inv_sqrt";
    case MathProbePolicyPath::vector_length: return "math.vector.length";
    case MathProbePolicyPath::vector_normalize: return "math.vector.normalize";
    case MathProbePolicyPath::vector_dot: return "math.vector.dot";
    case MathProbePolicyPath::vector_cross: return "math.vector.cross";
    case MathProbePolicyPath::matrix22_solve: return "math.matrix22.solve";
    case MathProbePolicyPath::matrix33_solve: return "math.matrix33.solve";
    case MathProbePolicyPath::matrix22_inverse: return "math.matrix22.inverse";
    case MathProbePolicyPath::matrix33_symmetric_inverse: return "math.matrix33.symmetric_inverse";
    case MathProbePolicyPath::rotation: return "math.rotation";
    case MathProbePolicyPath::transform_operation: return "math.transform.operation";
    case MathProbePolicyPath::transform_steps_32: return "math.transform.steps_32";
    case MathProbePolicyPath::sweep_transform: return "math.sweep.transform";
    case MathProbePolicyPath::sweep_advance_steps_4: return "math.sweep.advance_steps_4";
    case MathProbePolicyPath::sweep_normalize: return "math.sweep.normalize";
    case MathProbePolicyPath::arithmetic_cancellation: return "math.arithmetic.cancellation";
    case MathProbePolicyPath::arithmetic_halfway_rounding: return "math.arithmetic.halfway_rounding";
    case MathProbePolicyPath::arithmetic_overflow: return "math.arithmetic.overflow";
    case MathProbePolicyPath::arithmetic_underflow: return "math.arithmetic.underflow";
    case MathProbePolicyPath::arithmetic_fma_witness: return "math.arithmetic.fma_witness";
  }
  throw std::runtime_error("unreachable math probe policy path");
}

std::string_view math_value_field_name(MathProbeValueField field) {
  switch (field) {
    case MathProbeValueField::value: return "value";
    case MathProbeValueField::x: return "x";
    case MathProbeValueField::y: return "y";
    case MathProbeValueField::z: return "z";
    case MathProbeValueField::length: return "length";
    case MathProbeValueField::sine: return "sine";
    case MathProbeValueField::cosine: return "cosine";
    case MathProbeValueField::position_x: return "position_x";
    case MathProbeValueField::position_y: return "position_y";
    case MathProbeValueField::angle: return "angle";
    case MathProbeValueField::initial_center_x: return "initial_center_x";
    case MathProbeValueField::initial_center_y: return "initial_center_y";
    case MathProbeValueField::initial_angle: return "initial_angle";
    case MathProbeValueField::initial_fraction: return "initial_fraction";
    case MathProbeValueField::left_associated: return "left_associated";
    case MathProbeValueField::right_associated: return "right_associated";
    case MathProbeValueField::even_midpoint: return "even_midpoint";
    case MathProbeValueField::odd_midpoint: return "odd_midpoint";
  }
  throw std::runtime_error("unreachable math probe value field");
}

std::string_view math_discrete_field_name(MathProbeDiscreteField field) {
  switch (field) {
    case MathProbeDiscreteField::predicate: return "predicate";
    case MathProbeDiscreteField::non_zero_determinant: return "non_zero_determinant";
    case MathProbeDiscreteField::normalized: return "normalized";
    case MathProbeDiscreteField::advanced: return "advanced";
  }
  throw std::runtime_error("unreachable math probe discrete field");
}

std::string_view float_class_name(std::uint32_t bits) {
  const auto exponent = bits & 0x7F800000U;
  const auto fraction = bits & 0x007FFFFFU;
  if (exponent == 0) return fraction == 0 ? "zero" : "subnormal";
  if (exponent == 0x7F800000U) return fraction == 0 ? "infinite" : "nan";
  return "normal";
}
