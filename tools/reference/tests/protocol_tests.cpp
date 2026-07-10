#include "oracle_adapter.hpp"
#include "protocol.hpp"

#include <filesystem>
#include <fstream>
#include <iostream>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

namespace {

using liquidfun::reference::BuildIdentity;
using liquidfun::reference::OracleAdapter;
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
  return BuildIdentity{
      "7f20402173fd143a3988c921bc384459c6a858f2",
      "fixture-adapter-v1",
      "c7f36eaf2f184a36b9c9a04636d3e22785d815c4948d55d0b3cbf44ee7245fc8",
      "oracle-debug",
      "Clang",
      "22.1.8",
      "x86_64-unknown-linux-gnu",
      "Debug",
      "-O0 -g",
      "-lc++",
      "none"};
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
  } catch (const std::exception& error) {
    std::cerr << "protocol test failure: " << error.what() << '\n';
    return 1;
  }

  return 0;
}
