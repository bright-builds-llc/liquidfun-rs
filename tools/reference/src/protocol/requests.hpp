RequestKind decode_request_kind(std::string_view record) {
  const auto root = decode_record_node(record);
  const auto& object = as_object(root, "protocol request");
  const auto& kind = as_string(
      member(object, "record_kind", "protocol request"), "record kind");
  if (kind == "scenario_request") return RequestKind::scenario;
  if (kind == "math_probe_request") return RequestKind::math_probe;
  if (kind == "collision_probe_request") return RequestKind::collision_probe;
  if (kind == "rigid_world_request") return RequestKind::rigid_world;
  if (kind == "catalog_run_request") return RequestKind::catalog_run;
  if (kind == "benchmark_run_request") return RequestKind::benchmark_run;
  throw std::runtime_error("unsupported record kind");
}

BenchmarkRunRequest decode_benchmark_run_request(std::string_view record) {
  constexpr std::string_view reviewed_policy_sha256 =
      "75c0253d9f1eaa0b4cd6097031ed85f3c530fe47606049b5ac060a5267a3f05f";
  constexpr std::array<std::string_view, 14> workloads{
      "world_step",          "broad_phase",       "narrow_phase",
      "contact_solve",       "ccd",               "joints",
      "particle_lifecycle",  "particle_contacts", "particle_sort",
      "particle_pressure",   "large_particle_system",
      "mixed_world",         "aabb_query",        "ray_cast"};
  constexpr std::array<std::string_view, 4> size_points{
      "fixed", "work_units128", "work_units1024", "work_units8192"};

  const auto root = decode_record_node(record);
  const auto& object = as_object(root, "benchmark run request");
  require_members(
      object,
      {"protocol_version", "record_kind", "identity", "resolved_bytes"},
      "benchmark run request");
  if (as_u32(
          member(object, "protocol_version", "benchmark run request"),
          "protocol version") != kProtocolVersion ||
      as_string(
          member(object, "record_kind", "benchmark run request"),
          "record kind") != "benchmark_run_request") {
    throw std::runtime_error("unsupported benchmark protocol version or kind");
  }

  const auto& identity_node =
      as_object(member(object, "identity", "benchmark run request"),
                "benchmark run identity");
  require_members(
      identity_node,
      {"request_id", "resolved_sha256", "settings", "workload",
       "size_point", "optimization_mode", "warmup_count",
       "measured_horizon", "sample_ordinal", "policy_sha256",
       "profile_enabled"},
      "benchmark run identity");
  BenchmarkRunIdentity identity;
  identity.request_id = as_string(
      member(identity_node, "request_id", "benchmark run identity"),
      "benchmark request ID");
  require_id(identity.request_id, "benchmark request ID");
  identity.resolved_sha256 = as_string(
      member(identity_node, "resolved_sha256", "benchmark run identity"),
      "benchmark resolved SHA-256");
  require_sha256(
      identity.resolved_sha256, "benchmark resolved SHA-256");

  const auto& settings =
      as_object(member(identity_node, "settings", "benchmark run identity"),
                "benchmark settings");
  require_members(
      settings,
      {"timestep_bits", "velocity_iterations", "position_iterations",
       "particle_iterations"},
      "benchmark settings");
  identity.settings = {
      as_u32(member(settings, "timestep_bits", "benchmark settings"),
             "benchmark timestep"),
      as_u32(
          member(settings, "velocity_iterations", "benchmark settings"),
          "benchmark velocity iterations"),
      as_u32(
          member(settings, "position_iterations", "benchmark settings"),
          "benchmark position iterations"),
      as_u32(
          member(settings, "particle_iterations", "benchmark settings"),
          "benchmark particle iterations")};
  const auto timestep = float_from_bits(identity.settings.timestep_bits);
  if (!std::isfinite(timestep) || timestep <= 0.0F) {
    throw std::runtime_error("benchmark timestep must be finite and positive");
  }
  for (const auto iterations :
       {identity.settings.velocity_iterations,
        identity.settings.position_iterations,
        identity.settings.particle_iterations}) {
    if (iterations == 0U || iterations > 1024U) {
      throw std::runtime_error(
          "benchmark iteration count is outside reviewed bounds");
    }
  }

  identity.workload = as_string(
      member(identity_node, "workload", "benchmark run identity"),
      "benchmark workload");
  if (std::find(workloads.begin(), workloads.end(), identity.workload) ==
      workloads.end()) {
    throw std::runtime_error("unknown benchmark workload");
  }
  identity.size_point = as_string(
      member(identity_node, "size_point", "benchmark run identity"),
      "benchmark size point");
  if (std::find(size_points.begin(), size_points.end(), identity.size_point) ==
      size_points.end()) {
    throw std::runtime_error("unknown benchmark size point");
  }
  identity.optimization_mode = as_string(
      member(identity_node, "optimization_mode", "benchmark run identity"),
      "benchmark optimization mode");
  if (identity.optimization_mode != "release_scalar") {
    throw std::runtime_error("unsupported benchmark optimization mode");
  }
  identity.warmup_count = as_u32(
      member(identity_node, "warmup_count", "benchmark run identity"),
      "benchmark warmup count");
  if (identity.warmup_count != 1U) {
    throw std::runtime_error("benchmark warmup count violates policy");
  }
  identity.measured_horizon = as_u32(
      member(identity_node, "measured_horizon", "benchmark run identity"),
      "benchmark measured horizon");
  if (identity.measured_horizon == 0U ||
      identity.measured_horizon > 4096U) {
    throw std::runtime_error(
        "benchmark measured horizon is outside reviewed bounds");
  }
  identity.sample_ordinal = as_u32(
      member(identity_node, "sample_ordinal", "benchmark run identity"),
      "benchmark sample ordinal");
  if (identity.sample_ordinal == 0U || identity.sample_ordinal > 30U) {
    throw std::runtime_error(
        "benchmark sample ordinal is outside reviewed bounds");
  }
  identity.policy_sha256 = as_string(
      member(identity_node, "policy_sha256", "benchmark run identity"),
      "benchmark policy SHA-256");
  require_sha256(identity.policy_sha256, "benchmark policy SHA-256");
  if (identity.policy_sha256 != reviewed_policy_sha256) {
    throw std::runtime_error("benchmark policy hash mismatch");
  }
  identity.profile_enabled = as_bool(
      member(identity_node, "profile_enabled", "benchmark run identity"),
      "benchmark profile flag");

  const auto& bytes = as_array(
      member(object, "resolved_bytes", "benchmark run request"),
      "benchmark resolved bytes");
  if (bytes.size() > kMaximumRecordBytes) {
    throw std::runtime_error(
        "benchmark resolved bytes exceed reviewed bound");
  }
  std::string resolved_bytes;
  resolved_bytes.reserve(bytes.size());
  for (const auto& byte : bytes) {
    const auto value = as_u32(byte, "benchmark resolved byte");
    if (value > 255U) {
      throw std::runtime_error("benchmark resolved byte exceeds u8");
    }
    resolved_bytes.push_back(static_cast<char>(value));
  }
  if (sha256_hex(resolved_bytes) != identity.resolved_sha256) {
    throw std::runtime_error("benchmark resolved bytes hash mismatch");
  }
  return {std::move(identity), std::move(resolved_bytes)};
}

MathProbeRequest decode_math_probe_request(std::string_view record) {
  const auto root = decode_record_node(record);
  const auto& object = as_object(root, "math probe request");
  require_members(
      object,
      {"protocol_version", "record_kind", "request_id",
       "scenario_schema_version", "requested_trace_schema_version",
       "tolerance_profile_version", "tolerance_profile_sha256", "scenario"},
      "math probe request");
  if (as_u32(member(object, "protocol_version", "math probe request"),
             "protocol version") != kProtocolVersion ||
      as_string(member(object, "record_kind", "math probe request"),
                "record kind") != "math_probe_request" ||
      as_u32(member(object, "scenario_schema_version", "math probe request"),
             "scenario version") != kScenarioSchemaVersion ||
      as_u32(member(object, "requested_trace_schema_version", "math probe request"),
             "trace version") != kTraceSchemaVersion ||
      as_u32(member(object, "tolerance_profile_version", "math probe request"),
             "tolerance version") != kToleranceProfileVersion) {
    throw std::runtime_error("unsupported math probe protocol version");
  }
  const auto& request_id = as_string(
      member(object, "request_id", "math probe request"), "request ID");
  require_id(request_id, "request ID");
  require_sha256(
      as_string(
          member(object, "tolerance_profile_sha256", "math probe request"),
          "tolerance digest"),
      "tolerance digest");
  const auto& scenario = as_object(
      member(object, "scenario", "math probe request"), "math probe scenario");
  require_members(
      scenario, {"scenario_id", "source", "cases"}, "math probe scenario");
  const auto& scenario_id = as_string(
      member(scenario, "scenario_id", "math probe scenario"), "scenario ID");
  require_id(scenario_id, "scenario ID");
  static_cast<void>(decode_source(member(scenario, "source", "math probe scenario")));
  const auto& raw_cases = as_array(
      member(scenario, "cases", "math probe scenario"), "math probe cases");
  if (raw_cases.empty() || raw_cases.size() > 256) {
    throw std::runtime_error("math probe case count is outside reviewed bounds");
  }
  std::unordered_set<std::string> case_ids;
  std::vector<MathProbeCase> cases;
  cases.reserve(raw_cases.size());
  for (const auto& raw_case : raw_cases) {
    auto probe = decode_math_case(raw_case);
    if (!case_ids.insert(probe.case_id).second) {
      throw std::runtime_error("duplicate math probe case ID");
    }
    cases.push_back(std::move(probe));
  }
  return {request_id, std::move(cases)};
}

ScenarioRequest decode_scenario_request(std::string_view record) {
  const auto root = decode_record_node(record);
  const auto& object = as_object(root, "scenario request");
  require_members(
      object,
      {"protocol_version", "record_kind", "request_id", "scenario_schema_version",
       "requested_trace_schema_version", "tolerance_profile_version",
       "tolerance_profile_sha256", "scenario"},
      "scenario request");
  if (as_u32(member(object, "protocol_version", "scenario request"), "protocol version") != kProtocolVersion) {
    throw std::runtime_error("unsupported protocol version");
  }
  if (as_string(member(object, "record_kind", "scenario request"), "record kind") != "scenario_request") {
    throw std::runtime_error("unsupported record kind");
  }
  const auto request_id =
      as_string(member(object, "request_id", "scenario request"), "request ID");
  require_id(request_id, "request ID");
  if (as_u32(member(object, "scenario_schema_version", "scenario request"), "scenario version") != kScenarioSchemaVersion ||
      as_u32(member(object, "requested_trace_schema_version", "scenario request"), "trace version") != kTraceSchemaVersion ||
      as_u32(member(object, "tolerance_profile_version", "scenario request"), "tolerance version") != kToleranceProfileVersion) {
    throw std::runtime_error("unsupported schema or tolerance version");
  }
  ScenarioRequest request{
      request_id,
      as_string(member(object, "tolerance_profile_sha256", "scenario request"), "tolerance digest"),
      decode_scenario(member(object, "scenario", "scenario request"))};
  require_sha256(request.tolerance_profile_sha256, "tolerance digest");
  return request;
}

std::string encode_scenario(const ScenarioV1& scenario) {
  std::string output = "{\"scenario_id\":" + quote(scenario.scenario_id) +
                       ",\"source\":" + encode_source(scenario.source) +
                       ",\"gravity_x_bits\":" + std::to_string(scenario.gravity_x_bits) +
                       ",\"gravity_y_bits\":" + std::to_string(scenario.gravity_y_bits) +
                       ",\"entities\":[],\"commands\":[";
  for (std::size_t index = 0; index < scenario.commands.size(); ++index) {
    if (index != 0) output += ',';
    const auto& command = scenario.commands[index];
    output += "{\"kind\":\"step\",\"command_id\":" + quote(command.command_id) +
              ",\"timestep_bits\":" + std::to_string(command.timestep_bits) +
              ",\"velocity_iterations\":" + std::to_string(command.velocity_iterations) +
              ",\"position_iterations\":" + std::to_string(command.position_iterations) +
              ",\"particle_iterations\":" + std::to_string(command.particle_iterations) + "}";
  }
  output += "],\"checkpoints\":[";
  for (std::size_t index = 0; index < scenario.checkpoints.size(); ++index) {
    if (index != 0) output += ',';
    const auto& checkpoint = scenario.checkpoints[index];
    output += "{\"checkpoint_id\":" + quote(checkpoint.checkpoint_id) +
              ",\"after_command_id\":" + quote(checkpoint.after_command_id) +
              ",\"phase\":" + quote(checkpoint.phase) + ",\"observables\":[";
    for (std::size_t observable = 0; observable < checkpoint.observables.size(); ++observable) {
      if (observable != 0) output += ',';
      output += checkpoint.observables[observable] == Observable::world_counts
                    ? "\"world_counts\""
                    : "\"simulation_time\"";
    }
    output += "]}";
  }
  return output + "]}";
}

std::string encode_scenario_request(const ScenarioRequest& request) {
  return "{\"protocol_version\":1,\"record_kind\":\"scenario_request\",\"request_id\":" +
         quote(request.request_id) +
         ",\"scenario_schema_version\":1,\"requested_trace_schema_version\":1,\"tolerance_profile_version\":1,\"tolerance_profile_sha256\":" +
         quote(request.tolerance_profile_sha256) + ",\"scenario\":" +
         encode_scenario(request.scenario) + "}\n";
}

std::string encode_handshake(const BuildIdentity& identity) {
  const auto identity_sha256 = build_identity_sha256(identity);
  return "{\"protocol_version\":1,\"record_kind\":\"handshake\",\"supported_scenario_versions\":[1],\"supported_trace_versions\":[1],\"supported_tolerance_versions\":[1],\"build_identity\":{\"oracle_revision\":" +
         quote(identity.oracle_revision) + ",\"adapter_revision\":" + quote(identity.adapter_revision) +
         ",\"adapter_content_sha256\":" + quote(identity.adapter_content_sha256) +
         ",\"cmake_preset\":" + quote(identity.cmake_preset) +
         ",\"compiler_id\":" + quote(identity.compiler_id) +
         ",\"compiler_version\":" + quote(identity.compiler_version) +
         ",\"target\":" + quote(identity.target) +
         ",\"build_type\":" + quote(identity.build_type) +
         ",\"effective_compile_flags\":" + quote(identity.effective_compile_flags) +
         ",\"effective_link_flags\":" + quote(identity.effective_link_flags) +
         ",\"sanitizer_mode\":" + quote(identity.sanitizer_mode) +
         ",\"compile_command_sha256\":" + quote(identity.compile_command_sha256) +
         ",\"target_triple\":" + quote(identity.target_triple) +
         ",\"target_cpu\":" + quote(identity.target_cpu) +
         ",\"target_features\":" + quote(identity.target_features) +
         ",\"sdk_or_sysroot\":" + quote(identity.sdk_or_sysroot) +
         ",\"optimization\":" + quote(identity.optimization) +
         ",\"fp_model\":" + quote(identity.fp_model) +
         ",\"fp_contract\":" + quote(identity.fp_contract) +
         ",\"denormal_mode\":" + quote(identity.denormal_mode) +
         ",\"feature_set\":" + quote(identity.feature_set) +
         ",\"os\":" + quote(identity.os) +
         ",\"libc\":" + quote(identity.libc) +
         ",\"libm\":" + quote(identity.libm) +
         ",\"rounding_mode\":" + quote(identity.rounding_mode) +
         ",\"gradual_underflow\":" + (identity.gradual_underflow ? "true" : "false") +
         "},\"identity_sha256\":" + quote(identity_sha256) + "}";
}

std::string encode_trace_begin(
    const ScenarioRequest& request,
    std::string_view scenario_sha256,
    std::string_view identity_sha256) {
  return "{\"protocol_version\":1,\"record_kind\":\"trace_begin\",\"request_id\":" + quote(request.request_id) +
         ",\"trace_schema_version\":1,\"scenario_id\":" + quote(request.scenario.scenario_id) +
         ",\"scenario_sha256\":" + quote(scenario_sha256) + ",\"source\":" + encode_source(request.scenario.source) +
         ",\"tolerance_profile_version\":1,\"tolerance_profile_sha256\":" + quote(request.tolerance_profile_sha256) +
         ",\"engine_kind\":\"cpp_oracle\",\"identity_sha256\":" + quote(identity_sha256) + "}";
}

std::string encode_checkpoint(
    const ScenarioRequest& request,
    const CheckpointRequest& checkpoint,
    std::uint32_t ordinal,
    std::uint32_t simulation_time_bits,
    const WorldCounts& counts,
    std::string_view identity_sha256) {
  return "{\"protocol_version\":1,\"record_kind\":\"checkpoint\",\"request_id\":" + quote(request.request_id) +
         ",\"checkpoint_id\":" + quote(checkpoint.checkpoint_id) +
         ",\"ordinal\":" + std::to_string(ordinal) + ",\"phase\":" + quote(checkpoint.phase) +
         ",\"simulation_time_bits\":" + std::to_string(simulation_time_bits) +
         ",\"world_counts\":" + encode_world_counts(counts) +
         ",\"identity_sha256\":" + quote(identity_sha256) + "}";
}

std::string encode_trace_end(
    const ScenarioRequest& request,
    std::uint32_t checkpoint_count,
    std::string_view trace_payload_sha256,
    std::uint64_t reset_epoch,
    bool reset_verified,
    std::string_view identity_sha256) {
  return "{\"protocol_version\":1,\"record_kind\":\"trace_end\",\"request_id\":" + quote(request.request_id) +
         ",\"checkpoint_count\":" + std::to_string(checkpoint_count) +
         ",\"trace_payload_sha256\":" + quote(trace_payload_sha256) +
         ",\"reset_epoch\":" + std::to_string(reset_epoch) +
         ",\"reset_verified\":" + (reset_verified ? "true" : "false") +
         ",\"identity_sha256\":" + quote(identity_sha256) + "}";
}

std::string encode_math_probe_result(const MathProbeResult& result) {
  std::string output = "{\"case_id\":" + quote(result.case_id) +
                       ",\"operation\":" + quote(math_operation_name(result.operation)) +
                       ",\"policy_path\":" + quote(math_policy_path_name(result.policy_path)) +
                       ",\"horizon\":{\"kind\":" +
                       quote(result.horizon.is_operation ? "operation" : "scenario_steps");
  if (!result.horizon.is_operation) {
    output += ",\"steps\":" + std::to_string(result.horizon.steps);
  }
  output += "},\"values\":[";
  for (std::size_t index = 0; index < result.values.size(); ++index) {
    if (index != 0) output += ',';
    const auto& value = result.values[index];
    output += "{\"field\":" + quote(math_value_field_name(value.field)) +
              ",\"bits\":" + std::to_string(value.bits) +
              ",\"class\":" + quote(float_class_name(value.bits)) +
              ",\"negative\":" +
              ((value.bits & 0x80000000U) != 0 ? "true" : "false") + "}";
  }
  output += "],\"discrete\":[";
  for (std::size_t index = 0; index < result.discrete.size(); ++index) {
    if (index != 0) output += ',';
    const auto& discrete = result.discrete[index];
    output += "{\"field\":" + quote(math_discrete_field_name(discrete.field)) +
              ",\"value\":" + (discrete.value ? "true" : "false") + "}";
  }
  return output + "]}";
}

std::string encode_math_probe_end(
    const MathProbeRequest& request,
    std::uint32_t result_count,
    std::uint64_t reset_epoch) {
  return "{\"protocol_version\":1,\"record_kind\":\"math_probe_end\",\"request_id\":" +
         quote(request.request_id) + ",\"result_count\":" +
         std::to_string(result_count) + ",\"reset_epoch\":" +
         std::to_string(reset_epoch) +
         ",\"reset_verified\":true}";
}

bool read_bounded_record(std::istream& input, std::string& record) {
  record.clear();
  char byte = 0;
  while (input.get(byte)) {
    if (record.size() == kMaximumRecordBytes) {
      throw std::runtime_error("input record exceeds reviewed byte limit");
    }
    record.push_back(byte);
    if (byte == '\n') return true;
  }
  if (!input.eof()) {
    throw std::runtime_error("failed while reading protocol stdin");
  }
  return !record.empty();
}

void validate_bounded_json_record(std::string_view record) {
  static_cast<void>(decode_record_node(record));
}

void write_record(std::ostream& output, std::string_view record) {
  if (record.size() + 1 > kMaximumRecordBytes) {
    throw std::runtime_error("output record exceeds reviewed byte limit");
  }
  output << record << '\n';
  output.flush();
  if (!output) throw std::runtime_error("failed to write protocol record");
}
