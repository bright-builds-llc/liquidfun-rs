#include "collision_probe.hpp"
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
      "origin_shift_covariance"};
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

}  // namespace

int main() {
  try {
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
  } catch (const std::exception& error) {
    std::cerr << "protocol test failure: " << error.what() << '\n';
    return 1;
  }

  return 0;
}
