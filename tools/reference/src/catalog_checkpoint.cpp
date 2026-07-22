#include "catalog_checkpoint.hpp"

#include "nlohmann/json.hpp"

#include <array>
#include <stdexcept>
#include <string>

namespace liquidfun::reference {
namespace {

nlohmann::json count_observation(std::string_view id, std::uint64_t count) {
  return {{"observation_id", id},
          {"value", {{"kind", "count"}, {"value", count}}}};
}

}  // namespace

std::string encode_catalog_checkpoint(const CatalogCheckpointInput& input) {
  if (input.logical_step == 0U || input.checkpoint_id.empty()) {
    throw std::runtime_error("invalid catalog checkpoint identity");
  }
  auto observations = nlohmann::json::array();
  observations.push_back(
      count_observation("world-body-count", input.counts.bodies));
  observations.push_back(
      count_observation("world-contact-count", input.counts.contacts));
  observations.push_back(count_observation(
      "world-debug-primitive-count", input.debug_primitive_count));
  observations.push_back(
      count_observation("world-fixture-count", input.counts.fixtures));
  observations.push_back(
      count_observation("world-joint-count", input.counts.joints));
  observations.push_back(
      count_observation("world-particle-count", input.counts.particles));

  const nlohmann::json checkpoint = {
      {"protocol_version", kProtocolVersion},
      {"record_kind", "canonical_checkpoint"},
      {"checkpoint_schema_version", 1U},
      {"request_id", input.request_id},
      {"resolved_sha256", input.resolved_sha256},
      {"checkpoint_id", input.checkpoint_id},
      {"position", {{"kind", "logical_step"},
                    {"ordinal", input.logical_step}}},
      {"simulation_time_bits", input.simulation_time_bits},
      {"observations", std::move(observations)},
      {"numeric_observations", nlohmann::json::array()},
      {"ordered_occurrences", nlohmann::json::array()},
      {"unordered_sets", nlohmann::json::array()},
      {"debug_primitives", nlohmann::json::array()},
      {"profile_names", nlohmann::json::array()},
  };
  const auto encoded = checkpoint.dump();
  if (encoded.size() > kMaximumTraceBytes) {
    throw std::runtime_error("catalog checkpoint exceeds output limit");
  }
  return encoded;
}

std::string encode_catalog_run_end(
    std::string_view request_id,
    std::string_view resolved_sha256,
    std::uint32_t checkpoint_count,
    std::uint64_t reset_epoch) {
  return nlohmann::json{
      {"protocol_version", kProtocolVersion},
      {"record_kind", "catalog_run_end"},
      {"request_id", request_id},
      {"resolved_sha256", resolved_sha256},
      {"checkpoint_count", checkpoint_count},
      {"reset_epoch", reset_epoch},
      {"reset_verified", true}}
      .dump();
}

}  // namespace liquidfun::reference
