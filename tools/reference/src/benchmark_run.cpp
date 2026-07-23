#include "benchmark_run.hpp"

#include "catalog_run_session.hpp"
#include "protocol.hpp"

#include "nlohmann/json.hpp"

#include <algorithm>
#include <chrono>
#include <iomanip>
#include <limits>
#include <memory>
#include <optional>
#include <sstream>
#include <stdexcept>
#include <string>
#include <string_view>

namespace liquidfun::reference {
namespace {

using Json = nlohmann::json;
using catalog_run_detail::CatalogExecutionSession;
using catalog_run_detail::CatalogRequest;

constexpr std::size_t kMaximumEntities = 4096U;
constexpr std::size_t kMaximumActions = 128U;
constexpr std::size_t kMaximumCheckpoints = 128U;

void require_members(
    const Json& value,
    std::initializer_list<std::string_view> expected,
    std::string_view context) {
  if (!value.is_object() || value.size() != expected.size()) {
    throw std::runtime_error(std::string(context) + " has invalid members");
  }
  for (const auto field : expected) {
    if (!value.contains(field)) {
      throw std::runtime_error(
          std::string(context) + " is missing required member");
    }
  }
}

void require_array_bound(
    const Json& value,
    std::size_t maximum,
    std::string_view context) {
  if (!value.is_array() || value.size() > maximum) {
    throw std::runtime_error(std::string(context) + " exceeds reviewed bound");
  }
}

Json decode_payload(const BenchmarkRunRequest& request) {
  validate_bounded_json_record(request.resolved_bytes + '\n');
  const auto payload = Json::parse(request.resolved_bytes);
  require_members(
      payload, {"identity", "entities", "actions", "checkpoints"},
      "benchmark resolved payload");
  const auto& identity = payload.at("identity");
  require_members(
      identity,
      {"catalog_schema_version", "slug", "scenario_version", "generator_id",
       "generator_version", "maybe_seed", "settings"},
      "benchmark resolved identity");
  const auto& settings = identity.at("settings");
  require_members(
      settings,
      {"timestep_bits", "velocity_iterations", "position_iterations",
       "particle_iterations"},
      "benchmark resolved settings");
  if (catalog_run_detail::as_u32(
          identity.at("catalog_schema_version"),
          "benchmark catalog schema version") != 1U ||
      catalog_run_detail::as_u32(
          identity.at("scenario_version"),
          "benchmark scenario version") != 1U ||
      catalog_run_detail::as_u32(
          identity.at("generator_version"),
          "benchmark generator version") != 1U) {
    throw std::runtime_error(
        "unsupported benchmark resolved identity version");
  }
  static_cast<void>(catalog_run_detail::as_id(
      identity.at("slug"), "benchmark catalog slug"));
  static_cast<void>(catalog_run_detail::as_id(
      identity.at("generator_id"), "benchmark generator ID"));
  if (!identity.at("maybe_seed").is_null() &&
      !identity.at("maybe_seed").is_number_unsigned()) {
    throw std::runtime_error(
        "benchmark catalog seed must be null or unsigned");
  }
  const auto& expected = request.identity.settings;
  if (catalog_run_detail::as_u32(
          settings.at("timestep_bits"), "benchmark resolved timestep") !=
          expected.timestep_bits ||
      catalog_run_detail::as_u32(
          settings.at("velocity_iterations"),
          "benchmark resolved velocity iterations") !=
          expected.velocity_iterations ||
      catalog_run_detail::as_u32(
          settings.at("position_iterations"),
          "benchmark resolved position iterations") !=
          expected.position_iterations ||
      catalog_run_detail::as_u32(
          settings.at("particle_iterations"),
          "benchmark resolved particle iterations") !=
          expected.particle_iterations) {
    throw std::runtime_error("benchmark resolved settings mismatch");
  }
  require_array_bound(
      payload.at("entities"), kMaximumEntities, "benchmark entities");
  require_array_bound(
      payload.at("actions"), kMaximumActions, "benchmark actions");
  require_array_bound(
      payload.at("checkpoints"), kMaximumCheckpoints,
      "benchmark checkpoints");
  if (payload.at("actions").empty() || payload.at("checkpoints").empty()) {
    throw std::runtime_error("benchmark resolved schedule is incomplete");
  }
  if (payload.at("checkpoints").size() !=
      request.identity.measured_horizon) {
    throw std::runtime_error(
        "benchmark measured horizon does not match resolved checkpoints");
  }
  for (std::size_t ordinal = 0; ordinal < payload.at("entities").size();
       ++ordinal) {
    const auto& entity = payload.at("entities").at(ordinal);
    require_members(
        entity, {"semantic_id", "scenario_id"}, "benchmark entity");
    const auto& semantic_id = entity.at("semantic_id");
    require_members(
        semantic_id, {"kind", "ordinal"}, "benchmark semantic entity ID");
    auto kind = catalog_run_detail::as_id(
        semantic_id.at("kind"), "benchmark semantic entity kind");
    if (kind != "body" && kind != "fixture" && kind != "joint" &&
        kind != "rope" && kind != "particle_system" &&
        kind != "particle_group" && kind != "particle") {
      throw std::runtime_error("unknown benchmark semantic entity kind");
    }
    if (catalog_run_detail::as_u32(
            semantic_id.at("ordinal"),
            "benchmark semantic entity ordinal") != ordinal) {
      throw std::runtime_error(
          "benchmark semantic entity order is invalid");
    }
    std::replace(kind.begin(), kind.end(), '_', '-');
    std::ostringstream expected_id;
    expected_id << "entity-" << kind << '-' << std::setw(4)
                << std::setfill('0') << ordinal;
    if (catalog_run_detail::as_id(
            entity.at("scenario_id"), "benchmark entity scenario ID") !=
        expected_id.str()) {
      throw std::runtime_error(
          "benchmark semantic entity identity is invalid");
    }
  }
  return payload;
}

CatalogRequest catalog_request(
    const BenchmarkRunRequest& request,
    const Json& payload) {
  return {
      request.identity.request_id, request.identity.resolved_sha256, payload};
}

std::string run_untimed(const CatalogRequest& request) {
  CatalogExecutionSession session(request);
  std::string final_checkpoint;
  while (!session.finished()) {
    session.execute_next_logical_action();
    final_checkpoint = session.capture_current_checkpoint();
  }
  if (final_checkpoint.empty()) {
    throw std::runtime_error("benchmark produced no semantic checkpoint");
  }
  return final_checkpoint;
}

Json encode_identity(const BenchmarkRunIdentity& identity) {
  return {
      {"request_id", identity.request_id},
      {"resolved_sha256", identity.resolved_sha256},
      {"settings",
       {{"timestep_bits", identity.settings.timestep_bits},
        {"velocity_iterations", identity.settings.velocity_iterations},
        {"position_iterations", identity.settings.position_iterations},
        {"particle_iterations", identity.settings.particle_iterations}}},
      {"workload", identity.workload},
      {"size_point", identity.size_point},
      {"optimization_mode", identity.optimization_mode},
      {"warmup_count", identity.warmup_count},
      {"measured_horizon", identity.measured_horizon},
      {"sample_ordinal", identity.sample_ordinal},
      {"policy_sha256", identity.policy_sha256},
      {"profile_enabled", identity.profile_enabled}};
}

std::string encode_result(
    const BenchmarkRunRequest& request,
    std::uint64_t reset_epoch,
    std::uint64_t elapsed_nanoseconds,
    std::string_view checkpoint_record,
    const std::optional<std::pair<std::string, std::uint64_t>>&
        maybe_diagnostic) {
  const auto checkpoint = Json::parse(checkpoint_record);
  auto diagnostics = Json(nullptr);
  if (maybe_diagnostic.has_value()) {
    diagnostics = Json::array(
        {{{"phase", maybe_diagnostic->first},
          {"nanoseconds", maybe_diagnostic->second}}});
  }
  return Json{
      {"protocol_version", 1U},
      {"record_kind", "benchmark_run_result"},
      {"identity", encode_identity(request.identity)},
      {"engine_role", "pinned_cpp_oracle"},
      {"reset_epoch", reset_epoch},
      {"outcome",
       {{"outcome_kind", "performance"},
        {"outcome",
         {{"unprofiled_nanoseconds", elapsed_nanoseconds},
          {"maybe_common_parent_diagnostics", diagnostics},
          {"semantic_checkpoint_identity",
           {{"request_id", request.identity.request_id},
            {"resolved_sha256", request.identity.resolved_sha256},
            {"checkpoint_id", checkpoint.at("checkpoint_id")},
            {"checkpoint_sha256", sha256_hex(checkpoint_record)}}}}}}}}
      .dump();
}

std::optional<std::string> diagnostic_phase(std::string_view workload) {
  if (workload == "broad_phase") return "broad_phase";
  if (workload == "narrow_phase") return "narrow_phase";
  if (workload == "contact_solve") return "contact_solve";
  if (workload == "joints") return "joint_solve";
  if (workload == "particle_lifecycle" ||
      workload == "particle_contacts" || workload == "particle_sort" ||
      workload == "particle_pressure" ||
      workload == "large_particle_system") {
    return "particle_solve";
  }
  if (workload == "aabb_query" || workload == "ray_cast") {
    return "query_traversal";
  }
  return std::nullopt;
}

std::optional<std::pair<std::string, std::uint64_t>> profile_diagnostic(
    const BenchmarkRunRequest& request,
    const CatalogRequest& execution_request,
    std::string_view authority) {
  const auto maybe_phase = diagnostic_phase(request.identity.workload);
  if (!request.identity.profile_enabled || !maybe_phase.has_value()) {
    return std::nullopt;
  }
  CatalogExecutionSession profiled(execution_request);
  const auto started = std::chrono::steady_clock::now();
  while (!profiled.finished()) {
    profiled.execute_next_logical_action();
  }
  const auto stopped = std::chrono::steady_clock::now();
  if (profiled.capture_current_checkpoint() != authority) {
    throw std::runtime_error(
        "benchmark profile semantic checkpoint mismatch");
  }
  const auto raw_elapsed =
      std::chrono::duration_cast<std::chrono::nanoseconds>(
          stopped - started)
          .count();
  return std::pair{
      *maybe_phase,
      std::max<std::uint64_t>(
          1U, static_cast<std::uint64_t>(raw_elapsed))};
}

}  // namespace

BenchmarkRunAdapter::BenchmarkRunAdapter(
    BenchmarkRunObserver* maybe_observer)
    : maybe_observer_(maybe_observer) {}

BenchmarkRunTrace BenchmarkRunAdapter::execute(std::string_view record) {
  if (reset_epoch_ == std::numeric_limits<std::uint64_t>::max()) {
    throw std::runtime_error("benchmark reset counter overflow");
  }
  ++reset_epoch_;
  const auto request = decode_benchmark_run_request(record);
  const auto payload = decode_payload(request);
  const auto execution_request = catalog_request(request, payload);

  const auto authority = run_untimed(execution_request);
  observe(BenchmarkRunEvent::authority_prepared);
  for (std::uint32_t index = 0; index < request.identity.warmup_count;
       ++index) {
    if (run_untimed(execution_request) != authority) {
      throw std::runtime_error(
          "benchmark warmup semantic checkpoint mismatch");
    }
  }
  observe(BenchmarkRunEvent::warmup_complete);

  auto measured =
      std::make_unique<CatalogExecutionSession>(execution_request);
  observe(BenchmarkRunEvent::measured_setup_complete);
  observe(BenchmarkRunEvent::timer_started);
  const auto started = std::chrono::steady_clock::now();
  while (!measured->finished()) {
    measured->execute_next_logical_action();
  }
  const auto stopped = std::chrono::steady_clock::now();
  observe(BenchmarkRunEvent::timer_stopped);
  const auto measured_checkpoint = measured->capture_current_checkpoint();
  if (measured_checkpoint != authority) {
    throw std::runtime_error(
        "benchmark measured semantic checkpoint mismatch");
  }
  observe(BenchmarkRunEvent::checkpoint_validated);
  measured.reset();
  observe(BenchmarkRunEvent::teardown_complete);

  const auto raw_elapsed =
      std::chrono::duration_cast<std::chrono::nanoseconds>(
          stopped - started)
          .count();
  const auto elapsed_nanoseconds = std::max<std::uint64_t>(
      1U, static_cast<std::uint64_t>(raw_elapsed));
  const auto maybe_diagnostic =
      profile_diagnostic(request, execution_request, authority);
  return {
      encode_result(
          request, reset_epoch_, elapsed_nanoseconds, measured_checkpoint,
          maybe_diagnostic),
      reset_epoch_};
}

void BenchmarkRunAdapter::observe(BenchmarkRunEvent event) const {
  if (maybe_observer_ != nullptr) {
    maybe_observer_->observe(event);
  }
}

}  // namespace liquidfun::reference
