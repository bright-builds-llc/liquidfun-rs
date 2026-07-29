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

#include "protocol_tests/catalog.hpp"

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

#include "protocol_tests/scenario.hpp"

#include "protocol_tests/rigid_execution.hpp"

#include "protocol_tests/rigid_validation.hpp"

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
