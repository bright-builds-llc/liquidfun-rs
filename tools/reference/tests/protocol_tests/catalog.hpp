nlohmann::json catalog_payload() {
  return {
      {"identity",
       {{"catalog_schema_version", 1U},
        {"slug", "cpp-catalog-smoke"},
        {"scenario_version", 1U},
        {"generator_id", "cpp-catalog-test"},
        {"generator_version", 1U},
        {"maybe_seed", nullptr},
        {"settings",
         {{"timestep_bits", 0x3c888889U},
          {"velocity_iterations", 8U},
          {"position_iterations", 3U},
          {"particle_iterations", 2U}}}}},
      {"entities",
       {{{"semantic_id", {{"kind", "body"}, {"ordinal", 0U}}},
         {"scenario_id", "entity-body-0000"}},
        {{"semantic_id", {{"kind", "fixture"}, {"ordinal", 1U}}},
         {"scenario_id", "entity-fixture-0001"}}}},
      {"actions",
       {{{"action_id", "action-0000"},
         {"schedule", {{"kind", "setup"}, {"ordinal", 0U}}},
         {"action", {{"kind", "create_body"},
                     {"body_id", "entity-body-0000"}}}},
        {{"action_id", "action-0001"},
         {"schedule", {{"kind", "setup"}, {"ordinal", 1U}}},
         {"action", {{"kind", "create_fixture"},
                     {"fixture_id", "entity-fixture-0001"}}}},
        {{"action_id", "action-0002"},
         {"schedule", {{"kind", "logical_step"}, {"ordinal", 1U}}},
         {"action",
          {{"kind", "configured_step"},
           {"timestep_bits", 0x3c888889U},
           {"velocity_iterations", 8U},
           {"position_iterations", 3U},
           {"continuous_work_budget", 1U}}}}}},
      {"checkpoints",
       {{{"checkpoint_id", "checkpoint-0001"},
         {"after_action_id", "action-0002"},
         {"logical_step", 1U}}}}};
}

nlohmann::json catalog_distance_joint_payload() {
  auto payload = catalog_payload();
  payload["identity"]["slug"] = "joint-distance-behavior";
  payload["identity"]["generator_id"] = "native-joint-v1";
  payload["entities"] = {
      {{"semantic_id", {{"kind", "body"}, {"ordinal", 0U}}},
       {"scenario_id", "entity-body-0000"}},
      {{"semantic_id", {{"kind", "body"}, {"ordinal", 1U}}},
       {"scenario_id", "entity-body-0001"}},
      {{"semantic_id", {{"kind", "joint"}, {"ordinal", 2U}}},
       {"scenario_id", "entity-joint-0002"}}};
  payload["actions"] = {
      {{"action_id", "action-0000"},
       {"schedule", {{"kind", "setup"}, {"ordinal", 0U}}},
       {"action",
        {{"kind", "create_body"}, {"body_id", "entity-body-0000"}}}},
      {{"action_id", "action-0001"},
       {"schedule", {{"kind", "setup"}, {"ordinal", 1U}}},
       {"action",
        {{"kind", "create_body"}, {"body_id", "entity-body-0001"}}}},
      {{"action_id", "action-0002"},
       {"schedule", {{"kind", "setup"}, {"ordinal", 2U}}},
       {"action",
        {{"kind", "create_joint"}, {"joint_id", "entity-joint-0002"}}}},
      {{"action_id", "action-0003"},
       {"schedule", {{"kind", "setup"}, {"ordinal", 3U}}},
       {"action",
        {{"kind", "mutate_joint"},
         {"joint_id", "entity-joint-0002"},
         {"mutation", {{"kind", "length"}, {"length_bits", 0x40000000U}}}}}},
      {{"action_id", "action-0004"},
       {"schedule", {{"kind", "logical_step"}, {"ordinal", 1U}}},
       {"action",
        {{"kind", "configured_step"},
         {"timestep_bits", 0x3c888889U},
         {"velocity_iterations", 8U},
         {"position_iterations", 3U},
         {"continuous_work_budget", 1U}}}}};
  payload["checkpoints"] = {
      {{"checkpoint_id", "checkpoint-0001"},
       {"after_action_id", "action-0004"},
       {"logical_step", 1U}}};
  return payload;
}

nlohmann::json catalog_large_resolved_payload() {
  auto payload = catalog_payload();
  payload["identity"]["slug"] = "rigid-stack-stability";
  payload["identity"]["generator_id"] = "native-rigid-v1";
  payload["entities"] = nlohmann::json::array();
  payload["actions"] = nlohmann::json::array();
  for (std::uint32_t ordinal = 0; ordinal < 32U; ++ordinal) {
    std::ostringstream entity_id;
    entity_id << "entity-body-" << std::setw(4) << std::setfill('0')
              << ordinal;
    std::ostringstream action_id;
    action_id << "action-" << std::setw(4) << std::setfill('0') << ordinal;
    payload["entities"].push_back(
        {{"semantic_id", {{"kind", "body"}, {"ordinal", ordinal}}},
         {"scenario_id", entity_id.str()}});
    payload["actions"].push_back(
        {{"action_id", action_id.str()},
         {"schedule", {{"kind", "setup"}, {"ordinal", ordinal}}},
         {"action", {{"kind", "create_body"},
                     {"body_id", entity_id.str()}}}});
  }
  payload["actions"].push_back(
      {{"action_id", "action-0032"},
       {"schedule", {{"kind", "logical_step"}, {"ordinal", 1U}}},
       {"action",
        {{"kind", "configured_step"},
         {"timestep_bits", 0x3c888889U},
         {"velocity_iterations", 8U},
         {"position_iterations", 3U},
         {"continuous_work_budget", 1U}}}});
  payload["checkpoints"] = {
      {{"checkpoint_id", "checkpoint-0001"},
       {"after_action_id", "action-0032"},
       {"logical_step", 1U}}};
  return payload;
}

std::string catalog_request_from_payload(
    const nlohmann::json& payload,
    std::string_view identity_sha256) {
  const auto payload_text = payload.dump();
  const auto bytes = std::vector<std::uint8_t>(
      payload_text.begin(), payload_text.end());
  const auto resolved_sha256 = liquidfun::reference::sha256_hex(payload_text);
  const auto& identity = payload.at("identity");
  return nlohmann::json{
      {"protocol_version", 1U},
      {"record_kind", "catalog_run_request"},
      {"request_id", "cpp-catalog-request"},
      {"catalog_schema_version", identity.at("catalog_schema_version")},
      {"slug", identity.at("slug")},
      {"scenario_version", identity.at("scenario_version")},
      {"generator_id", identity.at("generator_id")},
      {"generator_version", identity.at("generator_version")},
      {"maybe_seed", identity.at("maybe_seed")},
      {"settings", identity.at("settings")},
      {"resolved_bytes", bytes},
      {"resolved_sha256", resolved_sha256},
      {"provenance_requirements",
       {{"required_identity_sha256", identity_sha256},
        {"limits_profile_sha256", std::string(64, 'b')},
        {"evidence_tier", "d2_supported"}}}}
      .dump() +
      '\n';
}

std::string catalog_request(std::string_view identity_sha256) {
  return catalog_request_from_payload(catalog_payload(), identity_sha256);
}

std::string benchmark_request(
    bool profile_enabled = false,
    std::string_view workload = "world_step",
    std::string_view size_point = "fixed") {
  const auto payload_text = catalog_payload().dump();
  const auto bytes = std::vector<std::uint8_t>(
      payload_text.begin(), payload_text.end());
  return nlohmann::json{
      {"protocol_version", 1U},
      {"record_kind", "benchmark_run_request"},
      {"identity",
       {{"request_id", "cpp-benchmark-request"},
        {"resolved_sha256",
         liquidfun::reference::sha256_hex(payload_text)},
        {"settings",
         {{"timestep_bits", 0x3c888889U},
          {"velocity_iterations", 8U},
          {"position_iterations", 3U},
          {"particle_iterations", 2U}}},
        {"workload", workload},
        {"size_point", size_point},
        {"optimization_mode", "release_scalar"},
        {"warmup_count", 1U},
        {"measured_horizon", 1U},
        {"sample_ordinal", 1U},
        {"policy_sha256",
         "75c0253d9f1eaa0b4cd6097031ed85f3c530fe47606049b5ac060a5267a3f05f"},
        {"profile_enabled", profile_enabled}}},
      {"resolved_bytes", bytes}}
      .dump() +
      '\n';
}

class BenchmarkEventRecorder final
    : public liquidfun::reference::BenchmarkRunObserver {
 public:
  void observe(liquidfun::reference::BenchmarkRunEvent event) override {
    events.push_back(event);
  }

  std::vector<liquidfun::reference::BenchmarkRunEvent> events;
};

void benchmark_run_executes_with_strict_timing_boundaries() {
  // Arrange
  BenchmarkEventRecorder recorder;
  liquidfun::reference::BenchmarkRunAdapter adapter(&recorder);

  // Act
  const auto trace = adapter.execute(benchmark_request());
  const auto result = nlohmann::json::parse(trace.result_record);

  // Assert
  expect(trace.reset_epoch == 1U, "first benchmark reset epoch changed");
  expect(result.at("record_kind") == "benchmark_run_result",
         "benchmark result record kind changed");
  expect(result.at("engine_role") == "pinned_cpp_oracle",
         "benchmark result lost its engine role");
  expect(result.at("reset_epoch") == 1U,
         "benchmark result lost its reset identity");
  expect(result.at("outcome").at("outcome_kind") == "performance",
         "valid benchmark did not produce performance evidence");
  expect(
      result.at("outcome").at("outcome").at("unprofiled_nanoseconds")
          .get<std::uint64_t>() > 0U,
      "benchmark returned a zero authoritative duration");
  expect(
      recorder.events ==
          std::vector<liquidfun::reference::BenchmarkRunEvent>{
              liquidfun::reference::BenchmarkRunEvent::authority_prepared,
              liquidfun::reference::BenchmarkRunEvent::warmup_complete,
              liquidfun::reference::BenchmarkRunEvent::measured_unit_setup,
              liquidfun::reference::BenchmarkRunEvent::measured_setup_complete,
              liquidfun::reference::BenchmarkRunEvent::timer_started,
              liquidfun::reference::BenchmarkRunEvent::timer_stopped,
              liquidfun::reference::BenchmarkRunEvent::checkpoint_validated,
              liquidfun::reference::BenchmarkRunEvent::teardown_complete},
      "benchmark lifecycle crossed the authoritative timer boundary");
}

void benchmark_run_prepares_every_scalable_unit_before_timing() {
  // Arrange
  BenchmarkEventRecorder recorder;
  liquidfun::reference::BenchmarkRunAdapter adapter(&recorder);

  // Act
  static_cast<void>(
      adapter.execute(benchmark_request(false, "world_step", "work_units128")));

  // Assert
  const auto timer_started = std::find(
      recorder.events.begin(), recorder.events.end(),
      liquidfun::reference::BenchmarkRunEvent::timer_started);
  const auto timer_stopped = std::find(
      recorder.events.begin(), recorder.events.end(),
      liquidfun::reference::BenchmarkRunEvent::timer_stopped);
  expect(timer_started != recorder.events.end() &&
             timer_stopped != recorder.events.end(),
         "scalable benchmark timer boundary was not observed");
  expect(
      std::count(
          recorder.events.begin(), timer_started,
          liquidfun::reference::BenchmarkRunEvent::measured_unit_setup) ==
          128,
      "scalable benchmark did not prepare every unit before timing");
  expect(
      std::find(
          timer_started, timer_stopped,
          liquidfun::reference::BenchmarkRunEvent::measured_unit_setup) ==
          timer_stopped,
      "scalable benchmark constructed a unit inside the timer");
}

void benchmark_run_rejection_advances_epoch_and_recovers() {
  // Arrange
  auto rejected = nlohmann::json::parse(benchmark_request());
  rejected["identity"]["resolved_sha256"] = std::string(64, 'f');
  liquidfun::reference::BenchmarkRunAdapter adapter;

  // Act / Assert
  try {
    static_cast<void>(adapter.execute(rejected.dump() + '\n'));
  } catch (const std::exception& error) {
    expect(std::string(error.what()).find("hash mismatch") !=
               std::string::npos,
           "benchmark hash rejection produced an unstable diagnostic");
    const auto recovered = adapter.execute(benchmark_request());
    expect(recovered.reset_epoch == 2U,
           "benchmark rejection did not advance reset identity");
    const auto result = nlohmann::json::parse(recovered.result_record);
    expect(result.at("reset_epoch") == 2U,
           "recovered benchmark emitted a stale reset epoch");
    return;
  }
  throw std::runtime_error("benchmark request with wrong hash was accepted");
}

void benchmark_run_keeps_profile_diagnostics_non_authoritative() {
  // Arrange
  liquidfun::reference::BenchmarkRunAdapter adapter;

  // Act
  const auto trace =
      adapter.execute(benchmark_request(true, "broad_phase"));
  const auto result = nlohmann::json::parse(trace.result_record);
  const auto& outcome = result.at("outcome").at("outcome");
  const auto& diagnostics =
      outcome.at("maybe_common_parent_diagnostics");

  // Assert
  expect(outcome.at("unprofiled_nanoseconds").get<std::uint64_t>() > 0U,
         "profiled request lost authoritative unprofiled timing");
  expect(diagnostics.size() == 1U &&
             diagnostics.at(0).at("phase") == "broad_phase" &&
             diagnostics.at(0).at("nanoseconds").get<std::uint64_t>() > 0U,
         "profiled request lost its common-parent diagnostic");
  expect(!outcome.contains("profiled_nanoseconds"),
         "profiled total became benchmark authority");
}

void benchmark_run_rejects_malformed_and_bounded_inputs() {
  // Arrange
  auto wrong_horizon = nlohmann::json::parse(benchmark_request());
  wrong_horizon["identity"]["measured_horizon"] = 2U;
  auto unknown = nlohmann::json::parse(benchmark_request());
  unknown["private_slot"] = 7U;
  auto oversized =
      std::string(liquidfun::reference::kMaximumRecordBytes, 'x') + '\n';
  liquidfun::reference::BenchmarkRunAdapter adapter;

  // Act / Assert
  for (const auto& [record, expected] :
       std::array<std::pair<std::string, std::string>, 3>{
           std::pair{wrong_horizon.dump() + '\n', "horizon"},
           std::pair{unknown.dump() + '\n', "unknown member"},
           std::pair{oversized, "reviewed byte limit"}}) {
    try {
      static_cast<void>(adapter.execute(record));
    } catch (const std::exception& error) {
      expect(std::string(error.what()).find(expected) != std::string::npos,
             "benchmark rejection produced an unstable diagnostic");
      continue;
    }
    throw std::runtime_error("invalid benchmark request was accepted");
  }
  const auto recovered = adapter.execute(benchmark_request());
  expect(recovered.reset_epoch == 4U,
         "benchmark rejection categories did not advance reset identity");
}

void catalog_run_executes_exact_resolved_bytes_and_reuses_cleanly() {
  // Arrange
  const auto identity_sha256 = std::string(64, 'a');
  const auto request = catalog_request(identity_sha256);
  liquidfun::reference::CatalogRunAdapter adapter;

  // Act
  const auto first = adapter.execute(request, identity_sha256);
  const auto second = adapter.execute(request, identity_sha256);

  // Assert
  expect(first.reset_epoch == 1U, "first catalog reset epoch changed");
  expect(second.reset_epoch == 2U, "second catalog reset epoch changed");
  expect(first.checkpoint_records == second.checkpoint_records,
         "catalog request leaked state across reuse");
  expect(first.checkpoint_records.size() == 1U,
         "catalog request emitted an unexpected checkpoint count");
  const auto checkpoint = nlohmann::json::parse(first.checkpoint_records.at(0));
  expect(checkpoint.at("record_kind") == "canonical_checkpoint",
         "catalog checkpoint record kind changed");
  expect(checkpoint.at("checkpoint_id") == "checkpoint-0001",
         "catalog checkpoint identity changed");
  expect(checkpoint.at("resolved_sha256") ==
             nlohmann::json::parse(request).at("resolved_sha256"),
         "catalog checkpoint lost exact resolved identity");
  expect(checkpoint.size() == 14U &&
             checkpoint.at("observations").size() == 6U &&
             checkpoint.at("numeric_observations").empty() &&
             checkpoint.at("ordered_occurrences").empty() &&
             checkpoint.at("unordered_sets").empty() &&
             checkpoint.at("debug_primitives").empty() &&
             checkpoint.at("profile_names").empty(),
         "catalog checkpoint diverged from the canonical schema");
  expect(checkpoint.at("observations")[0]["value"]["value"] == 1U &&
             checkpoint.at("observations")[3]["value"]["value"] == 1U,
         "catalog rigid semantics lost stable body or fixture counts");
  const auto text = first.checkpoint_records.at(0) + first.end_record;
  for (const auto forbidden :
       {"pointer", "dense_index", "arena_slot", "proxy_id", "duration"}) {
    expect(text.find(forbidden) == std::string::npos,
           "catalog result leaked private or nondeterministic state");
  }
}

void catalog_run_preserves_distance_joint_kind_and_mutation() {
  // Arrange
  const auto identity_sha256 = std::string(64, 'a');
  const auto request = catalog_request_from_payload(
      catalog_distance_joint_payload(), identity_sha256);
  liquidfun::reference::CatalogRunAdapter adapter;

  // Act
  const auto result = adapter.execute(request, identity_sha256);

  // Assert
  expect(result.checkpoint_records.size() == 1U,
         "distance joint scenario lost its checkpoint");
  const auto checkpoint = nlohmann::json::parse(result.checkpoint_records.at(0));
  expect(checkpoint.at("observations")[4]["value"]["value"] == 1U,
         "distance joint scenario did not create one typed joint");
}

void catalog_run_accepts_large_bounded_resolved_bytes() {
  // Arrange
  const auto identity_sha256 = std::string(64, 'a');
  const auto request = catalog_request_from_payload(
      catalog_large_resolved_payload(), identity_sha256);
  const auto resolved_size =
      nlohmann::json::parse(request).at("resolved_bytes").size();
  liquidfun::reference::CatalogRunAdapter adapter;

  // Act
  const auto result = adapter.execute(request, identity_sha256);

  // Assert
  expect(resolved_size > liquidfun::reference::kMaximumCollectionItems,
         "large catalog regression did not cross the generic bound");
  expect(resolved_size < liquidfun::reference::kMaximumRecordBytes,
         "large catalog regression crossed the record bound");
  expect(result.checkpoint_records.size() == 1U,
         "large bounded catalog request lost its checkpoint");
}

void catalog_run_rejects_hash_and_nested_shape_tampering() {
  // Arrange
  const auto identity_sha256 = std::string(64, 'a');
  auto wrong_hash = nlohmann::json::parse(catalog_request(identity_sha256));
  wrong_hash["resolved_sha256"] = std::string(64, 'f');
  auto unknown_member_payload = catalog_payload();
  unknown_member_payload["actions"][2]["action"]["private_row"] = 7U;
  const auto unknown_member =
      catalog_request_from_payload(unknown_member_payload, identity_sha256);
  liquidfun::reference::CatalogRunAdapter adapter;

  // Act / Assert
  for (const auto& [record, expected] :
       std::array<std::pair<std::string, std::string>, 2>{
           std::pair{wrong_hash.dump() + '\n', "hash mismatch"},
           std::pair{unknown_member, "invalid members"}}) {
    try {
      static_cast<void>(adapter.execute(record, identity_sha256));
    } catch (const std::exception& error) {
      expect(std::string(error.what()).find(expected) != std::string::npos,
             "catalog tampering produced an unstable diagnostic");
      continue;
    }
    throw std::runtime_error("tampered catalog request was accepted");
  }
}

void catalog_run_rejection_does_not_poison_the_next_request() {
  // Arrange
  const auto identity_sha256 = std::string(64, 'a');
  auto rejected = catalog_request(identity_sha256);
  rejected.insert(1, "\"protocol_version\":1,");
  const auto valid = catalog_request(identity_sha256);
  liquidfun::reference::CatalogRunAdapter adapter;

  // Act / Assert
  try {
    static_cast<void>(adapter.execute(rejected, identity_sha256));
  } catch (const std::exception& error) {
    expect(std::string(error.what()).find("duplicate member") !=
               std::string::npos,
           "duplicate catalog request produced an unstable diagnostic");
    const auto recovered = adapter.execute(valid, identity_sha256);
    expect(recovered.reset_epoch == 1U,
           "rejected catalog request advanced reset state");
    return;
  }
  throw std::runtime_error("duplicate catalog request was accepted");
}

void catalog_run_rejects_oversized_input_before_allocation() {
  // Arrange
  const auto identity_sha256 = std::string(64, 'a');
  auto oversized = std::string(liquidfun::reference::kMaximumRecordBytes, 'x');
  oversized.push_back('\n');
  liquidfun::reference::CatalogRunAdapter adapter;

  // Act / Assert
  try {
    static_cast<void>(adapter.execute(oversized, identity_sha256));
  } catch (const std::exception& error) {
    expect(std::string(error.what()).find("reviewed byte limit") !=
               std::string::npos,
           "oversized catalog input produced an unstable diagnostic");
    return;
  }
  throw std::runtime_error("oversized catalog input was accepted");
}

using liquidfun::reference::BuildIdentity;
using liquidfun::reference::OracleAdapter;
using liquidfun::reference::RigidWorldAdapter;
using liquidfun::reference::decode_rigid_world_request;
using liquidfun::reference::decode_scenario_request;
using liquidfun::reference::encode_scenario_request;
using liquidfun::reference::write_record;
