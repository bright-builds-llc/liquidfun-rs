#include "benchmark_run.hpp"
#include "collision_probe.hpp"
#include "catalog_run.hpp"
#include "oracle_adapter.hpp"
#include "protocol.hpp"
#include "rigid_world.hpp"

#include "../vendor/nlohmann/json.hpp"

#include <algorithm>
#include <array>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <set>
#include <sstream>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace {

void expect(bool condition, const std::string& message);

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

std::string read_fixture(const std::string& relative_path) {
  const auto path =
      std::filesystem::path(LIQUIDFUN_REPOSITORY_ROOT) / relative_path;
  std::ifstream input(path, std::ios::binary);
  if (!input) {
    throw std::runtime_error("could not open fixture: " + path.string());
  }
  return std::string(
      std::istreambuf_iterator<char>(input),
      std::istreambuf_iterator<char>());
}

void expect(bool condition, const std::string& message) {
  if (!condition) {
    throw std::runtime_error(message);
  }
}

void expect_rejected(
    const std::string& record,
    const std::string& expected_message) {
  try {
    static_cast<void>(decode_scenario_request(record));
  } catch (const std::exception& error) {
    expect(
        std::string(error.what()).find(expected_message) != std::string::npos,
        "unexpected rejection: " + std::string(error.what()));
    return;
  }
  throw std::runtime_error("record was unexpectedly accepted");
}

BuildIdentity fixture_identity() {
  BuildIdentity identity;
  identity.oracle_revision = "7f20402173fd143a3988c921bc384459c6a858f2";
  identity.adapter_revision = "fixture-adapter-v1";
  identity.adapter_content_sha256 =
      "c7f36eaf2f184a36b9c9a04636d3e22785d815c4948d55d0b3cbf44ee7245fc8";
  identity.cmake_preset = "oracle-debug";
  identity.compiler_id = "Clang";
  identity.compiler_version = "22.1.8";
  identity.target = "x86_64-unknown-linux-gnu";
  identity.build_type = "Debug";
  identity.effective_compile_flags = "-O0 -g";
  identity.effective_link_flags = "-lc++";
  identity.sanitizer_mode = "none";
  return identity;
}

std::vector<std::string> split_jsonl(const std::string& jsonl) {
  std::vector<std::string> records;
  std::istringstream input(jsonl);
  std::string record;
  while (std::getline(input, record)) {
    records.push_back(record);
  }
  return records;
}

nlohmann::json& custom_mass_action(nlohmann::json& request) {
  auto& actions =
      request.at("scenario").at("timelines").at(0).at("actions");
  auto found = std::find_if(
      actions.begin(), actions.end(), [](const auto& action) {
        return action.at("action_id") == "nc-custom-mass";
      });
  expect(found != actions.end(), "custom mass action is missing");
  return found->at("action");
}

nlohmann::json& query_timeline(nlohmann::json& request) {
  auto& timelines = request.at("scenario").at("timelines");
  const auto found = std::find_if(
      timelines.begin(), timelines.end(), [](const auto& timeline) {
        return timeline.at("witness_family") == "world_query_and_ray_cast";
      });
  expect(found != timelines.end(), "query timeline is missing");
  return *found;
}

void accepted_fixture_round_trips_exact_bits() {
  // Arrange
  const auto fixture = read_fixture(
      "protocol/fixtures/accepted/empty-world-request.jsonl");

  // Act
  const auto request = decode_scenario_request(fixture);
  const auto encoded = encode_scenario_request(request);

  // Assert
  expect(request.scenario.gravity_x_bits == 0, "gravity x bits changed");
  expect(
      request.scenario.gravity_y_bits == 3240099840U,
      "gravity y bits changed");
  expect(
      request.scenario.commands.front().timestep_bits == 1056964608U,
      "timestep bits changed");
  expect(encoded == fixture, "accepted fixture did not round trip exactly");
}

void framing_and_shape_fail_closed() {
  // Arrange
  const auto duplicate =
      read_fixture("protocol/fixtures/rejected/duplicate-member.jsonl");
  const auto partial =
      read_fixture("protocol/fixtures/rejected/partial-record.jsonl");
  const std::string invalid_utf8 = std::string("{\"x\":\"") +
                                   static_cast<char>(0xFF) + "\"}\n";

  // Act / Assert
  expect_rejected(duplicate, "duplicate member");
  expect_rejected(partial, "newline");
  expect_rejected(invalid_utf8, "parse");
}

void unknown_versions_members_and_kinds_fail_closed() {
  // Arrange
  const auto accepted = read_fixture(
      "protocol/fixtures/accepted/empty-world-request.jsonl");
  const auto unsupported_version =
      read_fixture("protocol/fixtures/rejected/unsupported-version.jsonl");
  const auto unknown_kind =
      read_fixture("protocol/fixtures/rejected/unknown-record-kind.jsonl");
  const auto oversized_id =
      read_fixture("protocol/fixtures/rejected/oversized-id.jsonl");
  auto unknown_member = accepted;
  unknown_member.insert(1, "\"unexpected\":true,");

  // Act / Assert
  expect_rejected(unsupported_version, "unsupported protocol version");
  expect_rejected(unknown_kind, "unsupported record kind");
  expect_rejected(oversized_id, "valid ID");
  expect_rejected(unknown_member, "unknown member");
}

void parser_bounds_fail_before_execution() {
  // Arrange
  std::string excessive_depth;
  for (std::size_t index = 0; index < 33; ++index) {
    excessive_depth += '[';
  }
  for (std::size_t index = 0; index < 33; ++index) {
    excessive_depth += ']';
  }
  excessive_depth += '\n';
  const auto oversized_string =
      std::string("{\"value\":\"") + std::string(4097, 'a') + "\"}\n";
  const auto oversized_record =
      std::string(liquidfun::reference::kMaximumRecordBytes, ' ') + "\n";
  std::string oversized_collection = "[";
  for (std::size_t index = 0; index < 4097; ++index) {
    if (index != 0) {
      oversized_collection += ',';
    }
    oversized_collection += '0';
  }
  oversized_collection += "]\n";

  // Act / Assert
  expect_rejected(excessive_depth, "depth");
  expect_rejected(oversized_string, "string");
  expect_rejected(oversized_record, "byte limit");
  expect_rejected(oversized_collection, "collection");
}

void scenario_references_and_phase_scope_are_validated() {
  // Arrange
  const auto accepted = read_fixture(
      "protocol/fixtures/accepted/empty-world-request.jsonl");
  const auto empty_phase = read_fixture(
      "protocol/fixtures/rejected/empty-checkpoint-phase.jsonl");
  auto bad_reference = accepted;
  bad_reference.replace(
      bad_reference.find("\"after_command_id\":\"step-1\""),
      std::string("\"after_command_id\":\"step-1\"").size(),
      "\"after_command_id\":\"missing\"");
  auto nonempty_entities = accepted;
  nonempty_entities.replace(
      nonempty_entities.find("\"entities\":[]"),
      std::string("\"entities\":[]").size(),
      "\"entities\":[{}]");

  // Act / Assert
  expect_rejected(bad_reference, "command reference");
  expect_rejected(nonempty_entities, "entities must be empty");
  expect_rejected(empty_phase, "checkpoint phase must not be empty");
}

void reused_adapter_resets_between_requests() {
  // Arrange
  auto first = decode_scenario_request(read_fixture(
      "protocol/fixtures/accepted/empty-world-request.jsonl"));
  auto second = first;
  second.request_id = "second-request";
  second.scenario.scenario_id = "empty-world-second";
  second.scenario.commands.resize(1);
  second.scenario.checkpoints.resize(1);
  OracleAdapter adapter;
  const auto identity = liquidfun::reference::build_identity_sha256(
      fixture_identity());

  // Act
  const auto first_trace = adapter.execute(first, identity);
  const auto second_trace = adapter.execute(second, identity);

  // Assert
  expect(first_trace.reset_verified, "first reset was not verified");
  expect(second_trace.reset_verified, "second reset was not verified");
  expect(first_trace.reset_epoch == 1, "first reset epoch was not one");
  expect(second_trace.reset_epoch == 2, "second reset epoch was not two");
  expect(first_trace.records.size() == 4, "first trace count leaked");
  expect(second_trace.records.size() == 3, "second trace count leaked");
}

void adapter_matches_the_cross_language_trace_fixture() {
  // Arrange
  const auto request = decode_scenario_request(read_fixture(
      "protocol/fixtures/accepted/empty-world-request.jsonl"));
  const auto fixture_records = split_jsonl(read_fixture(
      "protocol/fixtures/accepted/empty-world-trace.jsonl"));
  OracleAdapter adapter;
  const auto identity = liquidfun::reference::build_identity_sha256(
      fixture_identity());

  // Act
  const auto trace = adapter.execute(request, identity);

  // Assert
  expect(
      identity == "56b1b4d459fef5fc7abcd7072566ac92732284e73f99c79885a80770a9f0fafd",
      "build identity hash differs from the Rust protocol authority");
  expect(fixture_records.size() == trace.records.size() + 1, "fixture shape changed");
  for (std::size_t index = 0; index < trace.records.size(); ++index) {
    expect(
        fixture_records[index + 1] == trace.records[index],
        "C++ trace record differs from accepted fixture at index " +
            std::to_string(index));
  }
}

void record_writer_keeps_stdout_protocol_only() {
  // Arrange
  std::ostringstream output;

  // Act
  write_record(output, "{\"record_kind\":\"checkpoint\"}");

  // Assert
  expect(
      output.str() == "{\"record_kind\":\"checkpoint\"}\n",
      "record writer added non-protocol output or wrong framing");
}

void protocol_bits_preserve_exceptional_classes() {
  // Arrange
  const std::vector<std::uint32_t> bits{
      0x00000000U, 0x80000000U, 0x00000001U, 0x007FFFFFU,
      0x7F800000U, 0xFF800000U, 0x7FC00042U, 0x7FA00001U};

  // Act / Assert
  for (const auto value : bits) {
    expect(
        liquidfun::reference::bits_from_float(
            liquidfun::reference::float_from_bits(value)) == value,
        "exceptional float bits changed during transport");
  }
}

void math_probe_matches_operation_contract() {
  // Arrange
  const auto fixture = read_fixture(
      "protocol/fixtures/accepted/math-probe-request.jsonl");
  const auto request = liquidfun::reference::decode_math_probe_request(fixture);
  auto unknown = fixture;
  unknown.replace(unknown.find("\"is_valid\""),
                  std::string("\"is_valid\"").size(), "\"run_function\"");

  // Act
  const auto results = liquidfun::reference::execute_math_probe(request);
  std::set<liquidfun::reference::MathProbeOperation> operations;
  for (const auto& result : results) operations.insert(result.operation);

  // Assert
  expect(results.size() == 39, "math probe corpus result count changed");
  expect(operations.size() == 24, "math probe operation coverage is incomplete");
  try {
    static_cast<void>(liquidfun::reference::decode_math_probe_request(unknown));
  } catch (const std::exception& error) {
    expect(
        std::string(error.what()).find("unsupported math probe operation") !=
            std::string::npos,
        "unknown operation produced the wrong rejection");
    return;
  }
  throw std::runtime_error("unknown math probe operation was accepted");
}

void collision_probe_uses_existing_protocol_loop() {
  const auto fixture = read_fixture(
      "protocol/fixtures/accepted/collision-probe-request.jsonl");
  expect(
      liquidfun::reference::decode_request_kind(fixture) ==
          liquidfun::reference::RequestKind::collision_probe,
      "collision request kind should share the existing loop");
  const auto batch = liquidfun::reference::execute_collision_probe(fixture);
  expect(
      batch.result_records.size() == 78,
      "collision request should emit every required witness family");
  expect(
      liquidfun::reference::encode_collision_probe_end(batch, 1).find(
          "collision_probe_end") != std::string::npos,
      "collision request should emit its terminal record");
}

void rigid_world_executes_all_complete_witness_families() {
  // Arrange
  const auto fixture = read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl");
  RigidWorldAdapter adapter;

  // Act
  const auto trace = adapter.execute(fixture);
  const auto result = nlohmann::json::parse(trace.result_record);

  // Assert
  expect(trace.reset_verified, "rigid-world reset was not verified");
  expect(trace.reset_epoch == 1, "first rigid-world reset epoch was not one");
  expect(
      liquidfun::reference::decode_request_kind(fixture) ==
          liquidfun::reference::RequestKind::rigid_world,
      "rigid-world request did not use the existing process loop");
  expect(
      result.at("record_kind") == "rigid_world_result",
      "rigid-world result kind changed");
  const std::array expected_families{
      "non_colliding_body_fixture_lifecycle",
      "single_contact_lifecycle",
      "body_control_and_force_policy",
      "multi_contact_island_and_warm_start",
      "sleeping_and_waking",
      "continuous_collision_and_sub_stepping",
      "continuous_budget_resume",
      "world_query_and_ray_cast",
      "origin_shift_covariance",
      "joint_definitions_and_mutations",
      "revolute_prismatic_limits_and_motors",
      "distance_pulley_mouse_constraints",
      "wheel_weld_friction_rope_motor_constraints",
      "gear_dependencies_and_four_body_solver",
      "mixed_joint_island_order_and_collision_suppression",
      "standalone_rope_evolution",
      "contact_filter_listener_and_pre_solve_timing",
      "destruction_listener_and_dependency_cascades",
      "diagnostic_reconstruction_and_dump_order"};
  const std::array expected_phase7_checkpoints{
      "control-checkpoint",
      "island-checkpoint",
      "sleep-checkpoint",
      "ccd-checkpoint",
      "budget-checkpoint",
      "query-checkpoint",
      "origin-checkpoint"};
  const auto& timelines = result.at("timelines");
  expect(
      timelines.size() == expected_families.size(),
      "rigid witness families are incomplete");
  for (std::size_t index = 0; index < expected_families.size(); ++index) {
    expect(
        timelines.at(index).at("witness_family") == expected_families[index],
        "rigid witness family order changed");
  }
  for (std::size_t index = 0; index < expected_phase7_checkpoints.size(); ++index) {
    const auto& checkpoints = timelines.at(index + 2).at("checkpoints");
    expect(
        checkpoints.size() == 1 &&
            checkpoints.at(0).at("checkpoint_id") ==
                expected_phase7_checkpoints[index],
        "Phase 7 rigid checkpoint coverage is incomplete");
  }
  const auto& joint_observations =
      timelines.at(9).at("checkpoints").at(0).at("observations");
  expect(joint_observations.size() == 23, "Phase 8 joint coverage is incomplete");
  expect(
      joint_observations.at(10).at("snapshot").at("dependencies").size() == 2,
      "Phase 8 gear dependencies are incomplete");
  std::set<std::string> stepped_joint_kinds;
  for (std::size_t index = 12; index < joint_observations.size(); ++index) {
    stepped_joint_kinds.insert(
        joint_observations.at(index).at("snapshot").at("joint_kind"));
  }
  expect(
      stepped_joint_kinds.size() == 11,
      "Phase 8 did not inspect every pinned joint kind after stepping");
  const auto nontrivial_joint_count = std::count_if(
          joint_observations.begin() + 12,
          joint_observations.end(),
          [](const auto& observation) {
            const auto& snapshot = observation.at("snapshot");
            return snapshot.at("branch_state") != "inactive" ||
                   snapshot.at("coordinate_bits") != 0U ||
                   snapshot.at("speed_bits") != 0U ||
                   snapshot.at("reaction_force").at("x_bits") != 0U ||
                   snapshot.at("reaction_force").at("y_bits") != 0U ||
                   snapshot.at("reaction_torque_bits") != 0U;
          });
  expect(
      nontrivial_joint_count >= 10,
      "Phase 8 post-step joint observations remained trivial");
  const auto& rope_observations =
      timelines.at(15).at("checkpoints").at(0).at("observations");
  expect(rope_observations.size() == 3, "Phase 8 rope coverage is incomplete");
  std::set<std::string> gear_ids;
  for (const auto& observation :
       timelines.at(13).at("checkpoints").at(0).at("observations")) {
    if (observation.at("kind") == "joint" &&
        observation.at("snapshot").at("joint_kind") == "gear") {
      gear_ids.insert(observation.at("snapshot").at("joint_id"));
    }
  }
  expect(
      gear_ids == std::set<std::string>{
                      "gear-0-joint", "gear-1-joint", "gear-2-joint",
                      "gear-3-joint"},
      "Phase 8 did not execute all four gear source combinations");
  const auto& diagnostic_observations =
      timelines.at(18).at("checkpoints").at(0).at("observations");
  expect(
      diagnostic_observations.size() == 17 &&
          diagnostic_observations.back().at("kind") == "diagnostics" &&
          std::any_of(
              diagnostic_observations.begin(),
              diagnostic_observations.end(),
              [](const auto& observation) {
                return observation.at("kind") == "reconstruction" &&
                       observation.at("record").at("support") ==
                           "unsupported_mouse_joint";
              }),
      "Phase 8 reconstruction or diagnostics are incomplete");
  const auto& callback_lifecycle =
      timelines.at(16).at("checkpoints").at(0).at("observations");
  expect(
      callback_lifecycle == nlohmann::json::parse(
          R"([{"kind":"lifecycle","event":{"ordinal":0,"kind":"filter_decision","maybe_contact":null,"maybe_entity_id":"callback-fa"}},{"kind":"lifecycle","event":{"ordinal":1,"kind":"filter_decision","maybe_contact":null,"maybe_entity_id":"callback-fa"}},{"kind":"lifecycle","event":{"ordinal":2,"kind":"contact_created","maybe_contact":{"fixture_a_id":"callback-fa","child_a":0,"fixture_b_id":"callback-fb","child_b":0,"occurrence":1},"maybe_entity_id":null}},{"kind":"lifecycle","event":{"ordinal":3,"kind":"begin_contact","maybe_contact":{"fixture_a_id":"callback-fa","child_a":0,"fixture_b_id":"callback-fb","child_b":0,"occurrence":1},"maybe_entity_id":null}},{"kind":"lifecycle","event":{"ordinal":4,"kind":"pre_solve","maybe_contact":{"fixture_a_id":"callback-fa","child_a":0,"fixture_b_id":"callback-fb","child_b":0,"occurrence":1},"maybe_entity_id":null}},{"kind":"lifecycle","event":{"ordinal":5,"kind":"post_solve","maybe_contact":{"fixture_a_id":"callback-fa","child_a":0,"fixture_b_id":"callback-fb","child_b":0,"occurrence":1},"maybe_entity_id":null}},{"kind":"lifecycle","event":{"ordinal":6,"kind":"pre_solve","maybe_contact":{"fixture_a_id":"callback-fa","child_a":0,"fixture_b_id":"callback-fb","child_b":0,"occurrence":1},"maybe_entity_id":null}},{"kind":"lifecycle","event":{"ordinal":7,"kind":"post_solve","maybe_contact":{"fixture_a_id":"callback-fa","child_a":0,"fixture_b_id":"callback-fb","child_b":0,"occurrence":1},"maybe_entity_id":null}},{"kind":"lifecycle","event":{"ordinal":8,"kind":"pre_solve","maybe_contact":{"fixture_a_id":"callback-fa","child_a":0,"fixture_b_id":"callback-fb","child_b":0,"occurrence":1},"maybe_entity_id":null}}])"),
      "Phase 8 callback lifecycle order or multiplicity changed");
  const auto& destruction_observations =
      timelines.at(17).at("checkpoints").at(0).at("observations");
  std::vector<std::string> destruction_kinds;
  for (const auto& observation : destruction_observations) {
    if (observation.at("kind") == "lifecycle") {
      destruction_kinds.push_back(observation.at("event").at("kind"));
    }
  }
  expect(
      destruction_kinds == std::vector<std::string>{
          "filter_decision", "filter_decision", "contact_created",
          "begin_contact", "pre_solve", "post_solve", "pre_solve",
          "post_solve", "joint_goodbye", "end_contact",
          "contact_destroyed", "fixture_goodbye", "body_destroyed"},
      "Phase 8 destruction lifecycle order or multiplicity changed");
  const auto& non_colliding = result.at("timelines").at(0).at("checkpoints");
  const auto& single_contact = result.at("timelines").at(1).at("checkpoints");
  expect(non_colliding.size() == 8, "non-colliding checkpoints are incomplete");
  expect(single_contact.size() == 10, "contact checkpoints are incomplete");
  expect(
      non_colliding.at(1).at("checkpoint_id") ==
              "nc-static-kinematic-rejected" &&
          non_colliding.at(1).at("counts").at("contacts") == 0 &&
          non_colliding.at(1).at("counts").at("manifold_points") == 0 &&
          non_colliding.at(1).at("counts").at("events") == 0,
      "static/kinematic admission checkpoint changed");
  expect(
      non_colliding.at(3).at("checkpoint_id") ==
              "nc-kinematic-kinematic-rejected" &&
          non_colliding.at(3).at("counts").at("contacts") == 0 &&
          non_colliding.at(3).at("counts").at("manifold_points") == 0 &&
          non_colliding.at(3).at("counts").at("events") == 0,
      "kinematic/kinematic admission checkpoint changed");
  const auto& begin = single_contact.at(1);
  expect(
      begin.at("events") == nlohmann::json::parse(
          R"([{"kind":"created","contact":{"fixture_a_id":"contact-static-fixture","child_a":0,"fixture_b_id":"contact-dynamic-fixture","child_b":0,"occurrence":1}},{"kind":"begin","contact":{"fixture_a_id":"contact-static-fixture","child_a":0,"fixture_b_id":"contact-dynamic-fixture","child_b":0,"occurrence":1}},{"kind":"pre_solve","contact":{"fixture_a_id":"contact-static-fixture","child_a":0,"fixture_b_id":"contact-dynamic-fixture","child_b":0,"occurrence":1}},{"kind":"post_solve","contact":{"fixture_a_id":"contact-static-fixture","child_a":0,"fixture_b_id":"contact-dynamic-fixture","child_b":0,"occurrence":1}}])"),
      "contact begin event order or identity changed");
  expect(
      begin.at("contacts").at(0).at("maybe_manifold").at("points").size() == 1,
      "active contact manifold is incomplete");
  expect(
      single_contact.at(3).at("contacts").at(0).at("sensor") &&
          single_contact.at(3).at("contacts").at(0).at("maybe_manifold").is_null(),
      "sensor contact exposed an inactive manifold payload");
  expect(
      single_contact.at(8).at("destructions").at(0).at("kind") == "contact" &&
          single_contact.at(8).at("destructions").at(1).at("kind") == "fixture",
      "fixture teardown order changed");
  expect(
      single_contact.at(9).at("destructions").at(0).at("body_id") ==
              "contact-dynamic" &&
          single_contact.at(9).at("destructions").at(1).at("body_id") ==
              "contact-static",
      "body teardown order changed");
  expect(
      trace.result_record.find("pointer") == std::string::npos &&
          trace.result_record.find("address") == std::string::npos,
      "rigid trace leaked layout identity");
  expect(
      trace.end_record.find("\"reset_verified\":true") != std::string::npos,
      "terminal rigid-world reset proof is missing");
}

void rigid_world_rejects_expanding_ray_clip_during_execution() {
  // Arrange
  auto request = nlohmann::json::parse(read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl"));
  auto& timelines = request.at("scenario").at("timelines");
  const auto query_timeline = std::find_if(
      timelines.begin(), timelines.end(), [](const auto& timeline) {
        return timeline.at("witness_family") == "world_query_and_ray_cast";
      });
  expect(query_timeline != timelines.end(), "query timeline is missing");
  auto& actions = query_timeline->at("actions");
  const auto ray_action = std::find_if(
      actions.begin(), actions.end(), [](const auto& action) {
        return action.at("action_id") == "query-10";
      });
  expect(ray_action != actions.end(), "clip action is missing");
  ray_action->at("action")["directive_rules"] = nlohmann::json::parse(
      R"([{"target":{"fixture_id":"query-right-fixture","child_index":0},"directive":{"kind":"clip","fraction_bits":1056964608}},{"target":{"fixture_id":"query-center-fixture","child_index":0},"directive":{"kind":"clip","fraction_bits":1061158912}}])");
  RigidWorldAdapter adapter;

  // Act / Assert
  try {
    static_cast<void>(adapter.execute(request.dump() + '\n'));
  } catch (const std::exception& error) {
    expect(
        std::string(error.what()).find("expand current interval") !=
            std::string::npos,
        "expanding clip produced an unstable diagnostic");
    return;
  }
  throw std::runtime_error("expanding ray clip was accepted");
}

void rigid_world_rejects_signed_zero_clips_before_execution() {
  // Arrange and Act / Assert
  for (const auto fraction_bits :
       std::array<std::uint32_t, 2>{0U, 0x80000000U}) {
    auto request = nlohmann::json::parse(read_fixture(
        "protocol/fixtures/accepted/rigid-world-request.jsonl"));
    auto& timeline = query_timeline(request);
    for (auto& body : timeline.at("bodies")) {
      if (body.at("body_id") == "query-left" ||
          body.at("body_id") == "query-center") {
        body["transform"]["position"]["x_bits"] = 0xc0400000U;
      }
    }
    auto& actions = timeline.at("actions");
    const auto ray_action = std::find_if(
        actions.begin(), actions.end(), [](const auto& action) {
          return action.at("action_id") == "query-10";
        });
    expect(ray_action != actions.end(), "clip action is missing");
    ray_action->at("action")["directive_rules"][0]["directive"]
              ["fraction_bits"] = fraction_bits;

    try {
      static_cast<void>(decode_rigid_world_request(request.dump() + '\n'));
    } catch (const std::exception& error) {
      expect(
          std::string(error.what()).find("outside reviewed bounds") !=
              std::string::npos,
          "signed-zero clip produced an unstable diagnostic");
      continue;
    }
    throw std::runtime_error(
        "signed-zero clip reached multiple fraction-zero hit execution");
  }
}

void rigid_world_rejects_invalid_derived_ray_geometry_before_execution() {
  // Arrange and Act / Assert
  const std::array<std::array<std::uint32_t, 4>, 4> endpoints{{
      {0U, 0U, 0x80000000U, 0U},
      {0U, 0U, 1U, 0U},
      {0xff7fffffU, 0U, 0x7f7fffffU, 0U},
      {0U, 0U, 0x7f7fffffU, 0U},
  }};
  for (const auto& endpoint : endpoints) {
    auto request = nlohmann::json::parse(read_fixture(
        "protocol/fixtures/accepted/rigid-world-request.jsonl"));
    auto& actions = query_timeline(request).at("actions");
    const auto ray_action = std::find_if(
        actions.begin(), actions.end(), [](const auto& action) {
          return action.at("action_id") == "query-08";
        });
    expect(ray_action != actions.end(), "ray action is missing");
    ray_action->at("action")["start"] = {
        {"x_bits", endpoint[0]}, {"y_bits", endpoint[1]}};
    ray_action->at("action")["end"] = {
        {"x_bits", endpoint[2]}, {"y_bits", endpoint[3]}};

    try {
      static_cast<void>(decode_rigid_world_request(request.dump() + '\n'));
    } catch (const std::exception& error) {
      expect(
          std::string(error.what()).find("finite non-zero squared direction") !=
              std::string::npos,
          "derived ray geometry produced an unstable diagnostic");
      continue;
    }
    throw std::runtime_error("invalid derived ray geometry reached execution");
  }
}

void rigid_world_rejects_invalid_selector_children_before_execution() {
  // Arrange and Act / Assert
  for (const auto& test_case :
       std::array<std::pair<std::string, std::string>, 2>{
           std::pair{"query-07", "query directive"},
           std::pair{"query-11", "ray directive"}}) {
    const auto& action_id = test_case.first;
    const auto& context = test_case.second;
    auto request = nlohmann::json::parse(read_fixture(
        "protocol/fixtures/accepted/rigid-world-request.jsonl"));
    auto& actions = query_timeline(request).at("actions");
    const auto selected_action = std::find_if(
        actions.begin(), actions.end(), [&](const auto& action) {
          return action.at("action_id") == action_id;
        });
    expect(selected_action != actions.end(), "selector action is missing");
    selected_action->at("action")["directive_rules"][0]["target"]
                   ["child_index"] = 1U;
    RigidWorldAdapter adapter;

    try {
      static_cast<void>(adapter.execute(request.dump() + '\n'));
    } catch (const std::exception& error) {
      expect(
          std::string(error.what()).find(
              context + " references invalid fixture child") !=
              std::string::npos,
          "invalid selector child produced an unstable diagnostic");
      continue;
    }
    throw std::runtime_error(
        "invalid selector child reached rigid-world execution");
  }
}

void rigid_world_rejects_untrusted_records_before_execution() {
  // Arrange
  const auto fixture = read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl");
  auto duplicate = fixture;
  duplicate.insert(1, "\"protocol_version\":1,");
  auto unknown = fixture;
  unknown.insert(1, "\"unexpected\":true,");
  auto out_of_order_json = nlohmann::json::parse(fixture);
  auto& out_of_order_actions =
      out_of_order_json.at("scenario").at("timelines").at(0).at("actions");
  std::swap(out_of_order_actions.at(0), out_of_order_actions.at(3));
  const auto out_of_order = out_of_order_json.dump() + '\n';
  auto oversized = nlohmann::json::parse(fixture);
  auto& actions = oversized.at("scenario").at("timelines").at(0).at("actions");
  while (actions.size() <= liquidfun::reference::kRigidWorldMaximumActions) {
    actions.push_back(actions.back());
  }
  const auto oversized_record = oversized.dump() + '\n';
  auto missing_static_kinematic = fixture;
  const auto static_kinematic =
      missing_static_kinematic.find("static_kinematic_overlap_rejected");
  expect(
      static_kinematic != std::string::npos,
      "static/kinematic admission witness is missing from fixture");
  missing_static_kinematic.replace(
      static_kinematic,
      std::string("static_kinematic_overlap_rejected").size(),
      "removed_static_kinematic_witness");
  auto missing_kinematic_kinematic = fixture;
  const auto kinematic_kinematic =
      missing_kinematic_kinematic.find("kinematic_kinematic_overlap_rejected");
  expect(
      kinematic_kinematic != std::string::npos,
      "kinematic/kinematic admission witness is missing from fixture");
  missing_kinematic_kinematic.replace(
      kinematic_kinematic,
      std::string("kinematic_kinematic_overlap_rejected").size(),
      "removed_kinematic_kinematic_witness");

  // Act / Assert
  for (const auto& [record, expected] :
       std::vector<std::pair<std::string, std::string>>{
           {duplicate, "duplicate member"},
           {unknown, "unknown member"},
           {out_of_order, "action order"},
           {oversized_record, "action count"},
           {missing_static_kinematic, "witness registry is incomplete"},
           {missing_kinematic_kinematic, "witness registry is incomplete"}}) {
    try {
      static_cast<void>(decode_rigid_world_request(record));
    } catch (const std::exception& error) {
      expect(
          std::string(error.what()).find(expected) != std::string::npos,
          "unexpected rigid rejection: " + std::string(error.what()));
      continue;
    }
    throw std::runtime_error("untrusted rigid record was accepted");
  }
}

void rigid_world_boundary_matches_the_fixed_rust_contract() {
  // Arrange
  const auto fixture = read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl");
  auto maximum = nlohmann::json::parse(fixture);
  auto& maximum_actions =
      maximum.at("scenario").at("timelines").at(0).at("actions");
  const auto inspect_template = maximum_actions.at(9);
  while (maximum_actions.size() <
         liquidfun::reference::kRigidWorldMaximumActions) {
    auto action = inspect_template;
    action["action_id"] =
        "maximum-action-" + std::to_string(maximum_actions.size());
    maximum_actions.insert(maximum_actions.end() - 6, std::move(action));
  }
  auto maximum_plus_one = maximum;
  auto& maximum_plus_one_actions =
      maximum_plus_one.at("scenario").at("timelines").at(0).at("actions");
  auto extra = inspect_template;
  extra["action_id"] = "maximum-action-128";
  maximum_plus_one_actions.insert(
      maximum_plus_one_actions.end() - 6, std::move(extra));

  std::vector<nlohmann::json> alternate_steps;
  for (const auto& [field, value] :
       std::vector<std::pair<std::string, std::uint32_t>>{
           {"timestep_bits", liquidfun::reference::kRigidWorldTimestepBits + 1},
           {"velocity_iterations",
            liquidfun::reference::kRigidWorldVelocityIterations + 1},
           {"position_iterations",
            liquidfun::reference::kRigidWorldPositionIterations + 1}}) {
    auto alternate = nlohmann::json::parse(fixture);
    auto& timeline_actions =
        alternate.at("scenario").at("timelines").at(0).at("actions");
    auto step = std::find_if(
        timeline_actions.begin(), timeline_actions.end(),
        [](const auto& action) {
          return action.at("action_id") == "nc-step-zero";
        });
    expect(step != timeline_actions.end(), "fixed step action is missing");
    step->at("action")[field] = value;
    alternate_steps.push_back(std::move(alternate));
  }

  auto invalid_mass = nlohmann::json::parse(fixture);
  auto& invalid_mass_actions =
      invalid_mass.at("scenario").at("timelines").at(0).at("actions");
  auto custom_mass = std::find_if(
      invalid_mass_actions.begin(), invalid_mass_actions.end(),
      [](const auto& action) {
        return action.at("action_id") == "nc-custom-mass";
      });
  expect(custom_mass != invalid_mass_actions.end(), "custom mass action is missing");
  custom_mass->at("action")["mass_bits"] = 0x3f800000U;
  custom_mass->at("action")["center"]["x_bits"] = 0x40000000U;
  custom_mass->at("action")["center"]["y_bits"] = 0U;
  custom_mass->at("action")["inertia_bits"] = 0x3f800000U;

  // Act
  const auto accepted = decode_rigid_world_request(maximum.dump() + '\n');

  // Assert
  expect(
      accepted.timelines.at(0).actions.size() ==
          liquidfun::reference::kRigidWorldMaximumActions,
      "exact rigid action maximum was rejected");
  for (const auto& [record, expected] :
       std::vector<std::pair<std::string, std::string>>{
           {maximum_plus_one.dump() + '\n', "action count"},
           {alternate_steps.at(0).dump() + '\n', "fixed Phase 6 tuple"},
           {alternate_steps.at(1).dump() + '\n', "fixed Phase 6 tuple"},
           {alternate_steps.at(2).dump() + '\n', "fixed Phase 6 tuple"},
           {invalid_mass.dump() + '\n', "centered inertia"}}) {
    try {
      static_cast<void>(decode_rigid_world_request(record));
    } catch (const std::exception& error) {
      expect(
          std::string(error.what()).find(expected) != std::string::npos,
          "unexpected rigid boundary rejection: " + std::string(error.what()));
      continue;
    }
    throw std::runtime_error("invalid rigid boundary record was accepted");
  }
}

void rigid_world_rejects_zero_centered_inertia_before_execution() {
  // Arrange
  const auto fixture = read_fixture(
      "protocol/fixtures/rejected/rigid-world-zero-centered-inertia.jsonl");

  // Act / Assert
  try {
    static_cast<void>(decode_rigid_world_request(fixture));
  } catch (const std::exception& error) {
    expect(
        std::string(error.what()).find("centered inertia") != std::string::npos,
        "zero centered inertia produced an unstable diagnostic");
    return;
  }
  throw std::runtime_error("zero centered inertia reached adapter execution");
}

void rigid_world_accepts_zero_origin_inertia_with_nonzero_center() {
  // Arrange
  auto request = nlohmann::json::parse(read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl"));
  auto& action = custom_mass_action(request);
  action["mass_bits"] = 0x3f800000U;
  action["center"]["x_bits"] = 0x3f800000U;
  action["center"]["y_bits"] = 0U;
  action["inertia_bits"] = 0U;

  // Act
  const auto decoded = decode_rigid_world_request(request.dump() + '\n');

  // Assert
  expect(
      decoded.timelines.at(0).actions.size() ==
          request.at("scenario").at("timelines").at(0).at("actions").size(),
      "zero origin inertia did not preserve the reviewed action timeline");
}

void rigid_world_rejects_non_finite_centered_inertia_intermediates() {
  // Arrange
  auto request = nlohmann::json::parse(read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl"));
  auto& action = custom_mass_action(request);
  action["mass_bits"] = 0x3f800000U;
  action["center"]["x_bits"] = 0x7f7fffffU;
  action["center"]["y_bits"] = 0U;
  action["inertia_bits"] = 0x7f7fffffU;

  // Act / Assert
  try {
    static_cast<void>(decode_rigid_world_request(request.dump() + '\n'));
  } catch (const std::exception& error) {
    expect(
        std::string(error.what()).find("centered inertia") != std::string::npos,
        "non-finite centered inertia produced an unstable diagnostic");
    return;
  }
  throw std::runtime_error("non-finite centered inertia was accepted");
}

void rigid_world_reuse_advances_reset_without_state_leakage() {
  // Arrange
  const auto fixture = read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl");
  RigidWorldAdapter adapter;

  // Act
  const auto first = adapter.execute(fixture);
  const auto second = adapter.execute(fixture);
  const auto third = adapter.execute(fixture);

  // Assert
  expect(first.reset_epoch == 1, "first rigid reset epoch changed");
  expect(second.reset_epoch == 2, "second rigid reset epoch changed");
  expect(third.reset_epoch == 3, "third rigid reset epoch changed");
  expect(first.result_record == second.result_record, "rigid request leaked state");
  expect(second.result_record == third.result_record, "rigid allocation history changed identity");
}

void rigid_world_phase8_decode_fails_closed_at_reviewed_boundaries() {
  // Arrange
  constexpr std::size_t maximum_phase8_ropes = 8;
  constexpr std::size_t maximum_phase8_rope_vertices = 128;
  const auto fixture = read_fixture(
      "protocol/fixtures/accepted/rigid-world-request.jsonl");
  auto unknown_action = nlohmann::json::parse(fixture);
  unknown_action["scenario"]["timelines"][9]["actions"][0]["action"]["kind"] =
      "unknown_phase8_action";
  auto too_many_joints = nlohmann::json::parse(fixture);
  auto& joints = too_many_joints["scenario"]["timelines"][9]["joints"];
  while (joints.size() <= liquidfun::reference::kRigidWorldMaximumJoints) {
    auto duplicate = joints.at(0);
    duplicate["joint_id"] = "bounded-joint-" + std::to_string(joints.size());
    joints.push_back(std::move(duplicate));
  }
  auto missing_family = nlohmann::json::parse(fixture);
  missing_family["scenario"]["timelines"].erase(
      missing_family["scenario"]["timelines"].begin() + 9);
  auto zero_step = nlohmann::json::parse(fixture);
  zero_step["scenario"]["timelines"][9]["actions"][16]["action"]
           ["timestep_bits"] = 0U;
  auto missing_post_step_observation = nlohmann::json::parse(fixture);
  auto& observation_actions =
      missing_post_step_observation["scenario"]["timelines"][10]["actions"];
  const auto missing_observation = std::find_if(
      observation_actions.begin(), observation_actions.end(), [](const auto& record) {
        return record.at("action_id") ==
               "joint-rp-inspect-joint-rp-revolute-cold";
      });
  expect(
      missing_observation != observation_actions.end(),
      "reviewed Phase 8 observation action is missing from the fixture");
  observation_actions.erase(missing_observation);
  auto duplicate_id = nlohmann::json::parse(fixture);
  duplicate_id["scenario"]["timelines"][9]["bodies"][1]["body_id"] =
      duplicate_id["scenario"]["timelines"][9]["bodies"][0]["body_id"];
  auto too_many_actions = nlohmann::json::parse(fixture);
  auto& actions = too_many_actions["scenario"]["timelines"][9]["actions"];
  while (actions.size() <= liquidfun::reference::kRigidWorldMaximumActions) {
    auto duplicate = actions.back();
    duplicate["action_id"] = "bounded-action-" + std::to_string(actions.size());
    actions.push_back(std::move(duplicate));
  }
  auto too_many_ropes = nlohmann::json::parse(fixture);
  auto& ropes = too_many_ropes["scenario"]["timelines"][15]["ropes"];
  while (ropes.size() <= maximum_phase8_ropes) {
    auto duplicate = ropes.at(0);
    duplicate["rope_id"] = "bounded-rope-" + std::to_string(ropes.size());
    ropes.push_back(std::move(duplicate));
  }
  auto too_many_rope_vertices = nlohmann::json::parse(fixture);
  auto& bounded_rope =
      too_many_rope_vertices["scenario"]["timelines"][15]["ropes"][0];
  while (bounded_rope["vertices"].size() <= maximum_phase8_rope_vertices) {
    bounded_rope["vertices"].push_back(bounded_rope["vertices"].back());
    bounded_rope["masses_bits"].push_back(bounded_rope["masses_bits"].back());
  }

  // Act / Assert
  for (const auto& [request, expected] :
       std::array<std::pair<nlohmann::json, std::string_view>, 9>{
           std::pair{unknown_action, "unsupported Phase 8 action kind"},
           std::pair{too_many_joints, "collection count"},
           std::pair{missing_family, "timeline count"},
           std::pair{zero_step, "must be positive"},
           std::pair{missing_post_step_observation, "post-step observation"},
           std::pair{duplicate_id, "duplicate Phase 8 ID"},
           std::pair{too_many_actions, "collection count"},
           std::pair{too_many_ropes, "collection count"},
           std::pair{too_many_rope_vertices, "rope vertex count"}}) {
    try {
      static_cast<void>(decode_rigid_world_request(request.dump() + '\n'));
    } catch (const std::exception& error) {
      expect(
          std::string(error.what()).find(expected) != std::string::npos,
          "Phase 8 boundary expected diagnostic containing '" +
              std::string(expected) + "' but received '" + error.what() + "'");
      continue;
    }
    throw std::runtime_error("invalid Phase 8 request was accepted");
  }
}

void phase8_reactions_guard_uninitialized_solver_scratch() {
  // Arrange / Act / Assert
  expect(
      liquidfun::reference::phase8_reaction_guard_self_test(),
      "Phase 8 reaction guard did not separate undefined and initialized state");
}

}  // namespace

int main() {
  try {
    benchmark_run_executes_with_strict_timing_boundaries();
    benchmark_run_prepares_every_scalable_unit_before_timing();
    benchmark_run_rejection_advances_epoch_and_recovers();
    benchmark_run_keeps_profile_diagnostics_non_authoritative();
    benchmark_run_rejects_malformed_and_bounded_inputs();
    catalog_run_executes_exact_resolved_bytes_and_reuses_cleanly();
    catalog_run_preserves_distance_joint_kind_and_mutation();
    catalog_run_accepts_large_bounded_resolved_bytes();
    catalog_run_rejection_does_not_poison_the_next_request();
    catalog_run_rejects_hash_and_nested_shape_tampering();
    catalog_run_rejects_oversized_input_before_allocation();
    accepted_fixture_round_trips_exact_bits();
    framing_and_shape_fail_closed();
    unknown_versions_members_and_kinds_fail_closed();
    parser_bounds_fail_before_execution();
    scenario_references_and_phase_scope_are_validated();
    reused_adapter_resets_between_requests();
    adapter_matches_the_cross_language_trace_fixture();
    record_writer_keeps_stdout_protocol_only();
    protocol_bits_preserve_exceptional_classes();
    math_probe_matches_operation_contract();
    collision_probe_uses_existing_protocol_loop();
    rigid_world_executes_all_complete_witness_families();
    rigid_world_rejects_expanding_ray_clip_during_execution();
    rigid_world_rejects_signed_zero_clips_before_execution();
    rigid_world_rejects_invalid_derived_ray_geometry_before_execution();
    rigid_world_rejects_invalid_selector_children_before_execution();
    rigid_world_rejects_untrusted_records_before_execution();
    rigid_world_boundary_matches_the_fixed_rust_contract();
    rigid_world_rejects_zero_centered_inertia_before_execution();
    rigid_world_accepts_zero_origin_inertia_with_nonzero_center();
    rigid_world_rejects_non_finite_centered_inertia_intermediates();
    rigid_world_reuse_advances_reset_without_state_leakage();
    rigid_world_phase8_decode_fails_closed_at_reviewed_boundaries();
    phase8_reactions_guard_uninitialized_solver_scratch();
  } catch (const std::exception& error) {
    std::cerr << "protocol test failure: " << error.what() << '\n';
    return 1;
  }

  return 0;
}
